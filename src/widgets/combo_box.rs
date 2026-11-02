use super::label::Label;
use crate::motion::{InteractT, Motion, animate_with};
use crate::theme::Theme;
use crate::visuals::ComboBoxVisuals;
use egui::{
   Align2, Frame, Id, InnerResponse, NumExt, Painter, Popup, PopupCloseBehavior, PopupKind, Pos2,
   Rect, Response, ScrollArea, Sense, Stroke, TextWrapMode, Ui, Vec2, WidgetText,
   emath::Rot2,
   epaint::{RectShape, Shape, StrokeKind},
   style::WidgetVisuals,
};

use std::{fmt::Debug, hash::Hash, ops::Range};

/// Dropdown whose closed face is a [`Label`] (text and/or image).
///
/// ```
/// # use egui::__run_test_ui;
/// # use egui_elements::widgets::{ComboBox, Label};
/// # __run_test_ui(|ui| {
/// let current = Label::new("Tokyo Night", None);
/// ComboBox::new("theme", current).width(200.0).show_ui(ui, |ui| {
///     ui.label("Tokyo Night Light");
/// });
/// # });
/// ```
#[must_use = "You should call .show_ui()"]
pub struct ComboBox {
   id_salt: Id,
   visuals: Option<ComboBoxVisuals>,
   label: Option<WidgetText>,
   selected_item: Label,
   width: Option<f32>,
   popup_max_height: Option<f32>,
   /// Gap between the button and the popup. `None` → `ThemeSpacing::xs`.
   popup_gap: Option<f32>,
   /// Padding between the popup card and its rows. Default `5.0`.
   popup_content_margin: Option<f32>,
   icon: Option<Box<dyn FnOnce(&Ui, Rect, &WidgetVisuals, bool)>>,
   wrap_mode: Option<TextWrapMode>,
   close_behavior: Option<PopupCloseBehavior>,
}

impl ComboBox {
   /// Create a combo box keyed by `id_salt`, showing `selected_item` on the button.
   pub fn new(id_salt: impl Hash + Debug, selected_item: Label) -> Self {
      Self {
         id_salt: Id::unique(id_salt),
         visuals: None,
         label: None,
         selected_item,
         width: None,
         popup_max_height: None,
         popup_gap: None,
         popup_content_margin: None,
         icon: None,
         wrap_mode: None,
         close_behavior: None,
      }
   }

   /// Override the chrome that would otherwise come from the installed theme.
   pub fn visuals(mut self, visuals: ComboBoxVisuals) -> Self {
      self.visuals = Some(visuals);
      self
   }

   /// Optional caption painted to the left of the button.
   pub fn label(mut self, label: impl Into<WidgetText>) -> Self {
      self.label = Some(label.into());
      self
   }

   /// Set the exact width of the combo box button.
   /// If not set, the width adapts to the content, icon, and minimum width.
   pub fn width(mut self, width: f32) -> Self {
      self.width = Some(width);
      self
   }

   /// Set the maximum height of the popup menu.
   /// Default is `ui.spacing().combo_height`.
   pub fn popup_max_height(mut self, height: f32) -> Self {
      self.popup_max_height = Some(height);
      self
   }

   /// Gap between the button and the popup, in points. Default: `ThemeSpacing::xs`.
   pub fn popup_gap(mut self, gap: f32) -> Self {
      self.popup_gap = Some(gap);
      self
   }

   /// Padding between the popup card and its rows, in points. Default: `5.0`.
   pub fn popup_content_margin(mut self, margin: f32) -> Self {
      self.popup_content_margin = Some(margin);
      self
   }

   /// Replace the default disclosure triangle with a custom painter.
   ///
   /// The `bool` argument is `true` while the popup is open.
   pub fn icon(mut self, icon_fn: impl FnOnce(&Ui, Rect, &WidgetVisuals, bool) + 'static) -> Self {
      self.icon = Some(Box::new(icon_fn));
      self
   }

   /// Set the wrap mode for the selected text displayed *in the button*.
   pub fn wrap_mode(mut self, wrap_mode: TextWrapMode) -> Self {
      self.wrap_mode = Some(wrap_mode);
      self
   }

