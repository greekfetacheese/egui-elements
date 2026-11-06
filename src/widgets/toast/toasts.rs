//! The render half of the toast stack: stacking, measurement and paint.

use std::time::Duration;

use egui::{
   Align, Align2, Area, Context, CornerRadius, FontId, Id, Layout, Order, Pos2, Rect, Response,
   RichText, Role, Sense, Stroke, StrokeKind, Ui, UiBuilder, Vec2, WidgetInfo, accesskit,
};

use super::{
   CLEAR_ALL_GAP, CLEAR_ALL_HEIGHT, CLEAR_ALL_THRESHOLD, DEFAULT_MAX_VISIBLE, DEFAULT_WIDTH,
   STACK_GAP, ToastEntry, ToastState, storage_id, tone_accent,
};
use crate::theme::{Elevation, Surface, SurfaceState, Theme};
use crate::widgets::ChipRole;

/// Renderer for the enqueued toast stack.
///
/// Configure placement via the builder, then call [`Toasts::render`] once per
/// frame. Calling it more than once in a frame paints the whole stack twice.
///
/// ```no_run
/// # use egui_elements::widgets::Toasts;
/// # let ctx = egui::Context::default();
/// Toasts::new().anchor(egui::Align2::RIGHT_BOTTOM).render(&ctx);
/// ```
#[derive(Debug, Clone)]
#[must_use = "Call `.render(ctx)` to draw the toast stack."]
pub struct Toasts {
   anchor: Align2,
   offset: Vec2,
   max_visible: usize,
   width: f32,
   clear_all_button: bool,
}

impl Default for Toasts {
   fn default() -> Self {
      Self::new()
   }
}

impl Toasts {
   /// Start a new configuration. Defaults: anchored to the bottom-right with a
   /// 12-point offset, up to 5 toasts visible, 320 points wide.
   pub fn new() -> Self {
      Self {
         anchor: Align2::RIGHT_BOTTOM,
         offset: Vec2::new(12.0, 12.0),
         max_visible: DEFAULT_MAX_VISIBLE,
         width: DEFAULT_WIDTH,
         clear_all_button: false,
      }
   }

   /// Anchor corner for the stack. Default: [`Align2::RIGHT_BOTTOM`].
   pub fn anchor(mut self, anchor: Align2) -> Self {
      self.anchor = anchor;
      self
   }

   /// Offset from the anchor corner, in points. Default: `(12, 12)`.
   pub fn offset(mut self, offset: impl Into<Vec2>) -> Self {
      self.offset = offset.into();
      self
   }

   /// Maximum number of toasts rendered at once. Oldest are dropped when the
   /// cap is exceeded. Default: 5.
   pub fn max_visible(mut self, max_visible: usize) -> Self {
      self.max_visible = max_visible.max(1);
      self
   }

   /// Width of each toast card in points. Default: 320.
   pub fn width(mut self, width: f32) -> Self {
      self.width = width.max(120.0);
      self
   }

   /// Show a "Clear all" pill at the far end of the stack (above the topmost
   /// toast for bottom-anchored stacks, below the bottommost for top-anchored).
   /// It appears only when at least two non-dismissing toasts are visible;
   /// clicking it starts the fade-out on every entry at once. Default: `false`.
   pub fn clear_all_button(mut self, enabled: bool) -> Self {
      self.clear_all_button = enabled;
      self
   }

