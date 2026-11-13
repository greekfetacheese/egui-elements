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
//!
//! # Uniform rows
//!
//! Every body row is placed at one height: the tallest cell measured anywhere in the list, so a row
//! whose tallest cell is a `Button` cannot run taller than a text-only row. [`Table::row_height`]
//! pins that height instead of measuring it, and [`Table::row_divider`] draws a rule in the middle of
//! the gap under each row.
//!
//! [`Table::row_frame`] draws every row inside a frame instead — one card per row. The frame's margins
//! join the pitch and inset the cells, so the card never squeezes a row and the header still shares the
//! cells' columns; a cell is still not clipped, so a card's own chrome overhang is the host's to
//! budget.
//!
//! # Sticky header
//!
//! [`Table::scroll`] puts the rows in a vertical [`egui::ScrollArea`] and leaves the header outside
//! it, so the header stays put while the rows scroll. It is still one table: the measure pass covers
//! the whole list, so a [`Column::Auto`] column fits its widest body cell instead of settling on the
//! header's width, and the header and the rows share one set of column extents.
//!
//! # The viewport
//!
//! Only the rows the clip rect shows are laid out, so a table inside a `ScrollArea` — the host's or
//! the table's own — costs its viewport, not its list. The band still spans every row — the space an
//! off-screen row would take is reserved from the cached row height — so the scroll extent and the
//! column extents (measured over the whole list) stay correct. A row that is scrolled out of view
//! costs one height lookup and nothing else; the host needs no API for it, because only the table
//! knows the row heights.

use std::hash::{DefaultHasher, Hash, Hasher};

use egui::{
   Align, Color32, Id, InnerResponse, Layout, Margin, Rangef, Rect, Response, ScrollArea, Sense,
   Ui, UiBuilder, Vec2, pos2, vec2,
};

use super::solver::{Column, resolve_columns};
use super::state::TableState;
use crate::widgets::Frame;

/// A header-plus-rows table over a slice of data.
///
/// The table measures once, then places on every frame: the header and every row share the same column
/// extents, `Auto` columns fit their widest measured cell, and `Fr` columns share the leftover width.
/// Every row is placed at one height — the tallest measured row, or [`Table::row_height`] — and
/// [`Table::scroll`] keeps the header above a rows-only scroll area.
/// See the [module docs](self) for the measure-pass contract and [`Table::key`].
#[must_use = "You should call .show()"]
pub struct Table<'a, T> {
   id: Id,
   rows: &'a [T],
   columns: Vec<Column>,
   gap: Option<f32>,
   row_gap: Option<f32>,
   row_height: Option<f32>,
   row_divider: bool,
   row_frame: Option<Frame>,
   scroll: Option<TableScroll>,
   header_rule: bool,
   key: u64,
}

/// Whether the table is placing its cached sizes or measuring for the first time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
   Placing,
   Measuring,
}

/// The scroll area a [`Table`] puts its rows in, with the header left outside it.
///
/// Pass to [`Table::scroll`]. The default fills the space the table has left and shrinks in height to
/// its rows, so a short list does not leave a blank viewport under the last row.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TableScroll {
   max_height: Option<f32>,
   auto_shrink: [bool; 2],
   content_margin: Option<Margin>,
}

impl TableScroll {
   pub fn new() -> Self {
      Self {
         max_height: None,
         auto_shrink: [false, true],
         content_margin: None,
      }
   }

   /// Cap the viewport's height. Default: the space the table has left when the rows are added.
   pub fn max_height(mut self, max_height: f32) -> Self {
      self.max_height = Some(max_height);
      self
   }

   /// Shrink the viewport to its content on an axis. Default `[false, true]`: fill the width, shrink
   /// the height.
   pub fn auto_shrink(mut self, auto_shrink: [bool; 2]) -> Self {
      self.auto_shrink = auto_shrink;
      self
   }

   /// Pad the rows inside the viewport. The left/right insets also apply to the header band, so the
   /// two bands keep one set of column extents; the top/bottom insets sit between the rows and the
   /// viewport's edges. Default: the installed [`egui::ScrollStyle`]`::content_margin`.
   pub fn content_margin(mut self, content_margin: impl Into<Margin>) -> Self {
      self.content_margin = Some(content_margin.into());
      self
   }
}

impl Default for TableScroll {
   fn default() -> Self {
      Self::new()
   }
}

/// The layout every cell's child `Ui` starts from: left-to-right, vertically centred, not clipped.
fn cell_layout() -> Layout {
   Layout::left_to_right(Align::Center)
}

/// The height of the whole table band: the header, the pad under it, and every row plus the gaps
/// between them. `row_pitch` is one row's full height, its cells included.
fn band_height(
   has_header: bool,
   header_height: f32,
   header_pad: f32,
   rows: usize,
   row_pitch: f32,
   row_gap: f32,
) -> f32 {
   let mut height = header_height + header_pad;
   if has_header && rows > 0 {
      height += row_gap;
   }
   height += row_pitch * rows as f32;
   if rows > 1 {
      height += row_gap * (rows - 1) as f32;
   }
   height
}

/// The space one row frame adds around that row's cells.
#[derive(Clone, Copy, Debug, Default)]
struct FrameMargin {
   /// Left + right: taken out of the width the columns resolve against, so a card fits the table.
   x: f32,
   /// Top + bottom: added to every row's pitch, so a card pads its cells instead of squeezing them.
   y: f32,
   /// How far a row's cells start from its band's left edge, which is the header's left edge too.
   left: f32,
}

