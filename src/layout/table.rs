//! A data-driven table container: a header plus one row per item, with the column extents shared by
//! every line.
//!
//! The table resolves its [`Column`] specs once with [`resolve_columns`], measures a header plus every
//! cell in an invisible sizing pass, and then places the header and every row into the same column
//! bands.
//!
//! # The measure pass
//!
//! Measuring takes two passes on the frame a table first appears (or when the row count, the available
//! space, the column specs or [`Table::key`] change): the first pass measures into a `sizing_pass()`
//! `invisible()` child `Ui` and requests a discard, the second places. The cell and row closures
//! therefore run **twice** on a measuring frame. The measuring `Ui` is invisible and disabled, so no
//! widget can report an interaction from it, but a closure that mutates host state outside a widget
//! response would apply twice.
//!
//! # `key`
//!
//! The cache sees the row count and the `id_salt`, not the row *content*. Pass something that changes
//! with the content (a page index, a filter hash, a sort order) to [`Table::key`] and the table
//! measures again on the frame that value changes.
//!
//! # Columns and cells
//!
//! An [`Column::Auto`] column is as wide as its widest measured cell in the header or the body; a
//! [`Column::Fr`] column shares the space left over by the fixed and auto columns; a
//! [`Column::Fixed`] column keeps its width. Cells are **not clipped** — chrome such as a button's
//! hover expansion or a badge may paint outside its cell, so budget the gap for it.

use std::hash::{DefaultHasher, Hash, Hasher};

use egui::{
   Align, Color32, Id, InnerResponse, Layout, Rect, Response, Sense, Ui, UiBuilder, Vec2, pos2,
   vec2,
};

use super::solver::{Column, resolve_columns};
use super::state::TableState;

/// A header-plus-rows table over a slice of data.
///
/// The table measures once, then places on every frame: the header and every row share the same column
/// extents, `Auto` columns fit their widest measured cell, and `Fr` columns share the leftover width.
/// See the [module docs](self) for the measure-pass contract and [`Table::key`].
#[must_use = "You should call .show()"]
pub struct Table<'a, T> {
   id: Id,
   rows: &'a [T],
   columns: Vec<Column>,
   gap: Option<f32>,
   row_gap: Option<f32>,
   header_rule: bool,
   key: u64,
}

/// Whether the table is placing its cached sizes or measuring for the first time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
   Placing,
   Measuring,
}

/// The layout every cell's child `Ui` starts from: left-to-right, vertically centred, not clipped.
fn cell_layout() -> Layout {
   Layout::left_to_right(Align::Center)
}

/// The height of the table band: the header, the gap under it, and every row plus the gaps between
/// them.
fn band_height(has_header: bool, header_height: f32, heights: &[f32], row_gap: f32) -> f32 {
   let rows = heights.len();
   let mut height = header_height;
   if has_header && rows > 0 {
      height += row_gap;
   }
   height += heights.iter().sum::<f32>();
   if rows > 1 {
      height += row_gap * (rows - 1) as f32;
   }
   height
}

/// The column a cell call belongs to. A call past the column count is a host bug: it is asserted in
/// debug builds and rendered at the last column's extent instead of panicking.
fn column_index(i: usize, len: usize) -> Option<usize> {
   debug_assert!(
      i < len || len == 0,
      "Table cell {i} is past the {len} configured columns"
   );
   (len > 0).then(|| i.min(len - 1))
}

/// What a cell records while the table measures.
struct Measure<'a> {
   widths: &'a mut [f32],
   height: &'a mut f32,
}

impl<'a, T> Table<'a, T> {
   pub fn new(id_salt: impl Hash + std::fmt::Debug, rows: &'a [T]) -> Self {
      Self {
         id: Id::unique(id_salt),
         rows,
         columns: Vec::new(),
         gap: None,
         row_gap: None,
         header_rule: false,
         key: 0,
      }
   }

   pub fn columns(mut self, columns: impl IntoIterator<Item = Column>) -> Self {
      self.columns = columns.into_iter().collect();
      self
   }

