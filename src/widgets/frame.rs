use egui::layers::ShapeIdx;
use egui::{
   Color32, CornerRadius, InnerResponse, Margin, Rect, Response, Sense, Shadow, Shape, Stroke, Ui,
   UiBuilder, UiKind, UiStackInfo, epaint::MarginF32,
};

use crate::visuals::FrameVisuals;

/// A frame around some content: margin, fill ([`Fill`]), stroke, rounding, shadow.
///
/// Same geometry as [`egui::Frame`]:
/// `content_size + inner_margin + 2 * stroke.width + outer_margin`.
///
/// The difference is the fill is a [`Fill`], so it can be a two-stop gradient,
/// and state colors live on [`FrameVisuals`] like the other widgets.
///
/// Stroke width is taken from the idle border at `begin`. Changing stroke
/// width on hover after `begin` does not relayout.
#[must_use = "You should call .show()"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
   pub inner_margin: Margin,
   pub outer_margin: Margin,
   pub visuals: FrameVisuals,
   pub sense: Sense,
   /// When `false`, paint always uses idle fill / border, even if the
   /// pointer is over the frame. Hover / click fills need [`Self::interactive`].
   pub interactive: bool,
   /// Paint [`FrameVisuals::bg_selected`] instead of idle / hover / click.
   pub selected: bool,
   /// Stretch the frame to the available width (useful for sidebar rows).
   pub fill_width: bool,
   /// Set by [`Self::corner_radius`]. Wins over a
   /// later [`Self::visuals`], which would otherwise restore the theme radius.
   corner_radius_override: Option<CornerRadius>,
}

impl Default for Frame {
   fn default() -> Self {
      Self::NONE
   }
}

impl Frame {
   pub const NONE: Self = Self {
      inner_margin: Margin::ZERO,
      outer_margin: Margin::ZERO,
      visuals: FrameVisuals {
         bg: Color32::TRANSPARENT,
         bg_hover: Color32::TRANSPARENT,
         bg_click: Color32::TRANSPARENT,
         bg_selected: Color32::TRANSPARENT,
         border: Stroke::NONE,
         border_hover: Stroke::NONE,
         border_click: Stroke::NONE,
         corner_radius: CornerRadius::ZERO,
         shadow: Shadow::NONE,
      },
      sense: Sense::HOVER,
      interactive: false,
      selected: false,
      fill_width: false,
      corner_radius_override: None,
   };

   pub const fn new() -> Self {
      Self::NONE
   }

   /// Copy margins, idle fill, stroke, radius, and shadow from an [`egui::Frame`].
   /// Hover / click start equal to idle.
   pub fn from_egui(frame: egui::Frame) -> Self {
      let fill = frame.fill;
      Self {
         inner_margin: frame.inner_margin,
         outer_margin: frame.outer_margin,
         visuals: FrameVisuals {
            bg: fill,
            bg_hover: fill,
            bg_click: fill,
            bg_selected: fill,
            border: frame.stroke,
            border_hover: frame.stroke,
            border_click: frame.stroke,
            corner_radius: frame.corner_radius,
            shadow: frame.shadow,
         },
         sense: Sense::HOVER,
         interactive: false,
         selected: false,
         fill_width: false,
         corner_radius_override: None,
      }
   }

   #[inline]
   pub fn inner_margin(mut self, inner_margin: impl Into<Margin>) -> Self {
      self.inner_margin = inner_margin.into();
      self
   }

   #[inline]
   pub fn outer_margin(mut self, outer_margin: impl Into<Margin>) -> Self {
      self.outer_margin = outer_margin.into();
      self
   }

   /// Idle fill. Does not change hover / click unless they were still the old idle fill.
   #[inline]
   pub fn fill(mut self, fill: impl Into<Color32>) -> Self {
      let fill = fill.into();
      let old = self.visuals.bg;
      self.visuals.bg = fill;
      if self.visuals.bg_hover == old {
         self.visuals.bg_hover = fill;
      }
      if self.visuals.bg_click == old {
         self.visuals.bg_click = fill;
      }
      if self.visuals.bg_selected == old {
         self.visuals.bg_selected = fill;
      }
      self
   }

   #[inline]
   pub fn stroke(mut self, stroke: impl Into<Stroke>) -> Self {
      let stroke = stroke.into();
      let old = self.visuals.border;
      self.visuals.border = stroke;
      if self.visuals.border_hover == old {
         self.visuals.border_hover = stroke;
      }
      if self.visuals.border_click == old {
         self.visuals.border_click = stroke;
      }
      self
   }

   #[inline]
   pub fn corner_radius(mut self, corner_radius: impl Into<CornerRadius>) -> Self {
      let corner_radius = corner_radius.into();
      self.visuals.corner_radius = corner_radius;
      self.corner_radius_override = Some(corner_radius);
      self
   }

