//! A glowing orb loader with a staggered letter row.
//!
//! Ported from [dexter-st / bright-lizard-8](https://uiverse.io/dexter-st/bright-lizard-8) (MIT).
//!
//! The reference is not a stack of CSS layers over a background: it is one round
//! element carrying three `inset` box-shadows, rotating on a 2 s linear loop while
//! the shadows' colours cross-fade at the halfway point.
//!
//! - the first shadow is white — a thin hot sheen that only exists near the rim;
//! - the second is the colour lobe (`#ad5fff` to `#d60a47`), a broad light that
//!   reaches a third of the radius in and is strongest where the rotation points;
//! - the third is the deep wash (`#471eec` to `#311e80`), which tints the whole
//!   shell and is dimmest where the light does not reach.
//!
//! egui has no inset shadow, so the port rebuilds each one from its geometry: a
//! circle of the ball's radius shifted away from the light, blurred by the
//! shadow's radius, painted over everything outside it. The three crescents
//! therefore have circular edges, and those arcs — not a radial gradient — are
//! what read as an inner sphere turning inside the ball. Every vertex of one
//! radial mesh ([Orb::paint_at]) evaluates the three coverage functions and
//! composites them base first.
//!
//! The label pulses letter by letter on the same 2 s clock, each one a step behind
//! the last: opacity `0.4` to `1.0` with a 1.15x scale at 20 % of its loop, down
//! to `0.7` at 40 %, then idle back to `0.4`.
//!
//! ```
//! # use egui::__run_test_ui;
//! # use egui_elements::widgets::Orb;
//! # __run_test_ui(|ui| {
//! ui.add(Orb::new().text("Generating"));
//! ui.add(Orb::new().size(64.0));
//! # });
//! ```
//!
//! The colours come from the installed [Theme]: every shell is a hue offset from
//! [ThemeColors::accent] carrying the reference's saturation and brightness, and
//! the disc the shadows fall on is a dark tone of the same hue — opaque, so the
//! ball reads the same on a light palette as on a dark one instead of turning
//! into a ring.
//!
//! [Theme]: crate::theme::Theme
//! [ThemeColors::accent]: crate::theme::ThemeColors::accent

use std::sync::Arc;

use egui::epaint::{Mesh, TextShape};
use egui::{
   Color32, FontId, Galley, Rect, Response, Role, Sense, Shape, TextStyle, Ui, Vec2, Widget,
   WidgetInfo, emath::TSTransform, pos2, vec2,
};

use crate::motion::Easing;
use crate::theme::{Theme, ThemeColors};
use crate::utils::Hsla;

/// Shell radii, as fractions of the orb's radius. Dense toward the rim, where the
/// sheen and the lobe are thin.
const RINGS: [f32; 9] = [0.35, 0.55, 0.68, 0.78, 0.86, 0.92, 0.96, 0.985, 1.0];

/// Fewest angular samples around the light axis.
const MIN_SEGMENTS: usize = 24;

/// Most angular samples around the light axis.
const MAX_SEGMENTS: usize = 96;

/// Angular samples per point of diameter, before clamping.
const SEGMENTS_PER_POINT: f32 = 0.6;

/// Side of the square the widget allocates without a label, as a multiple of the
/// style's `interact_size.y`.
const DEFAULT_SIZE_RATIO: f32 = 2.6;

/// Share of the diameter the ported label occupies (108 pt of text in 180 pt), so
/// a longer label grows the ball instead of spilling out of it.
const TEXT_FIT_RATIO: f32 = 0.60;

/// Label size as a multiple of the body text size.
const LABEL_EM: f32 = 1.5;

/// Seconds of one full turn, and of one letter's pulse — the ported `2s`.
const DEFAULT_PERIOD: f32 = 2.0;

/// Shortest loop, so a caller cannot divide the phase by zero.
const MIN_PERIOD: f32 = 0.05;

/// Step between two neighbouring letters, in seconds. The ported `0.1s` ladder.
const DEFAULT_LETTER_DELAY: f32 = 0.1;

/// Letters that carry a distinct delay before the ladder repeats (`:nth-child(1..10)`).
const DELAY_STEPS: usize = 10;

/// Direction the light starts at, in radians: straight up.
const LIGHT_START: f32 = -std::f32::consts::FRAC_PI_2;

/// Opacity of a letter at the start and the end of its own loop.
const LETTER_IDLE_OPACITY: f32 = 0.4;

/// Opacity of a letter at the peak of its pulse.
const LETTER_PEAK_OPACITY: f32 = 1.0;