   /// Render the enqueued toast stack. Call once per frame.
   pub fn render(&self, ctx: &Context) {
      let theme = Theme::current(ctx);
      let now = ctx.input(|i| i.time);

      // Snapshot the state under a short lock, then hand the lock back.
      let mut state = ctx.data_mut(|d| d.get_temp::<ToastState>(storage_id())).unwrap_or_default();

      // Expire fully-faded toasts, then cap the queue to `max_visible` by
      // dropping the oldest (front).
      state.queue.retain(|entry| !entry.is_expired(now));
      while state.queue.len() > self.max_visible {
         state.queue.pop_front();
      }

      if state.queue.is_empty() {
         ctx.data_mut(|d| d.insert_temp(storage_id(), state));
         return;
      }

      // Lay the stack out manually (not via egui's own stacking) so each
      // toast's height is known before the Area is positioned.
      let screen = ctx.content_rect();
      let stack_up = matches!(self.anchor.y(), Align::Max);

      // Measure every toast first, so stacking does not depend on
      // previous-frame measurements.
      let entry_heights: Vec<f32> = state
         .queue
         .iter()
         .map(|entry| measure_height(ctx, &theme, entry, self.width))
         .collect();

      // x position of the stack.
      let x = match self.anchor.x() {
         Align::Min => screen.min.x + self.offset.x,
         Align::Center => screen.center().x - self.width * 0.5,
         Align::Max => screen.max.x - self.offset.x - self.width,
      };

      // Starting y and step direction.
      let (mut y, step_sign): (f32, f32) = if stack_up {
         (screen.max.y - self.offset.y, -1.0)
      } else {
         (screen.min.y + self.offset.y, 1.0)
      };

      // The newest toast sits closest to the anchor edge.
      let indices: Vec<usize> = if stack_up {
         (0..state.queue.len()).rev().collect()
      } else {
         (0..state.queue.len()).collect()
      };

      let mut dismiss_ids: Vec<u64> = Vec::new();
      let mut hovered_ids: Vec<u64> = Vec::new();
      let mut earliest_next_event: Option<f64> = None;
      let mut any_animating = false;

      for index in indices {
         let entry = &state.queue[index];
         let height = entry_heights[index];

         let (top, bottom) = if step_sign < 0.0 {
            (y - height, y)
         } else {
            (y, y + height)
         };
         let rect = Rect::from_min_max(
            Pos2::new(x, top),
            Pos2::new(x + self.width, bottom),
         );

         // Animating = currently in fade-out.
         let (alpha, is_animating, next_event) = entry.alpha_and_schedule(now);
         any_animating |= is_animating;
         if let Some(at) = next_event {
            earliest_next_event = Some(match earliest_next_event {
               Some(previous) => previous.min(at),
               None => at,
            });
         }

         let area_id = Id::unique(("egui_elements::toast", entry.id));
         let response =
            Area::new(area_id).order(Order::Tooltip).fixed_pos(rect.min).show(ctx, |ui| {
               paint_toast(ui, &theme, entry, rect, alpha)
            });

         let paint = response.inner;
         if paint.close_clicked {
            dismiss_ids.push(entry.id);
         }
         if paint.hovered {
            hovered_ids.push(entry.id);
         }

         // Advance the cursor for the next toast.
         y += (height + STACK_GAP) * step_sign;
      }

      // Record clicks into `dismiss_start` so the next frame's alpha math picks
      // them up. Also commit any in-flight hover pause and stop tracking it:
      // alpha is now driven by `dismiss_start`, and leaving
      // `hover_pause_started` set would advance `current_pause` against a
      // deadline that no longer applies.
      if !dismiss_ids.is_empty() {
         for entry in state.queue.iter_mut() {
            if dismiss_ids.contains(&entry.id) && entry.dismiss_start.is_none() {
               entry.dismiss_start = Some(now);
               if let Some(started) = entry.hover_pause_started.take() {
                  entry.hover_pause_total += (now - started).max(0.0);
               }
            }
         }
      }

      // Hover-pause bookkeeping. Hovering freezes the auto-dismiss countdown;
      // releasing commits the paused interval. If hover starts while the toast
      // is already mid auto-fade, "rescue" it: bump `hover_pause_total` so
      // the fade-out jumps to `now`, restoring alpha to 1.0 and giving the
      // user a fresh duration window after they release.
      //
      // Repaint scheduling: `alpha_and_schedule` returns the (pause-shifted)
      // deadline as `next_event`, so the `request_repaint_after` below sleeps
      // until then; on wake-up the deadline is recomputed against the latest
      // pause total. Hover transitions are pointer events and repaint anyway.
      for entry in state.queue.iter_mut() {
         let is_hovered = entry.dismiss_start.is_none() && hovered_ids.contains(&entry.id);
         match (is_hovered, entry.hover_pause_started) {
            (true, None) => {
               if let Some(duration) = entry.duration {
                  let fade_out_start = entry.birth + duration + entry.hover_pause_total;
                  if now > fade_out_start {
                     entry.hover_pause_total = now - entry.birth;
                  }
               }
               entry.hover_pause_started = Some(now);
            }
            (false, Some(started)) => {
               entry.hover_pause_total += (now - started).max(0.0);
               entry.hover_pause_started = None;
            }
            (true, Some(_)) | (false, None) => {}
         }
      }

      // Optional "Clear all" pill anchored at the far end of the stack. Counted
      // on non-dismissing entries so the pill hides as soon as a bulk dismiss
      // is triggered (those toasts then fade out).
      let active_count = state.queue.iter().filter(|entry| entry.dismiss_start.is_none()).count();
      if self.clear_all_button && active_count >= CLEAR_ALL_THRESHOLD {
         let total_height: f32 = entry_heights.iter().sum::<f32>()
            + STACK_GAP * entry_heights.len().saturating_sub(1) as f32;
         let pill_top = if stack_up {
            (screen.max.y - self.offset.y) - total_height - CLEAR_ALL_GAP - CLEAR_ALL_HEIGHT
         } else {
            (screen.min.y + self.offset.y) + total_height + CLEAR_ALL_GAP
         };
         let pill_rect = Rect::from_min_size(
            Pos2::new(x, pill_top),
            Vec2::new(self.width, CLEAR_ALL_HEIGHT),
         );

         let area_id = Id::unique("egui_elements::toast::clear_all");
         let response = Area::new(area_id)
            .order(Order::Tooltip)
            .fixed_pos(pill_rect.min)
            .show(ctx, |ui| paint_clear_all(ui, &theme, pill_rect));

         if response.inner {
            for entry in state.queue.iter_mut() {
               if entry.dismiss_start.is_none() {
                  entry.dismiss_start = Some(now);
               }
            }
            any_animating = true;
         }
      }

      ctx.data_mut(|d| d.insert_temp(storage_id(), state));

      // Keep animating smoothly; otherwise schedule the next transition.
      if any_animating {
         ctx.request_repaint();
      } else if let Some(at) = earliest_next_event {
         let remaining = (at - now).max(0.0);
         ctx.request_repaint_after(Duration::from_secs_f64(remaining));
      }
   }
}

