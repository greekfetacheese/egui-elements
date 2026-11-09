//! An anchored, themed popover: a picker, a menu or a small inspector.
//!
//! A picker, a menu or a small inspector anchored to a button hand-rolls the same four things:
//! anchoring a panel to the button, a shadowed overlay surface, close-on-outside-click and
//! close-on-Escape. `Popover` is that contract in the crate. The caller owns the open flag and the
//! anchor; the popover owns the overlay frame ([Surface::Overlay], the theme's window frame), the
//! dismissal rules and the placement.
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
//! When the open flag lives on your app struct, copy it out for the call and write it back:
//!
//! ```ignore
//! let mut open = app.emoji_open;
//! Popover::new("emoji", &mut open).show(ctx, anchor, |ui| {
//!    // `&mut app.composer` is free here: the flag is a local.
//! });
//! app.emoji_open = open;
//! ```
//!
//! The popover never flips itself: pick the side and let `constrain` keep it on screen. A host
//! that needs auto-placement should compute the side from a screen-space layout it already owns.

use std::hash::Hash;

use egui::{
   Align, Align2, Area, Context, Id, IdSalt, InnerResponse, Key, Margin, Order, Pos2, Rect, Ui,
   pos2,
};

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
   ///
   /// `align` is the cross-axis alignment: its x for [`Below`](Self::Below) /
   /// [`Above`](Self::Above), its y for [`Right`](Self::Right) / [`Left`](Self::Left). The side itself
   /// pins the other axis, so a card can hug its anchor's right edge without flipping.
   fn placement(self, align: Align2, anchor: Rect, gap: f32) -> (Align2, Pos2) {
      match self {
         Self::Below => (
            Align2::new(align.x(), Align::Max),
            pos2(x_at(anchor, align.x()), anchor.bottom() + gap),
         ),
         Self::Above => (
            Align2::new(align.x(), Align::Min),
            pos2(x_at(anchor, align.x()), anchor.top() - gap),
         ),
         Self::Right => (
            Align2::new(Align::Min, align.y()),
            pos2(anchor.right() + gap, y_at(anchor, align.y())),
         ),
         Self::Left => (
            Align2::new(Align::Max, align.y()),
            pos2(anchor.left() - gap, y_at(anchor, align.y())),
         ),
      }
   }
}

/// The x coordinate of `rect` at `align`.
fn x_at(rect: Rect, align: Align) -> f32 {
   match align {
      Align::Min => rect.left(),
      Align::Center => rect.center().x,
      Align::Max => rect.right(),
   }
}