/// Opacity a letter settles to after the peak, before idling out.
const LETTER_SETTLE_OPACITY: f32 = 0.7;

/// Scale of a letter at the peak of its pulse.
const LETTER_PEAK_SCALE: f32 = 1.15;

/// Where in the letter's loop the peak sits.
const LETTER_PEAK_AT: f32 = 0.20;

/// Where in the letter's loop the settle sits.
const LETTER_SETTLE_AT: f32 = 0.40;

/// Hue offset of the disc the shadows fall on from the accent's, in degrees. The
/// reference's interior is bluer than either shadow (it reads `#0a0a26`), which is
/// what keeps the ball from turning into a flat purple disc.
const BODY_HUE: f32 = -30.0;

/// Saturation of that disc, in percent.
const BODY_SATURATION: f32 = 58.0;

/// Lightness of that disc, in percent — dark enough to read as the ported ball's
/// interior on a light palette as well as a dark one.
const BODY_LIGHTNESS: f32 = 10.0;

/// Alpha of that disc, which is what keeps the ball opaque on any surface.
const BODY_ALPHA: f32 = 0.92;

/// Where the rim starts to feather out, as a fraction of the radius. A mesh is not
/// anti-aliased, so the outermost ring has to fade or the ball's edge is stepped.
const FEATHER_FROM: f32 = 0.97;

/// Hue offset of the deep wash from the accent's, in degrees (`#471eec` is 17° off
/// `#ad5fff`).
const DEEP_HUE: f32 = -17.3;

/// Saturation of the deep wash at the start of the loop, in percent.
const DEEP_SATURATION: f32 = 84.4;

/// Saturation of the deep wash at the halfway point, in percent.
const DEEP_SATURATION_END: f32 = 62.0;

/// Brightness of the deep wash as a share of the accent's, measured from the
/// ported keyframes (`#471eec` against `#ad5fff`): the wash keeps the
/// reference's light-and-dark structure whatever the accent's hue is.
const DEEP_LUMA_RATIO: f32 = 53.6 / 123.1;

/// Brightness of the deep wash at the halfway point (`#311e80`).
const DEEP_LUMA_MID_RATIO: f32 = 41.1 / 123.1;

/// Offset of the deep wash's circle at the 0 % keyframe, as a fraction of the
/// diameter (`60px` of the ported `180px`).
const DEEP_OFFSET: f32 = 60.0 / 180.0;

/// Offset of the deep wash's circle at the 50 % keyframe (`40px`).
const DEEP_OFFSET_MID: f32 = 40.0 / 180.0;

/// Blur radius of the deep wash (`60px`), constant across the loop.
const DEEP_BLUR: f32 = 60.0 / 180.0;

/// Hue offset of the colour lobe at the halfway point, in degrees (`#ad5fff` to
/// `#d60a47` walks the hue 73°).
const LOBE_HUE: f32 = 73.0;

/// Saturation of the colour lobe at the start of the loop, in percent.
const LOBE_SATURATION: f32 = 100.0;

/// Saturation of the colour lobe at the halfway point, in percent.
const LOBE_SATURATION_END: f32 = 91.1;

/// Brightness of the colour lobe as a share of the accent's (`#ad5fff` is the
/// reference's accent).
const LOBE_LUMA_RATIO: f32 = 1.0;

/// Brightness of the colour lobe at the halfway point (`#d60a47`).
const LOBE_LUMA_MID_RATIO: f32 = 57.8 / 123.1;

/// Offset of the colour lobe's circle (`20px`), constant across the loop.
const LOBE_OFFSET: f32 = 20.0 / 180.0;

/// Blur radius of the colour lobe at the 0 % keyframe (`30px`).
const LOBE_BLUR: f32 = 30.0 / 180.0;

/// Blur radius of the colour lobe at the 50 % keyframe (`10px`).
const LOBE_BLUR_MID: f32 = 10.0 / 180.0;

/// Offset of the sheen's circle (`10px`), constant across the loop.
const SHEEN_OFFSET: f32 = 10.0 / 180.0;

/// Blur radius of the sheen (`20px`).
const SHEEN_BLUR: f32 = 20.0 / 180.0;

/// Half-width of a blurred edge, in blur radii. A CSS `box-shadow` blur fades a
/// hard edge across roughly 1.5 x its radius, which a `smoothstep` over this band
/// matches.
const BLUR_SPREAD: f32 = 0.75;

/// Share of white in the sheen's colour. The ported `#fff` shadow is what the
/// rim's hot edge is made of.
const SHEEN_WHITE: f32 = 0.95;

