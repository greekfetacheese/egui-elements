//! A themed rule, horizontal or vertical.

use crate::theme::Theme;
use egui::{Color32, Response, Sense, Stroke, Ui, Widget, pos2, vec2};

/// A one-pixel rule in the theme's border colour.
///
/// Use it inside a card, where egui's `ui.separator()` follows the layout direction and the
/// default spacing does not match the theme. The rule is inset from the start edge with
/// [`Divider::inset`], and [`Divider::vertical`] gives the short rule between toolbar items.
///
/// ```
/// # use egui::__run_test_ui;
/// # use egui_elements::widgets::Divider;
/// # __run_test_ui(|ui| {
/// ui.add(Divider::new());
/// ui.add(Divider::new().inset(8.0));
/// ui.add(Divider::vertical().length(14.0));
/// # });
/// ```
#[must_use = "You should put this widget in a ui with `ui.add(widget);`"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Divider {
   vertical: bool,
   inset: f32,
   thickness: f32,
   color: Option<Color32>,
   length: f32,
}

impl Divider {
   /// A horizontal rule spanning the available width.
   pub fn new() -> Self {
      Self {
         vertical: false,
         inset: 0.0,
         thickness: 1.0,
         color: None,
         length: 16.0,
      }
   }

   /// A vertical rule, [`Self::length`] tall.
   pub fn vertical() -> Self {
      Self {
         vertical: true,
         ..Self::new()
      }
   }

   /// Skip `inset` points at the start edge (left, or top when vertical).
   pub fn inset(mut self, inset: f32) -> Self {
      self.inset = inset;
      self
   }

   /// Line width in points. Default `1.0`.
   pub fn thickness(mut self, thickness: f32) -> Self {
      self.thickness = thickness;
      self
   }

   /// Override the theme border colour.
   pub fn color(mut self, color: Color32) -> Self {
      self.color = Some(color);
      self
   }

   /// Height of a vertical rule. Default `16.0`.
   pub fn length(mut self, length: f32) -> Self {
      self.length = length;
      self
   }
}

impl Default for Divider {
   fn default() -> Self {
      Self::new()
   }
}

impl Widget for Divider {
   fn ui(self, ui: &mut Ui) -> Response {
      let color = self.color.unwrap_or_else(|| Theme::colors_from_ctx(ui.ctx()).border);
      let size = if self.vertical {
         vec2(self.thickness, self.length)
      } else {
         vec2(ui.available_width(), self.thickness)
      };
      let (rect, response) = ui.allocate_exact_size(size, Sense::hover());

      if ui.is_rect_visible(rect) {
         let stroke = Stroke::new(self.thickness, color);
         let (start, end) = if self.vertical {
            let top = rect.top() + self.inset;
            (
               pos2(rect.center().x, top),
               pos2(rect.center().x, rect.bottom()),
            )
         } else {
            let left = rect.left() + self.inset;
            (
               pos2(left, rect.center().y),
               pos2(rect.right(), rect.center().y),
            )
         };
         if self.vertical {
            if end.y > start.y {
               ui.painter().line_segment([start, end], stroke);
            }
         } else if end.x > start.x {
            ui.painter().line_segment([start, end], stroke);
         }
      }
      response
   }
}

#[cfg(test)]
mod tests {
   use super::*;
   use egui::Rect;

   #[test]
   fn a_horizontal_divider_spans_the_available_width() {
      let mut available = 0.0;
      let mut rect = Rect::NOTHING;
      egui::__run_test_ui(|ui| {
         available = ui.available_width();
         rect = ui.add(Divider::new()).rect;
      });
      assert!(
         (rect.width() - available).abs() < 0.5,
         "{rect:?} {available}"
      );
      assert!((rect.height() - 1.0).abs() < 0.5, "{rect:?}");
   }

   #[test]
   fn a_vertical_divider_uses_its_length() {
      let mut rect = Rect::NOTHING;
      egui::__run_test_ui(|ui| {
         rect = ui.add(Divider::vertical().length(22.0)).rect;
      });
      assert!((rect.height() - 22.0).abs() < 0.5, "{rect:?}");
      assert!((rect.width() - 1.0).abs() < 0.5, "{rect:?}");
   }
}
