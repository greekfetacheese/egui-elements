//! Toast notifications — timed, stacked messages painted over the app.
//!
//! Two types cooperate:
//!
//! * [`Toast`] — a builder that **enqueues** one notification via [`Toast::show`].
//!   It takes only a `&Context`, so any callback that holds the context can fire
//!   one: a button handler, an input event, an async completion callback, …
//! * [`Toasts`] — the renderer. Call [`Toasts::new()`]`.render(ctx)` once per
//!   frame in your top-level `update`. Without that call, enqueued toasts
//!   accumulate silently and nothing is drawn.
//!
//! ```no_run
//! # use egui_elements::widgets::{ChipRole, Toast, Toasts};
//! # let ctx = egui::Context::default();
//! // Somewhere in your update loop:
//! Toasts::new().render(&ctx);
//!
//! // From any callback with access to the context:
//! Toast::new("Deploy complete")
//!    .tone(ChipRole::Success)
//!    .description("Rolled out to us-east-1")
//!    .show(&ctx);
//! ```
//!
//! Forked from <https://github.com/stephenberry/egui-elegance> (MIT OR
//! Apache-2.0, © Stephen Berry) and re-homed on this crate's
//! [`Theme`](crate::theme::Theme): the card, accent bar and text read the
//! installed palette, and the tone is a [`ChipRole`] — the semantic role enum
//! [`Chip`](crate::widgets::Chip) already uses, so a toast and a chip never
//! disagree about what "success" looks like.

mod toast;
mod toasts;

pub use toast::Toast;
pub use toasts::Toasts;

use std::collections::VecDeque;

use egui::{Color32, Id};

use crate::theme::Theme;
use crate::widgets::ChipRole;

/// How long the fade-out animation takes, in seconds. A toast disappears at
/// `birth + duration + FADE_OUT`.
const FADE_OUT: f64 = 0.20;
/// Default auto-dismiss duration, in seconds.
const DEFAULT_DURATION: f64 = 4.0;
/// Default stack cap: older toasts are dropped when it is exceeded.
const DEFAULT_MAX_VISIBLE: usize = 5;
/// Default width of a toast card, in points.
const DEFAULT_WIDTH: f32 = 320.0;
/// Vertical gap between stacked toasts, in points.
const STACK_GAP: f32 = 8.0;
/// Height of the optional "Clear all" pill, in points.
const CLEAR_ALL_HEIGHT: f32 = 26.0;
/// Gap between the "Clear all" pill and the nearest toast, in points.
const CLEAR_ALL_GAP: f32 = 6.0;
/// Stack size at or above which the "Clear all" pill becomes worth showing.
///
/// At a single toast the per-toast × is enough; the bulk-dismiss affordance
/// only earns its rendering cost once two or more pile up.
const CLEAR_ALL_THRESHOLD: usize = 2;

/// Temp-data key holding the queue, shared by the builder and the renderer.
fn storage_id() -> Id {
   Id::unique("egui_elements::toasts")
}

/// Accent bar colour for a semantic [`ChipRole`].
fn tone_accent(theme: &Theme, tone: ChipRole) -> Color32 {
   tone.color(&theme.colors)
}

/// The whole notification stack, stored in `Context` temp data.
#[derive(Clone, Default)]
struct ToastState {
   queue: VecDeque<ToastEntry>,
   next_id: u64,
}

/// One notification, with the clock the renderer advances it against.
#[derive(Clone)]
struct ToastEntry {
   id: u64,
   title: String,
   description: Option<String>,
   tone: ChipRole,
   /// Auto-dismiss duration in seconds. `None` = persistent.
   duration: Option<f64>,
   /// Context time when the toast was enqueued.
   birth: f64,
   /// Context time when the user clicked ×. Triggers an immediate fade-out.
   dismiss_start: Option<f64>,
   /// Total seconds the auto-dismiss timer has been frozen by the pointer
   /// hovering the toast (so the user can read or copy the text).
   hover_pause_total: f64,
   /// Context time when the current hover-pause started, or `None` when the
   /// pointer is not over this toast.
   hover_pause_started: Option<f64>,
}