/// Saturation under which an accent counts as neutral: a grey has no hue to walk,
/// so [`Orb::color`] then paints a silver ball instead of a saturated one.
const ACHROMATIC_SATURATION: f32 = 5.0;

/// The four layers of one frame, base disc first.
struct Shells {
   /// The disc the ported shadows fall on.
   body: Color32,
   /// The deep wash.
   deep: Shadow,
   /// The colour lobe under the light.
   lobe: Shadow,
   /// The hot sheen at the rim.
   sheen: Shadow,
}

/// One ported `inset` shadow: its colour and the circle it blurs onto the ball.
struct Shadow {
   color: Color32,
   /// Offset of the shadow's circle from the ball's centre, as a fraction of the
   /// diameter.
   offset: f32,
   /// Blur radius, as a fraction of the diameter.
   blur: f32,
}

impl Shadow {
   /// Blurred coverage of this shadow at `u` of the ball's radius and `angle` in
   /// radians, with the light at `spin`.
   ///
   /// The shadow is everything outside a circle of the ball's radius shifted
   /// *away* from the light, so the lit side is the side that circle does not
   /// reach. Its colour is fully saturated where the crescent is thickest and
   /// fades across the blur, so the arc reads as the edge of an inner sphere
   /// turning inside the ball.
   fn coverage(&self, u: f32, angle: f32, spin: f32) -> f32 {
      // the shifted circle's centre, in units of the ball's radius
      let shift = 2.0 * self.offset;
      let distance = (u * u + shift * shift + 2.0 * u * shift * (angle - spin).cos()).sqrt();

      smoothstep(
         -2.0 * self.blur * BLUR_SPREAD,
         2.0 * self.blur * BLUR_SPREAD,
         distance - 1.0,
      )
   }
}

/// A glowing orb loader with an optional staggered label.
///
/// Reads [`ThemeColors`] from the installed [`Theme`] unless you pass
/// [`.color(...)`](Self::color), which retints the whole shell: every shell is a
/// hue offset from that colour, so a pure grey paints a silver ball.
///
/// ```
/// # use egui::{Color32, __run_test_ui};
/// # use egui_elements::widgets::Orb;
/// # __run_test_ui(|ui| {
/// ui.add(Orb::new().text("Generating"));                    // accent shell, label-sized
/// ui.add(Orb::new().size(64.0));                            // ball only
/// ui.add(Orb::new().text("Loading").color(Color32::from_rgb(0xad, 0x5f, 0xff))); // retint
/// ui.add(Orb::new().period(1.0).letter_delay(0.05));        // faster, tighter stagger
/// # });
/// ```
///
/// [`ThemeColors`]: crate::theme::ThemeColors
/// [`Theme`]: crate::theme::Theme
#[derive(Clone, Debug, PartialEq)]
#[must_use = "Orbs should be shown on a ui, e.g. `ui.add(Orb::new())`"]
pub struct Orb {
   size: Option<f32>,
   text: Option<String>,
   color: Option<Color32>,
   period: f32,
   letter_delay: f32,
   text_size: Option<f32>,
   easing: Easing,
}

impl Orb {
   /// An orb in the theme accent, sized to 2.6 x the style's `interact_size.y`
   /// and grown to fit [`.text(...)`](Self::text).
   pub fn new() -> Self {
      Self {
         size: None,
         text: None,
         color: None,
         period: DEFAULT_PERIOD,
         letter_delay: DEFAULT_LETTER_DELAY,
         text_size: None,
         easing: Easing::EaseOut,
      }
   }

   /// Side of the square the orb occupies. Defaults to 2.6 x the style's
   /// `interact_size.y`, grown so the label stays inside the ball.
   #[inline]
   pub fn size(mut self, size: f32) -> Self {
      self.size = Some(size);
      self
   }

   /// The label painted inside the orb, one letter at a time. An empty string is
   /// the same as no label at all.
   #[inline]
   pub fn text(mut self, text: impl Into<String>) -> Self {
      self.text = Some(text.into());
      self
   }

   /// Accent colour of the shell. Defaults to
   /// [`ThemeColors::accent`](crate::theme::ThemeColors::accent).
   #[inline]
   pub fn color(mut self, color: impl Into<Color32>) -> Self {
      self.color = Some(color.into());
      self
   }

   /// Seconds of one full turn, which is also one letter's pulse. Default: `2.0`.
   #[inline]
   pub fn period(mut self, period: f32) -> Self {
      self.period = period;
      self
   }

