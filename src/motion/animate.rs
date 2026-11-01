//! Eased value animation, and the interpolation helpers that go with it.

use std::hash::Hash;

use egui::{Color32, Id, Pos2, Rect, Stroke, Ui, Vec2};

use super::{Motion, state::AnimState};

/// Linear interpolation, so an animated `0..=1` progress can drive anything, not just a bare `f32`.
pub trait Lerp: Copy {
   /// Interpolate from `self` toward `to`; `t` is expected in `0..=1` and is **not** clamped here.
   fn lerp(self, to: Self, t: f32) -> Self;
}

impl Lerp for f32 {
   fn lerp(self, to: Self, t: f32) -> Self {
      self + (to - self) * t
   }
}

impl Lerp for Vec2 {
   fn lerp(self, to: Self, t: f32) -> Self {
      Self::new(self.x.lerp(to.x, t), self.y.lerp(to.y, t))
   }
}

impl Lerp for Pos2 {
   fn lerp(self, to: Self, t: f32) -> Self {
      Self::new(self.x.lerp(to.x, t), self.y.lerp(to.y, t))
   }
}

impl Lerp for Rect {
   fn lerp(self, to: Self, t: f32) -> Self {
      Self::from_min_max(self.min.lerp(to.min, t), self.max.lerp(to.max, t))
   }
}

impl Lerp for Color32 {
   /// Gamma-space (sRGB channel) blend. Linear-light blending washes mid-tones out in a UI, and
   /// `lerp_to_gamma` is what epaint itself uses for this.
   fn lerp(self, to: Self, t: f32) -> Self {
      self.lerp_to_gamma(to, t)
   }
}

impl Lerp for Stroke {
   fn lerp(self, to: Self, t: f32) -> Self {
      Self {
         width: self.width.lerp(to.width, t),
         color: self.color.lerp(to.color, t),
      }
   }
}

/// Eased `0..=1` progress toward `target`, timed by the installed style's `animation_time`.
///
/// Registers per-`Id` state in `ctx.data`: a first sight of `id` starts at `0.0`, a changed target retargets
/// from the value currently on screen, and a settled animation requests **no** repaint.
pub fn animate(ui: &Ui, id: impl Hash + std::fmt::Debug, target: f32) -> f32 {
   animate_with(ui, id, target, Motion::from_ui(ui))
}

/// [`animate`] with an explicit [`Motion`] (duration + curve).
pub fn animate_with(ui: &Ui, id: impl Hash + std::fmt::Debug, target: f32, motion: Motion) -> f32 {
   let ctx = ui.ctx();
   let now = ctx.input(|i| i.time);
   let key = Id::unique(("motion", id));

   let (value, progress) = ctx.data_mut(|data| {
      let state: &mut AnimState =
         data.get_temp_mut_or_insert_with(key, || AnimState::new(target, now));
      if state.target != target {
         // Start the new leg from the value that was on screen, not from the previous leg's origin: an instant leg
         // (duration 0.0) displays `to` while leaving `from` at 0.0, and re-sampling the old leg with the incoming
         // motion would invent a value that was never painted.
         state.from = state.last;
         state.to = target;
         state.start = now;
         state.target = target;
      }
      let (value, progress) = state.sample(now, motion.duration, motion.easing);
      state.last = value;
      (value, progress)
   });

   if progress < 1.0 {
      ctx.request_repaint();
   }
   value
}

#[cfg(test)]
mod tests {
   use super::*;
   use crate::theme::{Theme, ThemeKind};
   use egui::{Context, RawInput};

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
   fn animate_reaches_its_target_after_the_duration() {
      let ctx = themed_ctx();
      let motion = Motion::new(0.2);
      let mut seen = Vec::new();
      for time in [10.0, 10.1, 10.2] {
         frame_at(&ctx, time, |ui| {
            seen.push(animate_with(ui, "fade", 100.0, motion));
         });
      }
      assert_eq!(seen.len(), 3);
      assert!(
         (seen[0] - 0.0).abs() < 0.001,
         "start: {}",
         seen[0]
      );
      assert!((seen[1] - 87.5).abs() < 0.01, "half: {}", seen[1]);
      assert_eq!(seen[2], 100.0);
   }

   #[test]
   fn first_sight_starts_from_zero() {
      let ctx = themed_ctx();
      let mut value = -1.0;
      frame_at(&ctx, 10.0, |ui| {
         value = animate_with(ui, "first", 1.0, Motion::new(0.2));
      });
      // egui's `animate_bool` returns `1.0` here, which is why hosts hand-roll fade-ins.
      assert!(value < 0.001, "first sight: {value}");
   }