   /// How the popup closes. Default: [`PopupCloseBehavior::CloseOnClick`].
   pub fn close_behavior(mut self, close_behavior: PopupCloseBehavior) -> Self {
      self.close_behavior = Some(close_behavior);
      self
   }

   /// Paint the button and, while open, the popup. Returns the inner response
   /// of the menu closure when the popup is visible.
   pub fn show_ui<R>(
      self,
      ui: &mut Ui,
      menu_contents: impl FnOnce(&mut Ui) -> R,
   ) -> Option<InnerResponse<R>> {
      self.show_popup(ui, None, |ui, _| menu_contents(ui))
   }

   /// Same as [`Self::show_ui`], but only the rows inside the viewport are built
   /// ([`ScrollArea::show_rows`]), so a long option list costs the visible rows and
   /// not the whole list.
   ///
   /// `row_height` is a row's height **without** `item_spacing.y` — egui adds that
   /// itself — so every row the closure emits must lay out at exactly that height.
   /// `total_rows` is the full count, not the number of rows the closure sees.
   pub fn show_rows_ui<R>(
      self,
      ui: &mut Ui,
      row_height: f32,
      total_rows: usize,
      rows: impl FnOnce(&mut Ui, Range<usize>) -> R,
   ) -> Option<InnerResponse<R>> {
      self.show_popup(ui, Some((row_height, total_rows)), rows)
   }

   fn show_popup<R>(
      self,
      ui: &mut Ui,
      virtual_rows: Option<(f32, usize)>,
      contents: impl FnOnce(&mut Ui, Range<usize>) -> R,
   ) -> Option<InnerResponse<R>> {
      let button_id = ui.make_persistent_id(self.id_salt);
      let popup_id = button_id.with("popup");

      let is_popup_open = Popup::is_id_open(ui.ctx(), popup_id);
      let visuals = self.visuals.or_else(|| Theme::combo_box_visuals_from_ctx(ui.ctx()));

      // Button Rendering
      let button_response = combo_box_with_image_button(
         ui,
         button_id,
         is_popup_open,
         visuals.as_ref(),
         &self.selected_item,
         self.icon,
         self.wrap_mode,
         self.width,
      );

      // Popup Handling
      let popup_max_h = self.popup_max_height.unwrap_or_else(|| ui.spacing().combo_height);
      let popup_max_w = self.width.unwrap_or(ui.available_width());
      let close_behavior = self.close_behavior.unwrap_or(PopupCloseBehavior::CloseOnClick);
      let gap = self.popup_gap.unwrap_or(Theme::spacing_from_ctx(ui.ctx()).xs);
      let content_margin = self.popup_content_margin.unwrap_or(5.0);

      // `Popup::menu` owns the open state and toggles it from the button's click, so the
      // popup id has to be the one this widget reads back above — and it must not be
      // toggled a second time here, or the popup opens and closes in the same frame.
      let mut popup = Popup::menu(&button_response)
         .id(popup_id)
         .close_behavior(close_behavior)
         .kind(PopupKind::Tooltip)
         .gap(gap);
      if let Some(vis) = visuals {
         popup = popup.frame(popup_frame(ui, &vis));
      }

      popup.show(|ui| {
         let scroll = ScrollArea::vertical()
            .max_height(popup_max_h)
            .max_width(popup_max_w)
            .content_margin(content_margin);
         let min_width = button_response.rect.width() - ui.spacing().button_padding.x * 2.0;

         match virtual_rows {
            Some((row_height, total_rows)) => {
               scroll
                  .show_rows(ui, row_height, total_rows, |ui, range| {
                     ui.set_width(ui.available_width().max(min_width));
                     ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                     contents(ui, range)
                  })
                  .inner
            }
            None => {
               scroll
                  .show(ui, |ui| {
                     ui.set_width(ui.available_width().max(min_width));
                     ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                     contents(ui, 0..0)
                  })
                  .inner
            }
         }
      })
   }
}

