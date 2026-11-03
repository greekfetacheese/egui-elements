//! A themed switch: the checkbox's plate drawn as a track with a sliding knob.
//!
//! Geometry is the classic one: the track is 1.9 times its height with fully rounded ends, the knob is
//! 0.82 of the track height and sits 0.09 in from either end. Every measurement is a ratio of
//! [`Switch::size`], so one knob scales the whole control.
//!
//! The plate, palette roles and clock are the family's: off, the track is a `border` outline and the knob
//! takes [`ThemeColors::border`]; on, the track fills with [`ThemeColors::accent`] and the knob takes
//! [`ThemeColors::bg`], exactly like the checkbox's fill and its tick. The knob travels on the same clock,
//! so [`Switch::motion`] and [`ToggleVisuals::from_theme`] behave here as they do on a
//! [`Checkbox`](crate::widgets::Checkbox).
//!
//! ```no_run
//! # use egui::__run_test_ui;
//! # use egui_elements::widgets::Switch;
//! # __run_test_ui(|ui| {
//! let mut dark_mode = true;
//! if ui.add(Switch::new(&mut dark_mode, "Dark mode")).changed() {
//!     // ...
//! }
//! # });
//! ```
//!
//! [`ThemeColors::accent`]: crate::theme::ThemeColors::accent
//! [`ThemeColors::bg`]: crate::theme::ThemeColors::bg
//! [`ThemeColors::border`]: crate::theme::ThemeColors::border

use super::{DEFAULT_SIZE, PlateGlyph, PlateShape, Row, TRACK_RATIO, ToggleShadow, ToggleVisuals};

use egui::{Response, Role, Ui, Widget, WidgetInfo, WidgetText, vec2};

use crate::motion::Motion;
use crate::theme::Theme;

/// A switch with an animated track fill and a sliding knob.
///
/// Shares the toggle family's plate: the same colours ([`.visuals(ToggleVisuals)`](Self::visuals)), lift
/// ([`.shadow(ToggleShadow)`](Self::shadow)) and clock ([`.motion(Motion)`](Self::motion)); only the
/// outline (a pill) and the mark (a knob that slides end to end) differ. Clicking anywhere in the row -
/// track or label - toggles it; a click reports [`Response::changed`].
///
/// ```no_run
/// # use egui::__run_test_ui;
/// # use egui_elements::widgets::Switch;
/// # __run_test_ui(|ui| {
/// let mut wifi = false;
/// ui.add(Switch::without_text(&mut wifi));
/// # });
/// ```
#[must_use = "Switches should be shown on a ui, e.g. `ui.add(Switch::new(&mut flag, \"Wi-Fi\"))`"]
pub struct Switch<'a> {
   checked: &'a mut bool,
   text: Option<WidgetText>,
   size: Option<f32>,
   visuals: Option<ToggleVisuals>,
   shadow: Option<ToggleShadow>,
   motion: Option<Motion>,
}

impl<'a> Switch<'a> {
   /// A switch with a label.
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

   /// A switch without a label, for a table cell or a toolbar.
   pub fn without_text(checked: &'a mut bool) -> Self {
      Self {
         text: None,
         ..Self::new(checked, "")
      }
   }

   /// Height of the track in points. Default `18.0`; the track is 1.9x wider and the knob 0.82x as tall,
   /// so every other measurement is a ratio of it.
   pub fn size(mut self, size: f32) -> Self {
      self.size = Some(size);
      self
   }

   /// Replace the palette-derived colours.
   pub fn visuals(mut self, visuals: ToggleVisuals) -> Self {
      self.visuals = Some(visuals);
      self
   }

   /// Replace the palette-derived shadow; [`ToggleShadow::NONE`] turns the lift off.
   pub fn shadow(mut self, shadow: ToggleShadow) -> Self {
      self.shadow = Some(shadow);
      self
   }

   /// How the track fill and the knob animate. Defaults to [`Motion::from_ui`] (the style's
   /// `animation_time`); [`Motion::instant`] makes it snap.
   pub fn motion(mut self, motion: Motion) -> Self {
      self.motion = Some(motion);
      self
   }
}