impl FrameMargin {
   /// All zeros for a table with no row frame.
   fn of(row_frame: Option<Frame>) -> Self {
      let Some(frame) = row_frame else {
         return Self::default();
      };
      let margin = frame.total_margin();
      Self {
         x: margin.left + margin.right,
         y: margin.top + margin.bottom,
         left: margin.left,
      }
   }
}

/// Fill `cells` with one rect per column, in column order, starting at `content`'s left edge.
fn place_cells(cells: &mut Vec<Rect>, widths: &[f32], content: Rect, height: f32, gap: f32) {
   cells.clear();
   let mut x = content.min.x;
   for width in widths {
      cells.push(Rect::from_min_size(
         pos2(x, content.min.y),
         vec2(*width, height),
      ));
      x += *width + gap;
   }
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
         row_height: None,
         row_divider: false,
         row_frame: None,
         scroll: None,
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

   /// Place every body row at this height. Default: the tallest row measured anywhere in the list, so
   /// all rows share one height.
   ///
   /// Set this to a pixel value when the host wants one pitch regardless of its content, and note that
   /// a cell taller than it is clipped by the column, not by the row.
   pub fn row_height(mut self, row_height: f32) -> Self {
      self.row_height = Some(row_height.max(0.0));
      self
   }

   /// Draw a 1px separator in the middle of the gap under every body row, in the header rule's stroke,
   /// so the space above and below the rule is equal. The last row has no gap under it and gets no
   /// rule. Default: `false`.
   pub fn row_divider(mut self, row_divider: bool) -> Self {
      self.row_divider = row_divider;
      self
   }

   /// Draw every body row inside `frame` — one card per row — instead of a rule or a zebra fill.
   ///
   /// The frame's margins join the row pitch — a row occupies its cells' height plus the margins'
   /// top and bottom — and inset its cells by their left and right, so the columns fit inside the
   /// card and the header keeps sharing their left edge. The card is as wide as the resolved
   /// columns, so a table with no `Fr` column keeps its intrinsic width.
   ///
   /// Takes the crate's own [`Frame`] (hover / selected / gradient fills) or an
   /// [`egui::Frame`](egui::Frame) through `Into`, so a themed card needs no new type.
   ///
   /// Independent of [`Table::row_divider`], which a framed list normally turns off. The frame is
   /// host chrome, not a selection state — [`TableRow::fill`] still paints behind the cells.
   pub fn row_frame(mut self, frame: impl Into<Frame>) -> Self {
      self.row_frame = Some(frame.into());
      self
   }

   /// Put the rows in a vertical [`egui::ScrollArea`], leaving the header above it so it stays put
   /// while the rows scroll. See [`TableScroll`].
   ///
   /// The header and the rows are still one table: the measure pass covers the whole list, so an
   /// `Auto` column fits its widest body cell. Do not also wrap the table in a host `ScrollArea` —
   /// that one would scroll the header too.
   pub fn scroll(mut self, scroll: TableScroll) -> Self {
      self.scroll = Some(scroll);
      self
   }

   /// Draw a 1px separator at the bottom of the header band. Default: `false`.
   ///
   /// The rule reserves one [`egui::Spacing::item_spacing`]`.y` step under the header text and draws
   /// the line there, so it never touches the text's descenders.
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
   ///
   /// Rows outside `ui.clip_rect()` are skipped, which is what makes a table inside a `ScrollArea`
   /// cost its viewport rather than its list — see the [module docs](self).
   pub fn show<R>(
      self,
      ui: &mut Ui,
      add: impl FnOnce(&mut TableUi<'_, '_, T>) -> R,
   ) -> InnerResponse<R> {
      let id = ui.make_persistent_id(self.id);
      let prev = TableState::load(ui.ctx(), id);
      // The header band sits outside the scroll area, so the columns resolve against the width the
      // rows will actually get: the viewport minus a solid scroll bar and the content margin. A
      // floating bar takes nothing by default, which is egui's default style.
      let content_margin = self
         .scroll
         .map(|scroll| scroll.content_margin.unwrap_or(ui.spacing().scroll.content_margin))
         .unwrap_or(Margin::ZERO);
      let scroll_bar = ui.spacing().scroll.allocated_width();
      let avail = if self.scroll.is_some() {
         vec2(
            (ui.available_width()
               - scroll_bar
               - content_margin.left as f32
               - content_margin.right as f32)
               .max(0.0),
            ui.available_height(),
         )
      } else {
         ui.available_size()
      };
      // A row frame insets its cells, so the columns resolve against the card's content width. The
      // margins ride in `avail`, which is also the cache's space key: changing the frame re-measures.
      let row_margin = FrameMargin::of(self.row_frame);
      let avail = vec2((avail.x - row_margin.x).max(0.0), avail.y);

      let gap = self.gap.unwrap_or(ui.spacing().item_spacing.x);
      let row_gap = self.row_gap.unwrap_or(ui.spacing().item_spacing.y);
      let header_pad = if self.header_rule {
         ui.spacing().item_spacing.y
      } else {
         0.0
      };

      let placing = prev.as_ref().is_some_and(|p| {
         p.same_space(avail)
            && p.columns == self.columns
            && p.row_count == self.rows.len()
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
      let row_height =
         self.row_height.unwrap_or_else(|| prev.as_ref().map_or(0.0, |p| p.row_height));

      let mut table = TableUi {
         ui,
         id,
         data: self.rows,
         columns: self.columns,
         widths,
         scratch: Vec::new(),
         mode,
         prev,
         gap,
         row_gap,
         row_height,
         row_divider: self.row_divider,
         row_frame: self.row_frame,
         content_margin,
         scroll: self.scroll,
         header_rule: self.header_rule,
         header_pad,
         key: self.key,
         avail,
         heights: Vec::new(),
         header_height: 0.0,
         has_header: false,
         band: None,
         rows_rect: None,
      };
      let inner = add(&mut table);
      let response = table.teardown(id, avail);
      InnerResponse { response, inner }
   }
}

/// The per-call state of a [`Table`]; the header and the rows are added through it.
pub struct TableUi<'a, 'ui, T> {
   ui: &'ui mut Ui,
   id: Id,
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
   /// The height every body row's cells are placed at: [`Table::row_height`] or the tallest
   /// measured row.
   row_height: f32,
   row_divider: bool,
   /// The card drawn around every body row, when the host asked for one.
   row_frame: Option<Frame>,
   /// The padding inside the table's own scroll viewport; the horizontal part is applied to the
   /// header band too, so the columns stay aligned.
   content_margin: Margin,
   scroll: Option<TableScroll>,
   header_rule: bool,
   /// The gap the header rule reserves under the header text: one `item_spacing.y` step when
   /// `header_rule` is on, `0.0` otherwise.
   header_pad: f32,
   key: u64,
   avail: Vec2,
   heights: Vec<f32>,
   header_height: f32,
   has_header: bool,
   band: Option<(Rect, Response)>,
   /// The rows' viewport when the table owns the scroll area; the table's response spans the header
   /// band and this rect.
   rows_rect: Option<Rect>,
}

impl<T> TableUi<'_, '_, T> {
   /// The number of columns this table was built with.
   pub fn columns(&self) -> usize {
      self.columns.len()
   }

   /// The height one body row occupies: its cells plus a row frame's vertical margins.
   fn row_pitch(&self) -> f32 {
      self.row_height + FrameMargin::of(self.row_frame).y
   }

   /// Resolve the cached widths and header height into this pass's extents. Returns the table's
   /// width.
   fn resolve(&mut self) -> f32 {
      let prev = self.prev.as_ref().expect("placing requires a cache");
      let out = resolve_columns(
         &self.columns,
         &prev.widths,
         self.avail.x,
         self.gap,
      );
      self.widths = out.widths;
      self.header_height = prev.header_height;
      out.total
   }

   /// Resolve the cached extents and allocate the header band — or the whole table band when the
   /// table is not scrolling — on the first `header()` or `rows()` call.
   fn ensure_band(&mut self) {
      if self.band.is_some() {
         return;
      }
      let total = self.resolve();
      let margin = FrameMargin::of(self.row_frame);
      let height = if self.scroll.is_some() {
         self.header_height + self.header_pad
      } else {
         band_height(
            self.has_header,
            self.header_height,
            self.header_pad,
            self.data.len(),
            self.row_pitch(),
            self.row_gap,
         )
      };
      let width = if self.scroll.is_some() {
         total + self.content_margin.left as f32 + self.content_margin.right as f32 + margin.x
      } else {
         total + margin.x
      };
      let (rect, response) = self.ui.allocate_exact_size(vec2(width, height), Sense::hover());
      self.band = Some((rect, response));
   }

   /// Add the header line. The `i`-th [`TableHeader::cell`] call is column `i`. Add it before
   /// [`TableUi::rows`]: the table band is allocated on the first of those two calls.
   pub fn header(&mut self, add: impl FnOnce(&mut TableHeader<'_>)) {
      self.has_header = true;
      match self.mode {
         Mode::Placing => {
            self.ensure_band();
            let band = self.band.as_ref().expect("the band is allocated above").0;
            let header_height = self.header_height;
            let margin = FrameMargin::of(self.row_frame);
            let mut cells = std::mem::take(&mut self.scratch);
            cells.clear();
            let mut x = band.min.x + self.content_margin.left as f32 + margin.left;
            for width in &self.widths {
               cells.push(Rect::from_min_size(
                  pos2(x, band.min.y),
                  vec2(*width, header_height),
               ));
               x += *width + self.gap;
            }
            if self.header_rule {
               let stroke = self.ui.visuals().widgets.noninteractive.bg_stroke;
               let inset = &self.content_margin;
               let rule = Rangef::new(
                  band.min.x + inset.left as f32,
                  band.max.x - inset.right as f32,
               );
               self.ui.painter().hline(
                  rule,
                  band.min.y + header_height + self.header_pad,
                  stroke,
               );
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
         Mode::Placing => match self.scroll {
            Some(scroll) => self.rows_scrolling(scroll, &mut add),
            None => {
               self.ensure_band();
               let band = self.band.as_ref().expect("the band is allocated above").0;
               let mut top = band.min.y + self.header_height + self.header_pad;
               if self.has_header {
                  top += self.row_gap;
               }
               let mut placer = RowPlacer {
                  data: self.data,
                  widths: &self.widths,
                  scratch: &mut self.scratch,
                  row_height: self.row_height,
                  row_gap: self.row_gap,
                  gap: self.gap,
                  row_divider: self.row_divider,
                  row_frame: self.row_frame,
               };
               placer.place(
                  &mut *self.ui,
                  top,
                  band.min.x,
                  band.width(),
                  &mut add,
               );
            }
         },
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

   /// Place the rows in the table's own scroll area, under the header `header()` already placed.
   fn rows_scrolling(&mut self, scroll: TableScroll, add: &mut impl FnMut(&mut TableRow<'_>, &T)) {
      self.resolve();
      let content_height = self.content_height();

      // The rows start exactly `row_gap` under the header band, or at the table's top when there is
      // no header. Allocating the scroll area at the cursor would insert `item_spacing.y` instead, so
      // the viewport is placed into a child ui at the rect the table chose.
      let available = self.ui.available_rect_before_wrap();
      let band = self.band.as_ref().map(|(band, _)| *band);
      let top = band.map_or(available.min.y, |band| band.max.y + self.row_gap);
      let viewport = scroll.max_height.unwrap_or_else(|| available.bottom() - top).max(0.0);
      let rect = Rect::from_min_size(
         pos2(available.min.x, top),
         vec2(available.width(), viewport),
      );
      let id = self.id;

      let rows_rect = {
         let TableUi {
            ui,
            data,
            widths,
            scratch,
            row_height,
            row_gap,
            gap,
            row_divider,
            row_frame,
            content_margin,
            ..
         } = self;
         let mut placer = RowPlacer {
            data: *data,
            widths: &widths[..],
            scratch,
            row_height: *row_height,
            row_gap: *row_gap,
            gap: *gap,
            row_divider: *row_divider,
            row_frame: *row_frame,
         };
         ui.scope_builder(
            UiBuilder::new().max_rect(rect).layout(Layout::top_down(Align::Min)),
            |ui| {
               ScrollArea::vertical()
                  .id_salt(id.with("rows"))
                  .auto_shrink(scroll.auto_shrink)
                  .max_height(viewport)
                  .content_margin(*content_margin)
                  .show(ui, |ui| {
                     let (band, _) = ui.allocate_exact_size(
                        vec2(ui.available_width(), content_height),
                        Sense::hover(),
                     );
                     placer.place(ui, band.min.y, band.min.x, band.width(), add);
                  })
                  .inner_rect
            },
         )
         .inner
      };
      self.rows_rect = Some(rows_rect);
   }

   /// The height of the rows alone, without the header: the scroll area's content extent.
   fn content_height(&self) -> f32 {
      let rows = self.data.len();
      let mut height = self.row_pitch() * rows as f32;
      if rows > 1 {
         height += self.row_gap * (rows - 1) as f32;
      }
      height
   }

   /// Resolve the column extents, store the cache and return the table's band response.
   fn teardown(mut self, id: Id, avail: Vec2) -> Response {
      let out = resolve_columns(&self.columns, &self.widths, avail.x, self.gap);
      let response = match (self.band.take(), self.rows_rect) {
         (Some((band, _)), Some(rows)) => {
            self.ui.interact(band.union(rows), id.with("band"), Sense::hover())
         }
         (Some((_, response)), None) => response,
         (None, Some(rows)) => self.ui.interact(rows, id.with("band"), Sense::hover()),
         (None, None) => {
            let height = band_height(
               self.has_header,
               self.header_height,
               self.header_pad,
               0,
               self.row_pitch(),
               self.row_gap,
            );
            self.ui.allocate_exact_size(vec2(out.total, height), Sense::hover()).1
         }
      };

      if self.mode == Mode::Measuring {
         let row_height = self.heights.iter().copied().fold(0.0, f32::max);
         let state = TableState {
            columns: self.columns,
            widths: self.widths,
            row_count: self.data.len(),
            row_height,
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

/// The cached extents one row needs, split out of [`TableUi`] so a scroll area can own the child
/// [`Ui`] while the rows still read the table's cache.
struct RowPlacer<'a, 'x, T> {
   data: &'a [T],
   widths: &'x [f32],
   scratch: &'x mut Vec<Rect>,
   row_height: f32,
   row_gap: f32,
   gap: f32,
   row_divider: bool,
   row_frame: Option<Frame>,
}

impl<T> RowPlacer<'_, '_, T> {
   /// Stamp every row into a band that starts at `top` / `left`.
   fn place(
      &mut self,
      ui: &mut Ui,
      top: f32,
      left: f32,
      width: f32,
      add: &mut impl FnMut(&mut TableRow<'_>, &T),
   ) {
      let data = self.data;
      let margin = FrameMargin::of(self.row_frame);
      let pitch = self.row_height + margin.y;
      // The cells' own extent, gaps included: a card hugs it, where the band a scroll area hands over
      // is the viewport width and would stretch the card past the columns.
      let content_width =
         self.widths.iter().sum::<f32>() + self.gap * self.widths.len().saturating_sub(1) as f32;
      let band_width = if self.row_frame.is_some() {
         content_width + margin.x
      } else {
         width
      };
      let clip = ui.clip_rect();
      let mut y = top;
      for (i, item) in data.iter().enumerate() {
         let row_band = Rect::from_min_size(pos2(left, y), vec2(band_width, pitch));
         y += pitch + self.row_gap;

         // The band already reserved this row's space, so a row the viewport does not show is
         // skipped rather than placed: no cell is built and no closure runs for it.
         if !clip.intersects(row_band) {
            continue;
         }

         let mut cells = std::mem::take(self.scratch);

         match self.row_frame {
            Some(frame) => {
               // The frame owns the row's Ui: it shrinks the band by its own margins and paints
               // behind what it holds, which is exactly the geometry a card needs.
               let mut row_ui = ui.new_child(
                  UiBuilder::new()
                     .max_rect(row_band)
                     .layout(Layout::top_down(Align::Min))
                     .id_salt(("row_frame", i)),
               );
               let widths = self.widths;
               let gap = self.gap;
               let row_height = self.row_height;

               frame.show(&mut row_ui, |ui| {
                  // A frame sizes itself from this Ui's `min_rect`, and a cell's child Ui never grows
                  // it, so the cells' extent is reserved here and the cells are placed inside it.
                  let (content, _) =
                     ui.allocate_exact_size(vec2(content_width, row_height), Sense::hover());
                  place_cells(&mut cells, widths, content, row_height, gap);

                  let mut row = TableRow {
                     ui,
                     cells: &cells,
                     measure: None,
                     next: 0,
                     rect: row_band,
                     index: i,
                  };
                  add(&mut row, item);
               });
            }
            None => {
               place_cells(
                  &mut cells,
                  self.widths,
                  row_band,
                  self.row_height,
                  self.gap,
               );

               let mut row = TableRow {
                  ui: &mut *ui,
                  cells: &cells,
                  measure: None,
                  next: 0,
                  rect: row_band,
                  index: i,
               };
               add(&mut row, item);
            }
         }

         // The rule sits in the middle of the gap under the row, so the space above and below it is
         // equal. The last row has no gap under it, so it gets no rule.
         if self.row_divider && i + 1 < data.len() {
            let stroke = ui.visuals().widgets.noninteractive.bg_stroke;
            let y = row_band.max.y + self.row_gap * 0.5;
            ui.painter().hline(row_band.x_range(), y, stroke);
         }
         *self.scratch = cells;
      }
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

   /// The row's band: the table's width and the row's height, which with [`Table::row_frame`]
   /// includes the frame's margins.
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
   use egui::{Pos2, Shape, epaint::RectShape, vec2};

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
   fn table_columns_and_rows_follow_their_specs() {
      // Case: table_auto_column_fits_the_widest_cell
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

      // Case: table_fr_columns_fill_the_available_width
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

      // Case: table_row_height_is_its_tallest_cell
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

      // Case: table_header_rule_reserves_space_under_the_header
      let ctx = themed_ctx();
      let rows = [Row("Deployer", "Swap")];
      let rects = std::cell::RefCell::new(Vec::new());
      let spacing_y = std::cell::Cell::new(0.0f32);
      for _ in 0..2 {
         drop(frame(&ctx, vec2(500.0, 400.0), |ui| {
            rects.borrow_mut().clear();
            scope(ui, |ui| {
               spacing_y.set(ui.spacing().item_spacing.y);
               for (id, rule) in [("rule_off", false), ("rule_on", true)] {
                  Table::new(id, &rows).columns([Column::Auto]).header_rule(rule).show(
                     ui,
                     |table| {
                        table.header(|header| {
                           let rect = header.cell(|ui| ui.max_rect()).inner;
                           rects.borrow_mut().push(rect);
                        });
                        table.rows(|row, _item| {
                           row.cell(|ui| {
                              ui.add(crate::widgets::Label::new("Swap", None));
                           });
                           rects.borrow_mut().push(row.rect());
                        });
                     },
                  );
               }
            });
         }));
      }
      let rects = rects.borrow();
      assert_eq!(rects.len(), 4, "expected two header/row pairs");
      let spacing = spacing_y.get();
      let rule_off_gap = rects[1].top() - rects[0].bottom();
      assert!(
         (rule_off_gap - spacing).abs() < 0.5,
         "rule-less gap {rule_off_gap} != row_gap {spacing}"
      );
      let rule_on_gap = rects[3].top() - rects[2].bottom();
      assert!(
         (rule_on_gap - (spacing + spacing)).abs() < 0.5,
         "ruled gap {rule_on_gap} != row_gap + item_spacing.y {}",
         spacing + spacing
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

   /// A table in a `ScrollArea` lays out its viewport, not its list.
   ///
   /// The band still reserves every row's height — that is what a `ScrollArea` reads as its content
   /// extent — while the rows the clip rect does not show are skipped outright.
   #[test]
   fn table_places_only_the_rows_the_viewport_shows() {
      let rows: Vec<Row> = (0..6).map(|_| Row("row", "cell")).collect();
      let ctx = themed_ctx();
      let drawn = std::cell::Cell::new(0usize);
      let band = std::cell::Cell::new(0.0f32);

      let mut body = |ui: &mut Ui| {
         let mut child = ui.new_child(UiBuilder::new().max_rect(Rect::from_min_size(
            Pos2::ZERO,
            vec2(500.0, 400.0),
         )));
         // The viewport a `ScrollArea` imposes: the first row and the gap under it, and nothing of
         // the second (rows are 40 pt tall, 10 pt apart).
         child.set_clip_rect(Rect::from_min_size(Pos2::ZERO, vec2(500.0, 45.0)));

         let response = Table::new("viewport", &rows)
            .columns([Column::Auto, Column::Fr(1.0)])
            .row_gap(10.0)
            .show(&mut child, |table| {
               table.rows(|row, _item| {
                  drawn.set(drawn.get() + 1);
                  row.cell(|ui| {
                     ui.allocate_space(vec2(10.0, 40.0));
                  });
                  row.cell(|ui| {
                     ui.allocate_space(vec2(10.0, 40.0));
                  });
               });
            })
            .response;
         band.set(response.rect.height());
      };

      drop(frame(&ctx, vec2(500.0, 400.0), &mut body));
      drawn.set(0);
      let out = frame(&ctx, vec2(500.0, 400.0), &mut body);

      assert_eq!(
         extra_passes(&out),
         0,
         "the second frame should place in one pass"
      );
      assert_eq!(
         drawn.get(),
         1,
         "only the row the viewport shows is laid out"
      );
      assert!(
         (band.get() - 290.0).abs() < 0.5,
         "band height {} != 6 rows + 5 gaps",
         band.get()
      );
   }

   /// Every cell of a row sits in that row's band.
   ///
   /// `TableRow::fill` paints `rect()`, so if `cell()` lands anywhere else the host's chrome and its
   /// content drift apart — which is exactly how the viewport change first broke: the row cursor
   /// advanced before the cells were built, every row's content moved down one slot, and the band
   /// tint above it looked like an empty row.
   #[test]
   fn table_row_cells_sit_in_their_band() {
      let rows: Vec<Row> = (0..3).map(|_| Row("row", "cell")).collect();
      let ctx = themed_ctx();
      let seen = std::cell::RefCell::new(Vec::new());

      for _ in 0..2 {
         drop(frame(&ctx, vec2(500.0, 400.0), |ui| {
            seen.borrow_mut().clear();
            scope(ui, |ui| {
               Table::new("bands", &rows)
                  .columns([Column::Auto, Column::Fr(1.0)])
                  .row_gap(10.0)
                  .show(ui, |table| {
                     table.rows(|row, _item| {
                        let band = row.rect();
                        let cell = row
                           .cell(|ui| {
                              ui.allocate_space(vec2(10.0, 40.0));
                              ui.max_rect()
                           })
                           .inner;
                        seen.borrow_mut().push((band, cell));
                     });
                  });
            });
         }));
      }

      let seen = seen.borrow();
      assert_eq!(seen.len(), 3);
      for (band, cell) in seen.iter() {
         assert!(
            (cell.min.y - band.min.y).abs() < 0.5,
            "cell top {} != band top {}",
            cell.min.y,
            band.min.y
         );
         assert!(
            (cell.height() - band.height()).abs() < 0.5,
            "cell height {} != band height {}",
            cell.height(),
            band.height()
         );
      }
      for pair in seen.windows(2) {
         let (above, _) = pair[0];
         let (below, _) = pair[1];
         assert!(
            (below.min.y - (above.max.y + 10.0)).abs() < 0.5,
            "band {} does not follow band {} at the row gap",
            below.min.y,
            above.max.y
         );
      }
   }

   /// One row's tall cell sets the pitch for every row: the table no longer alternates heights.
   #[test]
   fn table_every_row_takes_the_tallest_cell_height() {
      let rows = [Row("text", "a"), Row("text", "b"), Row("text", "c")];
      let ctx = themed_ctx();
      let rects = std::cell::RefCell::new(Vec::new());
      for _ in 0..2 {
         drop(frame(&ctx, vec2(500.0, 400.0), |ui| {
            rects.borrow_mut().clear();
            scope(ui, |ui| {
               Table::new("uniform", &rows)
                  .columns([Column::Auto, Column::Fixed(80.0)])
                  .row_gap(10.0)
                  .show(ui, |table| {
                     table.rows(|row, item| {
                        rects.borrow_mut().push(row.rect());
                        row.cell(|ui| {
                           ui.add(crate::widgets::Label::new("Height", None));
                        });
                        row.cell(|ui| {
                           if item.1 == "b" {
                              ui.add(crate::widgets::Button::new("x").min_size(vec2(80.0, 44.0)));
                           }
                        });
                     });
                  });
            });
         }));
      }
      let rects = rects.borrow();
      assert_eq!(rects.len(), 3, "expected three placed rows");
      for rect in rects.iter() {
         assert!(
            (rect.height() - 44.0).abs() < 0.5,
            "row height {} != the tallest row's 44",
            rect.height()
         );
      }
      for pair in rects.windows(2) {
         assert!(
            (pair[1].min.y - (pair[0].max.y + 10.0)).abs() < 0.5,
            "the pitch broke between rows at {} and {}",
            pair[0].min.y,
            pair[1].min.y
         );
      }
   }

   /// `row_height` wins over the measured tallest cell, and the band follows the pinned pitch.
   #[test]
   fn table_row_height_pins_every_row_to_one_pitch() {
      let rows = [Row("a", "1"), Row("b", "2")];
      let ctx = themed_ctx();
      let rects = std::cell::RefCell::new(Vec::new());
      let table_rect = std::cell::Cell::new(Rect::ZERO);
      for _ in 0..2 {
         drop(frame(&ctx, vec2(500.0, 400.0), |ui| {
            rects.borrow_mut().clear();
            scope(ui, |ui| {
               let response = Table::new("pitch", &rows)
                  .columns([Column::Auto])
                  .row_gap(10.0)
                  .row_height(30.0)
                  .show(ui, |table| {
                     table.rows(|row, _item| {
                        rects.borrow_mut().push(row.rect());
                        row.cell(|ui| {
                           // Taller than the pinned height: the row must still be 30.
                           ui.add(crate::widgets::Button::new("x").min_size(vec2(80.0, 44.0)));
                        });
                     });
                  })
                  .response;
               table_rect.set(response.rect);
            });
         }));
      }
      let rects = rects.borrow();
      assert_eq!(rects.len(), 2);
      for rect in rects.iter() {
         assert!(
            (rect.height() - 30.0).abs() < 0.5,
            "row height {} != the pinned 30",
            rect.height()
         );
      }
      assert!(
         (table_rect.get().height() - 70.0).abs() < 0.5,
         "band height {} != 2 rows + one gap",
         table_rect.get().height()
      );
   }

   /// The header is placed above the table's own scroll area, so it cannot scroll away, and the
   /// rows still virtualise to the viewport.
   #[test]
   fn table_scroll_keeps_the_header_above_the_rows_viewport() {
      let rows: Vec<Row> = (0..40).map(|_| Row("row", "cell")).collect();
      let ctx = themed_ctx();
      let header = std::cell::Cell::new(Rect::ZERO);
      let first_row = std::cell::Cell::new(Rect::ZERO);
      let drawn = std::cell::Cell::new(0usize);
      let table_rect = std::cell::Cell::new(Rect::ZERO);

      let mut outputs = Vec::new();
      for _ in 0..3 {
         outputs.push(frame(&ctx, vec2(500.0, 400.0), |ui| {
            drawn.set(0);
            scope(ui, |ui| {
               let response = Table::new("sticky", &rows)
                  .columns([Column::Auto, Column::Fr(1.0)])
                  .row_gap(10.0)
                  .row_height(30.0)
                  .scroll(TableScroll::new().max_height(120.0))
                  .show(ui, |table| {
                     table.header(|header_ui| {
                        header.set(
                           header_ui
                              .cell(|ui| {
                                 ui.add(crate::widgets::Label::new("Token", None));
                                 ui.max_rect()
                              })
                              .inner,
                        );
                     });
                     table.rows(|row, _item| {
                        if drawn.get() == 0 {
                           first_row.set(row.rect());
                        }
                        drawn.set(drawn.get() + 1);
                        row.cell(|ui| {
                           ui.allocate_space(vec2(10.0, 30.0));
                        });
                     });
                  })
                  .response;
               table_rect.set(response.rect);
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

      let header = header.get();
      let table_rect = table_rect.get();
      assert!(
         (header.min.y - table_rect.min.y).abs() < 0.5,
         "header top {} is not the table's top {}",
         header.min.y,
         table_rect.min.y
      );
      assert!(
         (first_row.get().min.y - (header.max.y + 10.0)).abs() < 1.0,
         "first row top {} is not one row_gap under the header {}",
         first_row.get().min.y,
         header.max.y
      );
      assert!(
         (table_rect.height() - (header.height() + 10.0 + 120.0)).abs() < 1.0,
         "table height {} != header {} + gap + viewport 120",
         table_rect.height(),
         header.height()
      );
      assert!(
         (1..=6).contains(&drawn.get()),
         "the viewport laid out {} rows instead of its 4-5",
         drawn.get()
      );
   }

   /// `row_divider` paints one hairline per row, in the middle of the gap under it, so the space
   /// above and below the rule is equal.
   #[test]
   fn table_row_divider_sits_in_the_middle_of_the_gap() {
      let rows = [Row("a", "1"), Row("b", "2")];
      let ctx = themed_ctx();
      let bands = std::cell::RefCell::new(Vec::new());
      let mut output = None;
      for _ in 0..2 {
         output = Some(frame(&ctx, vec2(500.0, 400.0), |ui| {
            bands.borrow_mut().clear();
            scope(ui, |ui| {
               Table::new("divider", &rows)
                  .columns([Column::Fixed(120.0)])
                  .row_gap(10.0)
                  .row_divider(true)
                  .show(ui, |table| {
                     table.rows(|row, _item| {
                        bands.borrow_mut().push(row.rect());
                        row.cell(|ui| {
                           ui.allocate_space(vec2(10.0, 20.0));
                        });
                     });
                  });
            });
         }));
      }
      let bands = bands.borrow();
      let output = output.expect("two frames ran");
      let lines: Vec<[Pos2; 2]> = output
         .shapes
         .iter()
         .filter_map(|clipped| match &clipped.shape {
            egui::Shape::LineSegment { points, .. } => Some(*points),
            _ => None,
         })
         .collect();
      assert_eq!(
         lines.len(),
         bands.len() - 1,
         "expected a rule between the rows, and none after the last"
      );
      for (line, band) in lines.iter().zip(bands.iter()) {
         let rule_y = band.max.y + 5.0;
         assert!(
            (line[0].y - rule_y).abs() < 0.5 && (line[1].y - rule_y).abs() < 0.5,
            "rule {line:?} is not in the middle of the gap under row {}",
            band.max.y
         );
         assert!(
            (line[0].x - band.min.x).abs() < 0.5 && (line[1].x - band.max.x).abs() < 0.5,
            "rule {line:?} does not span the band {band:?}"
         );
      }
   }

   /// A scrolling table still resolves one column set: an `Auto` column fits its widest body cell,
   /// and the header above the scroll area lands on the same extents.
   #[test]
   fn table_scroll_header_and_rows_share_auto_column_extents() {
      const LONG: &str = "Buy the entire liquidity pool";
      let rows = [Row(LONG, "row")];
      let ctx = themed_ctx();
      let rects = std::cell::RefCell::new(Vec::new());
      for _ in 0..2 {
         drop(frame(&ctx, vec2(500.0, 400.0), |ui| {
            rects.borrow_mut().clear();
            scope(ui, |ui| {
               Table::new("scroll_auto", &rows)
                  .columns([Column::Auto, Column::Fixed(60.0)])
                  .scroll(TableScroll::new().max_height(80.0))
                  .show(ui, |table| {
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
                        let rect = row.cell(|ui| ui.max_rect()).inner;
                        rects.borrow_mut().push(rect);
                     });
                  });
            });
         }));
      }
      let rects = rects.borrow();
      assert_eq!(
         rects.len(),
         4,
         "expected a header and a row of two cells"
      );
      for column in 0..2 {
         let header = rects[column];
         let cell = rects[2 + column];
         assert!(
            (cell.min.x - header.min.x).abs() < 0.5,
            "column {column} row min.x {} != header min.x {}",
            cell.min.x,
            header.min.x
         );
         assert!(
            (cell.width() - header.width()).abs() < 0.5,
            "column {column} row width {} != header width {}",
            cell.width(),
            header.width()
         );
      }
   }

   /// The scroll content margin pads the rows inside the viewport, and its horizontal part insets
   /// the header band by the same amount, so the two bands keep one column set.
   #[test]
   fn table_scroll_content_margin_insets_both_bands() {
      let rows = [Row("a", "1"), Row("b", "2")];
      let ctx = themed_ctx();
      let rects = std::cell::RefCell::new(Vec::new());
      let table_rect = std::cell::Cell::new(Rect::ZERO);
      let first_row = std::cell::Cell::new(Rect::ZERO);
      for _ in 0..2 {
         drop(frame(&ctx, vec2(500.0, 400.0), |ui| {
            rects.borrow_mut().clear();
            scope(ui, |ui| {
               let response = Table::new("margin", &rows)
                  .columns([Column::Fr(1.0), Column::Fixed(60.0)])
                  .row_gap(10.0)
                  .scroll(
                     TableScroll::new().max_height(120.0).content_margin(Margin::symmetric(8, 4)),
                  )
                  .show(ui, |table| {
                     table.header(|header| {
                        for _ in 0..2 {
                           let rect = header.cell(|ui| ui.max_rect()).inner;
                           rects.borrow_mut().push(rect);
                        }
                     });
                     table.rows(|row, _item| {
                        if row.index() == 0 {
                           first_row.set(row.rect());
                        }
                        let rect = row.cell(|ui| ui.max_rect()).inner;
                        rects.borrow_mut().push(rect);
                        let rect = row.cell(|ui| ui.max_rect()).inner;
                        rects.borrow_mut().push(rect);
                     });
                  })
                  .response;
               table_rect.set(response.rect);
            });
         }));
      }
      let rects = rects.borrow();
      assert_eq!(
         rects.len(),
         6,
         "expected a header and two rows of two cells"
      );
      let table = table_rect.get();
      assert!(
         (rects[0].min.x - (table.min.x + 8.0)).abs() < 0.5,
         "header starts at {} instead of the table's left + 8",
         rects[0].min.x
      );
      for row in [2, 4] {
         assert!(
            (rects[row].min.x - rects[0].min.x).abs() < 0.5,
            "row cell starts at {} instead of the header's {}",
            rects[row].min.x,
            rects[0].min.x
         );
         assert!(
            (rects[row].width() - rects[0].width()).abs() < 0.5,
            "row column width {} != header {}",
            rects[row].width(),
            rects[0].width()
         );
      }
      // The Fr column resolves against the viewport minus both horizontal insets: 500 - 16.
      assert!(
         (rects[0].width() - 412.0).abs() < 0.5,
         "Fr column width {} != 500 - 16 - 60 - 12 of gap",
         rects[0].width()
      );
      assert!(
         (first_row.get().min.y - rects[0].max.y - 10.0 - 4.0).abs() < 1.0,
         "first row top {} != header bottom + row_gap + margin.top",
         first_row.get().min.y
      );
   }

   /// Every filled rect painted this frame, including the ones nested in a `Shape::Vec`.
   fn filled_rects(shape: &Shape, out: &mut Vec<RectShape>) {
      match shape {
         Shape::Rect(rect) => out.push(rect.clone()),
         Shape::Vec(shapes) => {
            for shape in shapes {
               filled_rects(shape, out);
            }
         }
         _ => {}
      }
   }

   #[test]
   fn row_frame_insets_the_cells_and_paints_a_card_per_row() {
      let ctx = themed_ctx();
      let card = Color32::from_rgb(0x12, 0x34, 0x56);
      let card_frame = Frame::new().inner_margin(Margin::same(8)).fill(card);
      let rects = std::cell::RefCell::new(Vec::new());
      let mut painted = Vec::new();

      for _ in 0..2 {
         let out = frame(&ctx, vec2(500.0, 400.0), |ui| {
            rects.borrow_mut().clear();
            scope(ui, |ui| {
               Table::new("row_frame", &ROWS)
                  .columns([Column::Fixed(100.0), Column::Fr(1.0)])
                  .row_gap(0.0)
                  .row_frame(card_frame)
                  .show(ui, |table| {
                     table.header(|header| {
                        for _ in 0..2 {
                           let rect = header.cell(|ui| ui.max_rect()).inner;
                           rects.borrow_mut().push(rect);
                        }
                     });
                     table.rows(|row, _item| {
                        for _ in 0..2 {
                           let rect = row.cell(|ui| ui.max_rect()).inner;
                           rects.borrow_mut().push(rect);
                        }
                     });
                  });
            });
         });

         painted.clear();
         for clipped in &out.shapes {
            filled_rects(&clipped.shape, &mut painted);
         }
      }

      let rects = rects.borrow();
      assert_eq!(
         rects.len(),
         6,
         "a header and two rows of two cells"
      );

      // The header shares the inset cells' left edge, not the card's.
      for column in 0..2 {
         assert!(
            (rects[column].min.x - rects[2 + column].min.x).abs() < 0.5,
            "header column {column} starts at {} but its cells start at {}",
            rects[column].min.x,
            rects[2 + column].min.x
         );
      }
      assert!(
         (rects[2].min.x - 8.0).abs() < 0.5,
         "a cell starts at {} instead of the frame's 8 pt inner margin",
         rects[2].min.x
      );

      // The card's margins are added to the pitch, so the cells keep their measured height: two rows
      // of `row_height` plus 8 pt above and below.
      let pitch = rects[2].height() + 16.0;
      assert!(
         (rects[4].min.y - rects[2].min.y - pitch).abs() < 0.5,
         "row pitch {} != cells {} + 16 of margins",
         rects[4].min.y - rects[2].min.y,
         rects[2].height()
      );

      let cards: Vec<_> = painted.iter().filter(|painted| painted.fill == card).collect();
      assert_eq!(
         cards.len(),
         2,
         "one card per row, got {}",
         cards.len()
      );
      for painted in &cards {
         assert!(
            (painted.rect.height() - pitch).abs() < 0.5,
            "card height {} != the row pitch {pitch}",
            painted.rect.height()
         );
         assert!(
            (painted.rect.width() - 500.0).abs() < 0.5,
            "card width {} != the table's 500",
            painted.rect.width()
         );
      }
      assert!(
         (cards[1].rect.min.y - cards[0].rect.min.y - pitch).abs() < 0.5,
         "cards stack by the pitch, not by the viewport"
      );
   }
}