impl ToastEntry {
   /// Total seconds the auto-dismiss timer is currently frozen, including any
   /// in-progress hover that hasn't been committed to `hover_pause_total`.
   fn current_pause(&self, now: f64) -> f64 {
      self.hover_pause_total + self.hover_pause_started.map(|t| (now - t).max(0.0)).unwrap_or(0.0)
   }

   /// Has the fade-out animation completed?
   fn is_expired(&self, now: f64) -> bool {
      if let Some(dismiss_start) = self.dismiss_start {
         return now >= dismiss_start + FADE_OUT;
      }
      if let Some(duration) = self.duration {
         return now >= self.birth + duration + self.current_pause(now) + FADE_OUT;
      }
      false
   }

   /// Returns `(alpha, is_animating, next_transition_time)`.
   ///
   /// Toasts appear at full opacity and fade out only. `is_animating` is true
   /// while the fade-out is in progress (the renderer repaints continuously
   /// during it). `next_transition_time` is `Some(t)` when the toast is still
   /// at full opacity and wants a single deferred repaint at `t` to start the
   /// fade-out.
   fn alpha_and_schedule(&self, now: f64) -> (f32, bool, Option<f64>) {
      // Fade-out: either an explicit dismiss, or past the (paused-shifted)
      // auto-dismiss instant.
      let fade_out_start = match self.dismiss_start {
         Some(dismiss_start) => Some(dismiss_start),
         None => self.duration.map(|duration| self.birth + duration + self.current_pause(now)),
      };

      match fade_out_start {
         Some(t0) if now >= t0 => {
            let progress = ((now - t0) / FADE_OUT).clamp(0.0, 1.0) as f32;
            (1.0 - progress, progress < 1.0, None)
         }
         // While hovered, the next transition slips forward (or jumps far
         // forward, if hover started during a fade and rescued alpha back to
         // 1.0). An already-scheduled repaint at `t0` is fine: next frame the
         // deadline is recomputed against the new total.
         Some(t0) => (1.0, false, Some(t0)),
         None => (1.0, false, None),
      }
   }
}

#[cfg(test)]
mod tests {
   use std::time::Duration;

   use egui::{Context, RawInput};

   use super::*;

   /// Render the stack at an explicit context time, inside one frame.
   fn render_at(ctx: &Context, time: f64, toasts: Toasts) {
      ctx.run_pass(
         RawInput {
            time: Some(time),
            ..Default::default()
         },
         |ctx| toasts.render(ctx),
      )
      .drop_without_applying_deltas();
   }

   fn state(ctx: &Context) -> ToastState {
      ctx.data_mut(|d| d.get_temp::<ToastState>(storage_id()).unwrap_or_default())
   }

   fn entry(duration: Option<f64>) -> ToastEntry {
      ToastEntry {
         id: 0,
         title: "Alerts".to_owned(),
         description: None,
         tone: ChipRole::Info,
         duration,
         birth: 0.0,
         dismiss_start: None,
         hover_pause_total: 0.0,
         hover_pause_started: None,
      }
   }

   #[test]
   fn show_enqueues_the_configured_toast() {
      let ctx = Context::default();

      Toast::new("Saved")
         .tone(ChipRole::Success)
         .description("Three files")
         .show(&ctx);

      let state = state(&ctx);
      assert_eq!(state.queue.len(), 1);

      let entry = &state.queue[0];
      assert_eq!(entry.title, "Saved");
      assert_eq!(entry.description.as_deref(), Some("Three files"));
      assert_eq!(entry.tone, ChipRole::Success);
      assert_eq!(entry.duration, Some(DEFAULT_DURATION));
   }

   #[test]
   fn enqueue_assigns_distinct_ids() {
      let ctx = Context::default();
      Toast::new("first").show(&ctx);
      Toast::new("second").show(&ctx);

      let state = state(&ctx);
      assert_ne!(state.queue[0].id, state.queue[1].id);
   }