   /// The gap between columns. Defaults to [`egui::Spacing::item_spacing`]`.x`.
   pub fn gap(mut self, gap: f32) -> Self {
      self.gap = Some(gap);
      self
   }

   /// The gap between rows. Defaults to [`egui::Spacing::item_spacing`]`.y`.
   pub fn row_gap(mut self, row_gap: f32) -> Self {
      self.row_gap = Some(row_gap);
      self
   }

   /// Draw a 1px separator under the header band. Default: `false`.
   pub fn header_rule(mut self, header_rule: bool) -> Self {
      self.header_rule = header_rule;
      self
   }

   /// Mix a content key into the cache key. See the [module docs](self).
   pub fn key(mut self, key: impl Hash) -> Self {
      let mut hasher = DefaultHasher::new();
      key.hash(&mut hasher);
      self.key = hasher.finish();
      self
   }

   /// Show the table, running `add` against its header and rows.
   ///
   /// `add` runs **twice** on a measuring frame: the table first measures its header and every cell in
   /// an invisible, disabled sizing pass, then places them into shared column extents on the second
   /// pass. Widget responses from the measuring pass cannot report an interaction, but a closure that
   /// mutates host state outside a response would apply twice. Pass [`Table::key`] when the row
   /// *content* changes without changing the row count.
   ///
   /// The header and every row are placed into the same column extents: `Auto` columns fit their
   /// widest measured cell, `Fr` columns share the width left over by the fixed and auto columns, and
   /// `Fixed` columns keep their width.
   pub fn show<R>(
      self,
      ui: &mut Ui,
      add: impl FnOnce(&mut TableUi<'_, '_, T>) -> R,
   ) -> InnerResponse<R> {
      let id = ui.make_persistent_id(self.id);
      let prev = TableState::load(ui.ctx(), id);
      let avail = ui.available_size();
      let gap = self.gap.unwrap_or(ui.spacing().item_spacing.x);
      let row_gap = self.row_gap.unwrap_or(ui.spacing().item_spacing.y);

      let placing = prev.as_ref().is_some_and(|p| {
         p.same_space(avail)
            && p.columns == self.columns
            && p.heights.len() == self.rows.len()
            && p.key == self.key
      });
      let mode = if placing {
         Mode::Placing
      } else {
         Mode::Measuring
      };
      let widths = if mode == Mode::Measuring {
         vec![0.0; self.columns.len()]
      } else {
         Vec::new()
      };

      let mut table = TableUi {
         ui,
         data: self.rows,
         columns: self.columns,
         widths,
         scratch: Vec::new(),
         mode,
         prev,
         gap,
         row_gap,
         header_rule: self.header_rule,
         key: self.key,
         avail,
         heights: Vec::new(),
         header_height: 0.0,
         has_header: false,
         band: None,
      };
      let inner = add(&mut table);
      let response = table.teardown(id, avail);
      InnerResponse { response, inner }
   }
}

/// The per-call state of a [`Table`]; the header and the rows are added through it.
pub struct TableUi<'a, 'ui, T> {
   ui: &'ui mut Ui,
   data: &'a [T],
   columns: Vec<Column>,
   /// The resolved column extents when placing, the measured natural widths when measuring.
   widths: Vec<f32>,
   /// Reused per-row cell rects.
   scratch: Vec<Rect>,
   mode: Mode,
   prev: Option<TableState>,
   gap: f32,
   row_gap: f32,
   header_rule: bool,
   key: u64,
   avail: Vec2,
   heights: Vec<f32>,
   header_height: f32,
   has_header: bool,
   band: Option<(Rect, Response)>,
}

impl<T> TableUi<'_, '_, T> {
   /// The number of columns this table was built with.
   pub fn columns(&self) -> usize {
      self.columns.len()
   }

   /// Resolve the cached widths and allocate the whole table band once, on the first `header()` or
   /// `rows()` call.
   fn ensure_band(&mut self) {
      if self.band.is_some() {
         return;
      }
      let prev = self.prev.as_ref().expect("placing requires a cache");
      let out = resolve_columns(
         &self.columns,
         &prev.widths,
         self.avail.x,
         self.gap,
      );
      self.widths = out.widths;
      self.heights = prev.heights.clone();
      self.header_height = prev.header_height;
      let height = band_height(
         self.has_header,
         self.header_height,
         &self.heights,
         self.row_gap,
      );
      let (rect, response) = self.ui.allocate_exact_size(vec2(out.total, height), Sense::hover());
      self.band = Some((rect, response));
   }

