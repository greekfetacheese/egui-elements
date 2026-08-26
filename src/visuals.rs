//! Per-widget visuals stored on [`Theme`](crate::theme::Theme) and optionally
//! overridden on a single widget via `.visuals(...)`.
//!
//! Custom widgets resolve visuals in this order: the override → values
//! written by [`Theme::install`](crate::theme::Theme::install) → stock
//! [`egui::Style`].

use egui::{
   Color32, CornerRadius, Painter, Rect, Response, Shadow, Shape, Stroke, StrokeKind,
   epaint::RectShape,
};

/// Alias: labels use the same visuals as buttons.
pub type LabelVisuals = ButtonVisuals;

/// Visuals for a button
#[derive(Copy, Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ButtonVisuals {
   /// Glyph / label color.
   pub text: Color32,
   /// Idle fill.
   pub bg: Color32,
   /// Fill while hovered.
   pub bg_hover: Color32,
   /// Fill while the pointer is down.
   pub bg_click: Color32,
   /// Fill when [`crate::widgets::Button::selected`] is set.
   pub bg_selected: Color32,
   /// Idle border.
   pub border: Stroke,
   /// Border while hovered.
   pub border_hover: Stroke,
   /// Border while the pointer is down.
   pub border_click: Stroke,
   pub corner_radius: CornerRadius,
   pub shadow: Shadow,
}

impl PartialEq for ButtonVisuals {
   fn eq(&self, other: &Self) -> bool {
      self.text == other.text
         && self.bg == other.bg
         && self.bg_hover == other.bg_hover
         && self.bg_click == other.bg_click
         && self.bg_selected == other.bg_selected
         && self.border == other.border
         && self.border_hover == other.border_hover
         && self.border_click == other.border_click
         && self.corner_radius == other.corner_radius
         && self.shadow == other.shadow
   }
}

impl Eq for ButtonVisuals {}

impl ButtonVisuals {
   /// Fill for the current interaction state (click > hover > idle).
   pub fn bg_from_res(&self, res: &Response) -> Color32 {
      if res.is_pointer_button_down_on() || res.has_focus() || res.clicked() {
         self.bg_click
      } else if res.hovered() || res.highlighted() {
         self.bg_hover
      } else {
         self.bg
      }
   }

   /// Border for the current interaction state (click > hover > idle).
   pub fn border_from_res(&self, res: &Response) -> Stroke {
      if res.is_pointer_button_down_on() || res.has_focus() || res.clicked() {
         self.border_click
      } else if res.hovered() || res.highlighted() {
         self.border_hover
      } else {
         self.border
      }
   }
}

/// Visuals for a TextEdit
#[derive(Copy, Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextEditVisuals {
   /// Typed text color.
   pub text: Color32,
   /// Field fill.
   pub bg: Color32,
   /// Idle border.
   pub border: Stroke,
   /// Border while hovered.
   pub border_hover: Stroke,
   /// Border while focused / open.
   pub border_open: Stroke,
   pub corner_radius: CornerRadius,
   pub shadow: Shadow,
}

impl PartialEq for TextEditVisuals {
   fn eq(&self, other: &Self) -> bool {
      self.text == other.text
         && self.bg == other.bg
         && self.border == other.border
         && self.border_hover == other.border_hover
         && self.border_open == other.border_open
         && self.corner_radius == other.corner_radius
         && self.shadow == other.shadow
   }
}

impl Eq for TextEditVisuals {}

impl TextEditVisuals {
   /// Border for the current interaction state (open > hover > idle).
   pub fn border_from_res(&self, res: &Response) -> Stroke {
      if res.is_pointer_button_down_on() || res.has_focus() || res.clicked() {
         self.border_open
      } else if res.hovered() || res.highlighted() {
         self.border_hover
      } else {
         self.border
      }
   }
}

/// Visuals for a ComboBox
#[derive(Copy, Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ComboBoxVisuals {
   /// Idle fill of the closed button.
   pub bg: Color32,
   /// Disclosure-icon color.
   pub icon: Color32,
   /// Fill while hovered.
   pub bg_hover: Color32,
   /// Fill while the popup is open.
   pub bg_open: Color32,
   /// Idle border.
   pub border: Stroke,
   /// Border while hovered.
   pub border_hover: Stroke,
   /// Border while the popup is open.
   pub border_open: Stroke,
   pub corner_radius: CornerRadius,
   pub shadow: Shadow,
}

