use egui::layers::ShapeIdx;
use egui::{
   Color32, CornerRadius, InnerResponse, Margin, Rect, Response, Sense, Shadow, Shape, Stroke, Ui,
   UiBuilder, UiKind, UiStackInfo, epaint::MarginF32,
};

use crate::gradient::Fill;
use crate::motion::{InteractT, Motion};
use crate::theme::{Surface, Theme};
use crate::visuals::FrameVisuals;

/// A frame around some content: margin, fill, stroke, rounding, shadow.
///
/// Same geometry as [`egui::Frame`]:
/// `content_size + inner_margin + 2 * stroke.width + outer_margin`.
///
/// The difference is that the fill is a [`Fill`] — a solid colour or a gradient — and the state
/// colours live on [`FrameVisuals`] like the other widgets.
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
         bg: Fill::solid(Color32::TRANSPARENT),
         bg_hover: Fill::solid(Color32::TRANSPARENT),
         bg_click: Fill::solid(Color32::TRANSPARENT),
         bg_selected: Fill::solid(Color32::TRANSPARENT),
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
   ///
   /// The inverse of [`Self::to_egui`], and what `Frame::from(frame)` calls.
   pub fn from_egui(frame: egui::Frame) -> Self {
      let fill = frame.fill;
      Self {
         inner_margin: frame.inner_margin,
         outer_margin: frame.outer_margin,
         visuals: FrameVisuals {
            bg: Fill::solid(fill),
            bg_hover: Fill::solid(fill),
            bg_click: Fill::solid(fill),
            bg_selected: Fill::solid(fill),
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

   /// The static chrome this frame paints, as an [`egui::Frame`].
   ///
   /// Use it where egui owns the frame — `Panel::frame`, `CentralPanel::frame`, `Window::frame`,
   /// a popup — but the margins, radius, shadow, border and fill should come from this crate. The
   /// conversion is lossy by construction:
   ///
   /// - A [`Fill`] is handed over as [`Fill::to_solid`], so a gradient lands on the midpoint of its
   ///   ramp (egui paints a frame with one colour).
   /// - `.selected(true)` hands over [`FrameVisuals::bg_selected`]; otherwise the idle fill is used.
   /// - Hover / click visuals, `sense` and `fill_width` have no `egui::Frame` counterpart and are
   ///   dropped.
   #[inline]
   pub fn to_egui(self) -> egui::Frame {
      let fill = if self.selected {
         self.visuals.bg_selected
      } else {
         self.visuals.bg
      };

      egui::Frame {
         inner_margin: self.inner_margin,
         fill: fill.to_solid(),
         stroke: self.visuals.border,
         corner_radius: self.visuals.corner_radius,
         outer_margin: self.outer_margin,
         shadow: self.visuals.shadow,
      }
   }

   /// A themed [`Surface`] frame, with that surface's hover / click visuals applied.
   ///
   /// ```
   /// # use egui::__run_test_ui;
   /// # use egui_elements::{Frame, Surface};
   /// # use egui_elements::theme::{Theme, ThemeKind};
   /// # __run_test_ui(|ui| {
   /// let theme = Theme::new(ThemeKind::TokyoNight);
   /// Frame::surface(&theme, Surface::Panel.card()).show(ui, |ui| {
   ///    ui.label("A well inside a panel");
   /// });
   /// # });
   /// ```
   pub fn surface(theme: &Theme, surface: Surface) -> Self {
      let mut frame = Self::from_egui(theme.surface_frame(surface));
      if let Some(visuals) = theme.surface_visuals(surface) {
         frame = frame.visuals(visuals);
      }
      frame
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
   pub fn fill(mut self, fill: impl Into<Fill>) -> Self {
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

   /// Shrink-wrap to the content width; same as `fill_width(false)`.
   ///
   /// The default for a bare [`Frame`], but `.surface(..)` and hand-built frames may have
   /// stretched, and `fit()` reads better than a boolean at the call site.
   #[inline]
   pub fn fit(mut self) -> Self {
      self.fill_width = false;
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
         fill: Color32::TRANSPARENT,
         stroke: Stroke::new(self.visuals.border.width, Color32::TRANSPARENT),
         ..self.to_egui()
      }
   }

   fn paint_shape(&self, widget_rect: Rect, fill: Fill, stroke: Stroke) -> Shape {
      crate::visuals::paint_shape(
         widget_rect,
         fill,
         stroke,
         self.visuals.corner_radius,
         self.visuals.shadow,
      )
   }
}

impl From<egui::Frame> for Frame {
   fn from(frame: egui::Frame) -> Self {
      Self::from_egui(frame)
   }
}

impl From<Frame> for egui::Frame {
   fn from(frame: Frame) -> Self {
      frame.to_egui()
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
         let visuals = &self.frame.visuals;
         let across = InteractT::of(
            ui,
            response.id,
            response.contains_pointer() || response.hovered() || response.highlighted(),
            response.is_pointer_button_down_on() || response.clicked() || response.has_focus(),
            Motion::from_ui(ui),
         );
         (
            across.blend(visuals.bg, visuals.bg_hover, visuals.bg_click),
            across.blend(
               visuals.border,
               visuals.border_hover,
               visuals.border_click,
            ),
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
   fn fill_propagates_to_matching_states() {
      let frame = Frame::new().fill(Color32::RED);
      assert_eq!(frame.visuals.bg, Fill::solid(Color32::RED));
      assert_eq!(frame.visuals.bg_hover, Fill::solid(Color32::RED));
      assert_eq!(frame.visuals.bg_click, Fill::solid(Color32::RED));
      assert_eq!(
         frame.visuals.bg_selected,
         Fill::solid(Color32::RED)
      );
   }

   fn theme_visuals() -> FrameVisuals {
      FrameVisuals {
         bg: Fill::solid(Color32::RED),
         bg_hover: Fill::solid(Color32::BLUE),
         bg_click: Fill::solid(Color32::RED),
         bg_selected: Fill::solid(Color32::GREEN),
         border: Stroke::NONE,
         border_hover: Stroke::NONE,
         border_click: Stroke::NONE,
         corner_radius: CornerRadius::same(6),
         shadow: Shadow::NONE,
      }
   }

   #[test]
   fn a_zero_corner_radius_survives_any_call_order() {
      // Case: zero_radius_survives_later_visuals / corner_radius_zero_survives_later_visuals
      let frame = Frame::new().corner_radius(0).visuals(theme_visuals());
      assert_eq!(frame.visuals.corner_radius, CornerRadius::ZERO);

      // Case: visuals_then_zero_corner_radius
      let frame = Frame::new().visuals(theme_visuals()).corner_radius(0);
      assert_eq!(frame.visuals.corner_radius, CornerRadius::ZERO);

      // Case: visuals_without_override_keeps_theme_radius
      let frame = Frame::new().visuals(theme_visuals());
      assert_eq!(frame.visuals.corner_radius, CornerRadius::same(6));
   }

   #[test]
   fn fit_shrinks_a_stretched_frame() {
      let frame = Frame::new().fill_width(true).fit();
      assert!(!frame.fill_width);
   }

   #[test]
   fn an_egui_frame_round_trips_through_the_crate_frame() {
      let source = egui::Frame::NONE
         .inner_margin(Margin::symmetric(10, 6))
         .outer_margin(Margin::same(2))
         .fill(Color32::RED)
         .stroke(Stroke::new(2.0, Color32::BLUE))
         .corner_radius(CornerRadius::same(5))
         .shadow(Shadow {
            offset: (1, 2).into(),
            blur: 4,
            spread: 0,
            color: Color32::BLACK,
         });

      let frame = Frame::from(source);
      assert_eq!(frame.inner_margin, Margin::symmetric(10, 6));
      assert_eq!(frame.visuals.bg, Fill::solid(Color32::RED));

      let back: egui::Frame = frame.into();
      assert_eq!(back, source);
   }

   #[test]
   fn to_egui_collapses_a_gradient_to_its_midpoint() {
      let fill = Fill::vertical(Color32::BLACK, Color32::WHITE);
      assert_eq!(
         Frame::new().fill(fill).to_egui().fill,
         fill.to_solid()
      );
   }

   #[test]
   fn to_egui_hands_over_the_selected_fill() {
      let selected = Frame::new().visuals(theme_visuals()).selected(true);
      assert_eq!(selected.to_egui().fill, Color32::GREEN);

      let idle = Frame::new().visuals(theme_visuals());
      assert_eq!(idle.to_egui().fill, Color32::RED);
   }
}