   #[test]
   fn render_caps_the_stack_at_max_visible() {
      let ctx = Context::default();
      for index in 0..4 {
         Toast::new(format!("toast {index}")).show(&ctx);
      }

      render_at(&ctx, 0.0, Toasts::new().max_visible(2));

      let state = state(&ctx);
      assert_eq!(state.queue.len(), 2);
      // The cap drops the oldest entries, so the newest survive.
      assert_eq!(state.queue[0].title, "toast 2");
      assert_eq!(state.queue[1].title, "toast 3");
   }

   #[test]
   fn max_visible_is_clamped_to_one() {
      let ctx = Context::default();
      Toast::new("first").show(&ctx);
      Toast::new("second").show(&ctx);

      render_at(&ctx, 0.0, Toasts::new().max_visible(0));

      assert_eq!(state(&ctx).queue.len(), 1);
   }

   #[test]
   fn auto_dismiss_expires_after_duration_plus_fade() {
      let ctx = Context::default();
      Toast::new("brief").duration(Duration::from_secs(1)).show(&ctx);

      render_at(&ctx, 0.5, Toasts::new());
      assert_eq!(
         state(&ctx).queue.len(),
         1,
         "still inside its lifetime"
      );

      render_at(&ctx, 1.0 + FADE_OUT * 0.5, Toasts::new());
      assert_eq!(
         state(&ctx).queue.len(),
         1,
         "fade-out still in flight"
      );

      render_at(&ctx, 1.0 + FADE_OUT + 0.05, Toasts::new());
      assert!(
         state(&ctx).queue.is_empty(),
         "fully faded and pruned"
      );
   }

   #[test]
   fn render_paints_a_card_of_the_configured_width() {
      let ctx = Context::default();
      Toast::new("Saved").persistent().show(&ctx);

      let input = || RawInput {
         time: Some(1.0),
         screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(800.0, 600.0),
         )),
         ..Default::default()
      };

      // An area's first frame only measures (its contents are invisible), so the
      // frame that paints is the second one.
      ctx.run_ui(input(), |ui| {
         Toasts::new().width(320.0).render(ui.ctx())
      })
      .drop_without_applying_deltas();

      let output = ctx.run_ui(input(), |ui| {
         Toasts::new().width(320.0).render(ui.ctx())
      });

      // The card is a rect of the builder's width; the shadow under it is a wider,
      // blurred rect, so the exact width is the discriminator.
      let card_painted = output.shapes.iter().any(|clipped| match &clipped.shape {
         egui::Shape::Rect(rect) => (rect.rect.width() - 320.0).abs() < 0.5,
         _ => false,
      });
      output.drop_without_applying_deltas();

      assert!(card_painted, "the toast card was never painted");
   }

   #[test]
   fn persistent_toast_never_expires() {
      let ctx = Context::default();
      Toast::new("sticky").persistent().show(&ctx);

      render_at(&ctx, 600.0, Toasts::new());

      assert_eq!(state(&ctx).queue.len(), 1);
   }

   #[test]
   fn hovering_freezes_the_auto_dismiss_clock() {
      let mut entry = entry(Some(1.0));
      entry.hover_pause_started = Some(0.5);

      // Hovered from 0.5 s to 2.0 s: the deadline slips by that interval.
      assert!((entry.current_pause(2.0) - 1.5).abs() < f64::EPSILON);
      assert!(!entry.is_expired(2.0));
   }

   #[test]
   fn fade_runs_from_full_to_transparent() {
      let entry = entry(Some(1.0));

      let (alpha, animating, next) = entry.alpha_and_schedule(0.5);
      assert_eq!(alpha, 1.0);
      assert!(!animating);
      assert_eq!(next, Some(1.0));

      let (alpha, animating, next) = entry.alpha_and_schedule(1.0 + FADE_OUT * 0.5);
      assert!((alpha - 0.5).abs() < 1e-6);
      assert!(animating);
      assert_eq!(next, None);
   }

   #[test]
   fn dismiss_starts_the_fade_immediately() {
      let mut entry = entry(None);
      entry.dismiss_start = Some(10.0);

      let (alpha, animating, _) = entry.alpha_and_schedule(10.0 + FADE_OUT * 0.5);
      assert!((alpha - 0.5).abs() < 1e-6);
      assert!(animating);
      assert!(entry.is_expired(10.0 + FADE_OUT));
   }
}
