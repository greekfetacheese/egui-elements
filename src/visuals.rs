//! Per-widget visuals stored on [`Theme`](crate::theme::Theme) and optionally
//! overridden on a single widget via `.visuals(...)`.
//!
//! Custom widgets resolve visuals in this order: the override → values
//! written by [`Theme::install`](crate::theme::Theme::install) → stock
//! [`egui::Style`].

use crate::gradient::Fill;
use crate::theme::ThemeColors;
use crate::utils::mix;
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
   /// Idle fill: a solid colour or a gradient.
   pub bg: Fill,
   /// Fill while hovered.
   pub bg_hover: Fill,
   /// Fill while the pointer is down.
   pub bg_click: Fill,
   /// Fill when [`crate::widgets::Button::selected`] is set.
   pub bg_selected: Fill,
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
   pub fn bg_from_res(&self, res: &Response) -> Fill {
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

   /// This set filled with `fill`, and inked with whatever reads on it.
   ///
   /// [`Self::text`] becomes [`ThemeColors::ink_on`]`(fill)`, so a chromatic fill can never
   /// ship label copy under [`crate::utils::CONTRAST_AA_TEXT`]: on every built-in palette that
   /// ink is `bg` (10.6:1 on Tokyo Night's accent, 4.9:1 on Wasp's error). Hover and press mix
   /// the fill 16 % / 28 % toward [`ThemeColors::bg`], and every border clears. Corner radius and
   /// shadow are kept, so start from [`Theme::button_visuals`](crate::theme::Theme::button_visuals).
   pub fn filled(mut self, fill: Color32, colors: ThemeColors) -> Self {
      self.text = colors.ink_on(fill);
      self.bg = Fill::solid(fill);
      self.bg_hover = Fill::solid(mix(fill, colors.bg, 0.16));
      self.bg_click = Fill::solid(mix(fill, colors.bg, 0.28));
      self.bg_selected = Fill::solid(fill);
      self.border = Stroke::NONE;
      self.border_hover = Stroke::NONE;
      self.border_click = Stroke::NONE;
      self
   }
}

/// Visuals for a TextEdit
#[derive(Copy, Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextEditVisuals {
   /// Typed text color.
   pub text: Color32,
   /// Field fill: a solid colour or a gradient.
   pub bg: Fill,
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
   /// Idle fill of the closed button: a solid colour or a gradient.
   pub bg: Fill,
   /// Disclosure-icon color.
   pub icon: Color32,
   /// Fill while hovered.
   pub bg_hover: Fill,
   /// Fill while the popup is open.
   pub bg_open: Fill,
   /// Popup surface fill. Distinct from `bg` (which every palette aliases to the page),
   /// so the menu reads as a raised layer even before its shadow lands.
   ///
   /// Flat by design: an `egui::Frame` paints the menu, and it takes one colour. Widget fills
   /// take a [`Fill`].
   #[cfg_attr(feature = "serde", serde(default))]
   pub popup_bg: Color32,
   /// Popup border.
   #[cfg_attr(feature = "serde", serde(default))]
   pub popup_border: Stroke,
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
         && self.popup_bg == other.popup_bg
         && self.popup_border == other.popup_border
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
   pub fn bg_from_res(&self, res: &Response) -> Fill {
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
   pub bg: Fill,
   pub bg_hover: Fill,
   pub bg_click: Fill,
   /// Fill when [`crate::widgets::Frame::selected`] is set.
   pub bg_selected: Fill,
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
   pub fn bg_from_res(&self, res: &Response) -> Fill {
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
   pub fn paint_at(&self, painter: &Painter, rect: Rect, fill: Fill, stroke: Stroke) {
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
   fill: Fill,
   stroke: Stroke,
   corner_radius: CornerRadius,
   shadow: Shadow,
) -> Shape {
   if rect.width() <= 0.0 || rect.height() <= 0.0 {
      return Shape::Noop;
   }

   let has_fill = match fill {
      Fill::Solid(color) => color != Color32::TRANSPARENT,
      Fill::Gradient(_) => true,
   };

   let has_shadow = shadow != Shadow::NONE && shadow.color != Color32::TRANSPARENT;
   if !has_fill && stroke.is_empty() && !has_shadow {
      return Shape::Noop;
   }

   let mut shapes = Vec::new();

   if has_shadow {
      shapes.push(Shape::from(shadow.as_shape(rect, corner_radius)));
   }

   match fill {
      Fill::Solid(color) => {
         if has_fill || !stroke.is_empty() {
            shapes.push(Shape::Rect(RectShape::new(
               rect,
               corner_radius,
               color,
               stroke,
               StrokeKind::Inside,
            )));
         }
      }
      Fill::Gradient(gradient) => {
         shapes.push(gradient.shape_rounded(rect, corner_radius));

         if !stroke.is_empty() {
            shapes.push(Shape::Rect(RectShape::stroke(
               rect,
               corner_radius,
               stroke,
               StrokeKind::Inside,
            )));
         }
      }
   }

   match shapes.len() {
      0 => Shape::Noop,
      1 => shapes.remove(0),
      _ => Shape::Vec(shapes),
   }
}

#[cfg(test)]
mod tests {
   use crate::gradient::Fill;
   use crate::theme::{Theme, ThemeKind};
   use crate::utils::CONTRAST_AA_TEXT;

   #[test]
   fn filled_buttons_ink_reads_on_every_builtin_theme() {
      for kind in ThemeKind::to_vec() {
         let name = kind.to_str().to_owned();
         let theme = Theme::new(kind);
         let colors = theme.colors;

         for fill in [
            colors.accent,
            colors.error,
            colors.warning,
            colors.success,
            colors.info,
         ] {
            let visuals = theme.button_visuals().filled(fill, colors);
            let ratio = crate::utils::contrast_ratio(visuals.text, fill);
            assert!(
               ratio >= CONTRAST_AA_TEXT,
               "{name}: {ratio:.2}:1 on {fill:?}"
            );
            assert_eq!(visuals.text, colors.ink_on(fill));
            assert_eq!(visuals.bg, Fill::solid(fill));
            assert_eq!(visuals.bg_selected, Fill::solid(fill));
            assert!(
               visuals.border.is_empty(),
               "{name}: the fill needs no border"
            );
            assert_ne!(
               visuals.bg_hover,
               Fill::solid(fill),
               "{name}: hover must lift the fill"
            );
         }
      }
   }
}
