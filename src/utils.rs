//! Helpers: theme switcher, HSLA colors, and interactive frames.

use crate::theme::*;
use crate::visuals::FrameVisuals;
use crate::widgets::{ComboBox, Label};
use egui::{Color32, Frame, Response, RichText, Sense, Ui};

/// Should work for most images that are shown on a very dark background
pub const TINT_1: Color32 = Color32::from_rgba_premultiplied(216, 216, 216, 255);

/// HSLA color used by the theme editor (hue 0..=360, s/l 0..=100, a 0..=1).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct Hsla {
   /// Hue 0.0..=360.0
   pub h: f32,
   /// Saturation 0.0..=100.0
   pub s: f32,
   /// Lightness 0.0..=100.0
   pub l: f32,
   /// Alpha 0.0..=1.0
   pub a: f32,
}

impl Hsla {
   /// Convert a premultiplied [`Color32`] to unpremultiplied HSLA.
   pub fn from_color32(c: Color32) -> Self {
      let (r, g, b) = unpremultiply_srgb(c);
      let (h, s, l) = srgb_to_hsl(r, g, b);
      Hsla {
         h,
         s: s * 100.0,
         l: l * 100.0,
         a: c.a() as f32 / 255.0,
      }
   }

   /// Parse `#RGB`, `#RRGGBB`, or `#RRGGBBAA`.
   pub fn from_hex(hex: &str) -> Option<Self> {
      match Color32::from_hex(hex) {
         Ok(c) => Some(Self::from_color32(c)),
         Err(_) => None,
      }
   }

   /// Convert back to a premultiplied [`Color32`].
   pub fn to_color32(&self) -> Color32 {
      let (r, g, b, a) = self.to_rgba_components();
      Color32::from_rgba_unmultiplied(r, g, b, a)
   }

   /// Unpremultiplied `(r, g, b, a)` in 0..=255.
   pub fn to_rgba_components(&self) -> (u8, u8, u8, u8) {
      let (r, g, b) = hsl_to_srgb(self.h, self.s / 100.0, self.l / 100.0);
      (
         channel_to_u8(r),
         channel_to_u8(g),
         channel_to_u8(b),
         channel_to_u8(self.a),
      )
   }

   /// Walk lightness in 5% steps, including `self` as the first entry.
   pub fn shades(&self, num_shades: usize, direction: ShadeDirection) -> Vec<Color32> {
      let mut shades = Vec::new();
      let step = if direction == ShadeDirection::Lighter {
         5.0
      } else {
         -5.0
      };
      let mut current = *self;
      for _ in 0..num_shades {
         shades.push(current.to_color32());
         current.l = (current.l as f32 + step).clamp(0.0, 100.0);
      }
      shades
   }
}

/// Direction for [`Hsla::shades`].
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ShadeDirection {
   /// Increase lightness.
   Lighter,
   /// Decrease lightness.
   Darker,
}

/// `Color32` stores premultiplied sRGB. HSL is computed in unpremultiplied space.
fn unpremultiply_srgb(c: Color32) -> (f32, f32, f32) {
   let a = c.a();
   if a == 0 {
      (0.0, 0.0, 0.0)
   } else if a == 255 {
      (
         c.r() as f32 / 255.0,
         c.g() as f32 / 255.0,
         c.b() as f32 / 255.0,
      )
   } else {
      let a = a as f32;
      (
         c.r() as f32 / a,
         c.g() as f32 / a,
         c.b() as f32 / a,
      )
   }
}

/// Gamma-encoded sRGB → HSL (CSS / Wikipedia). `s` and `l` in 0..=1, `h` in 0..360.
fn srgb_to_hsl(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
   let max = r.max(g).max(b);
   let min = r.min(g).min(b);
   let l = (max + min) * 0.5;
   let d = max - min;

   if d == 0.0 {
      return (0.0, 0.0, l);
   }

   let s = d / (1.0 - (2.0 * l - 1.0).abs());
   let h = if max == r {
      (g - b) / d
   } else if max == g {
      (b - r) / d + 2.0
   } else {
      (r - g) / d + 4.0
   };
   (h.rem_euclid(6.0) * 60.0, s, l)
}