impl Widget for Switch<'_> {
   fn ui(self, ui: &mut Ui) -> Response {
      let Self {
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

      let mut row = Row::new(ui, text, vec2(size * TRACK_RATIO, size));
      if row.response.clicked() {
         *checked = !*checked;
         row.response.mark_changed();
      }
      row.response
         .widget_info(|| WidgetInfo::selected(Role::Switch, enabled, *checked, label.clone()));

      row.paint(
         *checked,
         PlateShape::Circle,
         PlateGlyph::Knob,
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
      Context, Event, FullOutput, Key, Modifiers, PointerButton, Pos2, RawInput, Rect, Shape, pos2,
      vec2,
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

   /// The knob is the only circle a switch paints.
   fn knob_centre(out: &FullOutput) -> Pos2 {
      let mut centres = Vec::new();
      for clipped in &out.shapes {
         if let Shape::Circle(circle) = &clipped.shape {
            centres.push(circle.center);
         }
      }
      assert_eq!(
         centres.len(),
         1,
         "a switch paints exactly one circle (the knob)"
      );
      centres[0]
   }

   #[test]
   fn settled_switch_requests_no_repaint() {
      let ctx = themed_ctx();
      let mut flag = true;
      // Two warm-up frames absorb egui's own first-frame repaint request.
      for _ in 0..2 {
         frame(&ctx, 10.0, vec![], |ui| {
            ui.add(Switch::without_text(&mut flag));
         });
      }
      let out = frame(&ctx, 10.0, vec![], |ui| {
         ui.add(Switch::without_text(&mut flag));
      });
      let delay = out.viewport_output[&egui::ViewportId::ROOT].repaint_delay;
      assert_eq!(
         delay,
         std::time::Duration::MAX,
         "a settled switch requested a repaint"
      );
   }

   #[test]
   fn label_click_toggles_and_reports_changed() {
      let ctx = themed_ctx();
      let mut flag = false;
      let mut rect = Rect::ZERO;
      frame(&ctx, 0.0, vec![], |ui| {
         rect = ui.add(Switch::new(&mut flag, "Wi-Fi")).rect;
      });
      let at = pos2(rect.right() - 4.0, rect.center().y);
      frame(&ctx, 0.1, vec![press(at)], |ui| {
         ui.add(Switch::new(&mut flag, "Wi-Fi"));
      });
      let mut changed = false;
      frame(&ctx, 0.2, vec![release(at)], |ui| {
         changed = ui.add(Switch::new(&mut flag, "Wi-Fi")).changed();
      });
      assert!(flag, "a click on the label must toggle the bool");
      assert!(
         changed,
         "the toggling frame must report `changed`"
      );

      let mut changed_after = true;
      frame(&ctx, 0.3, vec![], |ui| {
         changed_after = ui.add(Switch::new(&mut flag, "Wi-Fi")).changed();
      });
      assert!(!changed_after, "`changed` must not repeat");
   }

   #[test]
   fn space_toggles_the_focused_switch() {
      let ctx = themed_ctx();
      let mut flag = false;
      let mut id = None;
      frame(&ctx, 0.0, vec![], |ui| {
         id = Some(ui.add(Switch::without_text(&mut flag)).id);
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
            ui.add(Switch::without_text(&mut flag));
         },
      );
      assert!(flag, "Space on a focused switch must toggle it");
   }

   #[test]
   fn disabled_switch_does_not_toggle() {
      let ctx = themed_ctx();
      let mut flag = false;
      let mut center = pos2(0.0, 0.0);
      frame(&ctx, 0.0, vec![], |ui| {
         center = ui.add_enabled(false, Switch::without_text(&mut flag)).rect.center();
      });
      frame(&ctx, 0.1, vec![press(center)], |ui| {
         ui.add_enabled(false, Switch::without_text(&mut flag));
      });
      frame(&ctx, 0.2, vec![release(center)], |ui| {
         ui.add_enabled(false, Switch::without_text(&mut flag));
      });
      assert!(!flag, "a disabled switch must not toggle");
   }

   #[test]
   fn the_track_is_almost_twice_as_wide_as_it_is_tall() {
      let ctx = themed_ctx();
      let mut flag = false;
      let mut rect = Rect::ZERO;
      frame(&ctx, 0.0, vec![], |ui| {
         rect = ui.add(Switch::without_text(&mut flag).size(20.0)).rect;
      });
      assert!(
         (rect.width() - 38.0).abs() < 0.01,
         "track width must be 1.9 x height: {}",
         rect.width()
      );
      assert!(
         (rect.height() - 20.0).abs() < 0.01,
         "the row must be one track tall: {}",
         rect.height()
      );
   }

   #[test]
   fn the_knob_travels_between_the_ends_of_the_track() {
      let ctx = themed_ctx();
      let mut flag = false;
      frame(&ctx, 0.0, vec![], |ui| {
         ui.add(Switch::without_text(&mut flag).size(20.0).motion(Motion::instant()));
      });
      let off = {
         let out = frame(&ctx, 0.1, vec![], |ui| {
            ui.add(Switch::without_text(&mut flag).size(20.0).motion(Motion::instant()));
         });
         knob_centre(&out)
      };

      flag = true;
      let on = {
         let out = frame(&ctx, 0.2, vec![], |ui| {
            ui.add(Switch::without_text(&mut flag).size(20.0).motion(Motion::instant()));
         });
         knob_centre(&out)
      };

      assert!(
         (on.x - off.x - 18.0).abs() < 0.01,
         "the knob must travel 0.9 x the track height: {}",
         on.x - off.x
      );
      assert!(
         (on.y - off.y).abs() < 0.01,
         "the knob must stay on the track's centre line"
      );
      assert!(
         off.x < on.x,
         "off is the left end, on is the right end"
      );
   }
}
