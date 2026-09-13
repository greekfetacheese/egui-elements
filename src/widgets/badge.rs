//! Corner badges — small notification circles painted over a themed widget.
//!
//! A [`Badge`] is a circle with optional text, anchored to one of the four
//! corners of a widget. Attach it to a [`Button`](crate::widgets::Button) to
//! signal that the control opens something with unseen content (balance or
//! approval changes, unread messages, …):
//!
//! ```
//! # use egui::__run_test_ui;
//! # use egui_elements::widgets::{Badge, BadgeCorner, Button};
//! # __run_test_ui(|ui| {
//! let changes = 3;
//! ui.add(
//!    Button::new("Balance & approvals")
//!       .badge(Badge::new(changes.to_string()).corner(BadgeCorner::TopRight)),
//! );
//! # });
//! ```
//!
//! The badge is an *overlay*: it is painted on top of the widget and never
//! changes its size or layout. The fill defaults to
//! [`ThemeColors::error`](crate::theme::ThemeColors::error) and the text to
//! [`ThemeColors::bg`](crate::theme::ThemeColors::bg), matching the theme rule
//! that chromatic fills carry `bg`-colored glyphs.

use egui::{Align2, Color32, Rect, Stroke, TextWrapMode, TextStyle, Ui, Vec2, WidgetText};

#[cfg(test)]
use egui::Pos2;

use crate::theme::Theme;

/// Default size (in points) of a badge with an empty text — a plain dot.
const DEFAULT_MIN_DIAMETER: f32 = 16.0;
/// Default distance (in points) the circle is inset from the widget corner.
const DEFAULT_INSET: f32 = 2.0;
/// Default gap (in points) between the badge text and the circle edge.
const DEFAULT_PADDING: f32 = 3.0;

/// Which corner of a widget a [`Badge`] is anchored to.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BadgeCorner {
   TopLeft,
   /// Where notification badges usually live; the default.
   #[default]
   TopRight,
   BottomLeft,
   BottomRight,
}

impl BadgeCorner {
   /// Anchor used to place the circle inside the widget rect.
   pub const fn align2(self) -> Align2 {
      match self {
         Self::TopLeft => Align2::LEFT_TOP,
         Self::TopRight => Align2::RIGHT_TOP,
         Self::BottomLeft => Align2::LEFT_BOTTOM,
         Self::BottomRight => Align2::RIGHT_BOTTOM,
      }
   }

   pub const fn to_str(self) -> &'static str {
      match self {
         Self::TopLeft => "Top left",
         Self::TopRight => "Top right",
         Self::BottomLeft => "Bottom left",
         Self::BottomRight => "Bottom right",
      }
   }

   pub const fn to_vec() -> &'static [Self] {
      &[Self::TopLeft, Self::TopRight, Self::BottomLeft, Self::BottomRight]
   }
}

/// A notification circle painted in a corner of a widget.
///
/// Build one with [`Badge::new`] and attach it with
/// [`Button::badge`](crate::widgets::Button::badge). The colour and corner are
/// the two knobs you will reach for; everything else has a sane default.
///
/// Empty text ([`Badge::dot`]) renders a plain coloured dot.
#[derive(Clone, Debug)]
#[must_use = "Badges should be shown on a widget, e.g. `Button::new(\"…\").badge(badge)`"]
pub struct Badge {
   text: WidgetText,
   color: Option<Color32>,
   corner: BadgeCorner,
   text_color: Option<Color32>,
   min_diameter: f32,
   inset: f32,
   padding: f32,
   outline: Option<Stroke>,
}

impl Badge {
   /// Create a badge showing `text` (e.g. a change count) in the top-right
   /// corner, filled with [`ThemeColors::error`](crate::theme::ThemeColors::error).
   pub fn new(text: impl Into<WidgetText>) -> Self {
      Self {
         text: text.into(),
         color: None,
         corner: BadgeCorner::default(),
         text_color: None,
         min_diameter: DEFAULT_MIN_DIAMETER,
         inset: DEFAULT_INSET,
         padding: DEFAULT_PADDING,
         outline: None,
      }
   }

   /// Create a textless badge: a plain coloured dot.
   pub fn dot() -> Self {
      Self::new("")
   }

   /// Circle fill. Defaults to the theme error colour.
   #[inline]
   pub fn color(mut self, color: Color32) -> Self {
      self.color = Some(color);
      self
   }

   /// Which corner of the widget the badge sits in. Default: top-right.
   #[inline]
   pub fn corner(mut self, corner: BadgeCorner) -> Self {
      self.corner = corner;
      self
   }

   /// Colour of the badge text. Defaults to
   /// [`ThemeColors::bg`](crate::theme::ThemeColors::bg) so glyphs stay
   /// readable on chromatic fills.
   #[inline]
   pub fn text_color(mut self, text_color: Color32) -> Self {
      self.text_color = Some(text_color);
      self
   }

   /// Smallest diameter the circle will take, in points.
   ///
   /// The circle grows beyond this when the text needs the room. Default:
   /// `16.0`. Lower it for [`Button::small`](crate::widgets::Button::small).
   #[inline]
   pub fn min_diameter(mut self, min_diameter: f32) -> Self {
      self.min_diameter = min_diameter;
      self
   }