/// HSL → gamma-encoded sRGB. `h` in degrees, `s` and `l` in 0..=1.
fn hsl_to_srgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
   let s = s.clamp(0.0, 1.0);
   let l = l.clamp(0.0, 1.0);
   let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
   let hp = h.rem_euclid(360.0) / 60.0;
   let x = c * (1.0 - (hp.rem_euclid(2.0) - 1.0).abs());
   let m = l - c * 0.5;
   let (r, g, b) = match hp as u8 {
      0 => (c, x, 0.0),
      1 => (x, c, 0.0),
      2 => (0.0, c, x),
      3 => (0.0, x, c),
      4 => (x, 0.0, c),
      _ => (c, 0.0, x),
   };
   (r + m, g + m, b + m)
}

fn channel_to_u8(channel: f32) -> u8 {
   (channel.clamp(0.0, 1.0) * 255.0) as u8
}

/// WCAG 2.2 AA contrast for normal-size text.
pub const CONTRAST_AA_TEXT: f32 = 4.5;

/// WCAG 2.2 AA contrast for large text and UI component boundaries.
pub const CONTRAST_AA_UI: f32 = 3.0;

/// Mix two colours in unpremultiplied sRGB; `t = 0` returns `a`, `t = 1` returns `b`.
///
/// Channels are mixed gamma-encoded, which is what palette tinting wants: halfway between a role
/// colour and the canvas reads as halfway. Alpha is interpolated too.
pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
   let t = t.clamp(0.0, 1.0);
   let (ar, ag, ab) = unpremultiply_srgb(a);
   let (br, bg, bb) = unpremultiply_srgb(b);
   let alpha = a.a() as f32 / 255.0;
   let other_alpha = b.a() as f32 / 255.0;

   let channel = |x: f32, y: f32| channel_to_u8(x + (y - x) * t);
   Color32::from_rgba_unmultiplied(
      channel(ar, br),
      channel(ag, bg),
      channel(ab, bb),
      channel(alpha, other_alpha),
   )
}

/// `color` at `alpha` (0..=255), preserving its hue.
///
/// The alpha is applied unmultiplied, so fading a glow never dims its RGB.
pub fn with_alpha(color: Color32, alpha: u8) -> Color32 {
   let [red, green, blue, _] = color.to_srgba_unmultiplied();
   Color32::from_rgba_unmultiplied(red, green, blue, alpha)
}

/// `color` as `#rrggbb` (the alpha channel is dropped).
pub fn hex(color: Color32) -> String {
   let [red, green, blue, _] = color.to_srgba_unmultiplied();
   format!("#{red:02x}{green:02x}{blue:02x}")
}

/// WCAG 2.2 contrast ratio between two opaque colours, from 1.0 to 21.0.
///
/// Use [`CONTRAST_AA_TEXT`] for body text and [`CONTRAST_AA_UI`] for large text and component
/// boundaries. Alpha is ignored: composite a translucent colour over its surface first.
pub fn contrast_ratio(a: Color32, b: Color32) -> f32 {
   let (first, second) = (relative_luminance(a), relative_luminance(b));
   let (lighter, darker) = if first >= second {
      (first, second)
   } else {
      (second, first)
   };
   (lighter + 0.05) / (darker + 0.05)
}

fn relative_luminance(c: Color32) -> f32 {
   let (r, g, b) = unpremultiply_srgb(c);
   0.2126 * linearize(r) + 0.7152 * linearize(g) + 0.0722 * linearize(b)
}

fn linearize(s: f32) -> f32 {
   if s <= 0.04045 {
      s / 12.92
   } else {
      ((s + 0.055) / 1.055).powf(2.4)
   }
}

/// `true` if any frame-related palette slot changed.
pub fn frame_palette_changed(old: &ThemeColors, new: &ThemeColors) -> bool {
   old.title_bar != new.title_bar
      || old.bg != new.bg
      || old.widget_bg != new.widget_bg
      || old.hover != new.hover
      || old.highlight != new.highlight
      || old.border != new.border
}

/// Write `new` into `slot` only if it still equals `old`.
pub fn remap_if_eq(slot: &mut Color32, old: Color32, new: Color32) {
   if *slot == old {
      *slot = new;
   }
}

