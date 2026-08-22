//! Per-widget visuals stored on [`Theme`](crate::theme::Theme) and optionally
//! overridden on a single widget via `.visuals(...)`.
//!
//! Custom widgets resolve visuals in this order: the override → values
//! written by [`Theme::install`](crate::theme::Theme::install) → stock
//! [`egui::Style`].

use egui::{Color32, CornerRadius, Response, Shadow, Stroke};

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

/// Hover / click overrides used by [`crate::utils::frame`].
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Default)]
pub struct FrameVisuals {
   /// Fill while hovered.
   pub bg_on_hover: Color32,
   /// Fill while the pointer is down.
   pub bg_on_click: Color32,
   /// `(width, color)` border while hovered.
   pub border_on_hover: (f32, Color32),
   /// `(width, color)` border while the pointer is down.
   pub border_on_click: (f32, Color32),
}

impl PartialEq for FrameVisuals {
   fn eq(&self, other: &Self) -> bool {
      self.bg_on_hover == other.bg_on_hover
         && self.bg_on_click == other.bg_on_click
         && self.border_on_hover == other.border_on_hover
         && self.border_on_click == other.border_on_click
   }
}

impl Eq for FrameVisuals {}