fn combo_box_with_image_button(
   ui: &mut Ui,
   id: Id,
   is_popup_open: bool,
   combo_box_visuals: Option<&ComboBoxVisuals>,
   selected_item: &Label,
   icon_painter: Option<Box<dyn FnOnce(&Ui, Rect, &WidgetVisuals, bool)>>,
   wrap_mode_override: Option<TextWrapMode>,
   width_override: Option<f32>,
) -> Response {
   let button_padding = ui.spacing().button_padding;
   let icon_width = ui.spacing().icon_width;
   let icon_spacing = ui.spacing().icon_spacing;
   let minimum_height = ui.spacing().interact_size.y;

   let wrap_mode = wrap_mode_override.unwrap_or_else(|| ui.wrap_mode());

   // Size Calculation
   let available_width = ui.available_width();
   let width_for_layout = if let Some(w) = width_override {
      (w - button_padding.x * 2.0 - icon_width - icon_spacing).max(0.0)
   } else {
      (available_width - button_padding.x * 2.0 - icon_width - icon_spacing).max(10.0)
   };

   let mut item_for_measurement = selected_item.clone();
   if wrap_mode_override.is_some() {
      item_for_measurement = item_for_measurement.wrap_mode(wrap_mode);
   }

   let (_, content_size) = item_for_measurement.galley_and_size(ui, width_for_layout);

   // Calculate the total inner size needed (content + icon)
   let inner_width = content_size.x + icon_spacing + icon_width;
   let inner_height = content_size.y.max(icon_width);

   let mut button_size = Vec2::new(
      inner_width + button_padding.x * 2.0,
      inner_height + button_padding.y * 2.0,
   );

   button_size.y = button_size.y.at_least(minimum_height);
   if let Some(w) = width_override {
      button_size.x = w;
   } else {
      button_size.x = button_size.x.at_least(ui.spacing().combo_width);
   }

   // Allocation & Interaction
   let (rect, response) = ui.allocate_exact_size(button_size, Sense::click());

   // The open flag is a state, not a transition: only the chevron eases, so it lands facing
   // up while the chrome has already switched.
   let arrow = animate_with(
      ui,
      id.with("arrow"),
      if is_popup_open { 1.0 } else { 0.0 },
      Motion::from_ui(ui),
   );

   // Painting
   if ui.is_rect_visible(rect) {
      let visuals = if is_popup_open {
         ui.visuals().widgets.open
      } else {
         ui.style().interact(&response).clone()
      };

      // Paint background
      let background_rect = rect.expand(visuals.expansion);
      let corner = combo_box_visuals.map(|v| v.corner_radius).unwrap_or(visuals.corner_radius);

      let across = InteractT::of(
         ui,
         response.id,
         response.hovered() || response.highlighted(),
         response.is_pointer_button_down_on() || response.clicked() || response.has_focus(),
         Motion::from_ui(ui),
      );
      // The popup being open outranks hover/press: it is a state, not a transition.
      let open = if is_popup_open { 1.0 } else { across.press };
      let weights = InteractT {
         hover: across.hover,
         press: open,
      };

      let widgets = &ui.style().visuals.widgets;
      let fill = combo_box_visuals
         .map(|v| weights.blend(v.bg, v.bg_hover, v.bg_open))
         .unwrap_or_else(|| {
            weights.blend(
               widgets.inactive.weak_bg_fill,
               widgets.hovered.weak_bg_fill,
               widgets.open.weak_bg_fill,
            )
         });

      let stroke = combo_box_visuals
         .map(|v| weights.blend(v.border, v.border_hover, v.border_open))
         .unwrap_or_else(|| {
            weights.blend(
               widgets.inactive.bg_stroke,
               widgets.hovered.bg_stroke,
               widgets.open.bg_stroke,
            )
         });

      if let Some(vis) = combo_box_visuals {
         let shadow_shape = vis.shadow.as_shape(background_rect, corner);
         ui.painter().add(shadow_shape);
      }

      let rect_shape = RectShape::new(
         background_rect,
         corner,
         fill,
         stroke,
         StrokeKind::Inside,
      );

      ui.painter().add(rect_shape);

      // Area for content (label + image) inside padding
      let content_total_rect = rect.shrink2(button_padding);

      let icon_rect = Align2::RIGHT_CENTER.align_size_within_rect(
         Vec2::splat(icon_width), // Square icon
         content_total_rect,
      );

      // Calculate rect for the LabelWithImage (remaining space to the left of the icon)
      let label_rect_width = (icon_rect.left() - content_total_rect.left() - icon_spacing).max(0.0);
      let label_rect = Rect::from_min_size(
         content_total_rect.min,
         Vec2::new(label_rect_width, content_total_rect.height()),
      );

      selected_item.paint_content_within_rect(ui, label_rect, &visuals);

      // Paint the icon
      if let Some(icon_painter) = icon_painter {
         icon_painter(ui, icon_rect, &visuals, is_popup_open);
      } else {
         paint_default_icon(
            ui.painter(),
            icon_rect,
            combo_box_visuals,
            &visuals,
            arrow,
         );
      }
   }

   response
}

