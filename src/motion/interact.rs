//! Interaction-state weights for widgets that transition between idle, hover and press.

use egui::{Id, Response, Ui};

use super::{Lerp, Motion, animate_with};

/// How far a widget is from its hover and press visuals, `0..=1`, already eased.
///
/// A widget blends its own colours with [`InteractT::blend`], so the transition costs one line per colour and no
/// widget needs to know about time at all.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct InteractT {
   /// `0.0` idle, `1.0` fully hovered.
   pub hover: f32,
   /// `0.0` released, `1.0` fully pressed.
   pub press: f32,
}

impl InteractT {
   /// Weights for an explicit state pair.
   ///
   /// The conditions are the caller's, not derived here: a `Frame` counts `contains_pointer()` as hover, a text edit
   /// counts focus as its open state, and a hold-to-confirm `Button` adds its own gesture — passing booleans keeps
   /// every widget's state selection exactly as it was and animates only the change.
   pub fn of(ui: &Ui, id: Id, hovered: bool, pressed: bool, motion: Motion) -> Self {
      let hover = animate_with(
         ui,
         id.with("hover_t"),
         if hovered { 1.0 } else { 0.0 },
         motion,
      );
      // Press snaps in and out. The release has to land on the frame the gesture ends: a hold-to-confirm button
      // drops its pressed chrome the frame it fires, so an eased release would paint PRESSED for another 300 ms
      // after a completed hold (`the_chrome_releases_when_the_hold_completes`). Hover is ambience; a press is a
      // state flag.
      let press = animate_with(
         ui,
         id.with("press_t"),
         if pressed { 1.0 } else { 0.0 },
         Motion::instant(),
      );
      Self { hover, press }
   }

   /// [`InteractT::of`] for the common pair: `hovered()` and `is_pointer_button_down_on()`.
   pub fn from_response(ui: &Ui, response: &Response, motion: Motion) -> Self {
      Self::of(
         ui,
         response.id,
         response.hovered(),
         response.is_pointer_button_down_on(),
         motion,
      )
   }

   /// Blend `idle` → `hover` → `press`; `press` wins where both are active.
   pub fn blend<T: Lerp>(self, idle: T, hover: T, press: T) -> T {
      idle.lerp(hover, self.hover).lerp(press, self.press)
   }
}

#[cfg(test)]
mod tests {
   use super::*;
   use crate::theme::{Theme, ThemeKind};
   use egui::{Context, Pos2, RawInput, Rect, Vec2};

   fn themed_ctx() -> Context {
      let ctx = Context::default();
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&ctx);
      ctx
   }

   /// One frame at an explicit `input.time` — no sleeps, no wall clock.
   fn frame_at(ctx: &Context, time: f64, mut body: impl FnMut(&mut Ui)) {
      let input = RawInput {
         time: Some(time),
         screen_rect: Some(Rect::from_min_size(
            Pos2::ZERO,
            Vec2::new(400.0, 600.0),
         )),
         ..Default::default()
      };
      let mut out = ctx.run_ui(input, |ui| body(ui));
      out.textures_delta.clear();
   }

   #[test]
   fn interact_t_snaps_the_press_both_ways() {
      let ctx = themed_ctx();
      let id = Id::unique("press");
      let motion = Motion::new(0.2);
      let mut t = InteractT::default();

      frame_at(&ctx, 10.0, |ui| {
         t = InteractT::of(ui, id, false, true, motion)
      });
      assert!(
         (t.press - 1.0).abs() < 1e-4,
         "press must land on the down frame: {}",
         t.press
      );

      // The up frame must land too: a hold-to-confirm button releases its pressed chrome on the frame it fires.
      frame_at(&ctx, 10.1, |ui| {
         t = InteractT::of(ui, id, false, false, motion)
      });
      assert!(
         t.press.abs() < 1e-4,
         "release must land on the up frame: {}",
         t.press
      );
   }
}