// -- internals ---------------------------------------------------------------

/// Result of painting a single toast for one frame.
struct ToastPaint {
   /// The close button was clicked this frame.
   close_clicked: bool,
   /// The pointer is currently over this toast (used to freeze the auto-dismiss
   /// timer so the user can read or copy the text).
   hovered: bool,
}

fn measure_height(ctx: &Context, theme: &Theme, entry: &ToastEntry, width: f32) -> f32 {
   use layout::*;

   let colors = &theme.colors;
   let typography = &theme.typography;

   // The same colours as the labels in `paint_toast` (fade is applied via
   // `ui.set_opacity`, never baked into the colour), so all three paths share
   // one cached galley.
   let text_width = text_wrap_width(width);
   let title_galley = ctx.fonts_mut(|fonts| {
      fonts.layout(
         entry.title.clone(),
         FontId::proportional(typography.normal),
         colors.text,
         text_width,
      )
   });

   let mut height = PAD_Y * 2.0 + title_galley.size().y;
   if let Some(description) = &entry.description {
      let description_galley = ctx.fonts_mut(|fonts| {
         fonts.layout(
            description.clone(),
            FontId::proportional(typography.small),
            colors.text_muted,
            text_width,
         )
      });
      height += TITLE_DESC_GAP + description_galley.size().y;
   }
   height.max(44.0)
}

