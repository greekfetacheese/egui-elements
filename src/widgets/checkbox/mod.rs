//! A themed checkbox whose fill, border, tick and shadow animate on toggle.
//!
//! Geometry follows the reference checkbox — a 16 px box with a 0.17 corner radius, a 1 pt border and a
//! 3-point tick with round ends — expressed as **ratios of the box**, so one [`Checkbox::size`] scales
//! everything downstream of it.
//!
//! Every colour derives from one palette: the checked fill is [`ThemeColors::accent`], the tick is
//! [`ThemeColors::bg`], and the idle box is [`ThemeColors::border`] over a transparent fill, so the widget
//! reads correctly on `bg`, `frame1` and `window` without a per-surface palette.
//! [`Checkbox::visuals`] replaces them for one widget; start from [`CheckboxVisuals::from_theme`] to keep the
//! rest.
//!
//! The toggle is one clock: fill, border and the shadow's alpha all follow `t`, and the tick pops in with a
//! slight overshoot. The duration is the installed style's `animation_time`, like every other widget;
//! [`Checkbox::motion`] overrides it.
//!
//! ```no_run
//! # use egui::__run_test_ui;
//! # use egui_elements::widgets::Checkbox;
//! # __run_test_ui(|ui| {
//! let mut two_factor = true;
//! if ui.add(Checkbox::new(&mut two_factor, "SMS verification")).changed() {
//!     // …
//! }
//! # });
//! ```
//!
//! [`ThemeColors::accent`]: crate::theme::ThemeColors::accent
//! [`ThemeColors::bg`]: crate::theme::ThemeColors::bg
//! [`ThemeColors::border`]: crate::theme::ThemeColors::border

mod glyph;

use egui::{
   Color32, CornerRadius, Rect, Response, Role, Sense, Shadow, Stroke, TextStyle, Ui, Vec2, Widget,
   WidgetInfo, WidgetText, pos2, vec2,
};

use crate::motion::{InteractT, Lerp, Motion, animate_from};
use crate::theme::{Theme, ThemeColors};
use crate::visuals::paint_shape;
use glyph::tick_shapes;

/// Default box size in points. The reference is 16 at 1×, up from egui's own `icon_width` of 14.
const DEFAULT_SIZE: f32 = 18.0;
/// Corner radius as a fraction of the box size.
const RADIUS_RATIO: f32 = 0.17;
/// Border width as a fraction of the box size, floored at one point.
const BORDER_RATIO: f32 = 0.0625;
/// Box scale while the pointer is down.
const PRESS_SCALE: f32 = 0.93;
/// Tick scale at `t = 0`; it pops to 1.0 and overshoots on the way.
const TICK_POP_FROM: f32 = 0.35;
/// How fast the tick's alpha reaches full, relative to `t`.
const TICK_FADE: f32 = 2.2;
/// Shadow alpha floor while unchecked; `1.0` at rest is the configured strength.
const SHADOW_T_FLOOR: f32 = 0.30;
/// Alpha multiplier applied to a disabled checkbox.
const DISABLED_ALPHA: f32 = 0.5;

/// The lift under a checkbox, in ratios of the box so it scales with [`Checkbox::size`].
///
/// [`CheckboxShadow::derived`] is the default (the checked fill at 42 %, under a 0.34·size blur, offset
/// 0.055·size down); [`CheckboxShadow::new`] keeps that geometry in a colour of your own;
/// [`CheckboxShadow::NONE`] turns the lift off.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CheckboxShadow {
   /// Shadow colour. `None` = the checked fill, so overriding the box colour repaints the lift with it.
   pub color: Option<Color32>,
   /// Vertical offset, as a fraction of the box size.
   pub offset_y: f32,
   /// Blur reach, as a fraction of the box size.
   pub blur: f32,
   /// Spread, as a fraction of the box size.
   pub spread: f32,
   /// Peak alpha, `0.0..=1.0`.
   pub strength: f32,
}

impl CheckboxShadow {
   /// No lift. The box still feathers its own edges.
   pub const NONE: Self = Self {
      color: None,
      offset_y: 0.0,
      blur: 0.0,
      spread: 0.0,
      strength: 0.0,
   };

   /// The default lift: the checked fill at 42 %, offset 0.055·size, blur 0.34·size.
   pub fn derived() -> Self {
      Self {
         color: None,
         offset_y: 0.055,
         blur: 0.34,
         spread: 0.0,
         strength: 0.42,
      }
   }

   /// The derived lift in `color`.
   pub fn new(color: Color32) -> Self {
      Self {
         color: Some(color),
         ..Self::derived()
      }
   }

