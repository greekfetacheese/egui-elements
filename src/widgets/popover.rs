//! An anchored, themed popover: a picker, a menu or a small inspector.
//!
//! The Telegram demo's emoji picker, attach menu and contact popover each hand-rolled the same
//! four things: anchoring a panel to a button, a shadowed overlay surface, close-on-outside-click
//! and close-on-Escape. `Popover` is that contract in the crate. The caller owns the open flag and
//! the anchor; the popover owns the overlay frame ([Surface::Overlay], the theme's window frame),
//! the dismissal rules and the placement.
//!
//! ```
//! # use egui::{pos2, vec2, Rect, __run_test_ui};
//! # use egui_elements::widgets::Popover;
//! # __run_test_ui(|ui| {
//! let mut open = true;
//! let anchor = Rect::from_min_size(pos2(12.0, 12.0), vec2(96.0, 32.0));
//! Popover::new("picker", &mut open).width(220.0).show(ui.ctx(), anchor, |ui| {
//!    ui.label("Item");
//! });
//! # });
//! ```
//!
//! The popover never flips itself: pick the side and let `constrain` keep it on screen. A host
//! that needs auto-placement should compute the side from a screen-space layout it already owns.

use std::hash::Hash;

use egui::{Align2, Area, Context, Id, IdSalt, InnerResponse, Key, Order, Pos2, Rect, Ui, vec2};

use crate::theme::{Surface, Theme};
use crate::widgets::Frame;

/// Which side of the anchor a [`Popover`] opens on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PopoverSide {
   /// Under the anchor, top-left corners aligned. The default.
   #[default]
   Below,
   /// Over the anchor, bottom-left corners aligned.
   Above,
   /// Right of the anchor, top-left corners aligned.
   Right,
   /// Left of the anchor, top-right corners aligned.
   Left,
}

impl PopoverSide {
   /// The area pivot and the point it sits on.
   fn placement(self, anchor: Rect, gap: f32) -> (Align2, Pos2) {
      match self {
         Self::Below => (
            Align2::LEFT_TOP,
            anchor.left_bottom() + vec2(0.0, gap),
         ),
         Self::Above => (
            Align2::LEFT_BOTTOM,
            anchor.left_top() + vec2(0.0, -gap),
         ),
         Self::Right => (
            Align2::LEFT_TOP,
            anchor.right_top() + vec2(gap, 0.0),
         ),
         Self::Left => (
            Align2::RIGHT_TOP,
            anchor.left_top() + vec2(-gap, 0.0),
         ),
      }
   }
}

/// An overlay panel anchored to a [`Rect`].
#[must_use = "You should call .show()"]
pub struct Popover<'a> {
   id_salt: IdSalt,
   open: &'a mut bool,
   side: PopoverSide,
   gap: f32,
   width: Option<f32>,
   close_on_click_outside: bool,
   close_on_escape: bool,
   frame: Option<Frame>,
   order: Order,
}

impl<'a> Popover<'a> {
   /// A popover anchored by `anchor` on its [`PopoverSide::Below`] side.
   pub fn new(id_salt: impl Hash + std::fmt::Debug, open: &'a mut bool) -> Self {
      Self {
         id_salt: IdSalt::new(id_salt),
         open,
         side: PopoverSide::Below,
         gap: 6.0,
         width: None,
         close_on_click_outside: true,
         close_on_escape: true,
         frame: None,
         order: Order::Foreground,
      }
   }

   /// Which side of the anchor to open on. Default [`PopoverSide::Below`].
   pub fn side(mut self, side: PopoverSide) -> Self {
      self.side = side;
      self
   }

   /// Distance between the anchor and the card, in points. Default `6.0`.
   pub fn gap(mut self, gap: f32) -> Self {
      self.gap = gap;
      self
   }

   /// Card width in points. Default: the content's natural width.
   ///
   /// The width includes the card's margins; content is laid out inside them.
   pub fn width(mut self, width: f32) -> Self {
      self.width = Some(width);
      self
   }