/// Remap fill, stroke, and shadow colors on an [`egui::Frame`] when they
/// still match the previous palette.
pub fn remap_frame(
   frame: &mut Frame,
   old_fill: Color32,
   new_fill: Color32,
   old_border: Color32,
   new_border: Color32,
) {
   remap_if_eq(&mut frame.fill, old_fill, new_fill);
   remap_if_eq(&mut frame.stroke.color, old_border, new_border);
   remap_if_eq(&mut frame.shadow.color, old_border, new_border);
}

/// Remap hover/click fills on [`FrameVisuals`] when they still match the
/// previous palette.
pub fn remap_frame_visuals(
   visuals: &mut FrameVisuals,
   old_hover: Color32,
   new_hover: Color32,
   old_click: Color32,
   new_click: Color32,
   old_highlight: Color32,
   new_highlight: Color32,
) {
   visuals.bg_hover.remap_color(old_hover, new_hover);
   visuals.bg_click.remap_color(old_click, new_click);
   visuals.bg_selected.remap_color(old_highlight, new_highlight);
   remap_if_eq(
      &mut visuals.border_hover.color,
      old_highlight,
      new_highlight,
   );
   remap_if_eq(
      &mut visuals.border_click.color,
      old_highlight,
      new_highlight,
   );
}

/// Button width and popup floor for the switcher: the longest name plus chrome.
///
/// The current kind is measured too: a [`ThemeKind::Custom`] name is not in
/// [`ThemeKind::to_vec`] and may be longer than every built-in.
fn switcher_width(ui: &Ui, theme: &Theme) -> (f32, f32) {
   let font = egui::FontId::proportional(theme.typography.normal);
   let mut names: Vec<String> =
      ThemeKind::to_vec().iter().map(|kind| kind.to_str().to_owned()).collect();
   names.push(theme.kind.to_str().to_owned());

   let widest = names
      .iter()
      .map(|name| {
         ui.painter()
            .layout_no_wrap(name.clone(), font.clone(), theme.colors.text)
            .size()
            .x
      })
      .fold(0.0_f32, f32::max);

   (widest, (widest + 56.0).max(200.0))
}

/// Show a ComboBox to change the theme
///
/// The button and its popup are sized to the longest palette name, so names like
/// "McLaren 650GTS" and "Shade Sanctuary" are not truncated.
///
/// Only built-in kinds are offered. A custom kind's definition is the app's
/// serialized [`Theme`], so it shows in the button but cannot be rebuilt from
/// the popup; reinstall the app's own theme instead.
///
/// Returns the new theme if we select one, the new theme is installed to the [egui::Context]
pub fn theme_switcher(current_theme: &Theme, ui: &mut Ui) -> Option<Theme> {
   let mut new_theme_opt = None;

   let text = RichText::new(current_theme.kind.to_str()).size(current_theme.typography.normal);
   let current = Label::new(text, None);
   let (widest, width) = switcher_width(ui, current_theme);

   ComboBox::new("Theme switcher", current).width(width).show_ui(ui, |ui| {
      ui.set_min_width(widest);
      ui.spacing_mut().item_spacing.y = 12.0;

      for kind in ThemeKind::to_vec() {
         let label = RichText::new(kind.to_str()).size(current_theme.typography.normal);
         let label = Label::new(label, None)
            .interactive(true)
            .fill_width(true)
            .sense(Sense::click())
            .expand(Some(4.0));

         if ui.add(label).clicked() {
            let mut new_theme = Theme::new(kind);
            new_theme_opt = Some(new_theme.clone());
            new_theme.install(ui.ctx());
         }
      }
   });
   new_theme_opt
}

/// Paint `frame` with hover/click fills from `visuals`.
///
/// The closure runs inside the frame. Returns the inner content response.
pub fn frame(
   frame: &mut Frame,
   visuals: FrameVisuals,
   ui: &mut Ui,
   add_contents: impl FnOnce(&mut Ui),
) -> Response {
   let mut frame = frame.begin(ui);
   let res = frame.content_ui.scope(|ui| add_contents(ui));

   if res.response.interact(Sense::click()).clicked() {
      frame.frame = frame.frame.fill(visuals.bg_click.to_solid());
      frame.frame = frame.frame.stroke(visuals.border_click);
   } else if res.response.hovered() {
      frame.frame = frame.frame.fill(visuals.bg_hover.to_solid());
      frame.frame = frame.frame.stroke(visuals.border_hover);
   }

   frame.end(ui);
   res.response
}