   /// Resolve the ratios against `size`. `t` (the toggle progress) lifts the alpha from
   /// `0.30 × strength` while unchecked to `strength` when checked, so the shadow blooms with the fill.
   pub fn to_shadow(self, size: f32, checked_fill: Color32, t: f32) -> Shadow {
      if self.strength <= 0.0 {
         return Shadow::NONE;
      }
      let alpha = self.strength * (SHADOW_T_FLOOR + (1.0 - SHADOW_T_FLOOR) * t);
      let color = self.color.unwrap_or(checked_fill).gamma_multiply(alpha);
      Shadow {
         offset: [0, (self.offset_y * size).round() as i8],
         blur: (self.blur * size).round().clamp(0.0, 255.0) as u8,
         spread: (self.spread * size).round().clamp(0.0, 255.0) as u8,
         color,
      }
   }
}

impl Default for CheckboxShadow {
   fn default() -> Self {
      Self::NONE
   }
}

/// Per-widget colours for [`Checkbox`].
///
/// [`Default`] is inert (transparent, no border, no shadow). The widget paints with
/// [`CheckboxVisuals::from_theme`] unless you pass your own set — start there to override one slot.
///
/// ```no_run
/// # use egui::{__run_test_ui, Color32};
/// # use egui_elements::theme::{Theme, ThemeKind};
/// # use egui_elements::widgets::{Checkbox, CheckboxVisuals};
/// # let colors = Theme::new(ThemeKind::TokyoNight).colors;
/// # __run_test_ui(|ui| {
/// # let mut on = false;
/// let visuals = CheckboxVisuals {
///    bg_checked: Color32::WHITE,
///    ..CheckboxVisuals::from_theme(colors)
/// };
/// ui.add(Checkbox::new(&mut on, "Lit").visuals(visuals));
/// # });
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CheckboxVisuals {
   /// Idle (unchecked) fill. Default `TRANSPARENT`, so the box picks up its surface.
   pub bg: Color32,
   /// Fill while hovered.
   pub bg_hover: Color32,
   /// Fill when checked.
   pub bg_checked: Color32,
   /// Idle border.
   pub border: Color32,
   /// Border while hovered.
   pub border_hover: Color32,
   /// Border when checked.
   pub border_checked: Color32,
   /// Tick colour.
   pub tick: Color32,
   /// The lift under the box.
   pub shadow: CheckboxShadow,
}

impl CheckboxVisuals {
   /// The defaults [`Checkbox`] derives from an installed palette.
   ///
   /// `bg` is transparent so the box sits on any surface; the checked fill is the accent and the tick is the
   /// page colour, which is the crate's rule for text on a chromatic fill.
   pub fn from_theme(colors: ThemeColors) -> Self {
      Self {
         bg: Color32::TRANSPARENT,
         bg_hover: colors.hover,
         bg_checked: colors.accent,
         border: colors.border,
         border_hover: colors.border.lerp_to_gamma(colors.accent, 0.5),
         border_checked: colors.accent,
         tick: colors.bg,
         shadow: CheckboxShadow::derived(),
      }
   }
}

/// A checkbox with an animated fill, tick and shadow.
///
/// Reads [`ThemeColors`] from the installed [`Theme`] unless you pass
/// [`.visuals(...)`](Self::visuals). Clicking anywhere in the row — box or label — toggles it; a click
/// reports [`Response::changed`].
///
/// ```
/// # use egui::__run_test_ui;
/// # use egui_elements::widgets::Checkbox;
/// # __run_test_ui(|ui| {
/// let mut triage = false;
/// ui.add(Checkbox::without_text(&mut triage));
/// # });
/// ```
#[must_use = "Checkboxes should be shown on a ui, e.g. `ui.add(Checkbox::new(&mut flag, \"Label\"))`"]
pub struct Checkbox<'a> {
   checked: &'a mut bool,
   text: Option<WidgetText>,
   size: Option<f32>,
   visuals: Option<CheckboxVisuals>,
   shadow: Option<CheckboxShadow>,
   motion: Option<Motion>,
}

impl<'a> Checkbox<'a> {
   /// A checkbox with a label.
   pub fn new(checked: &'a mut bool, text: impl Into<WidgetText>) -> Self {
      Self {
         checked,
         text: Some(text.into()),
         size: None,
         visuals: None,
         shadow: None,
         motion: None,
      }
   }

   /// A checkbox without a label, for a table cell or a toolbar.
   pub fn without_text(checked: &'a mut bool) -> Self {
      Self {
         text: None,
         ..Self::new(checked, "")
      }
   }

   /// Side of the box in points. Default `18.0`; every other measurement is a ratio of it.
   pub fn size(mut self, size: f32) -> Self {
      self.size = Some(size);
      self
   }

   /// Replace the palette-derived colours.
   pub fn visuals(mut self, visuals: CheckboxVisuals) -> Self {
      self.visuals = Some(visuals);
      self
   }