   /// Close when a click lands outside both the card and the anchor. Default `true`.
   ///
   /// The click that opens the popover sits on the anchor, so it never closes it on the same
   /// frame.
   pub fn close_on_click_outside(mut self, close: bool) -> Self {
      self.close_on_click_outside = close;
      self
   }

   /// Close on `Esc`. Default `true`.
   pub fn close_on_escape(mut self, close: bool) -> Self {
      self.close_on_escape = close;
      self
   }

   /// Override the card frame. Default: [`Theme::surface_frame`] for [`Surface::Overlay`].
   pub fn frame(mut self, frame: Frame) -> Self {
      self.frame = Some(frame);
      self
   }

   /// Layer to draw on. Default [`Order::Foreground`].
   pub fn order(mut self, order: Order) -> Self {
      self.order = order;
      self
   }

   /// Show the popover. `anchor` is the host widget's rect.
   ///
   /// Returns `None` when the popover is closed, so a caller can keep the body out of the frame.
   pub fn show<R>(
      self,
      ctx: &Context,
      anchor: Rect,
      add: impl FnOnce(&mut Ui) -> R,
   ) -> Option<InnerResponse<R>> {
      if !*self.open {
         return None;
      }

      let (pivot, pos) = self.side.placement(anchor, self.gap);
      let id = Id::unique(self.id_salt);
      let frame = self
         .frame
         .unwrap_or_else(|| Frame::from_egui(Theme::current(ctx).surface_frame(Surface::Overlay)));

      let area = Area::new(id)
         .order(self.order)
         .pivot(pivot)
         .fixed_pos(pos)
         .constrain(true)
         .show(ctx, |ui| {
            if let Some(width) = self.width {
               ui.set_width(width);
            }
            frame.show(ui, add).inner
         });

      let clicked_outside = self.close_on_click_outside
         && ctx.input(|input| {
            input.pointer.any_click()
               && input
                  .pointer
                  .interact_pos()
                  .is_some_and(|pos| !area.response.rect.contains(pos) && !anchor.contains(pos))
         });
      let escaped = self.close_on_escape && ctx.input(|input| input.key_pressed(Key::Escape));

      if clicked_outside || escaped {
         *self.open = false;
      }

      Some(InnerResponse::new(area.inner, area.response))
   }
}

#[cfg(test)]
mod tests {
   use super::*;
   use egui::{Event, Modifiers, RawInput, pos2};

   fn anchor() -> Rect {
      Rect::from_min_size(pos2(12.0, 12.0), egui::vec2(96.0, 32.0))
   }

   #[test]
   fn a_closed_popover_draws_nothing() {
      egui::__run_test_ui(|ui| {
         let mut open = false;
         let mut drawn = false;
         let shown = Popover::new("closed", &mut open).show(ui.ctx(), anchor(), |_ui| {
            drawn = true;
         });
         assert!(shown.is_none());
         assert!(!drawn);
      });
   }

   #[test]
   fn an_open_popover_draws_and_keeps_its_flag() {
      egui::__run_test_ui(|ui| {
         let mut open = true;
         let mut drawn = false;
         let shown = Popover::new("open", &mut open).show(ui.ctx(), anchor(), |_ui| {
            drawn = true;
         });
         assert!(shown.is_some());
         assert!(drawn);
         assert!(open);
      });
   }

   #[test]
   fn escape_closes_it() {
      let ctx = Context::default();
      let mut open = true;
      let mut input = RawInput::default();
      input.events.push(Event::Key {
         key: Key::Escape,
         physical_key: None,
         pressed: true,
         repeat: false,
         modifiers: Modifiers::NONE,
      });

      let mut output = ctx.run_ui(input, |ctx| {
         Popover::new("escape", &mut open).show(ctx, anchor(), |_ui| {});
      });
      output.textures_delta.clear();

      assert!(!open);
   }
}
