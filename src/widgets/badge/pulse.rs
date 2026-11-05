//! The pulse: a copy of the badge circle that grows and fades behind the core.
//!
//! Measured from the reference capture: one frame shows a flat alpha-0.51 band from the core's edge out to
//! 1.61 x the diameter, feathered over ~1 pt — a **disc** copy, not a sweeping ring. The band is flat
//! because the alpha holds for most of the loop; it only fades out at the end.

use egui::{Color32, Id};

use crate::motion::Easing;

/// Progress at which the alpha stops holding and starts fading.
const HOLD: f32 = 0.62;
/// Curve of the fade after [`HOLD`]; a linear drop reads as a hard pop at the end.
const FADE_POW: f32 = 1.3;

/// The looping halo a [`Badge`](super::Badge) paints behind its circle.
///
/// Defaults are the measured ones: a 1.9x copy at 50 % alpha, once every 1.5 s.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BadgePulse {
   /// Halo fill. `None` uses the badge's own resolved fill, so one `.color(..)` override repaints both.
   pub color: Option<Color32>,
   /// Final scale of the copy, as a multiple of the badge diameter.
   pub scale: f32,
   /// Opacity the halo holds before it fades.
   pub alpha: f32,
   /// Seconds for one full grow-and-fade loop.
   pub period: f32,
   /// Curve of the growth.
   pub easing: Easing,
}

impl BadgePulse {
   /// Measured defaults: `scale 1.9`, `alpha 0.50`, `period 1.5 s`, [`Easing::EaseOut`].
   pub fn new() -> Self {
      Self {
         color: None,
         scale: 1.9,
         alpha: 0.50,
         period: 1.5,
         easing: Easing::EaseOut,
      }
   }

   /// Halo fill. Defaults to the badge's own fill.
   #[inline]
   pub fn color(mut self, color: Color32) -> Self {
      self.color = Some(color);
      self
   }

   /// Final scale of the copy. Default: `1.9`.
   #[inline]
   pub fn scale(mut self, scale: f32) -> Self {
      self.scale = scale;
      self
   }

   /// Opacity the halo holds before fading. Default: `0.50`.
   #[inline]
   pub fn alpha(mut self, alpha: f32) -> Self {
      self.alpha = alpha;
      self
   }

   /// Seconds for one loop. Default: `1.5`.
   #[inline]
   pub fn period(mut self, period: f32) -> Self {
      self.period = period;
      self
   }

   /// Curve of the growth. Default: [`Easing::EaseOut`].
   #[inline]
   pub fn easing(mut self, easing: Easing) -> Self {
      self.easing = easing;
      self
   }

   /// `(scale, alpha)` at loop progress `p`, ready to paint.
   ///
   /// The alpha holds at [`BadgePulse::alpha`] for the first 62 % of the loop and eases to zero
   /// over the rest, which is what the capture's flat band measures.
   pub fn frame(&self, p: f32) -> (f32, f32) {
      let p = p.rem_euclid(1.0);
      let scale = 1.0 + (self.scale.max(1.0) - 1.0) * self.easing.apply(p);
      let alpha = if p < HOLD {
         self.alpha
      } else {
         self.alpha * (1.0 - ((p - HOLD) / (1.0 - HOLD)).powf(FADE_POW))
      };
      (scale, alpha.clamp(0.0, 1.0))
   }

   /// Per-id phase offset in `0..1`, so two pulsing badges do not beat in lockstep. Deterministic.
   pub fn phase(id: Id) -> f32 {
      (id.value() % 997) as f32 / 997.0
   }
}

impl Default for BadgePulse {
   fn default() -> Self {
      Self::new()
   }
}

#[cfg(test)]
mod tests {
   use super::*;

   #[test]
   fn phase_is_deterministic_and_off_zero_for_a_real_id() {
      let a = BadgePulse::phase(Id::unique("inbox"));
      assert_eq!(a, BadgePulse::phase(Id::unique("inbox")));
      assert!((0.0..1.0).contains(&a));
   }
}