#[cfg(test)]
mod tests {
   use super::*;

   fn assert_near_color(a: Color32, b: Color32) {
      let dr = (a.r() as i16 - b.r() as i16).abs();
      let dg = (a.g() as i16 - b.g() as i16).abs();
      let db = (a.b() as i16 - b.b() as i16).abs();
      assert!(
         dr <= 1 && dg <= 1 && db <= 1 && a.a() == b.a(),
         "roundtrip {a:?} -> {b:?}"
      );
   }

   #[test]
   fn color32_hsla_roundtrip() {
      let samples = [
         Color32::from_rgba_unmultiplied(0, 0, 0, 255),
         Color32::from_rgba_unmultiplied(255, 255, 255, 255),
         Color32::from_rgba_unmultiplied(255, 0, 0, 255),
         Color32::from_rgba_unmultiplied(0, 255, 0, 255),
         Color32::from_rgba_unmultiplied(0, 0, 255, 255),
         Color32::from_rgba_unmultiplied(128, 64, 192, 128),
         Color32::from_rgba_unmultiplied(26, 27, 38, 255),
         Color32::from_rgba_unmultiplied(122, 162, 247, 200),
      ];
      for c in samples {
         assert_near_color(c, Hsla::from_color32(c).to_color32());
      }
   }

   #[test]
   fn from_hex_parses_and_preserves_alpha() {
      let hsla = Hsla::from_hex("#7aa2f7").unwrap();
      let (r, g, b, a) = hsla.to_rgba_components();
      assert_eq!((r, g, b, a), (122, 162, 247, 255));

      let hsla = Hsla::from_hex("#7aa2f780").unwrap();
      assert!((hsla.a - 128.0 / 255.0).abs() < 1e-5);
      assert_eq!(hsla.to_color32().a(), 128);
   }

   #[test]
   fn mix_runs_between_the_endpoints() {
      let red = Color32::from_rgb(255, 0, 0);
      let blue = Color32::from_rgb(0, 0, 255);
      assert_eq!(mix(red, blue, 0.0), red);
      assert_eq!(mix(red, blue, 1.0), blue);
      assert_eq!(
         mix(red, blue, 0.5),
         Color32::from_rgb(127, 0, 127)
      );
   }

   #[test]
   fn mix_interpolates_alpha() {
      let opaque = Color32::from_rgba_unmultiplied(255, 0, 0, 255);
      let clear = Color32::from_rgba_unmultiplied(255, 0, 0, 0);
      assert!((mix(opaque, clear, 0.5).a() as i16 - 127).abs() <= 1);
   }

   #[test]
   fn contrast_ratio_matches_wcag_extremes() {
      assert!((contrast_ratio(Color32::BLACK, Color32::WHITE) - 21.0).abs() < 0.01);
      assert!((contrast_ratio(Color32::WHITE, Color32::BLACK) - 21.0).abs() < 0.01);
      assert!((contrast_ratio(Color32::WHITE, Color32::WHITE) - 1.0).abs() < 0.001);
   }

   #[test]
   fn the_theme_switcher_fits_its_longest_name() {
      let ctx = egui::Context::default();
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&ctx);

      let mut widest = 0.0;
      let mut width = 0.0;
      let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
         (widest, width) = switcher_width(ui, &theme);
      });
      out.textures_delta.clear();

      assert!(
         widest > 40.0,
         "the longest palette name measured {widest}"
      );
      assert!(
         width >= widest + 40.0,
         "button {width} vs text {widest}"
      );
   }

   #[test]
   fn with_alpha_replaces_only_the_alpha() {
      let color = with_alpha(Color32::from_rgb(10, 20, 30), 128);
      assert_eq!(color.to_srgba_unmultiplied(), [10, 20, 30, 128]);
   }

   #[test]
   fn hex_formats_the_rgb_channels() {
      assert_eq!(
         hex(Color32::from_rgb(0x12, 0xab, 0xff)),
         "#12abff"
      );
      assert_eq!(
         hex(Color32::from_rgba_unmultiplied(
            0x12, 0xab, 0xff, 128
         )),
         "#12abff",
         "alpha is dropped"
      );
   }
}
