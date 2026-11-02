//! A flex row/column container that measures its items once, then places them into exact cells.

use std::hash::Hash;

use egui::{
   Align, Id, InnerResponse, Layout, Pos2, Rect, Response, Sense, Ui, UiBuilder, Vec2, pos2, vec2,
};

use super::solver::{Axis, CrossAlign, ItemSize, ItemSpec, Justify, resolve};
use super::state::FlexState;

/// One flex item.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use = "add it to a FlexUi with `item()`"]
pub struct Item {
   size: ItemSize,
   align: Option<CrossAlign>,
   content: CrossAlign,
}

impl Item {
   /// Content sized: the measured natural size (CSS `flex-basis: auto`, no grow).
   pub fn new() -> Self {
      Self {
         size: ItemSize::Auto,
         align: None,
         content: CrossAlign::Start,
      }
   }

   /// A fixed main-axis extent in points (CSS `width: Npx`).
   pub fn fixed(main: f32) -> Self {
      Self {
         size: ItemSize::Fixed(main),
         align: None,
         content: CrossAlign::Start,
      }
   }

   /// Basis 0, shares the free main-axis space by weight (CSS `flex: N`). Equal weights give equal
   /// widths regardless of the captions — the equal-button-row case.
   pub fn grow(weight: f32) -> Self {
      Self {
         size: ItemSize::Grow(weight),
         align: None,
         content: CrossAlign::Start,
      }
   }

   /// `align-self` override for this item.
   pub fn align(self, align: CrossAlign) -> Self {
      Self {
         align: Some(align),
         ..self
      }
   }

   /// Where the item's content sits inside its cell along the main axis when the cell is wider (row) or
   /// taller (column) than the content. CSS `justify-self` for the cell's own box.
   ///
   /// `Start` (the default) is today's behaviour. `Center` is what makes a growing cell look centred
   /// instead of hugging the item before it. `Stretch` keeps the content box full-size (no effect on the
   /// widget's own size).
   pub fn align_content(self, content: CrossAlign) -> Self {
      Self { content, ..self }
   }

   pub(crate) fn spec(self) -> ItemSpec {
      ItemSpec {
         size: self.size,
         align: self.align,
      }
   }
}

impl Default for Item {
   fn default() -> Self {
      Self::new()
   }
}

/// A row or column of items laid out by the solver.
#[must_use = "You should call .show()"]
pub struct Flex {
   axis: Axis,
   id: Id,
   gap: Option<f32>,
   justify: Justify,
   align: CrossAlign,
   fill_main: bool,
}

/// Whether the container is placing cached items or measuring them for the first time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
   Placing,
   Measuring,
}

/// The layout a cell's child `Ui` starts from.
fn cell_layout(axis: Axis) -> Layout {
   match axis {
      Axis::Row => Layout::left_to_right(Align::Center),
      Axis::Column => Layout::top_down(Align::Min),
   }
}

/// The container's cross extent: `0.0` for a row, the parent's width for a column.
fn container_cross(axis: Axis, avail: Vec2) -> f32 {
   match axis {
      Axis::Row => 0.0,
      Axis::Column => axis.cross_of(avail),
   }
}

/// The container size the solver is handed: the main span plus the cross extent above.
fn container_size(axis: Axis, main: f32, avail: Vec2) -> Vec2 {
   axis.size(main, container_cross(axis, avail))
}

/// The main span of a set of natural sizes, gaps included.
fn content_main(axis: Axis, sizes: &[Vec2], gap: f32) -> f32 {
   let n = sizes.len();
   if n == 0 {
      return 0.0;
   }
   let items: f32 = sizes.iter().map(|s| axis.main_of(*s)).sum();
   items + gap * (n - 1) as f32
}

/// The container size for a set of measured sizes.
fn container_for(axis: Axis, fill_main: bool, sizes: &[Vec2], avail: Vec2, gap: f32) -> Vec2 {
   let main = if fill_main {
      axis.main_of(avail)
   } else {
      content_main(axis, sizes, gap)
   };
   container_size(axis, main, avail)
}