/// Paint a single toast inside its area.
fn paint_toast(
   ui: &mut Ui,
   theme: &Theme,
   entry: &ToastEntry,
   rect: Rect,
   alpha: f32,
) -> ToastPaint {
   use layout::*;

   let colors = &theme.colors;
   let typography = &theme.typography;

   // Upgrade the Area's Ui role from `GenericContainer` to an ARIA live
   // region. Danger/Warning toasts are `Alert` (assertive — interrupts the
   // user); the rest are `Status` (polite — announced after current speech).
   let role = match entry.tone {
      ChipRole::Error | ChipRole::Warning => accesskit::Role::Alert,
      _ => accesskit::Role::Status,
   };
   let label = entry.title.clone();
   let description = entry.description.clone();
   ui.ctx().accesskit_node_builder(ui.unique_id(), |node| {
      node.set_role(role);
      node.set_label(label);
      if let Some(description) = description {
         node.set_description(description);
      }
   });

   // Apply the fade as a painter-level opacity multiplier instead of baking
   // alpha into every colour, so the text-layout cache key stays stable across
   // the fade (one cached galley for the whole fade, not one per frame).
   ui.set_opacity(alpha);

   // Claim the whole card so clicks do not pass through to widgets beneath.
   // `contains_pointer` (not `hovered`) so the child labels below cannot
   // shadow it while the cursor is over the text.
   let card_response = ui.allocate_rect(rect, Sense::hover());
   let hovered = card_response.contains_pointer();
   let painter = ui.painter();

   // Card: a floating surface, lifted by the theme's overlay shadow.
   let radius = CornerRadius::same(theme.corner_radius);
   painter.add(theme.shadow(Elevation::Overlay).as_shape(rect, radius));
   painter.rect(
      rect,
      radius,
      theme.surface_fill(Surface::Overlay),
      Stroke::new(1.0, colors.border),
      StrokeKind::Inside,
   );

   // Left accent bar, tinted by the tone.
   let bar_rect = Rect::from_min_max(
      Pos2::new(rect.min.x + 4.0, rect.min.y + 6.0),
      Pos2::new(rect.min.x + 4.0 + BAR_W, rect.max.y - 6.0),
   );
   painter.rect_filled(
      bar_rect,
      CornerRadius::same(2),
      tone_accent(theme, entry.tone),
   );

   // Close × in the top-right.
   let close_rect = Rect::from_min_size(
      Pos2::new(
         rect.max.x - PAD_X * 0.5 - CLOSE_W,
         rect.min.y + 6.0,
      ),
      Vec2::new(CLOSE_W, CLOSE_W),
   );
   let close_response: Response = ui.allocate_rect(close_rect, Sense::click());
   let close_color = if close_response.hovered() {
      colors.text
   } else {
      colors.text_muted
   };
   ui.painter().text(
      close_rect.center(),
      Align2::CENTER_CENTER,
      "×",
      FontId::proportional(typography.normal + 2.0),
      close_color,
   );
   close_response.widget_info(|| {
      WidgetInfo::labeled(
         Role::Button,
         close_response.enabled(),
         "Dismiss notification",
      )
   });

   // Text block: title + optional description, to the right of the bar. Rendered
   // as a selectable label so the contents can be drag-selected and copied (the
   // dismiss timer pauses while hovered).
   let text_left = rect.min.x + PAD_X + BAR_W + BAR_GAP - TEXT_LEFT_NUDGE;
   let text_width = text_wrap_width(rect.width());

   // Pre-layout to learn each line's height. The same FontId/colour/wrap as
   // `measure_height` and the label below, so all three share one cached galley.
   let title_galley = ui.ctx().fonts_mut(|fonts| {
      fonts.layout(
         entry.title.clone(),
         FontId::proportional(typography.normal),
         colors.text,
         text_width,
      )
   });
   let title_height = title_galley.size().y;
   let title_rect = Rect::from_min_size(
      Pos2::new(text_left, rect.min.y + PAD_Y),
      Vec2::new(text_width, title_height),
   );
   let title_text = RichText::new(&entry.title).color(colors.text).size(typography.normal);
   put_wrapped_label(ui, title_rect, title_text);

   if let Some(description) = &entry.description {
      let description_galley = ui.ctx().fonts_mut(|fonts| {
         fonts.layout(
            description.clone(),
            FontId::proportional(typography.small),
            colors.text_muted,
            text_width,
         )
      });
      let description_rect = Rect::from_min_size(
         Pos2::new(
            text_left,
            rect.min.y + PAD_Y + title_height + TITLE_DESC_GAP,
         ),
         Vec2::new(text_width, description_galley.size().y),
      );
      let description_text =
         RichText::new(description).color(colors.text_muted).size(typography.small);
      put_wrapped_label(ui, description_rect, description_text);
   }

   ToastPaint {
      close_clicked: close_response.clicked(),
      hovered,
   }
}