   /// Replace the palette-derived shadow; [`CheckboxShadow::NONE`] turns the lift off.
   pub fn shadow(mut self, shadow: CheckboxShadow) -> Self {
      self.shadow = Some(shadow);
      self
   }

   /// How the toggle animates. Defaults to [`Motion::from_ui`] (the style's `animation_time`);
   /// [`Motion::instant`] makes it snap.
   pub fn motion(mut self, motion: Motion) -> Self {
      self.motion = Some(motion);
      self
   }
}

impl Widget for Checkbox<'_> {
   fn ui(self, ui: &mut Ui) -> Response {
      let Checkbox {
         checked,
         text,
         size,
         visuals,
         shadow,
         motion,
      } = self;

      let size = size.unwrap_or(DEFAULT_SIZE);
      let colors = Theme::colors_from_ctx(ui.ctx());
      let visuals = visuals.unwrap_or_else(|| CheckboxVisuals::from_theme(colors));
      let shadow = shadow.unwrap_or(visuals.shadow);
      let enabled = ui.is_enabled();
      let gap = ui.spacing().icon_spacing;

      let label = text.as_ref().map(|text| text.text().to_owned()).unwrap_or_default();
      let galley = text.map(|text| {
         let available = (ui.available_width() - size - gap).max(size);
         text.into_galley(
            ui,
            Some(egui::TextWrapMode::Wrap),
            available,
            TextStyle::Body,
         )
      });
      let text_size = galley.as_ref().map(|galley| galley.size()).unwrap_or(Vec2::ZERO);

      let row = vec2(
         size
            + if galley.is_some() {
               gap + text_size.x
            } else {
               0.0
            },
         size.max(text_size.y).max(ui.spacing().interact_size.y),
      );
      let (rect, mut response) = ui.allocate_exact_size(row, Sense::click());

      if response.clicked() {
         *checked = !*checked;
         response.mark_changed();
      }
      response
         .widget_info(|| WidgetInfo::selected(Role::CheckBox, enabled, *checked, label.clone()));

      if ui.is_rect_visible(rect) {
         let motion = motion.unwrap_or_else(|| Motion::from_ui(ui));
         let target = if *checked { 1.0 } else { 0.0 };
         // Seeded at the target: a checkbox that starts checked must be checked, not animate itself on.
         let t = animate_from(
            ui,
            response.id.with("check"),
            target,
            target,
            motion,
         );
         let across = InteractT::of(
            ui,
            response.id.with("face"),
            response.hovered(),
            response.is_pointer_button_down_on(),
            Motion::from_ui(ui),
         );

         let scale = 1.0 - (1.0 - PRESS_SCALE) * across.press;
         let box_rect = Rect::from_center_size(
            pos2(rect.left() + size * 0.5, rect.center().y),
            Vec2::splat(size * scale),
         );

         let shade = |color: Color32| {
            if enabled {
               color
            } else {
               color.gamma_multiply(DISABLED_ALPHA)
            }
         };

         // Each state's idle shade is blended into the checked colour by `t`, so at `t == 1.0` every state
         // collapses onto the accent and the pressed feedback is the scale, not a colour.
         let fill = across.blend(
            shade(visuals.bg.lerp(visuals.bg_checked, t)),
            shade(visuals.bg_hover.lerp(visuals.bg_checked, t)),
            shade(visuals.bg_hover.lerp(visuals.bg_checked, t)),
         );
         let border = across.blend(
            shade(visuals.border.lerp(visuals.border_checked, t)),
            shade(visuals.border_hover.lerp(visuals.border_checked, t)),
            shade(visuals.border_hover.lerp(visuals.border_checked, t)),
         );

         let stroke = Stroke::new((size * BORDER_RATIO).max(1.0), border);
         let radius = CornerRadius::same((size * RADIUS_RATIO).round() as u8);
         let lift = if enabled {
            shadow
         } else {
            CheckboxShadow::NONE
         };

         ui.painter().add(paint_shape(
            box_rect,
            fill,
            stroke,
            radius,
            lift.to_shadow(size * scale, shade(visuals.bg_checked), t),
         ));

         if t > 0.0 {
            let color = shade(visuals.tick).gamma_multiply((t * TICK_FADE).min(1.0));
            let pop = TICK_POP_FROM + (1.0 - TICK_POP_FROM) * egui::emath::easing::back_out(t);
            let tick_rect = Rect::from_center_size(box_rect.center(), box_rect.size() * pop);
            for shape in tick_shapes(
               tick_rect,
               color,
               glyph::stroke_width(size * scale) * pop,
            ) {
               ui.painter().add(shape);
            }
         }

         if let Some(galley) = galley {
            let pos = pos2(
               box_rect.right() + gap,
               rect.center().y - galley.size().y * 0.5,
            );
            let color = shade(colors.text);
            ui.painter().galley(pos, galley, color);
         }
      }

      response
   }
}