/// The per-call state of a [`Flex`]; items are added through it.
pub struct FlexUi<'a> {
   ui: &'a mut Ui,
   mode: Mode,
   prev: Option<FlexState>,
   axis: Axis,
   gap: f32,
   justify: Justify,
   align: CrossAlign,
   fill_main: bool,
   avail: Vec2,
   specs: Vec<ItemSpec>,
   sizes: Vec<Vec2>,
   cells: Option<Vec<Rect>>,
   origin: Pos2,
   acc: Option<(Rect, Response)>,
   needs_measure: bool,
   resized: bool,
}

impl Flex {
   pub fn row(id_salt: impl Hash + std::fmt::Debug) -> Self {
      Self {
         axis: Axis::Row,
         id: Id::unique(id_salt),
         gap: None,
         justify: Justify::Start,
         align: CrossAlign::Start,
         fill_main: true,
      }
   }

   pub fn column(id_salt: impl Hash + std::fmt::Debug) -> Self {
      Self {
         axis: Axis::Column,
         id: Id::unique(id_salt),
         gap: None,
         justify: Justify::Start,
         align: CrossAlign::Start,
         fill_main: false,
      }
   }

   pub fn gap(self, gap: f32) -> Self {
      Self {
         gap: Some(gap),
         ..self
      }
   }

   pub fn justify(self, justify: Justify) -> Self {
      Self { justify, ..self }
   }

   pub fn align_items(self, align: CrossAlign) -> Self {
      Self { align, ..self }
   }

   /// Span the available main-axis space. A row already does; a column does not by default, because a
   /// column that eats its parent's height is the bug this module exists to remove.
   pub fn fill_main(self, fill: bool) -> Self {
      Self {
         fill_main: fill,
         ..self
      }
   }

   pub fn show<R>(self, ui: &mut Ui, add: impl FnOnce(&mut FlexUi<'_>) -> R) -> InnerResponse<R> {
      let id = ui.make_persistent_id(self.id);
      let prev = FlexState::load(ui.ctx(), id);
      let avail = ui.available_size();
      let gap = self.gap.unwrap_or(match self.axis {
         Axis::Row => ui.spacing().item_spacing.x,
         Axis::Column => ui.spacing().item_spacing.y,
      });
      let mode = if prev.as_ref().is_some_and(|p| p.same_space(avail)) {
         Mode::Placing
      } else {
         Mode::Measuring
      };

      let mut flex = FlexUi {
         ui,
         mode,
         prev,
         axis: self.axis,
         gap,
         justify: self.justify,
         align: self.align,
         fill_main: self.fill_main,
         avail,
         specs: Vec::new(),
         sizes: Vec::new(),
         cells: None,
         origin: Pos2::ZERO,
         acc: None,
         needs_measure: false,
         resized: false,
      };

      let inner = add(&mut flex);
      let response = flex.teardown(id, avail);
      InnerResponse { response, inner }
   }
}

impl FlexUi<'_> {
   /// Resolve the measured specs into cells, allocate the container and update the cache.
   fn teardown(mut self, id: Id, avail: Vec2) -> Response {
      let n = self.specs.len();
      debug_assert_eq!(self.sizes.len(), n);
      if self.prev.as_ref().is_none_or(|p| p.specs.len() != n || p.specs != self.specs) {
         self.needs_measure = true;
      }
      let container = container_for(
         self.axis,
         self.fill_main,
         &self.sizes,
         avail,
         self.gap,
      );
      let out = resolve(
         self.axis,
         &self.specs,
         &self.sizes,
         container,
         self.gap,
         self.justify,
         self.align,
      );

      let response = match self.acc.take() {
         Some((_, response)) => response,
         None => self.ui.allocate_exact_size(out.size, Sense::hover()).1,
      };

      let state = FlexState {
         specs: self.specs,
         sizes: self.sizes,
         avail,
      };
      if self.needs_measure {
         state.store(self.ui.ctx(), id);
         if self.ui.is_visible() {
            self.ui.request_discard("new Flex");
         }
      } else if self.resized {
         state.store(self.ui.ctx(), id);
         self.ui.ctx().request_repaint();
      }

      response
   }

