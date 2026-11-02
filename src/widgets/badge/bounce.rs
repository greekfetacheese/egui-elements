//! The hop: a one-shot vertical jump, fired by a text change, by the host, or on a loop.
//!
//! The reference capture caught the hop 4.5 pt up on an 18 pt badge, so the amplitude is a ratio of the
//! diameter; the arc rises fast and falls slow, which is what reads as a "jump" rather than a bubble.

use std::f32::consts::PI;
use std::hash::{Hash, Hasher};

use egui::{Context, Id};

use crate::motion::Motion;

/// Shape of the arc: a low exponent rises quickly and settles slowly.
const HOP_SHAPE: f32 = 0.62;

/// What starts a hop.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BadgeBounceTrigger {
   /// The badge's text changed since the last painted frame. The default.
   #[default]
   OnChange,
   /// Only [`Badge::bounce_now`](super::Badge::bounce_now) starts a hop, so one shared counter can
   /// animate one badge and not five.
   Manual,
   /// The hop repeats every [`BadgeBounce::period`].
   Loop,
}

/// A vertical hop, once per trigger.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BadgeBounce {
   /// What starts the hop. Default: [`BadgeBounceTrigger::OnChange`].
   pub trigger: BadgeBounceTrigger,
   /// Peak height as a ratio of the badge diameter. Default: `0.5`.
   pub amplitude: f32,
   /// Duration and curve of one hop. Default: `Motion::new(0.42)`.
   pub motion: Motion,
   /// Seconds between hops for [`BadgeBounceTrigger::Loop`]. Default: `2.2`.
   pub period: f32,
}

impl BadgeBounce {
   /// A hop on every text change: `amplitude 0.5`, `Motion::new(0.42)`, `period 2.2 s`.
   pub fn new() -> Self {
      Self {
         trigger: BadgeBounceTrigger::OnChange,
         amplitude: 0.5,
         motion: Motion::new(0.42),
         period: 2.2,
      }
   }

   /// What starts the hop.
   #[inline]
   pub fn trigger(mut self, trigger: BadgeBounceTrigger) -> Self {
      self.trigger = trigger;
      self
   }

   /// Peak height as a ratio of the badge diameter. Default: `0.5`.
   #[inline]
   pub fn amplitude(mut self, amplitude: f32) -> Self {
      self.amplitude = amplitude;
      self
   }

   /// Duration and curve of one hop.
   #[inline]
   pub fn motion(mut self, motion: Motion) -> Self {
      self.motion = motion;
      self
   }

   /// Seconds between hops, for [`BadgeBounceTrigger::Loop`].
   #[inline]
   pub fn period(mut self, period: f32) -> Self {
      self.period = period;
      self
   }

   /// Vertical offset in points at hop progress `q`; negative is up. At rest outside `0..1`.
   pub fn offset(&self, diameter: f32, q: f32) -> f32 {
      if !(0.0..1.0).contains(&q) {
         return 0.0;
      }
      let amplitude = self.amplitude.clamp(0.0, 1.5);
      -amplitude * diameter * (PI * q.powf(HOP_SHAPE)).sin()
   }

   /// Offset to paint this frame, advancing the trigger state. Call once per painted frame.
   pub(crate) fn offset_at(
      &self,
      ctx: &Context,
      badge_id: Id,
      text: &str,
      diameter: f32,
      now: f64,
   ) -> f32 {
      let key = badge_id.with("badge-bounce");
      let start = match self.trigger {
         BadgeBounceTrigger::OnChange => {
            let hash = hash_text(text);
            ctx.data_mut(|d| {
               let state = d.get_temp_mut_or_default::<BounceState>(key);
               state.observe(hash, now);
               state.start
            })
         }
         BadgeBounceTrigger::Manual => {
            ctx.data_mut(|d| d.get_temp_mut_or_default::<BounceState>(key).start)
         }
         BadgeBounceTrigger::Loop => {
            let period = self.period.max(self.motion.duration * 1.2).max(0.05) as f64;
            Some(now - (now % period))
         }
      };

      let Some(start) = start else {
         return 0.0;
      };
      let duration = self.motion.duration.max(0.01);
      let q = ((now - start) as f32 / duration).clamp(0.0, 1.0);
      // A loop never settles, so it must always ask for the next frame.
      if self.trigger == BadgeBounceTrigger::Loop || q < 1.0 {
         ctx.request_repaint();
      }
      self.offset(diameter, self.motion.easing.apply(q))
   }
}

impl Default for BadgeBounce {
   fn default() -> Self {
      Self::new()
   }
}

/// Per-badge hop state: the text last seen, and the start of the leg in flight.
#[derive(Clone, Copy, Debug, Default)]
struct BounceState {
   last: Option<u64>,
   start: Option<f64>,
}

impl BounceState {
   /// Record the text seen this frame; a change after the first sight starts a leg.
   ///
   /// First sight does **not** hop: a badge that appears already showing `3` must be still.
   fn observe(&mut self, text_hash: u64, now: f64) {
      let changed = self.last.is_some_and(|last| last != text_hash);
      self.last = Some(text_hash);
      if changed {
         self.start = Some(now);
      }
   }
}

/// Start a manual hop on the badge carrying `badge_id`.
pub(crate) fn start_now(ctx: &Context, badge_id: Id) {
   let now = ctx.input(|i| i.time);
   let key = badge_id.with("badge-bounce");
   ctx.data_mut(|d| {
      d.get_temp_mut_or_default::<BounceState>(key).start = Some(now);
   });
}

fn hash_text(text: &str) -> u64 {
   let mut hasher = std::collections::hash_map::DefaultHasher::new();
   text.hash(&mut hasher);
   hasher.finish()
}

#[cfg(test)]
mod tests {
   use super::*;

   #[test]
   fn the_hop_peaks_then_returns_and_only_a_changed_text_starts_it() {
      // Case: hop_peaks_at_the_amplitude_then_returns
      let bounce = BadgeBounce::new();
      assert_eq!(bounce.offset(18.0, 0.0), 0.0);
      assert_eq!(bounce.offset(18.0, 1.0), 0.0);

      let peak = (1..100)
         .map(|i| bounce.offset(18.0, i as f32 / 100.0))
         .fold(0.0_f32, |acc, dy| acc.min(dy));
      assert!(
         (peak + 9.0).abs() < 0.2,
         "0.5 x 18 = 9 pt up, got {peak}"
      );

      // Case: first_sight_never_hops
      let mut state = BounceState::default();
      state.observe(7, 0.0);
      assert_eq!(
         state.start, None,
         "a badge that appears already showing its count is still"
      );

      // Case: a_changed_text_starts_a_leg_from_now
      let mut state = BounceState::default();
      state.observe(7, 0.0);
      state.observe(9, 1.0);
      assert_eq!(state.start, Some(1.0));
      state.observe(9, 1.5);
      assert_eq!(
         state.start,
         Some(1.0),
         "the same text must not retrigger"
      );
   }
}