impl PartialEq for ComboBoxVisuals {
   fn eq(&self, other: &Self) -> bool {
      self.bg == other.bg
         && self.icon == other.icon
         && self.bg_hover == other.bg_hover
         && self.bg_open == other.bg_open
         && self.border == other.border
         && self.border_hover == other.border_hover
         && self.border_open == other.border_open
         && self.corner_radius == other.corner_radius
         && self.shadow == other.shadow
   }
}

impl Eq for ComboBoxVisuals {}

impl ComboBoxVisuals {
   /// Fill for the current interaction state (open > hover > idle).
   pub fn bg_from_res(&self, res: &Response) -> Color32 {
      if res.is_pointer_button_down_on() || res.has_focus() || res.clicked() {
         self.bg_open
      } else if res.hovered() || res.highlighted() {
         self.bg_hover
      } else {
         self.bg
      }
   }

   /// Border for the current interaction state (open > hover > idle).
   pub fn border_from_res(&self, res: &Response) -> Stroke {
      if res.is_pointer_button_down_on() || res.has_focus() || res.clicked() {
         self.border_open
      } else if res.hovered() || res.highlighted() {
         self.border_hover
      } else {
         self.border
      }
   }
}

/// Visuals for Frame
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Default)]
pub struct FrameVisuals {
   pub bg: Color32,
   pub bg_hover: Color32,
   pub bg_click: Color32,
   /// Fill when [`crate::widgets::Frame::selected`] is set.
   pub bg_selected: Color32,
   pub border: Stroke,
   pub border_hover: Stroke,
   pub border_click: Stroke,
   pub corner_radius: CornerRadius,
   pub shadow: Shadow,
}

impl PartialEq for FrameVisuals {
   fn eq(&self, other: &Self) -> bool {
      self.bg == other.bg
         && self.bg_hover == other.bg_hover
         && self.bg_click == other.bg_click
         && self.bg_selected == other.bg_selected
         && self.border == other.border
         && self.border_hover == other.border_hover
         && self.border_click == other.border_click
         && self.corner_radius == other.corner_radius
         && self.shadow == other.shadow
   }
}

impl Eq for FrameVisuals {}

impl FrameVisuals {
   pub fn bg_from_res(&self, res: &Response) -> Color32 {
      if res.is_pointer_button_down_on() || res.has_focus() || res.clicked() {
         self.bg_click
      } else if res.contains_pointer() || res.hovered() || res.highlighted() {
         self.bg_hover
      } else {
         self.bg
      }
   }

   pub fn border_from_res(&self, res: &Response) -> Stroke {
      if res.is_pointer_button_down_on() || res.has_focus() || res.clicked() {
         self.border_click
      } else if res.contains_pointer() || res.hovered() || res.highlighted() {
         self.border_hover
      } else {
         self.border
      }
   }

   /// Shadow + fill + inside stroke on an already allocated widget rect.
   pub fn paint_at(&self, painter: &Painter, rect: Rect, fill: Color32, stroke: Stroke) {
      painter.add(paint_shape(
         rect,
         fill,
         stroke,
         self.corner_radius,
         self.shadow,
      ));
   }
}

pub(crate) fn paint_shape(
   rect: Rect,
   fill: Color32,
   stroke: Stroke,
   corner_radius: CornerRadius,
   shadow: Shadow,
) -> Shape {
   if rect.width() <= 0.0 || rect.height() <= 0.0 {
      return Shape::Noop;
   }

   let has_fill = fill != Color32::TRANSPARENT;

   let has_shadow = shadow != Shadow::NONE && shadow.color != Color32::TRANSPARENT;
   if !has_fill && stroke.is_empty() && !has_shadow {
      return Shape::Noop;
   }

   let mut shapes = Vec::new();

   if has_shadow {
      shapes.push(Shape::from(shadow.as_shape(rect, corner_radius)));
   }

   if has_fill || !stroke.is_empty() {
      shapes.push(Shape::Rect(RectShape::new(
         rect,
         corner_radius,
         fill,
         stroke,
         StrokeKind::Inside,
      )));
   }

   match shapes.len() {
      0 => Shape::Noop,
      1 => shapes.remove(0),
      _ => Shape::Vec(shapes),
   }
}

#[cfg(test)]
mod tests {
   use super::*;

   #[test]
   fn paint_shape_draws_fill_without_stroke() {
      let shape = paint_shape(
         Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(10.0, 10.0)),
         Color32::RED,
         Stroke::NONE,
         CornerRadius::ZERO,
         Shadow::NONE,
      );
      match shape {
         Shape::Rect(rect) => assert_eq!(rect.fill, Color32::RED),
         other => panic!("expected filled rect, got {other:?}"),
      }
   }
}