   /// Add one item. Items execute **immediately**; the returned `InnerResponse::inner` is the closure's
   /// return value and `.response` is the cell's response (`Ui::response` of the child). Use the widget's
   /// own response (from inside the closure) for hover/click state.
   ///
   /// The closure runs a **second time** on the frame the container measures (the item count, the
   /// available space or a specified size changed): the measuring pass is `invisible()` and therefore
   /// `disabled()`, so no interaction can fire from it, but a closure that mutates host state outside a
   /// widget response would apply twice.
   pub fn item<R>(&mut self, item: Item, add: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
      let i = self.specs.len();
      self.specs.push(item.spec());

      if self.mode == Mode::Placing {
         let prev = self.prev.as_ref().expect("placing requires a cache");
         if i == 0 {
            let cached_container = container_for(
               self.axis,
               self.fill_main,
               &prev.sizes,
               self.avail,
               self.gap,
            );
            let cached = resolve(
               self.axis,
               &prev.specs,
               &prev.sizes,
               cached_container,
               self.gap,
               self.justify,
               self.align,
            );
            self.cells = Some(cached.rects);
            let (rect, response) = self.ui.allocate_exact_size(cached.size, Sense::hover());
            self.acc = Some((rect, response));
            self.origin = rect.min;
         }

         if i < prev.sizes.len() && item.spec() == prev.specs[i] {
            let cells = self.cells.as_ref().expect("cells are placed on item 0");
            let cell = cells[i];
            // row: keep the cell's height, shrink the width to the content and place it per `content`
            // column: keep the cell's width, shrink the height and place it per `content`
            let natural = prev.sizes[i];
            let child_size = match self.axis {
               Axis::Row => vec2(natural.x.min(cell.width()), cell.height()),
               Axis::Column => vec2(cell.width(), natural.y.min(cell.height())),
            };
            let child_size = match item.content {
               // `Start` keeps today's full-cell box (and `Stretch` keeps it full-size).
               CrossAlign::Center | CrossAlign::End => child_size,
               _ => cell.size(),
            };
            let child_min = match (self.axis, item.content) {
               (Axis::Row, CrossAlign::Center) => {
                  pos2(cell.center().x - child_size.x / 2.0, cell.top())
               }
               (Axis::Row, CrossAlign::End) => pos2(cell.right() - child_size.x, cell.top()),
               (Axis::Column, CrossAlign::Center) => {
                  pos2(cell.left(), cell.center().y - child_size.y / 2.0)
               }
               (Axis::Column, CrossAlign::End) => pos2(cell.left(), cell.bottom() - child_size.y),
               _ => cell.min, // Start, and every Stretch case
            };
            let mut child = self.ui.new_child(
               UiBuilder::new()
                  .max_rect(Rect::from_min_size(
                     self.origin + child_min.to_vec2(),
                     child_size,
                  ))
                  .layout(cell_layout(self.axis)),
            );
            let inner = add(&mut child);
            self.sizes.push(prev.sizes[i]);

            let effective = item.align.unwrap_or(self.align);
            if matches!(item.size, ItemSize::Auto) && effective != CrossAlign::Stretch {
               let measured = child.min_rect().size();
               let cached = prev.sizes[i];
               if (measured.x - cached.x).abs() > 0.5 || (measured.y - cached.y).abs() > 0.5 {
                  self.sizes[i] = measured;
                  self.resized = true;
               }
            }

            return InnerResponse {
               response: child.response(),
               inner,
            };
         }

         self.needs_measure = true;
         self.mode = Mode::Measuring;
      }

      self.needs_measure = true;
      let mut child = self.ui.new_child(
         UiBuilder::new()
            .max_rect(self.ui.available_rect_before_wrap())
            .layout(cell_layout(self.axis))
            .sizing_pass()
            .invisible(),
      );
      let inner = add(&mut child);
      self.sizes.push(child.min_rect().size());
      InnerResponse {
         response: child.response(),
         inner,
      }
   }
}

#[cfg(test)]
mod tests {
   use super::*;
   use crate::theme::{Theme, ThemeKind};
   use egui::vec2;