   #[test]
   fn retargeting_starts_from_the_current_value() {
      let ctx = themed_ctx();
      let motion = Motion::new(0.2);
      let mut values = Vec::new();
      frame_at(&ctx, 10.0, |ui| {
         values.push(animate_with(ui, "retarget", 1.0, motion));
      });
      frame_at(&ctx, 10.1, |ui| {
         values.push(animate_with(ui, "retarget", 1.0, motion));
         // A changed target starts a new leg from the value on screen.
         values.push(animate_with(ui, "retarget", 0.0, motion));
      });
      frame_at(&ctx, 10.2, |ui| {
         values.push(animate_with(ui, "retarget", 0.0, motion));
      });
      assert!(
         (values[0] - 0.0).abs() < 0.001,
         "start: {}",
         values[0]
      );
      assert!(
         (values[1] - 0.875).abs() < 1e-3,
         "half: {}",
         values[1]
      );
      assert!(
         (values[2] - 0.875).abs() < 1e-3,
         "retarget: {}",
         values[2]
      );
      assert!(
         (values[3] - 0.109_375).abs() < 1e-3,
         "new leg: {}",
         values[3]
      );
   }

   #[test]
   fn retarget_after_an_instant_leg_starts_from_the_value_on_screen() {
      let ctx = themed_ctx();
      let mut v = 0.0_f32;

      frame_at(&ctx, 10.0, |ui| {
         v = animate_with(ui, "instant_leg", 1.0, Motion::instant())
      });
      assert!(
         (v - 1.0).abs() < 1e-4,
         "instant leg should land on its target: {v}"
      );

      // The release re-uses the same id with a real duration. It must start from 1.0 (what was painted), not from the
      // instant leg's origin, and must not re-sample the old leg with the new motion.
      frame_at(&ctx, 10.1, |ui| {
         v = animate_with(ui, "instant_leg", 0.0, Motion::new(0.2))
      });
      assert!(
         (v - 1.0).abs() < 1e-3,
         "release frame should still be 1.0: {v}"
      );

      frame_at(&ctx, 10.2, |ui| {
         v = animate_with(ui, "instant_leg", 0.0, Motion::new(0.2))
      });
      assert!(
         (v - 0.125).abs() < 0.01,
         "half a leg from 1.0 to 0.0 is 0.125: {v}"
      );
   }

   #[test]
   fn settled_animation_requests_no_repaint() {
      let ctx = themed_ctx();
      let motion = Motion::new(0.2);
      for time in [10.0, 10.2, 10.3] {
         frame_at(&ctx, time, |ui| {
            let _ = animate_with(ui, "settled", 100.0, motion);
         });
      }
      let mut settled = -1.0;
      let mut idle = true;
      frame_at(&ctx, 10.4, |ui| {
         settled = animate_with(ui, "settled", 100.0, motion);
         idle = !ui.ctx().has_requested_repaint();
      });
      assert_eq!(settled, 100.0);
      assert!(
         idle,
         "settled animation must not request a repaint"
      );

      let mut flight_idle = true;
      frame_at(&ctx, 10.5, |ui| {
         let _ = animate_with(ui, "flight", 1.0, motion);
         flight_idle = !ui.ctx().has_requested_repaint();
      });
      assert!(
         !flight_idle,
         "a fresh animation must request a repaint"
      );
   }

   #[test]
   fn instant_motion_is_immediate_and_requests_no_repaint() {
      let ctx = themed_ctx();
      // Absorb egui's own first-frame repaint request (the theme install queues a command, which
      // asks for one more frame); the `animate_with` call is still the id's first sight.
      frame_at(&ctx, 9.8, |_ui| {});
      frame_at(&ctx, 9.9, |_ui| {});
      let mut value = -1.0;
      let mut idle = true;
      frame_at(&ctx, 10.0, |ui| {
         value = animate_with(ui, "instant", 42.0, Motion::instant());
         idle = !ui.ctx().has_requested_repaint();
      });
      assert_eq!(value, 42.0);
      assert!(idle, "instant motion must not request a repaint");
   }

   #[test]
   fn color_lerp_mixes_channels() {
      let a = Color32::BLACK;
      assert_eq!(a.lerp(Color32::WHITE, 0.0), Color32::BLACK);
      assert_eq!(a.lerp(Color32::WHITE, 1.0), Color32::WHITE);

      let mid = a.lerp(Color32::WHITE, 0.5);
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
}