/// The y coordinate of `rect` at `align`.
fn y_at(rect: Rect, align: Align) -> f32 {
   match align {
      Align::Min => rect.top(),
      Align::Center => rect.center().y,
      Align::Max => rect.bottom(),
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
   surface: Option<Surface>,
   inner_margin: Option<Margin>,
   align: Align2,
   screen_margin: f32,
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
         surface: None,
         inner_margin: None,
         align: Align2::LEFT_TOP,
         screen_margin: 0.0,
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

   /// Card surface from the installed theme. Default [`Surface::Overlay`].
   ///
   /// The surface's frame carries that surface's margins and shadow; override the margin with
   /// [`inner_margin`](Self::inner_margin). Ignored when [`frame`](Self::frame) is set.
   pub fn surface(mut self, surface: Surface) -> Self {
      self.surface = Some(surface);
      self
   }

   /// Override the card's inner margin, on top of the surface frame's own.
   ///
   /// [`Surface::Overlay`] uses the window frame, whose 10 pt margin silently grows a picker that
   /// owns its padding; pass [`Margin::ZERO`](egui::Margin::ZERO) for those.
   pub fn inner_margin(mut self, inner_margin: impl Into<Margin>) -> Self {
      self.inner_margin = Some(inner_margin.into());
      self
   }

   /// Cross-axis alignment against the anchor. Default [`Align2::LEFT_TOP`].
   ///
   /// Only the cross axis is read: `.align(Align2::RIGHT_BOTTOM)` right-aligns a
   /// [`Below`](PopoverSide::Below) / [`Above`](PopoverSide::Above) card and bottom-aligns a
   /// [`Right`](PopoverSide::Right) / [`Left`](PopoverSide::Left) one.
   pub fn align(mut self, align: Align2) -> Self {
      self.align = align;
      self
   }

   /// Keep the card this far inside [`Context::content_rect`]. Default `0.0`.
   ///
   /// The default still constrains to the content rect; a positive margin insets that bound by
   /// the same amount on every side.
   pub fn screen_margin(mut self, margin: f32) -> Self {
      self.screen_margin = margin;
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

      let (pivot, pos) = self.side.placement(self.align, anchor, self.gap);
      let id = Id::unique(self.id_salt);
      let theme = Theme::current(ctx);
      let mut frame = match self.frame {
         Some(frame) => frame,
         None => Frame::from_egui(theme.surface_frame(self.surface.unwrap_or(Surface::Overlay))),
      };
      if let Some(margin) = self.inner_margin {
         frame = frame.inner_margin(margin);
      }

      let mut area = Area::new(id).order(self.order).pivot(pivot).fixed_pos(pos);
      area = if self.screen_margin > 0.0 {
         area.constrain_to(ctx.content_rect().shrink(self.screen_margin))
      } else {
         area.constrain(true)
      };

      let area = area.show(ctx, |ui| {
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
   use egui::{Event, Modifiers, RawInput, pos2, vec2};

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

   #[test]
   fn right_alignment_hugs_the_anchors_right_edge() {
      let ctx = Context::default();
      // Near the right edge, so the card has room on its left and constrain leaves it alone.
      let anchor = Rect::from_min_size(
         pos2(ctx.content_rect().right() - 180.0, 100.0),
         vec2(96.0, 32.0),
      );
      let mut open = true;
      let mut rect = None;
      // An `Area` knows its size only after the first pass; the pivot lands on the second.
      for _ in 0..2 {
         let mut output = ctx.run_ui(RawInput::default(), |ctx| {
            shown_rect(
               Popover::new("align", &mut open)
                  .side(PopoverSide::Above)
                  .align(Align2::RIGHT_BOTTOM)
                  .width(140.0)
                  .show(ctx, anchor, |ui| {
                     ui.label("x");
                  }),
               &mut rect,
            );
         });
         output.textures_delta.clear();
      }

      assert!((rect.expect("open").right() - anchor.right()).abs() < 0.5);
   }

   /// Record a shown popover's rect, if it drew.
   fn shown_rect(shown: Option<InnerResponse<()>>, rect: &mut Option<Rect>) {
      if let Some(shown) = shown {
         *rect = Some(shown.response.rect);
      }
   }

   #[test]
   fn screen_margin_insets_the_constraint() {
      egui::__run_test_ui(|ui| {
         let screen = ui.ctx().content_rect();
         let anchor = Rect::from_min_size(
            pos2(screen.right() - 48.0, 80.0),
            vec2(36.0, 32.0),
         );
         let mut open = true;
         let shown = Popover::new("margin", &mut open)
            .width(320.0)
            .screen_margin(12.0)
            .show(ui.ctx(), anchor, |ui| {
               ui.label("wide");
            })
            .expect("open");

         assert!(shown.response.rect.right() <= screen.right() - 12.0 + 0.5);
      });
   }

   #[test]
   fn inner_margin_overrides_the_surface_frame() {
      egui::__run_test_ui(|ui| {
         let mut tight = true;
         let tight = Popover::new("tight", &mut tight)
            .inner_margin(Margin::ZERO)
            .show(ui.ctx(), anchor(), |ui| {
               ui.label("x");
            })
            .expect("open");
         let mut loose = true;
         let loose = Popover::new("loose", &mut loose)
            .show(
               ui.ctx(),
               Rect::from_min_size(pos2(12.0, 80.0), vec2(96.0, 32.0)),
               |ui| {
                  ui.label("x");
               },
            )
            .expect("open");

         assert!(tight.response.rect.height() < loose.response.rect.height());
      });
   }
}