   fn themed_ctx() -> egui::Context {
      let ctx = egui::Context::default();
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&ctx);
      ctx
   }

   fn frame(ctx: &egui::Context, screen: Vec2, body: impl FnMut(&mut Ui)) -> egui::FullOutput {
      let input = egui::RawInput {
         screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, screen)),
         ..Default::default()
      };
      let mut out = ctx.run_ui(input, body);
      out.textures_delta.clear();
      out
   }

   fn extra_passes(output: &egui::FullOutput) -> usize {
      output.platform_output.num_completed_passes.saturating_sub(1)
   }

   #[test]
   fn a_conditional_item_change_re_measures_once() {
      let ctx = themed_ctx();
      let cells = std::cell::RefCell::new(Vec::new());

      let run = |has_seed: bool, cells: &std::cell::RefCell<Vec<Rect>>| {
         drop(frame(&ctx, vec2(500.0, 400.0), |ui| {
            cells.borrow_mut().clear();
            ui.scope_builder(
               UiBuilder::new().max_rect(Rect::from_min_size(
                  egui::Pos2::ZERO,
                  vec2(500.0, 400.0),
               )),
               |ui| {
                  Flex::row("conditional").gap(8.0).show(ui, |f| {
                     for caption in ["Copy Key", "Copy Seed Phrase"] {
                        f.item(Item::grow(1.0), |ui| {
                           cells.borrow_mut().push(ui.max_rect());
                           ui.add(crate::widgets::Button::new(caption));
                        });
                     }
                     if has_seed {
                        f.item(Item::grow(1.0), |ui| {
                           cells.borrow_mut().push(ui.max_rect());
                           ui.add(crate::widgets::Button::new("Show QR Code"));
                        });
                     }
                  });
               },
            );
         }));
         cells.borrow().clone()
      };

      let first = run(true, &cells);
      assert_eq!(first.len(), 3);
      for cell in first.iter() {
         assert!(
            (cell.width() - 161.33).abs() < 0.5,
            "frame 1 cell width {} != 161.33",
            cell.width()
         );
      }

      let second = run(false, &cells);
      assert_eq!(
         second.len(),
         2,
         "stale third cell survived: {:?}",
         second
      );
      for cell in second.iter() {
         assert!(
            (cell.width() - 246.0).abs() < 0.5,
            "frame 2 cell width {} != 246.0",
            cell.width()
         );
      }

      let third = frame(&ctx, vec2(500.0, 400.0), |ui| {
         cells.borrow_mut().clear();
         ui.scope_builder(
            UiBuilder::new().max_rect(Rect::from_min_size(
               egui::Pos2::ZERO,
               vec2(500.0, 400.0),
            )),
            |ui| {
               Flex::row("conditional").gap(8.0).show(ui, |f| {
                  for caption in ["Copy Key", "Copy Seed Phrase"] {
                     f.item(Item::grow(1.0), |ui| {
                        cells.borrow_mut().push(ui.max_rect());
                        ui.add(crate::widgets::Button::new(caption));
                     });
                  }
               });
            },
         );
      });
      assert_eq!(
         extra_passes(&third),
         0,
         "steady frame still ran a second pass"
      );
   }

   #[test]
   fn the_second_frame_requests_no_discard() {
      let ctx = themed_ctx();
      let mut outputs = Vec::new();
      for _ in 0..3 {
         outputs.push(frame(&ctx, vec2(500.0, 400.0), |ui| {
            ui.scope_builder(
               UiBuilder::new().max_rect(Rect::from_min_size(
                  egui::Pos2::ZERO,
                  vec2(500.0, 400.0),
               )),
               |ui| {
                  Flex::row("steady").gap(8.0).show(ui, |f| {
                     f.item(Item::new(), |ui| {
                        ui.add(crate::widgets::Label::new("Normal text", None));
                     });
                     f.item(Item::grow(1.0), |ui| {
                        ui.add(crate::widgets::Button::new("Click me"));
                     });
                  });
               },
            );
         }));
      }
      let passes: Vec<_> = outputs.iter().map(extra_passes).collect();
      assert!(
         passes[0] >= 1,
         "first frame did not measure: {:?}",
         passes
      );
      assert_eq!(
         passes[1], 0,
         "second frame ran a second pass: {:?}",
         passes
      );
      assert_eq!(
         passes[2], 0,
         "third frame ran a second pass: {:?}",
         passes
      );
   }
}