   /// Distance between the circle and the widget corner, in points.
   ///
   /// Negative values let the badge straddle the edge. Default: `2.0`.
   #[inline]
   pub fn inset(mut self, inset: f32) -> Self {
      self.inset = inset;
      self
   }

   /// Gap between the text and the circle edge, in points. Default: `3.0`.
   #[inline]
   pub fn padding(mut self, padding: f32) -> Self {
      self.padding = padding;
      self
   }

   /// Ring drawn around the circle, e.g. `Stroke::new(1.5, theme.colors.bg)`
   /// to lift the badge off a busy button fill.
   #[inline]
   pub fn outline(mut self, outline: impl Into<Stroke>) -> Self {
      self.outline = Some(outline.into());
      self
   }

   /// Rect the circle occupies for a laid-out text of `text_size`, inside
   /// `widget_rect`. Pure geometry — shared by painting and tests.
   fn circle_rect(&self, text_size: Vec2, widget_rect: Rect) -> Rect {
      let diameter = (text_size + Vec2::splat(2.0 * self.padding))
         .max_elem()
         .max(self.min_diameter);
      self
         .corner
         .align2()
         .align_size_within_rect(Vec2::splat(diameter), widget_rect.shrink(self.inset))
   }

   /// Paint the badge on top of `widget_rect` (usually a widget's response rect).
   ///
   /// Public so any widget can host a badge without re-implementing the
   /// placement and text metrics.
   pub fn paint_at(&self, ui: &Ui, widget_rect: Rect) {
      if !ui.is_rect_visible(widget_rect) {
         return;
      }

      let theme = Theme::current(ui.ctx());
      let fill = self.color.unwrap_or(theme.colors.error);
      let text_color = self.text_color.unwrap_or(theme.colors.bg);

      let galley = self.text.clone().into_galley(
         ui,
         Some(TextWrapMode::Extend),
         f32::INFINITY,
         TextStyle::Small,
      );

      let circle = self.circle_rect(galley.size(), widget_rect);
      let center = circle.center();

      let painter = ui.painter();
      painter.circle(
         center,
         circle.width() * 0.5,
         fill,
         self.outline.unwrap_or(Stroke::NONE),
      );
      if !galley.is_empty() {
         painter.galley(center - galley.size() * 0.5, galley, text_color);
      }
   }

   /// Center of the circle for a laid-out text of `text_size`. Test helper.
   #[cfg(test)]
   fn center_for(&self, text_size: Vec2, widget_rect: Rect) -> Pos2 {
      self.circle_rect(text_size, widget_rect).center()
   }
}

#[cfg(test)]
mod tests {
   use super::*;
   use egui::vec2;

   fn rect() -> Rect {
      Rect::from_min_size(Pos2::new(100.0, 100.0), vec2(200.0, 40.0))
   }

   const TEXT: Vec2 = Vec2::new(8.0, 12.0);

   #[test]
   fn corners_anchor_to_the_matching_side() {
      let r = rect();
      let inset = DEFAULT_INSET;
      let pad = DEFAULT_PADDING;
      let min = DEFAULT_MIN_DIAMETER;

      let expected_diameter = (TEXT.x + 2.0 * pad).max(TEXT.y + 2.0 * pad).max(min);
      let radius = expected_diameter * 0.5;

      let cases = [
         (BadgeCorner::TopLeft, r.left() + inset + radius, r.top() + inset + radius),
         (BadgeCorner::TopRight, r.right() - inset - radius, r.top() + inset + radius),
         (BadgeCorner::BottomLeft, r.left() + inset + radius, r.bottom() - inset - radius),
         (
            BadgeCorner::BottomRight,
            r.right() - inset - radius,
            r.bottom() - inset - radius,
         ),
      ];

      for (corner, x, y) in cases {
         let center = Badge::dot().corner(corner).center_for(TEXT, r);
         assert!(
            (center.x - x).abs() < 0.01 && (center.y - y).abs() < 0.01,
            "{corner:?}: expected ({x}, {y}), got {center:?}"
         );
      }
   }

   #[test]
   fn circle_covers_text_and_min_diameter() {
      let badge = Badge::new("12");
      let small = badge.clone().circle_rect(Vec2::ZERO, rect());
      assert_eq!(small.width(), DEFAULT_MIN_DIAMETER);

      let wide = badge.circle_rect(vec2(40.0, 12.0), rect());
      assert_eq!(wide.width(), 40.0 + 2.0 * DEFAULT_PADDING);
   }

   #[test]
   fn negative_inset_lets_the_badge_straddle_the_corner() {
      let inside = Badge::dot().center_for(TEXT, rect());
      let outside = Badge::dot().inset(-6.0).center_for(TEXT, rect());
      assert!(outside.x > inside.x);
   }

   #[test]
   fn defaults_are_top_right_error_and_bg_text() {
      let badge = Badge::dot();
      assert_eq!(badge.corner, BadgeCorner::TopRight);
      assert_eq!(badge.color, None);
      assert_eq!(badge.text_color, None);
   }

   #[test]
   fn paints_without_panicking() {
      egui::__run_test_ui(|ui| {
         let response = ui.add(crate::widgets::Button::new("Balance & approvals").badge(
            Badge::new("3")
               .corner(BadgeCorner::TopRight)
               .outline(Stroke::new(1.5, Color32::BLACK)),
         ));
         assert!(response.rect.width() > 0.0);
      });
   }
}