   /// Step between two neighbouring letters, in seconds. `0.0` pulses every
   /// letter at once. Default: `0.1`.
   #[inline]
   pub fn letter_delay(mut self, delay: f32) -> Self {
      self.letter_delay = delay;
      self
   }

   /// Label size in points. Defaults to 1.5 x the body text size.
   #[inline]
   pub fn text_size(mut self, text_size: f32) -> Self {
      self.text_size = Some(text_size);
      self
   }

   /// Curve of a letter's rise and fall. Default: [`Easing::EaseOut`], the CSS
   /// `ease` measured against the reference capture.
   #[inline]
   pub fn easing(mut self, easing: Easing) -> Self {
      self.easing = easing;
      self
   }

   /// The label's text, or `None` when it is empty.
   fn label(&self) -> Option<&str> {
      self.text.as_deref().filter(|text| !text.is_empty())
   }

   /// Seconds of one full loop, floored so the phase cannot divide by zero.
   fn loop_period(&self) -> f32 {
      self.period.max(MIN_PERIOD)
   }

   /// The label's font for `style`, honouring [`Orb::text_size`].
   fn font(&self, style: &egui::Style) -> FontId {
      let body = TextStyle::Body.resolve(style).size;

      FontId::proportional(self.text_size.unwrap_or(body * LABEL_EM))
   }

   /// The label's letters — one galley each, because each one pulses on its own
   /// clock — and their total width.
   fn letters(&self, ui: &Ui, font: &FontId) -> (Vec<Arc<Galley>>, f32) {
      let Some(text) = self.label() else {
         return (Vec::new(), 0.0);
      };

      let painter = ui.painter();
      let mut letters = Vec::with_capacity(text.len());
      let mut width = 0.0;

      for chr in text.chars() {
         let galley = painter.layout_no_wrap(chr.to_string(), font.clone(), Color32::WHITE);

         width += galley.size().x;
         letters.push(galley);
      }

      (letters, width)
   }

   /// The four layers at `mix` of the loop (`0.0` is the `0%` keyframe, `1.0` the
   /// `50%` one; the loop comes back through the same ramp).
   ///
   /// Only the accent's hue is taken from the palette: the ported shadows are fully
   /// saturated (`#ad5fff` is 100 % saturated), so a muted accent would read as a grey
   /// ball with a tint rather than as a light.
   fn shells(accent: Color32, mix: f32) -> Shells {
      let seed = Hsla::from_color32(accent);

      // a grey accent has no hue to offset, so keep the whole shell neutral
      let tint = if seed.s < ACHROMATIC_SATURATION {
         0.0
      } else {
         1.0
      };
      let accent_luma = luma(accent);
      let deep = shadow_color(
         seed.h + DEEP_HUE,
         tint * lerp(DEEP_SATURATION, DEEP_SATURATION_END, mix),
         accent_luma * lerp(DEEP_LUMA_RATIO, DEEP_LUMA_MID_RATIO, mix),
      );
      let lobe = shadow_color(
         seed.h + LOBE_HUE * mix,
         tint * lerp(LOBE_SATURATION, LOBE_SATURATION_END, mix),
         accent_luma * lerp(LOBE_LUMA_RATIO, LOBE_LUMA_MID_RATIO, mix),
      );
      let sheen = lobe.lerp_to_gamma(Color32::WHITE, SHEEN_WHITE);

      Shells {
         body: Hsla {
            h: seed.h + BODY_HUE,
            s: tint * BODY_SATURATION,
            l: BODY_LIGHTNESS,
            a: 1.0,
         }
         .to_color32(),
         deep: Shadow {
            color: deep,
            offset: lerp(DEEP_OFFSET, DEEP_OFFSET_MID, mix),
            blur: DEEP_BLUR,
         },
         lobe: Shadow {
            color: lobe,
            offset: LOBE_OFFSET,
            blur: lerp(LOBE_BLUR, LOBE_BLUR_MID, mix),
         },
         sheen: Shadow {
            color: sheen,
            offset: SHEEN_OFFSET,
            blur: SHEEN_BLUR,
         },
      }
   }

