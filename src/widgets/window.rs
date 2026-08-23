//! [`Window`] — [`egui::Window`] with a title fill that stays put.
//!
//! Stock egui `title_ui` overwrites [`egui::Window::title_frame`] fill
//! with `visuals.widgets.open.weak_bg_fill` whenever the window is the top
//! layer. Opening a second window (Theme Editor on top of the demo window,
//! for example) flips that slot and the title bar changes color even if you
//! passed an explicit title frame.
//!
//! This wrapper forwards the stock builder API. On [`Window::show`] it pins
//! `widgets.open.weak_bg_fill` to the requested title fill so focused and
//! unfocused titles stay the same. The style is restored when `show` returns.

use egui::{
   Align2, Color32, Context, Frame, Id, InnerResponse, IntoAtoms, Order, Pos2, Rect, Resize, Ui,
   Vec2, Vec2b, ViewportBuilder, ViewportId, WindowDrag,
   scroll_area::{DragScroll, ScrollBarVisibility},
};

/// Builder for a floating window. Same API as [`egui::Window`].
///
/// The only behavior change: [`Self::title_frame`] fill is honored even when
/// this window is the top layer. Stock egui replaces that fill from
/// `widgets.open`.
///
/// ```
/// # use egui::__run_test_ctx;
/// # use egui_elements::widgets::Window;
/// # __run_test_ctx(|ctx| {
/// let frame = egui::Frame::window(&ctx.global_style());
/// Window::new("Demo")
///     .frame(frame)
///     .title_frame(frame.stroke(egui::Stroke::NONE))
///     .show(ctx, |ui| {
///         ui.label("hello");
///     });
/// # });
/// ```
#[must_use = "You should call .show()"]
pub struct Window<'a> {
   inner: egui::Window<'a>,
   /// Fill taken from [`Self::title_frame`]. `None` means stock default
   /// (`visuals.window_fill`).
   title_fill: Option<Color32>,
}

impl<'a> Window<'a> {
   /// The window title is used as a unique [`Id`] and must be unique, and
   /// should not change. See [`egui::Window::new`].
   pub fn new(title: impl IntoAtoms<'a>) -> Self {
      Self {
         inner: egui::Window::new(title),
         title_fill: None,
      }
   }

   /// Construct a [`Window`] that follows the given viewport.
   pub fn from_viewport(id: ViewportId, viewport: ViewportBuilder) -> Self {
      Self {
         inner: egui::Window::from_viewport(id, viewport),
         title_fill: None,
      }
   }

   /// Assign a unique id. Required if the title changes or is shared.
   #[inline]
   pub fn id(mut self, id: Id) -> Self {
      self.inner = self.inner.id(id);
      self
   }

   /// Add a close button. See [`egui::Window::open`].
   #[inline]
   pub fn open(mut self, open: &'a mut bool) -> Self {
      self.inner = self.inner.open(open);
      self
   }

   /// If `false` the window is grayed out and non-interactive.
   #[inline]
   pub fn enabled(mut self, enabled: bool) -> Self {
      self.inner = self.inner.enabled(enabled);
      self
   }

   /// If `false`, clicks go through to what is behind us. Default: `true`.
   #[inline]
   pub fn interactable(mut self, interactable: bool) -> Self {
      self.inner = self.inner.interactable(interactable);
      self
   }

   /// If `false` the window cannot be moved by dragging.
   #[inline]
   pub fn movable(mut self, movable: bool) -> Self {
      self.inner = self.inner.movable(movable);
      self
   }

   /// Where the user can grab the window to move it.
   #[inline]
   pub fn drag_area(mut self, drag_area: WindowDrag) -> Self {
      self.inner = self.inner.drag_area(drag_area);
      self
   }

   /// `order(Order::Foreground)` for a window that should always be on top.
   #[inline]
   pub fn order(mut self, order: Order) -> Self {
      self.inner = self.inner.order(order);
      self
   }

   /// Fade in when the window first appears. Default: `true`.
   #[inline]
   pub fn fade_in(mut self, fade_in: bool) -> Self {
      self.inner = self.inner.fade_in(fade_in);
      self
   }

   /// Fade out when the window closes via [`Self::open`]. Default: `true`.
   #[inline]
   pub fn fade_out(mut self, fade_out: bool) -> Self {
      self.inner = self.inner.fade_out(fade_out);
      self
   }

   /// Mutate the builder in place.
   #[inline]
   pub fn mutate(mut self, mutate: impl Fn(&mut Self)) -> Self {
      mutate(&mut self);
      self
   }

   /// Mutate the inner [`Resize`].
   #[inline]
   pub fn resize(mut self, mutate: impl Fn(Resize) -> Resize) -> Self {
      self.inner = self.inner.resize(mutate);
      self
   }