   /// Add the header line. The `i`-th [`TableHeader::cell`] call is column `i`.
   pub fn header(&mut self, add: impl FnOnce(&mut TableHeader<'_>)) {
      self.has_header = true;
      match self.mode {
         Mode::Placing => {
            self.ensure_band();
            let band = self.band.as_ref().expect("the band is allocated above").0;
            let header_height = self.header_height;
            let mut cells = std::mem::take(&mut self.scratch);
            cells.clear();
            let mut x = band.min.x;
            for width in &self.widths {
               cells.push(Rect::from_min_size(
                  pos2(x, band.min.y),
                  vec2(*width, header_height),
               ));
               x += *width + self.gap;
            }
            if self.header_rule {
               let stroke = self.ui.visuals().widgets.noninteractive.bg_stroke;
               self.ui.painter().hline(band.x_range(), band.min.y + header_height, stroke);
            }
            {
               let mut header = TableHeader {
                  ui: &mut *self.ui,
                  cells: &cells,
                  measure: None,
                  next: 0,
               };
               add(&mut header);
            }
            self.scratch = cells;
         }
         Mode::Measuring => {
            let mut widths = std::mem::take(&mut self.widths);
            let mut height = self.header_height;
            {
               let mut header = TableHeader {
                  ui: &mut *self.ui,
                  cells: &[],
                  measure: Some(Measure {
                     widths: &mut widths,
                     height: &mut height,
                  }),
                  next: 0,
               };
               add(&mut header);
            }
            self.widths = widths;
            self.header_height = height;
         }
      }
   }

   /// Add one body row per item. The `i`-th [`TableRow::cell`] call is column `i`; a row that adds
   /// fewer cells than columns leaves the remaining columns empty.
   pub fn rows(&mut self, mut add: impl FnMut(&mut TableRow<'_>, &T)) {
      match self.mode {
         Mode::Placing => {
            self.ensure_band();
            let band = self.band.as_ref().expect("the band is allocated above").0;
            let mut y = band.min.y + self.header_height;
            if self.has_header {
               y += self.row_gap;
            }
            for (i, item) in self.data.iter().enumerate() {
               let height = self.heights[i];
               let row_band = Rect::from_min_size(pos2(band.min.x, y), vec2(band.width(), height));
               let mut cells = std::mem::take(&mut self.scratch);
               cells.clear();
               let mut x = band.min.x;
               for width in &self.widths {
                  cells.push(Rect::from_min_size(
                     pos2(x, y),
                     vec2(*width, height),
                  ));
                  x += *width + self.gap;
               }
               {
                  let mut row = TableRow {
                     ui: &mut *self.ui,
                     cells: &cells,
                     measure: None,
                     next: 0,
                     rect: row_band,
                     index: i,
                  };
                  add(&mut row, item);
               }
               self.scratch = cells;
               y += height + self.row_gap;
            }
         }
         Mode::Measuring => {
            let data = self.data;
            let measure_rect = self.ui.available_rect_before_wrap();
            let mut widths = std::mem::take(&mut self.widths);
            for (i, item) in data.iter().enumerate() {
               let mut height = 0.0f32;
               {
                  let mut row = TableRow {
                     ui: &mut *self.ui,
                     cells: &[],
                     measure: Some(Measure {
                        widths: &mut widths,
                        height: &mut height,
                     }),
                     next: 0,
                     rect: measure_rect,
                     index: i,
                  };
                  add(&mut row, item);
               }
               self.heights.push(height);
            }
            self.widths = widths;
         }
      }
   }

   /// Resolve the column extents, store the cache and return the table's band response.
   fn teardown(mut self, id: Id, avail: Vec2) -> Response {
      let out = resolve_columns(&self.columns, &self.widths, avail.x, self.gap);
      let response = match self.band.take() {
         Some((_, response)) => response,
         None => {
            let height = band_height(
               self.has_header,
               self.header_height,
               &self.heights,
               self.row_gap,
            );
            self.ui.allocate_exact_size(vec2(out.total, height), Sense::hover()).1
         }
      };

      if self.mode == Mode::Measuring {
         let state = TableState {
            columns: self.columns,
            widths: self.widths,
            heights: self.heights,
            header_height: self.header_height,
            key: self.key,
            avail,
         };
         state.store(self.ui.ctx(), id);
         if self.ui.is_visible() {
            self.ui.request_discard("new Table");
         }
      }

      response
   }
}

/// The header line of a [`Table`].
pub struct TableHeader<'a> {
   ui: &'a mut Ui,
   cells: &'a [Rect],
   measure: Option<Measure<'a>>,
   next: usize,
}

impl TableHeader<'_> {
   /// Add one header cell. The cell layout is left-to-right, vertically centred, and not clipped.
   pub fn cell<R>(&mut self, add: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
      let i = self.next;
      self.next += 1;

      if let Some(measure) = self.measure.as_mut() {
         let column = column_index(i, measure.widths.len());
         let mut child = self.ui.new_child(
            UiBuilder::new()
               .max_rect(self.ui.available_rect_before_wrap())
               .layout(cell_layout())
               .sizing_pass()
               .invisible(),
         );
         let inner = add(&mut child);
         let measured = child.min_rect();
         if let Some(column) = column {
            measure.widths[column] = measure.widths[column].max(measured.width());
         }
         *measure.height = measure.height.max(measured.height());
         return InnerResponse {
            response: child.response(),
            inner,
         };
      }

      let rect = match column_index(i, self.cells.len()) {
         Some(column) => self.cells[column],
         None => self.ui.available_rect_before_wrap(),
      };
      let mut child = self.ui.new_child(UiBuilder::new().max_rect(rect).layout(cell_layout()));
      let inner = add(&mut child);
      InnerResponse {
         response: child.response(),
         inner,
      }
   }
}

