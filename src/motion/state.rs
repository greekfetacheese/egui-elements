//! Per-`Id` animation state, kept in `ctx.data` and never persisted.

use super::Easing;

/// One `animate` value in flight.
#[derive(Clone, Copy, Debug)]
pub struct AnimState {
   /// Value the current leg started from.
   pub from: f32,
   /// Value the current leg moves toward.
   pub to: f32,
   /// `input.time` (seconds) of the last retarget.
   pub start: f64,
   /// The target last seen; a different target means "retarget from where we are".
   pub target: f32,
   /// The value displayed on the previous frame. A retarget starts from **this**, not from `from`: an instant leg
   /// (`duration == 0.0`) never advances `from`, so `from` alone loses the value the user actually saw.
   pub last: f32,
}

/// Where a [`Collapse`](crate::motion::Collapse) is: the body height measured on its last visible frame.
#[derive(Clone, Copy, Debug, Default)]
pub struct CollapseState {
   /// `None` until the body has been measured once (first open, or after a host restart).
   pub natural: Option<f32>,
}

impl AnimState {
   /// A freshly seen id starts at `0.0`, not at the target: a first open must fade in, which is the
   /// behaviour egui's own `animate_bool` cannot give (an unknown id returns the target immediately).
   pub fn new(target: f32, now: f64) -> Self {
      Self {
         from: 0.0,
         to: target,
         start: now,
         target,
         last: 0.0,
      }
   }

   /// `(value, progress)` at `now`. `progress` is `1.0` once settled, so callers can stop repainting.
   pub fn sample(&self, now: f64, duration: f32, easing: Easing) -> (f32, f32) {
      if duration <= 0.0 {
         return (self.to, 1.0);
      }
      let t = (((now - self.start) as f32) / duration).clamp(0.0, 1.0);
      let eased = easing.apply(t);
      (self.from + (self.to - self.from) * eased, t)
   }
}