   #[inline]
   pub fn shadow(mut self, shadow: Shadow) -> Self {
      self.visuals.shadow = shadow;
      self
   }

   /// Replace fill, stroke, radius, and shadow. Margins and sense are kept.
   ///
   /// [`Self::corner_radius`] win if already set
   #[inline]
   pub fn visuals(mut self, visuals: FrameVisuals) -> Self {
      self.visuals = visuals;
      if let Some(corner_radius) = self.corner_radius_override {
         self.visuals.corner_radius = corner_radius;
      }
      self
   }

   #[inline]
   pub fn sense(mut self, sense: Sense) -> Self {
      self.sense = sense;
      self
   }

   /// `true` → [`Sense::click`] and hover / click fills. `false` → idle chrome
   /// only (pointer hover does not swap fill).
   #[inline]
   pub fn interactive(mut self, interactive: bool) -> Self {
      self.interactive = interactive;
      self.sense = if interactive {
         Sense::click()
      } else {
         Sense::HOVER
      };
      self
   }

   /// Paint [`FrameVisuals::bg_selected`] even when not hovered.
   #[inline]
   pub fn selected(mut self, selected: bool) -> Self {
      self.selected = selected;
      self
   }

   /// Stretch the frame to the available width.
   ///
   /// The content `Ui` also gets that min width, so inner
   /// right-to-left rows (unread counts, timestamps) land on the far edge.
   #[inline]
   pub fn fill_width(mut self, fill: bool) -> Self {
      self.fill_width = fill;
      self
   }

   #[inline]
   pub fn multiply_with_opacity(mut self, opacity: f32) -> Self {
      self.visuals.bg = self.visuals.bg.gamma_multiply(opacity);
      self.visuals.bg_hover = self.visuals.bg_hover.gamma_multiply(opacity);
      self.visuals.bg_click = self.visuals.bg_click.gamma_multiply(opacity);
      self.visuals.bg_selected = self.visuals.bg_selected.gamma_multiply(opacity);
      self.visuals.border.color = self.visuals.border.color.gamma_multiply(opacity);
      self.visuals.border_hover.color = self.visuals.border_hover.color.gamma_multiply(opacity);
      self.visuals.border_click.color = self.visuals.border_click.color.gamma_multiply(opacity);
      self.visuals.shadow.color = self.visuals.shadow.color.gamma_multiply(opacity);
      self
   }

   /// Idle stroke width is part of the margin (same as [`egui::Frame`]).
   #[inline]
   pub fn total_margin(&self) -> MarginF32 {
      MarginF32::from(self.inner_margin)
         + MarginF32::from(self.visuals.border.width)
         + MarginF32::from(self.outer_margin)
   }

   pub fn fill_rect(&self, content_rect: Rect) -> Rect {
      content_rect + self.inner_margin
   }

   pub fn widget_rect(&self, content_rect: Rect) -> Rect {
      content_rect + self.inner_margin + MarginF32::from(self.visuals.border.width)
   }

   pub fn outer_rect(&self, content_rect: Rect) -> Rect {
      content_rect
         + self.inner_margin
         + MarginF32::from(self.visuals.border.width)
         + self.outer_margin
   }

   fn stack_frame(&self) -> egui::Frame {
      egui::Frame {
         inner_margin: self.inner_margin,
         fill: Color32::TRANSPARENT,
         stroke: Stroke::new(self.visuals.border.width, Color32::TRANSPARENT),
         corner_radius: self.visuals.corner_radius,
         outer_margin: self.outer_margin,
         shadow: self.visuals.shadow,
      }
   }

   fn paint_shape(&self, widget_rect: Rect, fill: Color32, stroke: Stroke) -> Shape {
      crate::visuals::paint_shape(
         widget_rect,
         fill,
         stroke,
         self.visuals.corner_radius,
         self.visuals.shadow,
      )
   }
}

pub struct Prepared {
   pub frame: Frame,
   where_to_put_background: ShapeIdx,
   pub content_ui: Ui,
   max_content_rect: Rect,
}

impl Frame {
   pub fn begin(self, ui: &mut Ui) -> Prepared {
      let where_to_put_background = ui.painter().add(Shape::Noop);
      let outer_rect_bounds = ui.available_rect_before_wrap();

      let mut max_content_rect = outer_rect_bounds - self.total_margin();
      max_content_rect.max.x = max_content_rect.max.x.max(max_content_rect.min.x);
      max_content_rect.max.y = max_content_rect.max.y.max(max_content_rect.min.y);

      let mut content_ui = ui.new_child(
         UiBuilder::new()
            .ui_stack_info(UiStackInfo::new(UiKind::Frame).with_frame(self.stack_frame()))
            .max_rect(max_content_rect)
            .sense(Sense::empty()),
      );
      if self.fill_width {
         content_ui.set_min_width(max_content_rect.width());
      }

      Prepared {
         frame: self,
         where_to_put_background,
         content_ui,
         max_content_rect,
      }
   }

   pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
      let mut prepared = self.begin(ui);
      let ret = add_contents(&mut prepared.content_ui);
      let response = prepared.end(ui);
      InnerResponse::new(ret, response)
   }
}

impl Prepared {
   fn content_rect(&self) -> Rect {
      let mut content_rect = self.content_ui.min_rect();
      if self.frame.fill_width {
         content_rect.min.x = content_rect.min.x.min(self.max_content_rect.min.x);
         content_rect.max.x = content_rect.max.x.max(self.max_content_rect.max.x);
      }
      content_rect
   }

   fn allocated_outer_rect(&self) -> Rect {
      self.frame.outer_rect(self.content_rect())
   }

   pub fn allocate_space(&self, ui: &mut Ui) -> Response {
      ui.allocate_rect(self.allocated_outer_rect(), self.frame.sense)
   }

   pub fn paint(&self, ui: &Ui, response: &Response) {
      let content_rect = self.content_rect();
      let widget_rect = self.frame.widget_rect(content_rect);
      if !ui.is_rect_visible(widget_rect) {
         return;
      }
      let (fill, stroke) = if self.frame.selected {
         (
            self.frame.visuals.bg_selected,
            self.frame.visuals.border,
         )
      } else if self.frame.interactive {
         (
            self.frame.visuals.bg_from_res(response),
            self.frame.visuals.border_from_res(response),
         )
      } else {
         (self.frame.visuals.bg, self.frame.visuals.border)
      };
      let shape = self.frame.paint_shape(widget_rect, fill, stroke);
      ui.painter().set(self.where_to_put_background, shape);
   }

   pub fn end(self, ui: &mut Ui) -> Response {
      let response = self.allocate_space(ui);
      self.paint(ui, &response);
      response
   }
}

#[cfg(test)]
mod tests {
   use super::*;

   #[test]
   fn total_margin_sums_parts() {
      let frame = Frame::new()
         .inner_margin(Margin::same(4))
         .outer_margin(Margin::same(3))
         .stroke(Stroke::new(2.0, Color32::WHITE));
      let m = frame.total_margin();
      assert_eq!(m.left, 9.0);
      assert_eq!(m.right, 9.0);
      assert_eq!(m.top, 9.0);
      assert_eq!(m.bottom, 9.0);
   }

   #[test]
   fn fill_propagates_to_matching_states() {
      let frame = Frame::new().fill(Color32::RED);
      assert_eq!(frame.visuals.bg, Color32::RED);
      assert_eq!(frame.visuals.bg_hover, Color32::RED);
      assert_eq!(frame.visuals.bg_click, Color32::RED);
      assert_eq!(frame.visuals.bg_selected, Color32::RED);
   }

   #[test]
   fn interactive_false_is_default() {
      let frame = Frame::new().fill(Color32::RED);
      assert!(!frame.interactive);
      assert!(!frame.selected);
      assert!(!frame.fill_width);
      assert!(frame.interactive(true).interactive);
      assert!(!frame.interactive(true).interactive(false).interactive);
      assert!(frame.selected(true).selected);
      assert!(frame.fill_width(true).fill_width);
   }

   fn theme_visuals() -> FrameVisuals {
      FrameVisuals {
         bg: Color32::RED,
         bg_hover: Color32::BLUE,
         bg_click: Color32::RED,
         bg_selected: Color32::GREEN,
         border: Stroke::NONE,
         border_hover: Stroke::NONE,
         border_click: Stroke::NONE,
         corner_radius: CornerRadius::same(6),
         shadow: Shadow::NONE,
      }
   }

   #[test]
   fn zero_radius_survives_later_visuals() {
      let frame = Frame::new().corner_radius(0).visuals(theme_visuals());
      assert_eq!(frame.visuals.corner_radius, CornerRadius::ZERO);
   }

   #[test]
   fn corner_radius_zero_survives_later_visuals() {
      let frame = Frame::new().corner_radius(0).visuals(theme_visuals());
      assert_eq!(frame.visuals.corner_radius, CornerRadius::ZERO);
   }

   #[test]
   fn visuals_then_zero_corner_radius() {
      let frame = Frame::new().visuals(theme_visuals()).corner_radius(0);
      assert_eq!(frame.visuals.corner_radius, CornerRadius::ZERO);
   }

   #[test]
   fn visuals_without_override_keeps_theme_radius() {
      let frame = Frame::new().visuals(theme_visuals());
      assert_eq!(frame.visuals.corner_radius, CornerRadius::same(6));
   }
}
