//! A scroll area sized for a themed desktop panel.

use std::hash::Hash;
use std::ops::Range;

use egui::scroll_area::{ScrollAreaOutput, ScrollBarVisibility};
use egui::{IdSalt, Layout, ScrollArea, Ui};

/// A [`ScrollArea`] with a thin bar and the theme's host spacing.
///
/// egui's default bar is wide enough to read as a gutter; this keeps the handle colours from
/// the theme but pins the geometry: a 6-point bar, 2-point inner margin, 4-point outer margin.
/// [`Scroll::bars`] hides it entirely for panes that scroll with the wheel only.
///
/// ```
/// # use egui::__run_test_ui;
/// # use egui_elements::widgets::Scroll;
/// # __run_test_ui(|ui| {
/// Scroll::vertical("reading").show(ui, |ui| {
///    ui.label("Scrolled content");
/// });
/// # });
/// ```
#[must_use = "You should call .show()"]
#[derive(Clone, Copy, Debug)]
pub struct Scroll {
   id_salt: IdSalt,
   horizontal: bool,
   visibility: ScrollBarVisibility,
   bar_width: f32,
   auto_shrink: [bool; 2],
   max_height: Option<f32>,
   stick_to_bottom: bool,
   layout: Option<Layout>,
}

impl Scroll {
   /// A vertical scroll area. Defaults to not shrinking in either axis.
   pub fn vertical(id_salt: impl Hash + std::fmt::Debug) -> Self {
      Self {
         id_salt: IdSalt::new(id_salt),
         horizontal: false,
         visibility: ScrollBarVisibility::VisibleWhenNeeded,
         bar_width: 6.0,
         auto_shrink: [false, false],
         max_height: None,
         stick_to_bottom: false,
         layout: None,
      }
   }

   /// A scroll area on both axes.
   pub fn both(id_salt: impl Hash + std::fmt::Debug) -> Self {
      Self {
         horizontal: true,
         ..Self::vertical(id_salt)
      }
   }

   /// When the bar is drawn. Default [`ScrollBarVisibility::VisibleWhenNeeded`].
   pub fn bars(mut self, visibility: ScrollBarVisibility) -> Self {
      self.visibility = visibility;
      self
   }

   /// Bar width in points. Default `6.0`.
   pub fn bar_width(mut self, width: f32) -> Self {
      self.bar_width = width;
      self
   }

   /// `[x, y]` shrink flags passed to [`ScrollArea::auto_shrink`]. Default `[false, false]`.
   pub fn auto_shrink(mut self, shrink: [bool; 2]) -> Self {
      self.auto_shrink = shrink;
      self
   }

   /// Cap the area's height in points. Default: unconstrained.
   pub fn max_height(mut self, height: f32) -> Self {
      self.max_height = Some(height);
      self
   }

   /// Keep the view pinned to the end of the content as it grows. Default `false`.
   pub fn stick_to_bottom(mut self, stick: bool) -> Self {
      self.stick_to_bottom = stick;
      self
   }

   /// Force the content's layout instead of inheriting the parent's.
   ///
   /// A `Scroll` inside a `Flex` row cell inherits the cell's left-to-right layout, which lays
   /// the content out sideways behind the clip. Pass a top-down layout for lists:
   ///
   /// ```
   /// # use egui::{Align, Layout, __run_test_ui};
   /// # use egui_elements::widgets::Scroll;
   /// # __run_test_ui(|ui| {
   /// Scroll::vertical("list")
   ///    .layout(Layout::top_down(Align::Min))
   ///    .show(ui, |ui| {
   ///       ui.label("Row");
   ///    });
   /// # });
   /// ```
   pub fn layout(mut self, layout: Layout) -> Self {
      self.layout = Some(layout);
      self
   }

   /// Show the scroll area. Mirrors [`ScrollArea::show`].
   pub fn show<R>(self, ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> ScrollAreaOutput<R> {
      let area = self.configure(if self.horizontal {
         ScrollArea::both()
      } else {
         ScrollArea::vertical()
      });
      Self::bar_style(self.bar_width, ui, |ui| {
         area.show(ui, |ui| self.content(ui, add))
      })
   }

   /// Show only the visible rows of a long list. Mirrors [`ScrollArea::show_rows`].
   ///
   /// The area always scrolls vertically here, whatever [`Self::both`] asked for.
   pub fn show_rows<R>(
      self,
      ui: &mut Ui,
      row_height_sans_spacing: f32,
      total_rows: usize,
      add: impl FnOnce(&mut Ui, Range<usize>) -> R,
   ) -> ScrollAreaOutput<R> {
      let area = self.configure(ScrollArea::vertical());
      Self::bar_style(self.bar_width, ui, |ui| {
         area.show_rows(
            ui,
            row_height_sans_spacing,
            total_rows,
            |ui, rows| self.content(ui, |ui| add(ui, rows)),
         )
      })
   }

   /// Apply this `Scroll`'s state to an egui area.
   fn configure(self, mut area: ScrollArea) -> ScrollArea {
      area = area
         .id_salt(self.id_salt)
         .scroll_bar_visibility(self.visibility)
         .auto_shrink(self.auto_shrink)
         .stick_to_bottom(self.stick_to_bottom);
      if let Some(height) = self.max_height {
         area = area.max_height(height);
      }
      area
   }

   /// Run `add` under the forced content layout, when one is set.
   fn content<R>(self, ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
      match self.layout {
         Some(layout) => ui.with_layout(layout, add).inner,
         None => add(ui),
      }
   }

   /// Scope the thin-bar geometry around `add`.
   fn bar_style<R>(bar_width: f32, ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
      ui.scope(|ui| {
         let style = ui.style_mut();
         style.spacing.scroll.bar_width = bar_width;
         style.spacing.scroll.bar_inner_margin = 2.0;
         style.spacing.scroll.bar_outer_margin = 4.0;
         add(ui)
      })
      .inner
   }
}

#[cfg(test)]
mod tests {
   use super::*;

   #[test]
   fn the_bar_geometry_reaches_the_content() {
      egui::__run_test_ui(|ui| {
         let out = Scroll::vertical("geometry").bar_width(11.0).show(ui, |ui| {
            assert!((ui.style().spacing.scroll.bar_width - 11.0).abs() < f32::EPSILON);
            assert!((ui.style().spacing.scroll.bar_inner_margin - 2.0).abs() < f32::EPSILON);
            7
         });
         assert_eq!(out.inner, 7);
      });
   }

   #[test]
   fn the_content_layout_can_be_forced() {
      egui::__run_test_ui(|ui| {
         let mut main_dir = None;
         Scroll::vertical("layout")
            .layout(Layout::top_down(egui::Align::Min))
            .show(ui, |ui| {
               main_dir = Some(ui.layout().main_dir());
            });
         assert_eq!(main_dir, Some(egui::Direction::TopDown));
      });
   }

   #[test]
   fn virtualized_rows_stay_within_bounds() {
      egui::__run_test_ui(|ui| {
         let mut seen = Vec::new();
         Scroll::vertical("rows").show_rows(ui, 20.0, 50, |ui, rows| {
            seen.extend(rows.clone());
            for row in rows {
               ui.label(format!("row {row}"));
            }
         });
         assert!(seen.iter().all(|row| *row < 50));
         assert!(seen.windows(2).all(|pair| pair[0] < pair[1]));
      });
   }
}
