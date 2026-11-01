//! Containers that animate what they hold: an opacity [`Fade`] and a height [`Collapse`].

use std::hash::Hash;

use egui::{Id, Rect, Sense, Ui, UiBuilder, Vec2};

use super::{Motion, animate_with, state::CollapseState};

/// What a [`Fade`] does when it closes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FadeMode {
   /// Close instantly; only opening is animated. The default, and what a panel that swaps to the next view
   /// wants — a fading-out panel stacks with whatever comes next and reads as a glitch.
   In,
   /// Animate both directions; the body keeps running while it fades out.
   Both,
}

/// Animated opacity for a body that must not pop into existence.
pub struct Fade {
   id: Id,
   open: bool,
   mode: FadeMode,
   motion: Option<Motion>,
}

impl Fade {
   /// `id` keys the animation: pass something stable per fade, not something that changes every frame.
   pub fn new(id: impl Hash + std::fmt::Debug, open: bool) -> Self {
      Self {
         id: Id::unique(id),
         open,
         mode: FadeMode::In,
         motion: None,
      }
   }

   /// Close by fading out instead of disappearing (default [`FadeMode::In`]).
   pub fn mode(self, mode: FadeMode) -> Self {
      Self { mode, ..self }
   }

   /// Override the duration/curve (default: the installed style's `animation_time`, ease-out).
   pub fn motion(self, motion: Motion) -> Self {
      Self {
         motion: Some(motion),
         ..self
      }
   }

   /// Run `add_contents` under the animated opacity. Does not run at all while the opacity is `0.0`.
   pub fn show(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
      let motion = self.motion.unwrap_or_else(|| Motion::from_ui(ui));

      let opacity = match self.mode {
         // Instant close, and no closed-frame ticking: `animate_with` starts a first sight at 0.0 by itself,
         // so the next open fades in even though we never call it while closed.
         FadeMode::In => {
            if !self.open {
               return;
            }
            animate_with(ui, self.id, 1.0, motion)
         }
         FadeMode::Both => {
            let t = animate_with(
               ui,
               self.id,
               if self.open { 1.0 } else { 0.0 },
               motion,
            );
            if t <= 0.0 {
               return;
            }
            t
         }
      };

      if opacity <= 0.0 {
         return;
      }

      ui.scope(|ui| {
         ui.multiply_opacity(opacity);
         add_contents(ui);
      });
   }
}

/// A body revealed by animating its own height, measured once and then followed.
///
/// The band is the animated height, the body lays out at its **natural** height and is clipped to the band, and
/// the cursor advances by the band — so whatever follows the collapse rides the animation without knowing it
/// exists. No host-side height cache is needed (that is what `zeus/src/gui/ui/account/mod.rs` carries today).
pub struct Collapse {
   id: Id,
   open: bool,
   motion: Option<Motion>,
}

impl Collapse {
   /// `id` keys both the animation and the measured height.
   pub fn new(id: impl Hash + std::fmt::Debug, open: bool) -> Self {
      Self {
         id: Id::unique(id),
         open,
         motion: None,
      }
   }

   /// Override the duration/curve (default: the installed style's `animation_time`, ease-out).
   pub fn motion(self, motion: Motion) -> Self {
      Self {
         motion: Some(motion),
         ..self
      }
   }

   /// Reveal `add_contents`, returning the height allocated this frame (the animated height, `0.0` when closed).
   pub fn show(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) -> f32 {
      let motion = self.motion.unwrap_or_else(|| Motion::from_ui(ui));
      let ctx = ui.ctx().clone();
      let key = self.id.with("collapse");

      // Tick the clock BEFORE anything can return early: the probe frame below is the frame the animation
      // starts on, so a probe-and-return placed above this call would lose a frame and the first eased height
      // would still be 0.0.
      let target = if self.open { 1.0 } else { 0.0 };
      let t = animate_with(ui, self.id.with("t"), target, motion);

      let natural =
         ctx.data_mut(|data| data.get_temp_mut_or_insert_with(key, CollapseState::default).natural);

      let Some(natural) = natural else {
         if !self.open {
            return 0.0;
         }
         // First open: measure the body once, invisibly, and let it run only once this frame.
         let rect = ui.available_rect_before_wrap();
         let mut probe = ui.new_child(
            UiBuilder::new().max_rect(rect).layout(*ui.layout()).sizing_pass().invisible(),
         );
         add_contents(&mut probe);
         let measured = probe.min_rect().height().max(0.0);
         ctx.data_mut(|data| {
            data.get_temp_mut_or_insert_with(key, CollapseState::default).natural = Some(measured)
         });
         return 0.0;
      };

      let height = (natural * t).max(0.0);
      if height <= 0.0 {
         // Closed: allocate nothing, so a closed collapse leaves no gap behind it.
         return 0.0;
      }

      let (band, _) = ui.allocate_exact_size(
         Vec2::new(ui.available_width(), height),
         Sense::hover(),
      );
      let clip = ui.clip_rect();
      let mut child = ui.new_child(
         UiBuilder::new()
            .max_rect(Rect::from_min_size(
               band.min,
               Vec2::new(band.width(), natural),
            ))
            .layout(*ui.layout()),
      );
      child.set_clip_rect(band.intersect(clip));
      add_contents(&mut child);

      let measured = child.min_rect().height().max(0.0);
      if measured > 0.0 {
         ctx.data_mut(|data| {
            data.get_temp_mut_or_insert_with(key, CollapseState::default).natural = Some(measured)
         });
      }

      height
   }
}