   /// Change the background color, margins, etc. of the window body.
   #[inline]
   pub fn frame(mut self, frame: Frame) -> Self {
      self.inner = self.inner.frame(frame);
      self
   }

   /// Change the background color, margins, etc. of the title bar.
   ///
   /// Unlike [`egui::Window::title_frame`], this fill is kept even when the
   /// window is the top layer.
   #[inline]
   pub fn title_frame(mut self, frame: Frame) -> Self {
      self.title_fill = Some(frame.fill);
      self.inner = self.inner.title_frame(frame);
      self
   }

   /// Set minimum width of the window.
   #[inline]
   pub fn min_width(mut self, min_width: f32) -> Self {
      self.inner = self.inner.min_width(min_width);
      self
   }

   /// Set minimum height of the window.
   #[inline]
   pub fn min_height(mut self, min_height: f32) -> Self {
      self.inner = self.inner.min_height(min_height);
      self
   }

   /// Set minimum outer size of the window.
   #[inline]
   pub fn min_size(mut self, min_size: impl Into<Vec2>) -> Self {
      self.inner = self.inner.min_size(min_size);
      self
   }

   /// Set maximum width of the window.
   #[inline]
   pub fn max_width(mut self, max_width: f32) -> Self {
      self.inner = self.inner.max_width(max_width);
      self
   }

   /// Set maximum height of the window.
   #[inline]
   pub fn max_height(mut self, max_height: f32) -> Self {
      self.inner = self.inner.max_height(max_height);
      self
   }

   /// Set maximum outer size of the window.
   #[inline]
   pub fn max_size(mut self, max_size: impl Into<Vec2>) -> Self {
      self.inner = self.inner.max_size(max_size);
      self
   }

   /// Set current position of the window.
   #[inline]
   pub fn current_pos(mut self, current_pos: impl Into<Pos2>) -> Self {
      self.inner = self.inner.current_pos(current_pos);
      self
   }

   /// Set initial position of the window.
   #[inline]
   pub fn default_pos(mut self, default_pos: impl Into<Pos2>) -> Self {
      self.inner = self.inner.default_pos(default_pos);
      self
   }

   /// Sets the window position and prevents it from being dragged.
   #[inline]
   pub fn fixed_pos(mut self, pos: impl Into<Pos2>) -> Self {
      self.inner = self.inner.fixed_pos(pos);
      self
   }

   /// Constrains this window to [`Context::content_rect`]. Default: `true`.
   #[inline]
   pub fn constrain(mut self, constrain: bool) -> Self {
      self.inner = self.inner.constrain(constrain);
      self
   }

   /// Constrain movement of the window to the given rectangle.
   #[inline]
   pub fn constrain_to(mut self, constrain_rect: Rect) -> Self {
      self.inner = self.inner.constrain_to(constrain_rect);
      self
   }

   /// Where the "root" of the window is. Default: [`Align2::LEFT_TOP`].
   #[inline]
   pub fn pivot(mut self, pivot: Align2) -> Self {
      self.inner = self.inner.pivot(pivot);
      self
   }

   /// Set anchor and distance. Also makes the window immovable.
   #[inline]
   pub fn anchor(mut self, align: Align2, offset: impl Into<Vec2>) -> Self {
      self.inner = self.inner.anchor(align, offset);
      self
   }

   /// Set initial collapsed state of the window.
   #[inline]
   pub fn default_open(mut self, default_open: bool) -> Self {
      self.inner = self.inner.default_open(default_open);
      self
   }

   /// Set initial outer size of the window.
   #[inline]
   pub fn default_size(mut self, default_size: impl Into<Vec2>) -> Self {
      self.inner = self.inner.default_size(default_size);
      self
   }

   /// Set initial width of the window.
   #[inline]
   pub fn default_width(mut self, default_width: f32) -> Self {
      self.inner = self.inner.default_width(default_width);
      self
   }

   /// Set initial height of the window.
   #[inline]
   pub fn default_height(mut self, default_height: f32) -> Self {
      self.inner = self.inner.default_height(default_height);
      self
   }

   /// Sets the window size and prevents resizing by dragging edges.
   #[inline]
   pub fn fixed_size(mut self, size: impl Into<Vec2>) -> Self {
      self.inner = self.inner.fixed_size(size);
      self
   }

   /// Set initial position and size of the window.
   #[inline]
   pub fn default_rect(self, rect: Rect) -> Self {
      self.default_pos(rect.min).default_size(rect.size())
   }