/// The popup card: the stock popup chrome (menu margin, menu corner radius, `popup_shadow`)
/// with the theme's elevated surface instead of `Visuals::window_fill`, which every palette
/// aliases to the page background.
fn popup_frame(ui: &Ui, visuals: &ComboBoxVisuals) -> Frame {
   Frame::popup(ui.style()).fill(visuals.popup_bg).stroke(visuals.popup_border)
}

/// The disclosure chevron: a stroked `∨` in the theme's icon colour that rotates to point up
/// as `open` goes `0.0 → 1.0`, so opening and closing are one continuous motion.
fn paint_default_icon(
   painter: &Painter,
   rect: Rect,
   combo_box_visuals: Option<&ComboBoxVisuals>,
   visuals: &WidgetVisuals,
   open: f32,
) {
   let colour = combo_box_visuals.map(|v| v.icon).unwrap_or(visuals.fg_stroke.color);
   let half = Vec2::new(rect.width() * 0.28, rect.height() * 0.14);
   let rot = Rot2::from_angle(open * std::f32::consts::PI);
   let centre = rect.center();
   // Closed pose: the middle vertex hangs below its two ends.
   let closed = [
      Vec2::new(-half.x, -half.y),
      Vec2::new(0.0, half.y),
      Vec2::new(half.x, -half.y),
   ];
   let points: Vec<Pos2> = closed.iter().map(|v| centre + rot * *v).collect();
   painter.add(Shape::line(points, Stroke::new(1.5, colour)));
}

#[cfg(test)]
mod tests {
   use super::*;
   use crate::theme::{Theme, ThemeKind};
   use egui::{Context, Event, FullOutput, Modifiers, PointerButton, Pos2, RawInput};
   use std::cell::Cell;