#[cfg(test)]
mod tests {
   use super::*;
   use crate::theme::{Theme, ThemeKind};
   use egui::{Context, Pos2, RawInput};
   use std::cell::Cell;

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

   /// A body whose natural height is exactly `100.0`.
   fn body(ui: &mut Ui) {
      ui.add_space(100.0);
   }

   #[test]
   fn collapse_expands_to_the_body_height() {
      let ctx = themed_ctx();
      let motion = Motion::new(0.2);
      let mut heights = Vec::new();

      frame_at(&ctx, 10.0, |ui| {
         heights.push(Collapse::new("expand", false).motion(motion).show(ui, body));
      });
      // The probe frame: the body is measured invisibly and allocated nothing yet.
      frame_at(&ctx, 10.1, |ui| {
         heights.push(Collapse::new("expand", true).motion(motion).show(ui, body));
      });
      frame_at(&ctx, 10.2, |ui| {
         heights.push(Collapse::new("expand", true).motion(motion).show(ui, body));
      });
      frame_at(&ctx, 10.3, |ui| {
         heights.push(Collapse::new("expand", true).motion(motion).show(ui, body));
      });

      assert_eq!(heights[0], 0.0, "closed: {}", heights[0]);
      assert_eq!(heights[1], 0.0, "probe frame: {}", heights[1]);
      assert!(
         (heights[2] - 87.5).abs() < 0.01,
         "mid-flight: {}",
         heights[2]
      );
      assert!(
         (heights[3] - 100.0).abs() < 0.01,
         "settled: {}",
         heights[3]
      );
   }

   #[test]
   fn collapse_mid_flight_is_the_eased_fraction() {
      let ctx = themed_ctx();
      let motion = Motion::new(0.2);
      let mut mid = -1.0;

      frame_at(&ctx, 10.0, |ui| {
         Collapse::new("mid", false).motion(motion).show(ui, body);
      });
      frame_at(&ctx, 10.1, |ui| {
         Collapse::new("mid", true).motion(motion).show(ui, body);
      });
      frame_at(&ctx, 10.2, |ui| {
         mid = Collapse::new("mid", true).motion(motion).show(ui, body);
      });

      // `cubic_out(0.5) = 0.875`, so a 100 pt body is 87.5 pt tall halfway through.
      assert!((mid - 87.5).abs() < 0.01, "mid-flight: {mid}");
      assert!(
         mid > 0.0 && mid < 100.0,
         "strictly mid-flight: {mid}"
      );
   }

   #[test]
   fn collapse_clips_its_body_while_expanding() {
      let ctx = themed_ctx();
      let motion = Motion::new(0.2);
      // `(clip, max_rect)` of the child ui on the mid-flight frame.
      let seen: Cell<Option<(Rect, Rect)>> = Cell::new(None);
      let frame = Cell::new(0);

      for time in [10.0, 10.1, 10.2] {
         let this_frame = frame.get();
         frame.set(this_frame + 1);
         frame_at(&ctx, time, |ui| {
            Collapse::new("clip", this_frame >= 1).motion(motion).show(ui, |ui| {
               if this_frame == 2 {
                  seen.set(Some((ui.clip_rect(), ui.max_rect())));
               }
               ui.add_space(100.0);
            });
         });
      }

      let (clip, max_rect) = seen.get().expect("mid-flight frame captured a rect");
      // The clip is the band, not the body: the body is laid out at `100.0` and clipped to the band.
      assert!(
         (clip.top() - max_rect.top()).abs() < 0.5,
         "clip top {} vs max_rect top {}",
         clip.top(),
         max_rect.top()
      );
      assert!(
         clip.height() <= 87.5 + 0.5,
         "clip height {} must be the band",
         clip.height()
      );
   }

