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
      // Press snaps in and eases out: a press that fades in reads as lag on the click.
      let press_motion = if pressed { Motion::instant() } else { motion };
      let press = animate_with(
         ui,
         id.with("press_t"),
         if pressed { 1.0 } else { 0.0 },
         press_motion,
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
   use egui::{Color32, Context, Pos2, RawInput, Rect, Stroke, Vec2};

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
   fn interact_t_eases_the_hover() {
      let ctx = themed_ctx();
      let motion = Motion::new(0.2);
      let mut seen = Vec::new();
      for time in [10.0, 10.1, 10.2] {
         frame_at(&ctx, time, |ui| {
            seen.push(InteractT::of(ui, Id::unique("h"), true, false, motion).hover);
         });
      }
      assert_eq!(seen.len(), 3);
      assert!(
         (seen[0] - 0.0).abs() < 1e-4,
         "first sight: {}",
         seen[0]
      );
      assert!(
         (seen[1] - 0.875).abs() < 0.01,
         "half: {}",
         seen[1]
      );
      assert!(
         (seen[2] - 1.0).abs() < 1e-4,
         "settled: {}",
         seen[2]
      );
   }

   #[test]
   fn interact_t_snaps_the_press_in_and_eases_it_out() {
      let ctx = themed_ctx();
      let motion = Motion::new(0.2);
      let mut pressed = Vec::new();
      let mut released = Vec::new();
      frame_at(&ctx, 10.0, |ui| {
         pressed.push(InteractT::of(ui, Id::unique("p"), false, true, motion).press);
      });
      frame_at(&ctx, 10.1, |ui| {
         released.push(InteractT::of(ui, Id::unique("p"), false, false, motion).press);
      });
      frame_at(&ctx, 10.2, |ui| {
         released.push(InteractT::of(ui, Id::unique("p"), false, false, motion).press);
      });
      assert_eq!(pressed[0], 1.0, "press must snap in");
      assert!(
         (released[0] - 1.0).abs() < 1e-4,
         "release frame: {}",
         released[0]
      );
      assert!(
         (released[1] - 0.125).abs() < 0.01,
         "release: {}",
         released[1]
      );
   }

   #[test]
   fn interact_t_blends_idle_hover_press() {
      let idle = Color32::BLACK;
      let hover = Color32::WHITE;
      let press = Color32::RED;

      assert_eq!(
         InteractT {
            hover: 0.0,
            press: 0.0
         }
         .blend(idle, hover, press),
         Color32::BLACK
      );
      assert_eq!(
         InteractT {
            hover: 1.0,
            press: 0.0
         }
         .blend(idle, hover, press),
         Color32::WHITE
      );
      assert_eq!(
         InteractT {
            hover: 0.0,
            press: 1.0
         }
         .blend(idle, hover, press),
         Color32::RED
      );

      let mid = InteractT {
         hover: 0.5,
         press: 0.0,
      }
      .blend(idle, hover, press);
      for channel in [mid.r(), mid.g(), mid.b()] {
         assert!(
            (124..=131).contains(&channel),
            "mid channel: {channel}"
         );
      }
      assert_eq!(mid.r(), mid.g());
      assert_eq!(mid.g(), mid.b());
      assert_eq!(mid.a(), 255);
   }

   #[test]
   fn stroke_lerp_blends_width_and_colour() {
      let stroke = Stroke::new(1.0, Color32::BLACK).lerp(Stroke::new(3.0, Color32::WHITE), 0.5);
      assert!(
         (stroke.width - 2.0).abs() < 1e-4,
         "width: {}",
         stroke.width
      );
      for channel in [stroke.color.r(), stroke.color.g(), stroke.color.b()] {
         assert!(
            (124..=131).contains(&channel),
            "mid channel: {channel}"
         );
      }
   }
}
