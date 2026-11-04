//! A scroll area sized for a themed desktop panel.

use std::hash::Hash;

use egui::scroll_area::{ScrollAreaOutput, ScrollBarVisibility};
use egui::{IdSalt, ScrollArea, Ui};

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

   /// Show the scroll area. Mirrors [`ScrollArea::show`].
   pub fn show<R>(self, ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> ScrollAreaOutput<R> {
      let mut area = if self.horizontal {
         ScrollArea::both()
      } else {
         ScrollArea::vertical()
      };
      area = area
         .id_salt(self.id_salt)
         .scroll_bar_visibility(self.visibility)
         .auto_shrink(self.auto_shrink);

      ui.scope(|ui| {
         let style = ui.style_mut();
         style.spacing.scroll.bar_width = self.bar_width;
         style.spacing.scroll.bar_inner_margin = 2.0;
         style.spacing.scroll.bar_outer_margin = 4.0;
         area.show(ui, add)
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
}
