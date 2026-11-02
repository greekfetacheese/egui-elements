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

use super::toggle::{DEFAULT_SIZE, PlateGlyph, PlateShape, Row, ToggleVisuals};
pub use super::toggle::{ToggleShadow as CheckboxShadow, ToggleVisuals as CheckboxVisuals};

use egui::{Response, Role, Ui, Widget, WidgetInfo, WidgetText};

use crate::motion::Motion;
use crate::theme::Theme;

/// Corner radius as a fraction of the box size.
const RADIUS_RATIO: f32 = 0.17;

/// A checkbox with an animated fill, tick and shadow.
///
/// Reads [`ThemeColors`](crate::theme::ThemeColors) from the installed [`Theme`] unless you pass
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
      let visuals = visuals.unwrap_or_else(|| ToggleVisuals::from_theme(colors));
      let shadow = shadow.unwrap_or(visuals.shadow);
      let enabled = ui.is_enabled();
      let label = text.as_ref().map(|text| text.text().to_owned()).unwrap_or_default();

      let mut row = Row::new(ui, text, size);
      if row.response.clicked() {
         *checked = !*checked;
         row.response.mark_changed();
      }
      row.response
         .widget_info(|| WidgetInfo::selected(Role::CheckBox, enabled, *checked, label.clone()));

      row.paint(
         *checked,
         PlateShape::Rounded {
            ratio: RADIUS_RATIO,
         },
         PlateGlyph::Tick,
         &visuals,
         shadow,
         motion,
      )
   }
}

#[cfg(test)]
mod tests {
   use super::*;
   use crate::theme::{Theme, ThemeKind};
   use egui::{
      Context, Event, FullOutput, Key, Modifiers, PointerButton, Pos2, RawInput, Rect, pos2, vec2,
   };

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
