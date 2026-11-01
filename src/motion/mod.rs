//! Time, in one place: easing curves and eased value animation.
//!
//! [`animate`] drives a value, and `Fade` / `Collapse` (next task) drive a container.
//! Durations come from the installed style's `animation_time`, so [`Motion::instant`]
//! (`duration = 0.0`) is both the reduce-motion answer and the deterministic-test answer.
//!
//! An animation's state is keyed by the caller's [`Id`](egui::Id), so pass a stable one — a
//! per-frame id animates nothing. A settled animation requests no repaint, which is what keeps this
//! usable in a battery-sensitive host.

mod animate;
mod easing;
mod state;

pub use animate::{Lerp, animate, animate_with};
pub use easing::Easing;

use egui::Ui;

/// How long an animation runs and along which curve.
///
/// The duration defaults to the installed style's `animation_time` ([`Motion::from_ui`]), so a host that
/// already tunes `Style::animation_time` does not configure motion twice.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Motion {
   /// Seconds. `0.0` means "no animation": the target is returned immediately.
   pub duration: f32,
   /// Curve applied to the normalized progress.
   pub easing: Easing,
}

impl Motion {
   /// `duration` seconds along [`Easing::default`] (ease-out).
   pub fn new(duration: f32) -> Self {
      Self {
         duration,
         easing: Easing::default(),
      }
   }

   /// The installed style's `animation_time`.
   pub fn from_ui(ui: &Ui) -> Self {
      Self::new(ui.style().animation_time)
   }

   /// No animation at all — every value lands immediately and nothing requests a repaint.
   pub fn instant() -> Self {
      Self {
         duration: 0.0,
         easing: Easing::Linear,
      }
   }

   /// Replace the curve.
   pub fn easing(self, easing: Easing) -> Self {
      Self { easing, ..self }
   }

   /// Replace the duration.
   pub fn duration(self, duration: f32) -> Self {
      Self { duration, ..self }
   }
}

impl Default for Motion {
   fn default() -> Self {
      Self::new(0.2)
   }
}