   /// The ball's colour and alpha at `u` of the radius, `angle` in radians, with
   /// the light at `spin`.
   ///
   /// The three ported shadows are composited base first; each one paints its own
   /// crescent with its own colour, and the crescents' blurred edges are the
   /// circular arcs that read as an inner sphere turning inside the ball.
   fn shade(u: f32, angle: f32, spin: f32, shells: &Shells) -> Color32 {
      let rim = u.max(0.0);
      let feather = 1.0 - smoothstep(FEATHER_FROM, 1.0, rim);
      let mut rgb = [
         shells.body.r() as f32,
         shells.body.g() as f32,
         shells.body.b() as f32,
      ];
      let mut alpha = BODY_ALPHA;

      for shadow in [&shells.deep, &shells.lobe, &shells.sheen] {
         let layer = shadow.coverage(u, angle, spin);
         if layer <= 0.0 {
            continue;
         }

         let combined = layer + alpha * (1.0 - layer);
         if combined <= 0.0 {
            continue;
         }

         let source = [
            shadow.color.r() as f32,
            shadow.color.g() as f32,
            shadow.color.b() as f32,
         ];
         for (channel, value) in rgb.iter_mut().zip(source) {
            *channel = (value * layer + *channel * alpha * (1.0 - layer)) / combined;
         }
         alpha = combined;
      }

      alpha *= feather;

      Color32::from_rgba_unmultiplied(
         rgb[0].round().clamp(0.0, 255.0) as u8,
         rgb[1].round().clamp(0.0, 255.0) as u8,
         rgb[2].round().clamp(0.0, 255.0) as u8,
         (alpha.clamp(0.0, 1.0) * 255.0).round() as u8,
      )
   }

   /// One radial mesh: a centre vertex, then one ring per [`RINGS`] entry, each
   /// vertex shaded.
   fn mesh(&self, orb: Rect, accent: Color32, phase: f32) -> Mesh {
      // the colours and the two animated blurs go out and back over one loop
      let mix = 1.0 - (2.0 * phase - 1.0).abs();
      let shells = Self::shells(accent, mix);
      let spin = LIGHT_START + std::f32::consts::TAU * phase;
      let radius = orb.width() * 0.5;
      let segments =
         ((orb.width() * SEGMENTS_PER_POINT).round() as usize).clamp(MIN_SEGMENTS, MAX_SEGMENTS);
      let mut mesh = Mesh::default();

      mesh.colored_vertex(
         orb.center(),
         Self::shade(0.0, spin, spin, &shells),
      );

      for u in RINGS {
         for segment in 0..segments {
            let angle = segment as f32 / segments as f32 * std::f32::consts::TAU;
            let (sin, cos) = angle.sin_cos();
            let position = orb.center() + vec2(cos, sin) * (radius * u);

            mesh.colored_vertex(position, Self::shade(u, angle, spin, &shells));
         }
      }

      let step = segments as u32;

      for segment in 0..step {
         let next = (segment + 1) % step;

         mesh.add_triangle(0, 1 + segment, 1 + next);
      }

      for ring in 0..(RINGS.len() as u32 - 1) {
         let inner = 1 + ring * step;
         let outer = inner + step;

         for segment in 0..step {
            let next = (segment + 1) % step;

            mesh.add_triangle(inner + segment, outer + segment, outer + next);
            mesh.add_triangle(inner + segment, outer + next, inner + next);
         }
      }

      mesh
   }

   /// The label, centred in `orb`, each letter on its own clock.
   fn paint_label(&self, ui: &Ui, orb: Rect, colors: ThemeColors, time: f64) {
      let font = self.font(ui.style());
      let (letters, width) = self.letters(ui, &font);
      if letters.is_empty() {
         return;
      }

      let period = self.loop_period() as f64;
      let painter = ui.painter();
      let mut cursor = orb.center().x - width * 0.5;

      for (index, galley) in letters.iter().enumerate() {
         let size = galley.size();
         let pos = pos2(cursor, orb.center().y - size.y * 0.5);
         let (opacity, scale) = self.letter_pulse(time, period, index);
         let mut shape = TextShape::new(pos, galley.clone(), colors.text);

         shape.opacity_factor = opacity;
         shape.transform(TSTransform {
            scaling: scale,
            translation: (pos + size * 0.5).to_vec2() * (1.0 - scale),
         });
         painter.add(shape);

         cursor += size.x;
      }
   }

