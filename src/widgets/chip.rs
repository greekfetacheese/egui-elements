//! Compact label for tags, filters, and statuses.
//!
//! A [`Chip`] sizes itself to its text and stays a pill at any length, unlike
//! [`Badge`](crate::widgets::Badge), whose circle grows with the label. The fill comes from
//! [`ThemeColors::tint`](crate::theme::ThemeColors::tint), so every [`ChipRole`] keeps AA ink
//! without per-theme tuning.

use crate::theme::{Theme, ThemeColors};
use egui::{
   Color32, Response, Sense, Stroke, StrokeKind, TextStyle, TextWrapMode, Ui, Vec2, Widget,
   WidgetText, epaint::RectShape, pos2, vec2,
};

/// Semantic colour of a [`Chip`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChipRole {
   /// Muted grey for metadata that carries no severity.
   #[default]
   Neutral,
   /// The theme accent, for the primary tag.
   Accent,
   /// Informational tag.
   Info,
   /// Positive status.
   Success,
   /// Caution.
   Warning,
   /// Destructive or failed status.
   Error,
}

impl ChipRole {
   /// Palette colour this role reads from.
   pub fn color(self, colors: &ThemeColors) -> Color32 {
      match self {
         ChipRole::Neutral => colors.text_muted,
         ChipRole::Accent => colors.accent,
         ChipRole::Info => colors.info,
         ChipRole::Success => colors.success,
         ChipRole::Warning => colors.warning,
         ChipRole::Error => colors.error,
      }
   }
}

const DOT_DIAMETER: f32 = 6.0;
const DOT_GAP: f32 = 6.0;

/// Auto-width pill for one short label.
///
/// ```
/// # use egui::__run_test_ui;
/// # use egui_elements::widgets::{Chip, ChipRole};
/// # __run_test_ui(|ui| {
/// ui.add(Chip::new("Design").role(ChipRole::Info));
/// ui.add(Chip::new("Needs reply").role(ChipRole::Warning).dot(true));
/// ui.add(Chip::new("Archived").outline(true));
/// # });
/// ```
#[must_use = "You should put this widget in a ui with `ui.add(widget);`"]
#[derive(Clone)]
pub struct Chip {
   text: WidgetText,
   role: ChipRole,
   dot: bool,
   outline: bool,
   text_style: TextStyle,
   padding: Vec2,
   corner_radius: Option<f32>,
}

impl Chip {
   /// A neutral chip showing `text`.
   pub fn new(text: impl Into<WidgetText>) -> Self {
      Self {
         text: text.into(),
         role: ChipRole::Neutral,
         dot: false,
         outline: false,
         text_style: TextStyle::Small,
         padding: vec2(8.0, 3.0),
         corner_radius: None,
      }
   }

   /// Fill and ink colour. Defaults to [`ChipRole::Neutral`].
   pub fn role(mut self, role: ChipRole) -> Self {
      self.role = role;
      self
   }

   /// Prefix a status dot in the role colour.
   pub fn dot(mut self, dot: bool) -> Self {
      self.dot = dot;
      self
   }

   /// Transparent fill, role-coloured border, theme text ink.
   pub fn outline(mut self, outline: bool) -> Self {
      self.outline = outline;
      self
   }

   /// Font the label falls back to when `text` carries no explicit size.
   pub fn text_style(mut self, text_style: TextStyle) -> Self {
      self.text_style = text_style;
      self
   }

   /// Outer padding; defaults to `(8, 3)`.
   pub fn padding(mut self, padding: Vec2) -> Self {
      self.padding = padding;
      self
   }

   /// Corner radius; the default is a pill (`height / 2`).
   pub fn corner_radius(mut self, radius: f32) -> Self {
      self.corner_radius = Some(radius);
      self
   }
}

impl Widget for Chip {
   fn ui(self, ui: &mut Ui) -> Response {
      let colors = Theme::colors_from_ctx(ui.ctx());
      let role_color = self.role.color(&colors);
      let galley = self.text.into_galley(
         ui,
         Some(TextWrapMode::Extend),
         f32::INFINITY,
         self.text_style,
      );
      let label = galley.text().to_owned();
      let text_size = galley.size();
      let dot_space = if self.dot {
         DOT_DIAMETER + DOT_GAP
      } else {
         0.0
      };
      let size = Vec2::new(
         text_size.x + dot_space + 2.0 * self.padding.x,
         text_size.y + 2.0 * self.padding.y,
      );
      let (rect, response) = ui.allocate_exact_size(size, Sense::hover());

      if ui.is_rect_visible(rect) {
         let corner = self.corner_radius.unwrap_or(size.y * 0.5).min(size.y * 0.5);
         let (fill, ink, stroke) = if self.outline {
            (
               Color32::TRANSPARENT,
               colors.text,
               Stroke::new(1.0, role_color),
            )
         } else {
            let (fill, ink) = colors.tint(role_color);
            (fill, ink, Stroke::NONE)
         };
         ui.painter().add(RectShape::new(
            rect,
            corner,
            fill,
            stroke,
            StrokeKind::Inside,
         ));

         let mut left = rect.left() + self.padding.x;
         if self.dot {
            ui.painter().circle_filled(
               pos2(left + DOT_DIAMETER * 0.5, rect.center().y),
               DOT_DIAMETER * 0.5,
               role_color,
            );
            left += DOT_DIAMETER + DOT_GAP;
         }
         ui.painter().galley(
            pos2(left, rect.center().y - text_size.y * 0.5),
            galley,
            ink,
         );
      }

      response
         .widget_info(|| egui::WidgetInfo::labeled(egui::Role::Label, ui.is_enabled(), &label));
      response
   }
}

#[cfg(test)]
mod tests {
   use super::*;
   use egui::Rect;

   #[test]
   fn a_chip_is_a_pill_not_a_circle() {
      let mut rect = Rect::NOTHING;
      egui::__run_test_ui(|ui| {
         rect = ui.add(Chip::new("Needs review")).rect;
      });
      assert!(rect.width() > rect.height() * 2.0, "{rect:?}");
      assert!(rect.height() < 26.0, "{rect:?}");
   }

   #[test]
   fn the_dot_adds_width_only() {
      let mut plain = Rect::NOTHING;
      let mut dotted = Rect::NOTHING;
      egui::__run_test_ui(|ui| plain = ui.add(Chip::new("Design")).rect);
      egui::__run_test_ui(|ui| dotted = ui.add(Chip::new("Design").dot(true)).rect);
      assert!((dotted.width() - plain.width() - 12.0).abs() < 1.0);
      assert!((dotted.height() - plain.height()).abs() < 0.5);
   }

   #[test]
   fn outline_keeps_the_layout() {
      let mut filled = Rect::NOTHING;
      let mut outlined = Rect::NOTHING;
      egui::__run_test_ui(|ui| filled = ui.add(Chip::new("Archived")).rect);
      egui::__run_test_ui(|ui| outlined = ui.add(Chip::new("Archived").outline(true)).rect);
      assert_eq!(filled.size(), outlined.size());
   }
}