   fn themed_ctx() -> Context {
      let ctx = Context::default();
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&ctx);
      ctx
   }

   fn screen() -> Rect {
      Rect::from_min_size(Pos2::ZERO, Vec2::new(420.0, 420.0))
   }

   fn run_frame(
      ctx: &Context,
      events: Vec<Event>,
      time: f64,
      body: impl FnMut(&mut Ui),
   ) -> FullOutput {
      let mut output = ctx.run_ui(
         RawInput {
            events,
            screen_rect: Some(screen()),
            time: Some(time),
            predicted_dt: 0.1,
            ..Default::default()
         },
         body,
      );
      output.textures_delta.clear();
      output
   }

   fn press(pos: Pos2, pressed: bool) -> Event {
      Event::PointerButton {
         pos,
         button: PointerButton::Primary,
         pressed,
         modifiers: Modifiers::NONE,
      }
   }

   /// Clicks the combo box open with a real press/release, parks the pointer away from the
   /// button (so only the open state can paint its chrome), settles the hover animation and
   /// returns the settled frame plus the button rect measured on the idle frame.
   fn open_popup(ctx: &Context, mut build: impl FnMut(&mut Ui)) -> (FullOutput, Rect) {
      // The idle button is the only rect painted with `bg` on a bare screen, so the frame
      // itself supplies the rect to click — never assume the ui starts at the origin.
      let idle = run_frame(ctx, Vec::new(), 10.0, &mut build);
      let button = rect_with_fill(&idle, combo_visuals(ctx).bg, -10.0)
         .map(|(rect, _)| rect)
         .expect("the idle button paints `bg`");

      let centre = button.center();
      run_frame(
         ctx,
         vec![Event::PointerMoved(centre), press(centre, true)],
         10.1,
         &mut build,
      );
      run_frame(ctx, vec![press(centre, false)], 10.2, &mut build);
      run_frame(
         ctx,
         vec![Event::PointerMoved(Pos2::new(400.0, 400.0))],
         10.3,
         &mut build,
      );
      run_frame(ctx, Vec::new(), 10.6, &mut build);
      let output = run_frame(ctx, Vec::new(), 10.9, &mut build);
      (output, button)
   }

   fn combo_visuals(ctx: &Context) -> ComboBoxVisuals {
      Theme::combo_box_visuals_from_ctx(ctx).expect("the test installs a theme")
   }

   /// The painted rect carrying `fill`, restricted to cards at or below `above`.
   ///
   /// A `Frame`'s card and shadow are painted as one `Shape::Vec`, so the walk descends.
   fn rect_with_fill(
      output: &FullOutput,
      fill: egui::Color32,
      above: f32,
   ) -> Option<(Rect, Stroke)> {
      fn find(shape: &Shape, fill: egui::Color32, above: f32) -> Option<(Rect, Stroke)> {
         match shape {
            Shape::Rect(rect) if rect.fill == fill && rect.rect.min.y >= above - 1.0 => {
               Some((rect.rect, rect.stroke))
            }
            Shape::Vec(shapes) => shapes.iter().find_map(|shape| find(shape, fill, above)),
            _ => None,
         }
      }
      output.shapes.iter().find_map(|clipped| find(&clipped.shape, fill, above))
   }

   /// Every fill painted at `centre` (a card's shadow and its body share one rect).
   fn fills_at(output: &FullOutput, centre: Pos2) -> Vec<egui::Color32> {
      fn collect(shape: &Shape, centre: Pos2, fills: &mut Vec<egui::Color32>) {
         match shape {
            Shape::Rect(rect) if (rect.rect.center() - centre).length() < 0.5 => {
               fills.push(rect.fill);
            }
            Shape::Vec(shapes) => shapes.iter().for_each(|shape| collect(shape, centre, fills)),
            _ => {}
         }
      }
      let mut fills = Vec::new();
      output
         .shapes
         .iter()
         .for_each(|clipped| collect(&clipped.shape, centre, &mut fills));
      fills
   }

   fn one_row(ui: &mut Ui) {
      ui.add(Label::new("Alpha", None));
   }

   #[test]
   fn the_popup_paints_the_elevated_surface() {
      let ctx = themed_ctx();
      let visuals = combo_visuals(&ctx);
      let (output, button) = open_popup(&ctx, |ui| {
         ComboBox::new("surface", Label::new("Pick", None))
            .width(160.0)
            .show_ui(ui, one_row);
      });

      let (rect, stroke) = rect_with_fill(&output, visuals.popup_bg, button.max.y)
         .expect("the popup card must be painted with `popup_bg`, not `Visuals::window_fill`");
      assert!(rect.min.y >= button.max.y, "popup {rect:?}");
      assert_eq!(
         stroke, visuals.popup_border,
         "the card keeps the theme border"
      );
   }

   #[test]
   fn the_popup_sits_a_gap_below_the_button() {
      let ctx = themed_ctx();
      let visuals = combo_visuals(&ctx);
      let (output, button) = open_popup(&ctx, |ui| {
         ComboBox::new("gap_default", Label::new("Pick", None))
            .width(160.0)
            .show_ui(ui, one_row);
      });
      let (rect, _) = rect_with_fill(&output, visuals.popup_bg, button.max.y).expect("popup card");
      let default_gap = rect.min.y - button.max.y;
      assert!(
         (default_gap - 4.0).abs() < 1.5,
         "the default gap is ThemeSpacing::xs (4), measured {default_gap}"
      );

      let ctx = themed_ctx();
      let (output, button) = open_popup(&ctx, |ui| {
         ComboBox::new("gap_twelve", Label::new("Pick", None))
            .width(160.0)
            .popup_gap(12.0)
            .show_ui(ui, one_row);
      });
      let (rect, _) = rect_with_fill(&output, visuals.popup_bg, button.max.y).expect("popup card");
      let custom_gap = rect.min.y - button.max.y;
      assert!(
         (custom_gap - 12.0).abs() < 1.5,
         "popup_gap(12) must move the card, measured {custom_gap}"
      );
   }

   #[test]
   fn the_popup_content_margin_is_adjustable() {
      let offset_with = |margin: Option<f32>| {
         let ctx = themed_ctx();
         let visuals = combo_visuals(&ctx);
         let row = Cell::new(Rect::NOTHING);
         let (output, button) = open_popup(&ctx, |ui| {
            let mut combo = ComboBox::new("margin", Label::new("Pick", None)).width(160.0);
            if let Some(margin) = margin {
               combo = combo.popup_content_margin(margin);
            }
            combo.show_ui(ui, |ui| {
               row.set(ui.add(Label::new("Alpha", None)).rect);
            });
         });
         let (rect, _) =
            rect_with_fill(&output, visuals.popup_bg, button.max.y).expect("popup card");
         row.get().min.y - rect.min.y
      };

      let default_offset = offset_with(None);
      assert!(
         default_offset >= 10.0,
         "menu margin (6) + the 5 pt content margin, measured {default_offset}"
      );

      let wide = offset_with(Some(20.0));
      assert!(
         wide - default_offset >= 14.0,
         "content margin 5 -> 20 must move the row by ~15, measured {}",
         wide - default_offset
      );
   }

   #[test]
   fn an_open_button_paints_the_open_chrome() {
      let ctx = themed_ctx();
      let visuals = combo_visuals(&ctx);
      let (output, button) = open_popup(&ctx, |ui| {
         ComboBox::new("open_chrome", Label::new("Pick", None))
            .width(160.0)
            .show_ui(ui, one_row);
      });

      let fills = fills_at(&output, button.center());
      assert!(
         fills.contains(&visuals.bg_open),
         "an open combo box paints `bg_open` while the pointer is elsewhere, painted {fills:?}"
      );
      assert!(
         !fills.contains(&visuals.bg),
         "an open combo box must not fall back to its idle fill, painted {fills:?}"
      );
   }

   /// The three points of the disclosure chevron, in paint order.
   fn chevron(output: &FullOutput) -> Option<Vec<Pos2>> {
      output.shapes.iter().find_map(|clipped| match &clipped.shape {
         Shape::Path(path) if path.points.len() == 3 => Some(path.points.clone()),
         _ => None,
      })
   }

   /// The y of the chevron's middle vertex — below its ends when closed, above when open.
   fn chevron_y(output: &FullOutput) -> f32 {
      let points = chevron(output).expect("the combo box paints its chevron");
      assert_eq!(points.len(), 3);
      points[1].y
   }

   fn closed_combo(salt: &'static str) -> impl FnMut(&mut Ui) {
      move |ui: &mut Ui| {
         ComboBox::new(salt, Label::new("Pick", None)).width(160.0).show_ui(ui, one_row);
      }
   }

   #[test]
   fn the_chevron_points_down_when_closed() {
      let ctx = themed_ctx();
      let mut build = closed_combo("arrow_closed");

      for time in [10.0, 10.1, 10.2] {
         let output = run_frame(&ctx, Vec::new(), time, &mut build);
         let points = chevron(&output).expect("the closed combo box paints its chevron");
         assert!(
            points[1].y > points[0].y && points[1].y > points[2].y,
            "a closed chevron points down, got {points:?}"
         );
      }
   }

   #[test]
   fn the_chevron_rotates_when_the_popup_opens() {
      let ctx = themed_ctx();
      let mut build = closed_combo("arrow_open");

      let idle = run_frame(&ctx, Vec::new(), 10.0, &mut build);
      let button = rect_with_fill(&idle, combo_visuals(&ctx).bg, -10.0)
         .map(|(rect, _)| rect)
         .expect("the idle button paints `bg`");
      let centre = button.center();

      // press (10.1), release (10.2, the click opens the popup), 10.3 is the frame the
      // chevron's target flips, so 10.35 is a fifth of the way into the 0.3 s leg.
      run_frame(
         &ctx,
         vec![Event::PointerMoved(centre), press(centre, true)],
         10.1,
         &mut build,
      );
      run_frame(&ctx, vec![press(centre, false)], 10.2, &mut build);
      run_frame(&ctx, Vec::new(), 10.3, &mut build);
      let mid = run_frame(&ctx, Vec::new(), 10.35, &mut build);
      let settled = run_frame(&ctx, Vec::new(), 10.75, &mut build);

      let down = chevron_y(&idle);
      let between = chevron_y(&mid);
      let up = chevron_y(&settled);

      assert!(
         (down - up).abs() > 1.0,
         "the chevron must actually swing: closed {down}, open {up}"
      );
      assert!(
         up < between && between < down,
         "10.35 is mid-rotation: {between} must sit strictly between {up} and {down}"
      );
   }

   #[test]
   fn the_chevron_rotates_back_when_the_popup_closes() {
      let ctx = themed_ctx();
      let mut build = closed_combo("arrow_close");

      let idle = run_frame(&ctx, Vec::new(), 10.0, &mut build);
      let button = rect_with_fill(&idle, combo_visuals(&ctx).bg, -10.0)
         .map(|(rect, _)| rect)
         .expect("the idle button paints `bg`");
      let centre = button.center();

      run_frame(
         &ctx,
         vec![Event::PointerMoved(centre), press(centre, true)],
         10.1,
         &mut build,
      );
      run_frame(&ctx, vec![press(centre, false)], 10.2, &mut build);
      // The popup reports open one frame after the release (as in the opening test), and a
      // first sight of the arrow id starts at 0.0, so the leg needs that frame to exist
      // before 10.9 can be a settled open chevron.
      run_frame(&ctx, Vec::new(), 10.3, &mut build);
      let open = run_frame(&ctx, Vec::new(), 10.9, &mut build);

      // Click again: the popup closes on 11.0, so the chevron starts back down there.
      run_frame(
         &ctx,
         vec![Event::PointerMoved(centre), press(centre, true)],
         10.95,
         &mut build,
      );
      run_frame(&ctx, vec![press(centre, false)], 11.0, &mut build);
      run_frame(&ctx, Vec::new(), 11.1, &mut build);
      let mid = run_frame(&ctx, Vec::new(), 11.15, &mut build);
      let settled = run_frame(&ctx, Vec::new(), 11.55, &mut build);

      let up = chevron_y(&open);
      let between = chevron_y(&mid);
      let down = chevron_y(&settled);

      assert!(
         (down - up).abs() > 1.0,
         "the chevron must swing back: open {up}, closed {down}"
      );
      assert!(
         up < between && between < down,
         "11.15 is mid-rotation: {between} must sit strictly between {up} and {down}"
      );
   }

   #[test]
   fn show_rows_builds_only_the_visible_rows() {
      const ROW_H: f32 = 24.0;
      const ROWS: usize = 10_000;

      let ctx = themed_ctx();
      let built = Cell::new(0usize);
      let (output, _) = open_popup(&ctx, |ui| {
         ComboBox::new("virtual", Label::new("Pick", None)).width(160.0).show_rows_ui(
            ui,
            ROW_H,
            ROWS,
            |ui, range| {
               built.set(built.get() + range.len());
               for index in range {
                  ui.label(format!("Row {index}"));
               }
            },
         );
      });

      assert!(
         !output.shapes.is_empty(),
         "the popup must have painted something"
      );

      let built = built.get();
      assert!(
         built > 0,
         "the visible rows must be built, built {built} of {ROWS}"
      );
      assert!(
         built < 200,
         "only the viewport is built — six frames over {ROWS} rows built {built}"
      );
   }
}