   /// Opacity and scale of letter `index` at `time`: the ported keyframes, each
   /// letter [`Orb::letter_delay`] behind the one before it.
   fn letter_pulse(&self, time: f64, period: f64, index: usize) -> (f32, f32) {
      let delay = self.letter_delay.max(0.0) as f64 * (index.min(DELAY_STEPS - 1)) as f64;
      let phase = ((time - delay) / period).rem_euclid(1.0) as f32;

      if phase < LETTER_PEAK_AT {
         let t = self.easing.apply(phase / LETTER_PEAK_AT);

         (
            lerp(LETTER_IDLE_OPACITY, LETTER_PEAK_OPACITY, t),
            lerp(1.0, LETTER_PEAK_SCALE, t),
         )
      } else if phase < LETTER_SETTLE_AT {
         let span = LETTER_SETTLE_AT - LETTER_PEAK_AT;
         let t = self.easing.apply((phase - LETTER_PEAK_AT) / span);

         (
            lerp(LETTER_PEAK_OPACITY, LETTER_SETTLE_OPACITY, t),
            lerp(LETTER_PEAK_SCALE, 1.0, t),
         )
      } else {
         let t = (phase - LETTER_SETTLE_AT) / (1.0 - LETTER_SETTLE_AT);

         (
            lerp(LETTER_SETTLE_OPACITY, LETTER_IDLE_OPACITY, t),
            1.0,
         )
      }
   }

   /// Paints the orb in `rect`, centred and inset to the shorter side.
   ///
   /// Public so a widget can host one without allocating a second `Ui`.
   pub fn paint_at(&self, ui: &Ui, rect: Rect) {
      if !ui.is_rect_visible(rect) {
         return;
      }

      // because it is animated
      ui.ctx().request_repaint();

      let size = rect.width().min(rect.height());
      if size <= 0.0 {
         return;
      }

      let orb = Rect::from_center_size(rect.center(), Vec2::splat(size));
      let colors = Theme::colors_from_ctx(ui.ctx());
      let accent = self.color.unwrap_or(colors.accent);
      let time = ui.ctx().input(|input| input.time);
      let phase = (time / self.loop_period() as f64).rem_euclid(1.0) as f32;

      ui.painter().add(Shape::mesh(self.mesh(orb, accent, phase)));
      self.paint_label(ui, orb, colors, time);
   }

   /// Side of the square the widget allocates: [`Orb::size`], else the default
   /// square grown so the label keeps [`TEXT_FIT_RATIO`] of the diameter.
   fn requested_size(&self, ui: &Ui) -> f32 {
      if let Some(size) = self.size {
         return size;
      }

      let floor = ui.spacing().interact_size.y * DEFAULT_SIZE_RATIO;
      if self.label().is_none() {
         return floor;
      }

      let font = self.font(ui.style());
      let (_, width) = self.letters(ui, &font);

      floor.max(width / TEXT_FIT_RATIO)
   }
}

impl Default for Orb {
   fn default() -> Self {
      Self::new()
   }
}

impl Widget for Orb {
   fn ui(self, ui: &mut Ui) -> Response {
      let size = self.requested_size(ui);
      let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());

      response.widget_info(|| match self.label() {
         Some(text) => WidgetInfo::labeled(Role::ProgressIndicator, ui.is_enabled(), text),
         None => WidgetInfo::new(Role::ProgressIndicator),
      });
      self.paint_at(ui, rect);

      response
   }
}

/// Perceived brightness of `color` in `0..=255`.
fn luma(color: Color32) -> f32 {
   f32::from(color.r()) * 0.2126 + f32::from(color.g()) * 0.7152 + f32::from(color.b()) * 0.0722
}

/// A colour at `hue` and `saturation` (percent) whose brightness matches the
/// reference's: the port keeps the source's light-and-dark structure, and only
/// the hue follows the theme.
fn shadow_color(hue: f32, saturation: f32, target: f32) -> Color32 {
   let (mut low, mut high) = (0.0f32, 100.0f32);

   for _ in 0..24 {
      let mid = (low + high) * 0.5;
      let color = Hsla {
         h: hue,
         s: saturation,
         l: mid,
         a: 1.0,
      }
      .to_color32();

      if luma(color) < target {
         low = mid;
      } else {
         high = mid;
      }
   }

   Hsla {
      h: hue,
      s: saturation,
      l: (low + high) * 0.5,
      a: 1.0,
   }
   .to_color32()
}

/// Linear interpolation of two scalars.
fn lerp(from: f32, to: f32, t: f32) -> f32 {
   from + (to - from) * t.clamp(0.0, 1.0)
}

