//! A themed radio sharing the checkbox's plate: same row, colours, shadow and clock.
//!
//! The plate, the palette roles and the toggle animation are [`Checkbox`](crate::widgets::Checkbox)'s;
//! only the outline (a circle) and the mark (a dot) differ. A group is every radio bound to the same
//! `&mut value`: selecting one sets it, clicking the selected one changes nothing, and the rest of the
//! group follows on the next frame.
//!
//! [`Radio::visuals`] and [`Radio::shadow`] take the same `ToggleVisuals` / `ToggleShadow` the checkbox
//! uses, and [`Radio::motion`] overrides the style's clock.

use super::ToggleShadow;
use super::toggle::{DEFAULT_SIZE, PlateGlyph, PlateShape, Row, ToggleVisuals};

use egui::{Response, Role, Ui, Widget, WidgetInfo, WidgetText};

use crate::motion::Motion;
use crate::theme::Theme;

/// A radio button with an animated fill, dot and shadow.
///
/// Shares the checkbox's plate: the plate, colours, shadow and clock are the checkbox's
/// ([`.visuals(ToggleVisuals)`](Self::visuals), [`.shadow(ToggleShadow)`](Self::shadow),
/// [`.motion(Motion)`](Self::motion)); the outline is a circle and the mark is a dot. Clicking anywhere
/// in the row — plate or label — selects the value; a click reports [`Response::changed`]. Clicking the
/// selected radio reports no change and never clears the group.
///
/// ```no_run
/// # use egui::__run_test_ui;
/// # use egui_elements::widgets::Radio;
/// # __run_test_ui(|ui| {
/// #[derive(Clone, Copy, PartialEq)]
/// enum Mode {
///     Fast,
///     Secure,
/// }
/// let mut mode = Mode::Fast;
/// ui.add(Radio::new(&mut mode, Mode::Fast, "Fast"));
/// ui.add(Radio::new(&mut mode, Mode::Secure, "Secure"));
/// # });
/// ```
#[must_use = "Radios should be shown on a ui, e.g. `ui.add(Radio::new(&mut mode, Mode::Fast, \"Fast\"))`"]
pub struct Radio<'a, T> {
   selected: &'a mut T,
   value: T,
   text: Option<WidgetText>,
   size: Option<f32>,
   visuals: Option<ToggleVisuals>,
   shadow: Option<ToggleShadow>,
   motion: Option<Motion>,
}

impl<'a, T: PartialEq> Radio<'a, T> {
   /// A radio for `value` in the group behind `selected`, with a label.
   pub fn new(selected: &'a mut T, value: T, text: impl Into<WidgetText>) -> Self {
      Self {
         selected,
         value,
         text: Some(text.into()),
         size: None,
         visuals: None,
         shadow: None,
         motion: None,
      }
   }

   /// A radio without a label, for a table cell or a toolbar.
   pub fn without_text(selected: &'a mut T, value: T) -> Self {
      Self {
         text: None,
         ..Self::new(selected, value, "")
      }
   }

   /// Diameter of the plate in points. Default `18.0`; every other measurement is a ratio of it.
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

   /// How the toggle animates. Defaults to [`Motion::from_ui`] (the style's `animation_time`);
   /// [`Motion::instant`] makes it snap.
   pub fn motion(mut self, motion: Motion) -> Self {
      self.motion = Some(motion);
      self
   }
}