   /// Sets pos and size and prevents moving and resizing by dragging.
   #[inline]
   pub fn fixed_rect(self, rect: Rect) -> Self {
      self.fixed_pos(rect.min).fixed_size(rect.size())
   }

   /// Can the user resize the window by dragging its edges? Default: `true`.
   #[inline]
   pub fn resizable(mut self, resizable: impl Into<Vec2b>) -> Self {
      self.inner = self.inner.resizable(resizable);
      self
   }

   /// Can the window be collapsed by clicking on its title?
   #[inline]
   pub fn collapsible(mut self, collapsible: bool) -> Self {
      self.inner = self.inner.collapsible(collapsible);
      self
   }

   /// Show a title bar? If `false`, the window is not collapsible and has no
   /// close button.
   #[inline]
   pub fn title_bar(mut self, title_bar: bool) -> Self {
      self.inner = self.inner.title_bar(title_bar);
      self
   }

   /// Not resizable, just takes the size of its contents. Also disables
   /// scrolling.
   #[inline]
   pub fn auto_sized(mut self) -> Self {
      self.inner = self.inner.auto_sized();
      self
   }

   /// Enable/disable horizontal/vertical scrolling. `false` by default.
   #[inline]
   pub fn scroll(mut self, scroll: impl Into<Vec2b>) -> Self {
      self.inner = self.inner.scroll(scroll);
      self
   }

   /// Enable/disable horizontal scrolling. `false` by default.
   #[inline]
   pub fn hscroll(mut self, hscroll: bool) -> Self {
      self.inner = self.inner.hscroll(hscroll);
      self
   }

   /// Enable/disable vertical scrolling. `false` by default.
   #[inline]
   pub fn vscroll(mut self, vscroll: bool) -> Self {
      self.inner = self.inner.vscroll(vscroll);
      self
   }

   /// Scroll by dragging the contents with the pointer.
   #[inline]
   pub fn drag_to_scroll(mut self, drag_to_scroll: DragScroll) -> Self {
      self.inner = self.inner.drag_to_scroll(drag_to_scroll);
      self
   }

   /// Sets the [`ScrollBarVisibility`] of the window.
   #[inline]
   pub fn scroll_bar_visibility(mut self, visibility: ScrollBarVisibility) -> Self {
      self.inner = self.inner.scroll_bar_visibility(visibility);
      self
   }

   /// Show the window.
   ///
   /// Returns `None` if the window is not open (if [`Self::open`] was called
   /// with `&mut false`). Returns `Some(InnerResponse { inner: None })` if
   /// the window is collapsed.
   #[inline]
   pub fn show<R>(
      self,
      ctx: &Context,
      add_contents: impl FnOnce(&mut Ui) -> R,
   ) -> Option<InnerResponse<Option<R>>> {
      let fill = self.title_fill.unwrap_or_else(|| ctx.global_style().visuals.window_fill);
      let _guard = PinOpenWeakFill::new(ctx, fill);
      self.inner.show(ctx, add_contents)
   }
}

/// Pins `widgets.open.weak_bg_fill` for the duration of [`Window::show`].
///
/// Stock `title_ui` reads that slot when the window is on top. Restoring on
/// drop keeps Theme Editor color pickers / stock combos from inheriting the
/// title fill after the window returns.
struct PinOpenWeakFill {
   ctx: Context,
   previous: Color32,
}

impl PinOpenWeakFill {
   fn new(ctx: &Context, fill: Color32) -> Self {
      let previous = ctx.global_style().visuals.widgets.open.weak_bg_fill;
      ctx.global_style_mut(|style| {
         style.visuals.widgets.open.weak_bg_fill = fill;
      });
      Self {
         ctx: ctx.clone(),
         previous,
      }
   }
}

impl Drop for PinOpenWeakFill {
   fn drop(&mut self) {
      self.ctx.global_style_mut(|style| {
         style.visuals.widgets.open.weak_bg_fill = self.previous;
      });
   }
}

#[cfg(test)]
mod tests {
   use super::*;

   #[test]
   fn title_frame_fill_is_pinned_during_show_then_restored() {
      egui::__run_test_ctx(|ctx| {
         let original = Color32::from_rgb(9, 9, 9);
         ctx.global_style_mut(|style| {
            style.visuals.widgets.open.weak_bg_fill = original;
         });

         let title_fill = Color32::from_rgb(1, 2, 3);
         Window::new("title").title_frame(Frame::NONE.fill(title_fill)).show(ctx, |ui| {
            assert_eq!(
               ui.ctx().global_style().visuals.widgets.open.weak_bg_fill,
               title_fill
            );
         });

         assert_eq!(
            ctx.global_style().visuals.widgets.open.weak_bg_fill,
            original
         );
      });
   }
}