#[cfg(test)]
mod tests {
   use super::*;
   use crate::theme::{Theme, ThemeKind};
   use egui::{Context, Event, FullOutput, Key, Modifiers, PointerButton, Pos2, RawInput, pos2};

   fn themed_ctx() -> Context {
      let ctx = Context::default();
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&ctx);
      ctx
   }

   /// One frame at an explicit `input.time` (no sleeps, no wall clock).
   fn frame(
      ctx: &Context,
      time: f64,
      events: Vec<Event>,
      mut body: impl FnMut(&mut Ui),
   ) -> FullOutput {
      let input = RawInput {
         events,
         time: Some(time),
         screen_rect: Some(Rect::from_min_size(
            pos2(0.0, 0.0),
            vec2(400.0, 300.0),
         )),
         ..Default::default()
      };
      let mut out = ctx.run_ui(input, |ui| body(ui));
      out.textures_delta.clear();
      out
   }

   fn press(pos: Pos2) -> Event {
      Event::PointerButton {
         pos,
         button: PointerButton::Primary,
         pressed: true,
         modifiers: Modifiers::NONE,
      }
   }

   fn release(pos: Pos2) -> Event {
      Event::PointerButton {
         pos,
         button: PointerButton::Primary,
         pressed: false,
         modifiers: Modifiers::NONE,
      }
   }

   #[test]
   fn settled_checkbox_requests_no_repaint() {
      let ctx = themed_ctx();
      let mut flag = true;
      // Two warm-up frames absorb egui's own first-frame repaint request.
      for _ in 0..2 {
         frame(&ctx, 10.0, vec![], |ui| {
            ui.add(Checkbox::without_text(&mut flag));
         });
      }
      let out = frame(&ctx, 10.0, vec![], |ui| {
         ui.add(Checkbox::without_text(&mut flag));
      });
      let delay = out.viewport_output[&egui::ViewportId::ROOT].repaint_delay;
      assert_eq!(
         delay,
         std::time::Duration::MAX,
         "a settled checkbox requested a repaint"
      );
   }

   #[test]
   fn label_click_toggles_and_reports_changed() {
      let ctx = themed_ctx();
      let mut flag = false;
      let mut rect = Rect::ZERO;
      frame(&ctx, 0.0, vec![], |ui| {
         rect = ui.add(Checkbox::new(&mut flag, "SMS verification")).rect;
      });
      let at = pos2(rect.right() - 4.0, rect.center().y);
      frame(&ctx, 0.1, vec![press(at)], |ui| {
         ui.add(Checkbox::new(&mut flag, "SMS verification"));
      });
      let mut changed = false;
      frame(&ctx, 0.2, vec![release(at)], |ui| {
         changed = ui.add(Checkbox::new(&mut flag, "SMS verification")).changed();
      });
      assert!(flag, "a click on the label must toggle the bool");
      assert!(
         changed,
         "the toggling frame must report `changed`"
      );

      let mut changed_after = true;
      frame(&ctx, 0.3, vec![], |ui| {
         changed_after = ui.add(Checkbox::new(&mut flag, "SMS verification")).changed();
      });
      assert!(!changed_after, "`changed` must not repeat");
   }

   #[test]
   fn space_toggles_the_focused_box() {
      let ctx = themed_ctx();
      let mut flag = false;
      let mut id = None;
      frame(&ctx, 0.0, vec![], |ui| {
         id = Some(ui.add(Checkbox::without_text(&mut flag)).id);
      });
      ctx.memory_mut(|memory| memory.request_focus(id.expect("an id")));
      frame(
         &ctx,
         0.1,
         vec![Event::Key {
            key: Key::Space,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
         }],
         |ui| {
            ui.add(Checkbox::without_text(&mut flag));
         },
      );
      assert!(flag, "Space on a focused checkbox must toggle it");
   }

   #[test]
   fn disabled_checkbox_does_not_toggle() {
      let ctx = themed_ctx();
      let mut flag = false;
      let mut center = pos2(0.0, 0.0);
      frame(&ctx, 0.0, vec![], |ui| {
         center = ui.add_enabled(false, Checkbox::without_text(&mut flag)).rect.center();
      });
      frame(&ctx, 0.1, vec![press(center)], |ui| {
         ui.add_enabled(false, Checkbox::without_text(&mut flag));
      });
      frame(&ctx, 0.2, vec![release(center)], |ui| {
         ui.add_enabled(false, Checkbox::without_text(&mut flag));
      });
      assert!(!flag, "a disabled checkbox must not toggle");
   }
}