   #[test]
   fn collapse_closed_takes_no_space() {
      let ctx = themed_ctx();
      let motion = Motion::new(0.2);
      let mut heights = Vec::new();

      // Settle open.
      for time in [10.0, 10.1, 10.2] {
         frame_at(&ctx, time, |ui| {
            heights.push(Collapse::new("space", true).motion(motion).show(ui, body));
         });
      }
      assert!(
         (heights[2] - 100.0).abs() < 0.01,
         "settled open: {}",
         heights[2]
      );

      // Close: the retarget frame must not jump.
      frame_at(&ctx, 10.3, |ui| {
         heights.push(Collapse::new("space", false).motion(motion).show(ui, body));
      });
      // p = 0.5 on the way down: `1.0 - 0.875 = 0.125` of 100.
      frame_at(&ctx, 10.4, |ui| {
         heights.push(Collapse::new("space", false).motion(motion).show(ui, body));
      });
      let mut cursor_moved = 1.0;
      frame_at(&ctx, 10.5, |ui| {
         let before = ui.cursor().top();
         heights.push(Collapse::new("space", false).motion(motion).show(ui, body));
         let after = ui.cursor().top();
         cursor_moved = after - before;
      });

      assert!(
         (heights[3] - 100.0).abs() < 0.01,
         "retarget frame: {}",
         heights[3]
      );
      assert!(
         (heights[4] - 12.5).abs() < 0.01,
         "mid close: {}",
         heights[4]
      );
      assert_eq!(heights[5], 0.0, "closed: {}", heights[5]);
      assert!(
         cursor_moved.abs() < 0.001,
         "a closed collapse must not move the cursor: {cursor_moved}"
      );
   }

   #[test]
   fn fade_in_skips_the_body_when_closed() {
      let ctx = themed_ctx();
      let motion = Motion::new(0.2);
      let runs = Cell::new(0usize);
      let mut seen = Vec::new();

      let mut run = |ctx: &Context, time: f64, open: bool| {
         frame_at(ctx, time, |ui| {
            Fade::new("fade_in", open)
               .mode(FadeMode::In)
               .motion(motion)
               .show(ui, |_ui| runs.set(runs.get() + 1));
         });
         seen.push(runs.get());
      };

      run(&ctx, 10.0, false);
      run(&ctx, 10.1, false);
      run(&ctx, 10.2, false);
      // First open frame: opacity is still `0.0`, so nothing is painted either.
      run(&ctx, 10.3, true);
      run(&ctx, 10.4, true);

      assert_eq!(
         &seen[..4],
         &[0, 0, 0, 0],
         "closed then first open: {seen:?}"
      );
      assert_eq!(seen[4], 1, "open body runs: {seen:?}");
   }

   #[test]
   fn fade_both_runs_the_body_while_closing() {
      let ctx = themed_ctx();
      let motion = Motion::new(0.2);
      let runs = Cell::new(0usize);
      let mut seen = Vec::new();

      let mut run = |ctx: &Context, time: f64, open: bool| {
         frame_at(ctx, time, |ui| {
            Fade::new("fade_both", open)
               .mode(FadeMode::Both)
               .motion(motion)
               .show(ui, |_ui| runs.set(runs.get() + 1));
         });
         seen.push(runs.get());
      };

      run(&ctx, 9.9, false);
      // First open frame at `10.0`: opacity is `0.0`, so nothing is painted.
      run(&ctx, 10.0, true);
      run(&ctx, 10.1, true);
      // Closing at full opacity: it still paints.
      run(&ctx, 10.2, false);
      run(&ctx, 10.3, false);
      run(&ctx, 10.5, false);

      assert_eq!(
         &seen[..3],
         &[0, 0, 1],
         "closed then open: {seen:?}"
      );
      assert_eq!(seen[3], 2, "first closing frame: {seen:?}");
      assert_eq!(seen[4], 3, "mid fade-out: {seen:?}");
      assert_eq!(seen[5], 3, "settled at zero: {seen:?}");
   }
}
