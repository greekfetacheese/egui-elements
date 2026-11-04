//! A segmented tab strip.

use std::hash::Hash;

use egui::{
   CornerRadius, FontId, IdSalt, Rect, Response, Role, Sense, Stroke, StrokeKind, Ui, WidgetInfo,
   pos2, vec2,
};

use crate::theme::{Surface, SurfaceState, Theme};

/// A horizontal segmented control: one selected segment, index written back.
///
/// ```rust
/// # use egui::__run_test_ui;
/// # use egui_elements::widgets::Tabs;
/// # __run_test_ui(|ui| {
/// let mut selected = 0;
/// Tabs::new("kind", &mut selected, ["All", "Media", "Files"]).show(ui);
/// # });
/// ```
#[must_use = "You should call .show()"]
pub struct Tabs<'a> {
   id_salt: IdSalt,
   selected: &'a mut usize,
   labels: Vec<&'a str>,
   fill_width: bool,
   height: Option<f32>,
   gap: f32,
}

impl<'a> Tabs<'a> {
   /// Tabs over `labels`, writing the chosen index into `selected`.
   pub fn new(
      id_salt: impl Hash + std::fmt::Debug,
      selected: &'a mut usize,
      labels: impl IntoIterator<Item = &'a str>,
   ) -> Self {
      Self {
         id_salt: IdSalt::new(id_salt),
         selected,
         labels: labels.into_iter().collect(),
         fill_width: true,
         height: None,
         gap: 2.0,
      }
   }

   /// Stretch the strip across the available width. Default `true`.
   pub fn fill_width(mut self, fill: bool) -> Self {
      self.fill_width = fill;
      self
   }

   /// Strip height in points. Default: body text plus the button padding.
   pub fn height(mut self, height: f32) -> Self {
      self.height = Some(height);
      self
   }

   /// Space between segments, in points. Default `2.0`.
   pub fn gap(mut self, gap: f32) -> Self {
      self.gap = gap;
      self
   }

   /// Draw the strip. The returned [`Response`] reports `changed` when the index moved.
   pub fn show(self, ui: &mut Ui) -> Response {
      let theme = Theme::current(ui.ctx());
      let colors = theme.colors;
      let pad_x = ui.spacing().button_padding.x.max(10.0);
      let pad_y = ui.spacing().button_padding.y.max(6.0);
      let font = FontId::proportional(theme.typography.normal);
      let height = self.height.unwrap_or(theme.typography.normal + 2.0 * pad_y);
      let gap = self.gap;

      let widths: Vec<f32> = if self.labels.is_empty() {
         Vec::new()
      } else if self.fill_width {
         let gap_total = gap * (self.labels.len() - 1) as f32;
         let each = ((ui.available_width() - gap_total) / self.labels.len() as f32).max(1.0);
         vec![each; self.labels.len()]
      } else {
         self
            .labels
            .iter()
            .map(|label| {
               ui.painter()
                  .layout_no_wrap((*label).to_owned(), font.clone(), colors.text)
                  .size()
                  .x
                  + 2.0 * pad_x
            })
            .collect()
      };

      let strip_width: f32 =
         widths.iter().sum::<f32>() + gap * (self.labels.len().saturating_sub(1) as f32);
      let (rect, mut response) = ui.allocate_exact_size(vec2(strip_width, height), Sense::hover());

      let radius = CornerRadius::same(theme.corner_radius);
      let painter = ui.painter().clone();
      painter.rect_filled(rect, radius, theme.surface_fill(Surface::Well));
      painter.rect_stroke(
         rect,
         radius,
         Stroke::new(1.0, colors.border),
         StrokeKind::Inside,
      );

      if self.labels.is_empty() {
         return response;
      }

      let selected = (*self.selected).min(self.labels.len() - 1);
      let hovered = ui.ctx().pointer_hover_pos();
      let segment_radius = CornerRadius::same(theme.corner_radius.saturating_sub(2));
      let mut changed = false;
      let mut x = rect.left();

      for (index, label) in self.labels.iter().enumerate() {
         let width = widths[index];
         let seg = Rect::from_min_size(pos2(x, rect.top()), vec2(width, rect.height()));
         x += width + gap;

         let segment = ui.interact(
            seg,
            ui.scope_id().with(&self.id_salt).with(index),
            Sense::click(),
         );
         let is_selected = index == selected;
         segment
            .widget_info(|| WidgetInfo::selected(Role::Tab, ui.is_enabled(), is_selected, *label));

         if segment.clicked() {
            *self.selected = index;
            changed = true;
         }

         let is_hovered = segment.hovered() || hovered.is_some_and(|pos| seg.contains(pos));
         if is_selected {
            painter.rect_filled(seg.shrink(2.0), segment_radius, colors.highlight);
         } else if is_hovered {
            painter.rect_filled(
               seg.shrink(2.0),
               segment_radius,
               theme.state_fill(Surface::Panel, SurfaceState::Hover),
            );
         }

         let ink = if is_selected {
            colors.ink_on(colors.highlight)
         } else if is_hovered {
            colors.text
         } else {
            colors.text_muted
         };
         let galley = painter.layout_no_wrap((*label).to_owned(), font.clone(), ink);
         let pos = seg.center() - galley.size() / 2.0;
         painter.galley(pos, galley, ink);
      }

      if changed {
         response.mark_changed();
      }
      response
   }
}