impl<T: PartialEq> Widget for Radio<'_, T> {
   fn ui(self, ui: &mut Ui) -> Response {
      let Self {
         selected,
         value,
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
      let mut checked = *selected == value;

      let mut row = Row::new(ui, text, size);
      if row.response.clicked() && !checked {
         *selected = value;
         checked = true;
         row.response.mark_changed();
      }
      row.response
         .widget_info(|| WidgetInfo::selected(Role::RadioButton, enabled, checked, label.clone()));

      row.paint(
         checked,
         PlateShape::Circle,
         PlateGlyph::Dot,
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

   #[derive(Clone, Copy, PartialEq, Debug)]
   enum Mode {
      Sms,
      Authenticator,
      Hardware,
   }

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
   fn clicking_the_label_selects_that_choice() {
      let ctx = themed_ctx();
      let mut selected = Mode::Sms;
      let mut rect = Rect::ZERO;
      frame(&ctx, 0.0, vec![], |ui| {
         ui.add(Radio::new(&mut selected, Mode::Sms, "SMS"));
         rect = ui
            .add(Radio::new(
               &mut selected,
               Mode::Authenticator,
               "Authenticator app",
            ))
            .rect;
         ui.add(Radio::new(
            &mut selected,
            Mode::Hardware,
            "Hardware key",
         ));
      });
      let at = pos2(rect.right() - 4.0, rect.center().y);
      frame(&ctx, 0.1, vec![press(at)], |ui| {
         ui.add(Radio::new(&mut selected, Mode::Sms, "SMS"));
         ui.add(Radio::new(
            &mut selected,
            Mode::Authenticator,
            "Authenticator app",
         ));
         ui.add(Radio::new(
            &mut selected,
            Mode::Hardware,
            "Hardware key",
         ));
      });
      let mut changed = false;
      frame(&ctx, 0.2, vec![release(at)], |ui| {
         ui.add(Radio::new(&mut selected, Mode::Sms, "SMS"));
         changed = ui
            .add(Radio::new(
               &mut selected,
               Mode::Authenticator,
               "Authenticator app",
            ))
            .changed();
         ui.add(Radio::new(
            &mut selected,
            Mode::Hardware,
            "Hardware key",
         ));
      });
      assert_eq!(
         selected,
         Mode::Authenticator,
         "a click on the label must select that radio's value"
      );
      assert!(
         changed,
         "the selecting frame must report `changed`"
      );

      let mut changed_after = true;
      frame(&ctx, 0.3, vec![], |ui| {
         ui.add(Radio::new(&mut selected, Mode::Sms, "SMS"));
         changed_after = ui
            .add(Radio::new(
               &mut selected,
               Mode::Authenticator,
               "Authenticator app",
            ))
            .changed();
         ui.add(Radio::new(
            &mut selected,
            Mode::Hardware,
            "Hardware key",
         ));
      });
      assert!(!changed_after, "`changed` must not repeat");
   }

   #[test]
   fn clicking_the_selected_radio_keeps_the_group() {
      let ctx = themed_ctx();
      let mut selected = Mode::Sms;
      let mut center = Pos2::ZERO;
      frame(&ctx, 0.0, vec![], |ui| {
         center = ui.add(Radio::new(&mut selected, Mode::Sms, "SMS")).rect.center();
      });
      frame(&ctx, 0.1, vec![press(center)], |ui| {
         ui.add(Radio::new(&mut selected, Mode::Sms, "SMS"));
      });
      let mut changed = true;
      frame(&ctx, 0.2, vec![release(center)], |ui| {
         changed = ui.add(Radio::new(&mut selected, Mode::Sms, "SMS")).changed();
      });
      assert_eq!(
         selected,
         Mode::Sms,
         "clicking the selected radio must keep the group"
      );
      assert!(
         !changed,
         "a selected radio must not report `changed`"
      );
   }

   #[test]
   fn space_selects_the_focused_radio() {
      let ctx = themed_ctx();
      let mut selected = Mode::Sms;
      let mut id = None;
      frame(&ctx, 0.0, vec![], |ui| {
         id = Some(
            ui.add(Radio::new(
               &mut selected,
               Mode::Hardware,
               "Hardware key",
            ))
            .id,
         );
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
            ui.add(Radio::new(
               &mut selected,
               Mode::Hardware,
               "Hardware key",
            ));
         },
      );
      assert_eq!(
         selected,
         Mode::Hardware,
         "Space on a focused radio must select it"
      );
   }

   #[test]
   fn disabled_radio_leaves_the_group_untouched() {
      let ctx = themed_ctx();
      let mut selected = Mode::Sms;
      let mut center = Pos2::ZERO;
      frame(&ctx, 0.0, vec![], |ui| {
         center = ui
            .add_enabled(
               false,
               Radio::new(&mut selected, Mode::Hardware, "Hardware key"),
            )
            .rect
            .center();
      });
      frame(&ctx, 0.1, vec![press(center)], |ui| {
         ui.add_enabled(
            false,
            Radio::new(&mut selected, Mode::Hardware, "Hardware key"),
         );
      });
      frame(&ctx, 0.2, vec![release(center)], |ui| {
         ui.add_enabled(
            false,
            Radio::new(&mut selected, Mode::Hardware, "Hardware key"),
         );
      });
      assert_eq!(
         selected,
         Mode::Sms,
         "a disabled radio must not select its value"
      );
   }

   #[test]
   fn a_radio_that_mounts_selected_does_not_animate_or_repaint() {
      let ctx = themed_ctx();
      let mut selected = Mode::Sms;
      // Two warm-up frames absorb egui's own first-frame repaint request.
      for _ in 0..2 {
         frame(&ctx, 10.0, vec![], |ui| {
            ui.add(Radio::new(&mut selected, Mode::Sms, "SMS"));
         });
      }
      let out = frame(&ctx, 10.0, vec![], |ui| {
         ui.add(Radio::new(&mut selected, Mode::Sms, "SMS"));
      });
      let delay = out.viewport_output[&egui::ViewportId::ROOT].repaint_delay;
      assert_eq!(
         delay,
         std::time::Duration::MAX,
         "a radio that mounts selected requested a repaint"
      );
   }
}