/// Hermite ramp from `0.0` at `edge0` to `1.0` at `edge1`.
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
   let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);

   t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
   use super::*;
   use egui::{Context, FullOutput, Pos2, RawInput};

   /// The square every test paints into.
   const SIDE: f32 = 180.0;

   fn orb_rect() -> Rect {
      Rect::from_min_size(pos2(20.0, 20.0), Vec2::splat(SIDE))
   }

   fn accent() -> Color32 {
      Theme::new(crate::theme::ThemeKind::TokyoNight).colors.accent
   }

   /// One frame at an explicit `time`, with the loader painted into a fixed square.
   fn frame(orb: &Orb, time: f64) -> FullOutput {
      let ctx = Context::default();
      let input = RawInput {
         screen_rect: Some(Rect::from_min_size(
            Pos2::ZERO,
            vec2(400.0, 400.0),
         )),
         time: Some(time),
         ..Default::default()
      };
      let mut out = ctx.run_ui(input, |ui| orb.paint_at(ui, orb_rect()));
      out.textures_delta.clear();

      out
   }

   /// The single radial mesh a frame paints.
   fn mesh(out: &FullOutput) -> Mesh {
      fn walk(shape: &Shape, found: &mut Option<Mesh>) {
         match shape {
            Shape::Mesh(mesh) => *found = Some((**mesh).clone()),
            Shape::Vec(shapes) => shapes.iter().for_each(|shape| walk(shape, found)),
            _ => {}
         }
      }

      let mut found = None;
      for clipped in &out.shapes {
         walk(&clipped.shape, &mut found);
      }

      found.expect("the orb paints one radial mesh")
   }

   /// The brightest vertex of `mesh` and its angle around the mesh's centre, in
   /// degrees.
   fn brightest(mesh: &Mesh, center: Pos2) -> (f32, f32) {
      mesh
         .vertices
         .iter()
         .map(|vertex| {
            let offset = vertex.pos - center;
            let angle = offset.y.atan2(offset.x).to_degrees().rem_euclid(360.0);

            (luma(vertex.color), angle)
         })
         .fold((0.0, 0.0), |best, next| {
            if next.0 > best.0 { next } else { best }
         })
   }

   #[test]
   fn the_interior_is_dark_and_the_lit_rim_is_bright() {
      let shells = Orb::shells(accent(), 0.0);
      let center = Orb::shade(0.0, 0.0, LIGHT_START, &shells);
      // 0.96 is the last ring before the rim feathers out
      let rim = Orb::shade(0.96, LIGHT_START, LIGHT_START, &shells);
      let far = Orb::shade(
         0.96,
         LIGHT_START + std::f32::consts::PI,
         LIGHT_START,
         &shells,
      );

      assert!(
         luma(center) < 50.0,
         "the interior stays dark: {center:?} is {}",
         luma(center)
      );
      assert!(
         luma(rim) > 200.0,
         "the lit rim runs to white: {rim:?} is {}",
         luma(rim)
      );
      assert!(
         luma(far) < luma(rim) * 0.5,
         "the unlit rim keeps only the wash: {} vs {}",
         luma(far),
         luma(rim)
      );

      let mesh = mesh(&frame(&Orb::new(), 0.0));
      assert!(
         mesh.vertices.iter().all(|vertex| vertex.pos.is_finite()),
         "every vertex is finite"
      );
   }

   #[test]
   fn the_light_orbits_the_ball_once_per_period() {
      let orb = Orb::new();
      let center = orb_rect().center();
      let (start_luma, start) = brightest(&mesh(&frame(&orb, 0.0)), center);
      let (half_luma, half) = brightest(&mesh(&frame(&orb, 1.0)), center);

      assert!(
         (start - 270.0).abs() < 20.0,
         "the light starts at the top: {start} degrees"
      );
      assert!(
         ((start - half).rem_euclid(360.0) - 180.0).abs() < 20.0,
         "half a period puts the light on the far side: {start} vs {half}"
      );
      assert!(
         start_luma > 150.0 && half_luma > 150.0,
         "the light stays bright at both keyframes: {start_luma} vs {half_luma}"
      );
   }

   #[test]
   fn the_ball_fills_the_allocation_and_feathers_at_the_rim() {
      let mesh = mesh(&frame(&Orb::new(), 0.0));
      let rect = orb_rect();
      let reach = SIDE * 0.5;

      assert!(
         mesh.vertices.iter().all(|vertex| rect.expand(0.5).contains(vertex.pos)),
         "no vertex spills out of the allocation"
      );
      assert!(
         mesh
            .vertices
            .iter()
            .any(|vertex| (vertex.pos - rect.center()).length() > reach - 1.0),
         "the ball reaches the allocation's edge"
      );
      assert!(
         mesh
            .vertices
            .iter()
            .filter(|vertex| (vertex.pos - rect.center()).length() > reach - 0.5)
            .all(|vertex| vertex.color.a() == 0),
         "the outermost ring fades out, so the edge is not stepped"
      );
   }

   #[test]
   fn each_shadow_is_bounded_by_a_shifted_circle() {
      let shells = Orb::shells(accent(), 0.0);
      let shadow = &shells.deep;
      let shift = 2.0 * shadow.offset;
      let spin = LIGHT_START;
      let center = -shift * vec2(spin.cos(), spin.sin());

      // the shadow's edge is a circle around its shifted centre: every point of
      // it that falls inside the ball is half-covered, whatever radius it sits at
      for degrees in [-20.0f32, -45.0, -90.0, -135.0, -160.0] {
         let around = degrees.to_radians();
         let point = center + vec2(around.cos(), around.sin());
         let u = point.length();
         let angle = point.y.atan2(point.x);
         let coverage = shadow.coverage(u, angle, spin);

         assert!(
            u <= 1.0,
            "the edge point at {degrees} degrees is inside the ball"
         );
         assert!(
            (coverage - 0.5).abs() < 0.01,
            "the shadow fades over its circle edge at {degrees} degrees: {coverage}"
         );
      }
   }

   #[test]
   fn a_label_pulses_letters_one_delay_apart() {
      let orb = Orb::new().letter_delay(0.1);
      let period = 2.0;

      for index in 0..6 {
         let direct = orb.letter_pulse(0.7, period, index);
         let shifted = orb.letter_pulse(0.7 - 0.1 * index as f64, period, 0);

         assert!(
            (direct.0 - shifted.0).abs() < 0.001 && (direct.1 - shifted.1).abs() < 0.001,
            "letter {index} is letter 0 one delay later: {direct:?} vs {shifted:?}"
         );
      }

      let (idle, scale) = orb.letter_pulse(0.0, period, 0);
      assert!(
         (idle - LETTER_IDLE_OPACITY).abs() < 0.001 && scale == 1.0,
         "the loop starts idle: {idle} at {scale}"
      );

      let (peak, peak_scale) = orb.letter_pulse(period * LETTER_PEAK_AT as f64, period, 0);
      assert!(
         (peak - LETTER_PEAK_OPACITY).abs() < 0.01 && peak_scale > 1.1,
         "the peak is opaque and enlarged: {peak} at {peak_scale}"
      );

      let (late, _) = orb.letter_pulse(period * 0.9, period, 0);
      assert!(
         late < LETTER_SETTLE_OPACITY,
         "a letter idles out in the last 60 % of its loop: {late}"
      );
   }

   /// The square `orb` asks for inside a fresh context.
   fn requested(orb: Orb) -> f32 {
      let ctx = Context::default();
      let input = RawInput {
         screen_rect: Some(Rect::from_min_size(
            Pos2::ZERO,
            vec2(400.0, 400.0),
         )),
         ..Default::default()
      };
      let mut size = 0.0;
      let mut out = ctx.run_ui(input, |ui| size = orb.requested_size(ui));
      out.textures_delta.clear();

      size
   }

   #[test]
   fn a_longer_label_grows_the_ball() {
      let bare = requested(Orb::new());
      let short = requested(Orb::new().text("Hi"));
      let long = requested(Orb::new().text("Saving the changes"));
      let explicit = requested(Orb::new().text("Saving the changes").size(64.0));

      assert!(bare > 0.0, "a bare orb still has a size");
      assert!(
         short >= bare && long > short,
         "a longer label grows the ball: {bare} / {short} / {long}"
      );
      assert_eq!(explicit, 64.0, ".size() wins over the label");
   }

   #[test]
   fn a_neutral_override_paints_a_neutral_ball() {
      let silver = Orb::shells(Color32::WHITE, 0.0);
      let accent = Orb::shells(accent(), 0.0);

      for color in [silver.deep.color, silver.lobe.color, silver.body] {
         let hsla = Hsla::from_color32(color);
         assert!(
            hsla.s < 1.0,
            "a white override leaves no hue behind: {color:?} is {} % saturated",
            hsla.s
         );
      }
      assert!(
         Hsla::from_color32(accent.lobe.color).s > 90.0,
         "a chromatic accent keeps the ported saturation"
      );
   }

   #[test]
   fn a_visible_loader_requests_a_repaint() {
      let ctx = Context::default();
      let input = RawInput {
         screen_rect: Some(Rect::from_min_size(
            Pos2::ZERO,
            vec2(400.0, 400.0),
         )),
         time: Some(0.2),
         ..Default::default()
      };
      let mut repaint = false;
      let mut out = ctx.run_ui(input, |ui| {
         Orb::new().paint_at(ui, orb_rect());
         repaint = ui.ctx().has_requested_repaint();
      });
      out.textures_delta.clear();

      assert!(repaint, "a running loop is never settled");
   }
}