#[cfg(test)]
mod tests {
   use super::*;
   use crate::theme::{Theme, ThemeKind};
   use egui::{
      Context, Event, FullOutput, Modifiers, PointerButton, Pos2, RawInput, Rect, pos2, vec2,
   };

   fn themed_ctx() -> Context {
      let ctx = Context::default();
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&ctx);
      ctx
   }

   fn frame(
      ctx: &Context,
      time: f64,
      events: Vec<Event>,
      mut body: impl FnMut(&mut Ui),
   ) -> FullOutput {
      let input = RawInput {
         events,
         time: Some(time),
         screen_rect: Some(Rect::from_min_size(
            pos2(0.0, 0.0),
            vec2(400.0, 300.0),
         )),
         ..Default::default()
      };
      let mut out = ctx.run_ui(input, |ui| body(ui));
      out.textures_delta.clear();
      out
   }

   fn press(pos: Pos2) -> Event {
      Event::PointerButton {
         pos,
         button: PointerButton::Primary,
         pressed: true,
         modifiers: Modifiers::NONE,
      }
   }

   fn release(pos: Pos2) -> Event {
      Event::PointerButton {
         pos,
         button: PointerButton::Primary,
         pressed: false,
         modifiers: Modifiers::NONE,
      }
   }

   #[test]
   fn clicking_a_segment_selects_it() {
      let ctx = themed_ctx();
      let mut selected = 0usize;
      let mut rect = Rect::ZERO;

      frame(&ctx, 0.0, vec![], |ui| {
         rect = Tabs::new("tabs", &mut selected, ["All", "Media"]).show(ui).rect;
      });

      let at = pos2(rect.left() + rect.width() * 0.75, rect.center().y);
      frame(&ctx, 0.1, vec![press(at)], |ui| {
         Tabs::new("tabs", &mut selected, ["All", "Media"]).show(ui);
      });
      frame(&ctx, 0.2, vec![release(at)], |ui| {
         Tabs::new("tabs", &mut selected, ["All", "Media"]).show(ui);
      });

      assert_eq!(
         selected, 1,
         "the second segment must take the selection"
      );
   }

   #[test]
   fn fill_width_stretches_to_the_row() {
      let ctx = themed_ctx();
      let mut wide = 0usize;
      let mut narrow = 0usize;
      let mut wide_rect = Rect::ZERO;
      let mut narrow_rect = Rect::ZERO;

      frame(&ctx, 0.0, vec![], |ui| {
         wide_rect = Tabs::new("wide", &mut wide, ["All", "Media"]).fill_width(true).show(ui).rect;
         narrow_rect = Tabs::new("narrow", &mut narrow, ["All", "Media"])
            .fill_width(false)
            .show(ui)
            .rect;
      });

      assert!(
         wide_rect.width() > 380.0,
         "fill_width must use the row"
      );
      assert!(narrow_rect.width() < wide_rect.width());
   }

   #[test]
   fn an_empty_strip_does_not_panic() {
      let ctx = themed_ctx();
      let mut selected = 0usize;
      frame(&ctx, 0.0, vec![], |ui| {
         Tabs::new("empty", &mut selected, [] as [&str; 0]).show(ui);
      });
   }
}
