//! The enqueue half of the toast stack: [`Toast`], its builder and `show`.

use std::time::Duration;

use egui::Context;

use super::{DEFAULT_DURATION, ToastEntry, ToastState, storage_id};
use crate::widgets::ChipRole;

/// A single enqueued notification.
///
/// Construct with [`Toast::new`], configure via the builder methods, then call
/// [`Toast::show`] to enqueue. The toast is rendered the next time
/// [`Toasts::render`](super::Toasts::render) runs.
///
/// ```no_run
/// # use egui_elements::widgets::{ChipRole, Toast};
/// # let ctx = egui::Context::default();
/// Toast::new("Backup finished")
///    .tone(ChipRole::Success)
///    .description("42 files")
///    .show(&ctx);
/// ```
#[derive(Debug, Clone)]
#[must_use = "Call `show(ctx)` to enqueue the toast."]
pub struct Toast {
   title: String,
   description: Option<String>,
   tone: ChipRole,
   duration: Option<Duration>,
}

impl Toast {
   /// Create a toast with a title. Defaults: [`ChipRole::Info`], auto-dismiss
   /// after four seconds.
   pub fn new(title: impl Into<String>) -> Self {
      Self {
         title: title.into(),
         description: None,
         tone: ChipRole::Info,
         duration: Some(Duration::from_secs_f64(DEFAULT_DURATION)),
      }
   }

   /// Pick the tone (drives the left accent bar colour).
   pub fn tone(mut self, tone: ChipRole) -> Self {
      self.tone = tone;
      self
   }

   /// Add a secondary line below the title.
   pub fn description(mut self, description: impl Into<String>) -> Self {
      self.description = Some(description.into());
      self
   }

   /// Override how long the toast stays visible before it starts fading out.
   pub fn duration(mut self, duration: Duration) -> Self {
      self.duration = Some(duration);
      self
   }

   /// Disable auto-dismiss. The toast stays until the user clicks × or another
   /// toast pushes it out of the stack (see
   /// [`Toasts::max_visible`](super::Toasts::max_visible)).
   pub fn persistent(mut self) -> Self {
      self.duration = None;
      self
   }

   /// Enqueue the toast. It is shown on the next frame that renders
   /// [`Toasts`](super::Toasts).
   pub fn show(self, ctx: &Context) {
      let now = ctx.input(|i| i.time);
      ctx.data_mut(|d| {
         let mut state = d.get_temp::<ToastState>(storage_id()).unwrap_or_default();
         let id = state.next_id;
         state.next_id = state.next_id.wrapping_add(1);
         state.queue.push_back(ToastEntry {
            id,
            title: self.title,
            description: self.description,
            tone: self.tone,
            duration: self.duration.map(|d| d.as_secs_f64()),
            birth: now,
            dismiss_start: None,
            hover_pause_total: 0.0,
            hover_pause_started: None,
         });
         d.insert_temp(storage_id(), state);
      });
      ctx.request_repaint();
   }
}