/// Place a wrapping, selectable label in `rect`.
///
/// Not `Ui::put`: that imposes `Layout::centered_and_justified`, which `Label`
/// copies into `LayoutJob::justify`. Justification stretches every row but the
/// last out to the full wrap width, so a row holding one long word (a path, a
/// URL) comes back letter-spaced.
fn put_wrapped_label(ui: &mut Ui, rect: Rect, text: RichText) {
   ui.scope_builder(
      UiBuilder::new().max_rect(rect).layout(Layout::top_down(Align::LEFT)),
      |ui| ui.add(egui::Label::new(text).selectable(true).wrap()),
   );
}

/// Paint the "Clear all" pill that sits above (or below) the stack when
/// [`Toasts::clear_all_button`] is enabled and at least
/// [`CLEAR_ALL_THRESHOLD`] non-dismissing entries are visible. Returns `true`
/// when it was clicked this frame.
fn paint_clear_all(ui: &mut Ui, theme: &Theme, rect: Rect) -> bool {
   let colors = &theme.colors;

   ui.ctx().accesskit_node_builder(ui.unique_id(), |node| {
      node.set_role(accesskit::Role::Button);
      node.set_label("Clear all notifications");
   });

   let response = ui.allocate_rect(rect, Sense::click());
   let painter = ui.painter();

   let surface = Surface::Overlay;
   let fill = if response.hovered() {
      theme.state_fill(surface, SurfaceState::Hover)
   } else {
      theme.surface_fill(surface)
   };
   painter.rect(
      rect,
      CornerRadius::same((rect.height() * 0.5).round() as u8),
      fill,
      Stroke::new(1.0, colors.border),
      StrokeKind::Inside,
   );

   let text_color = if response.hovered() {
      colors.text
   } else {
      colors.text_muted
   };
   painter.text(
      rect.center(),
      Align2::CENTER_CENTER,
      "Clear all",
      FontId::proportional(theme.typography.small),
      text_color,
   );

   response.clicked()
}

/// Layout constants shared between measurement and painting.
mod layout {
   pub const PAD_X: f32 = 14.0;
   pub const PAD_Y: f32 = 10.0;
   pub const BAR_W: f32 = 3.0;
   pub const BAR_GAP: f32 = 10.0;
   pub const TITLE_DESC_GAP: f32 = 3.0;
   pub const CLOSE_W: f32 = 18.0;
   pub const CLOSE_GAP: f32 = 8.0;
   pub const TEXT_LEFT_NUDGE: f32 = 4.0;

   /// Shared so `measure_height` and `paint_toast` wrap against the same width;
   /// otherwise the stack lays out against a height the paint path does not
   /// reproduce.
   pub fn text_wrap_width(card_width: f32) -> f32 {
      (card_width - PAD_X * 1.5 - BAR_W - BAR_GAP - CLOSE_W - CLOSE_GAP + TEXT_LEFT_NUDGE).max(1.0)
   }
}