/// One body row of a [`Table`].
pub struct TableRow<'a> {
   ui: &'a mut Ui,
   cells: &'a [Rect],
   measure: Option<Measure<'a>>,
   next: usize,
   rect: Rect,
   index: usize,
}

impl TableRow<'_> {
   /// The row's index in the data slice.
   pub fn index(&self) -> usize {
      self.index
   }

   /// The row's band: the table's full width and the row's height.
   pub fn rect(&self) -> Rect {
      self.rect
   }

   /// Paint the row band immediately, behind whatever the row's cells draw.
   pub fn fill(&mut self, color: Color32) {
      self.ui.painter().rect_filled(self.rect, 0.0, color);
   }

   /// Add one cell. The cell layout is left-to-right, vertically centred, and not clipped.
   pub fn cell<R>(&mut self, add: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
      let i = self.next;
      self.next += 1;

      if let Some(measure) = self.measure.as_mut() {
         let column = column_index(i, measure.widths.len());
         let mut child = self.ui.new_child(
            UiBuilder::new()
               .max_rect(self.ui.available_rect_before_wrap())
               .layout(cell_layout())
               .sizing_pass()
               .invisible(),
         );
         let inner = add(&mut child);
         let measured = child.min_rect();
         if let Some(column) = column {
            measure.widths[column] = measure.widths[column].max(measured.width());
         }
         *measure.height = measure.height.max(measured.height());
         return InnerResponse {
            response: child.response(),
            inner,
         };
      }

      let rect = match column_index(i, self.cells.len()) {
         Some(column) => self.cells[column],
         None => self.ui.available_rect_before_wrap(),
      };
      let mut child = self.ui.new_child(UiBuilder::new().max_rect(rect).layout(cell_layout()));
      let inner = add(&mut child);
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
   use egui::{Pos2, vec2};

   fn themed_ctx() -> egui::Context {
      let ctx = egui::Context::default();
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&ctx);
      ctx
   }

   fn frame(ctx: &egui::Context, screen: Vec2, body: impl FnMut(&mut Ui)) -> egui::FullOutput {
      let input = egui::RawInput {
         screen_rect: Some(Rect::from_min_size(Pos2::ZERO, screen)),
         ..Default::default()
      };
      let mut out = ctx.run_ui(input, body);
      out.textures_delta.clear();
      out
   }

   fn extra_passes(output: &egui::FullOutput) -> usize {
      output.platform_output.num_completed_passes.saturating_sub(1)
   }

   fn scope(ui: &mut Ui, body: impl FnOnce(&mut Ui)) {
      ui.scope_builder(
         UiBuilder::new().max_rect(Rect::from_min_size(
            Pos2::ZERO,
            vec2(500.0, 400.0),
         )),
         body,
      );
   }

   struct Row(&'static str, &'static str);

   const ROWS: [Row; 2] = [Row("Deployer", "Swap"), Row("Router", "Add liquidity")];

   #[test]
   fn table_header_and_body_share_column_extents() {
      let ctx = themed_ctx();
      let rects = std::cell::RefCell::new(Vec::new());
      for _ in 0..2 {
         drop(frame(&ctx, vec2(500.0, 400.0), |ui| {
            rects.borrow_mut().clear();
            scope(ui, |ui| {
               Table::new("shared", &ROWS)
                  .columns([Column::Auto, Column::Fr(1.0), Column::Fixed(60.0)])
                  .show(ui, |table| {
                     table.header(|header| {
                        for _ in 0..3 {
                           let rect = header.cell(|ui| ui.max_rect()).inner;
                           rects.borrow_mut().push(rect);
                        }
                     });
                     table.rows(|row, _item| {
                        for _ in 0..3 {
                           let rect = row.cell(|ui| ui.max_rect()).inner;
                           rects.borrow_mut().push(rect);
                        }
                     });
                  });
            });
         }));
      }
      let rects = rects.borrow();
      assert_eq!(
         rects.len(),
         9,
         "expected header + 2 body rows of 3 cells"
      );
      for column in 0..3 {
         let header = rects[column];
         for row in 0..2 {
            let cell = rects[3 + row * 3 + column];
            assert!(
               (cell.min.x - header.min.x).abs() < 0.5,
               "column {column} row {row} min.x {} != header min.x {}",
               cell.min.x,
               header.min.x
            );
            assert!(
               (cell.width() - header.width()).abs() < 0.5,
               "column {column} row {row} width {} != header width {}",
               cell.width(),
               header.width()
            );
         }
      }
   }

   #[test]
   fn table_auto_column_fits_the_widest_cell() {
      const LONG: &str = "Buy the entire liquidity pool";
      let rows = [Row("Swap", "row 0"), Row(LONG, "row 1")];
      let ctx = themed_ctx();
      let rects = std::cell::RefCell::new(Vec::new());
      let reference = std::cell::Cell::new(0.0f32);
      for _ in 0..2 {
         drop(frame(&ctx, vec2(500.0, 400.0), |ui| {
            rects.borrow_mut().clear();
            scope(ui, |ui| {
               reference.set(
                  ui.scope_builder(
                     UiBuilder::new().max_rect(Rect::from_min_size(Pos2::ZERO, vec2(500.0, 40.0))),
                     |ui| ui.add(crate::widgets::Label::new(LONG, None)).rect.width(),
                  )
                  .inner,
               );
               Table::new("auto-fit", &rows).columns([Column::Auto, Column::Fixed(60.0)]).show(
                  ui,
                  |table| {
                     table.header(|header| {
                        for _ in 0..2 {
                           let rect = header.cell(|ui| ui.max_rect()).inner;
                           rects.borrow_mut().push(rect);
                        }
                     });
                     table.rows(|row, item| {
                        let rect = row
                           .cell(|ui| {
                              ui.add(crate::widgets::Label::new(item.0, None));
                              ui.max_rect()
                           })
                           .inner;
                        rects.borrow_mut().push(rect);
                        let rect = row
                           .cell(|ui| {
                              ui.add(crate::widgets::Label::new(item.1, None));
                              ui.max_rect()
                           })
                           .inner;
                        rects.borrow_mut().push(rect);
                     });
                  },
               );
            });
         }));
      }
      let rects = rects.borrow();
      assert_eq!(rects.len(), 6);
      let widest = rects[4];
      assert!(
         (widest.width() - reference.get()).abs() < 0.5,
         "auto column width {} != reference text width {}",
         widest.width(),
         reference.get()
      );
      for fixed in [rects[3], rects[5]] {
         assert!(
            (fixed.width() - 60.0).abs() < 0.5,
            "fixed column width {} != 60",
            fixed.width()
         );
      }
   }

   #[test]
   fn table_fr_columns_fill_the_available_width() {
      let ctx = themed_ctx();
      let cells = std::cell::RefCell::new(Vec::new());
      for _ in 0..2 {
         drop(frame(&ctx, vec2(500.0, 400.0), |ui| {
            cells.borrow_mut().clear();
            scope(ui, |ui| {
               Table::new("fr", &ROWS)
                  .columns([Column::Fr(1.0), Column::Fr(3.0), Column::Fixed(80.0)])
                  .gap(10.0)
                  .show(ui, |table| {
                     table.rows(|row, _item| {
                        for _ in 0..3 {
                           let rect = row.cell(|ui| ui.max_rect()).inner;
                           cells.borrow_mut().push(rect);
                        }
                     });
                  });
            });
         }));
      }
      let cells = cells.borrow();
      assert_eq!(cells.len(), 6);
      for (column, expected) in [100.0, 300.0, 80.0].into_iter().enumerate() {
         assert!(
            (cells[column].width() - expected).abs() < 0.5,
            "column {column} width {} != {expected}",
            cells[column].width()
         );
      }
      assert!(
         (cells[2].max.x - 500.0).abs() < 0.5,
         "last cell max.x {} != 500",
         cells[2].max.x
      );
   }

   #[test]
   fn table_row_height_is_its_tallest_cell() {
      let rows = [Row("Label", "Button")];
      let ctx = themed_ctx();
      let rects = std::cell::RefCell::new(Vec::new());
      for _ in 0..2 {
         drop(frame(&ctx, vec2(500.0, 400.0), |ui| {
            rects.borrow_mut().clear();
            scope(ui, |ui| {
               Table::new("tallest", &rows).columns([Column::Auto, Column::Auto]).show(
                  ui,
                  |table| {
                     table.rows(|row, _item| {
                        row.cell(|ui| {
                           ui.add(crate::widgets::Label::new("Height", None));
                        });
                        row.cell(|ui| {
                           ui.add(crate::widgets::Button::new("x").min_size(vec2(80.0, 44.0)));
                        });
                        rects.borrow_mut().push(row.rect());
                     });
                  },
               );
            });
         }));
      }
      let rects = rects.borrow();
      assert_eq!(rects.len(), 1);
      assert!(
         (rects[0].height() - 44.0).abs() < 0.5,
         "row height {} != 44",
         rects[0].height()
      );
   }

   #[test]
   fn table_second_frame_requests_no_discard() {
      let ctx = themed_ctx();
      let mut outputs = Vec::new();
      for _ in 0..3 {
         outputs.push(frame(&ctx, vec2(500.0, 400.0), |ui| {
            scope(ui, |ui| {
               Table::new("steady", &ROWS)
                  .columns([Column::Auto, Column::Fr(1.0), Column::Fixed(60.0)])
                  .show(ui, |table| {
                     table.header(|header| {
                        for _ in 0..3 {
                           header.cell(|ui| {
                              ui.add(crate::widgets::Label::new("Header", None));
                           });
                        }
                     });
                     table.rows(|row, _item| {
                        for _ in 0..3 {
                           row.cell(|ui| {
                              ui.add(crate::widgets::Label::new("Cell", None));
                           });
                        }
                     });
                  });
            });
         }));
      }
      let passes: Vec<_> = outputs.iter().map(extra_passes).collect();
      assert!(
         passes[0] >= 1,
         "first frame did not measure: {passes:?}"
      );
      assert_eq!(
         passes[1], 0,
         "second frame ran a second pass: {passes:?}"
      );
      assert_eq!(
         passes[2], 0,
         "third frame ran a second pass: {passes:?}"
      );
   }
}
