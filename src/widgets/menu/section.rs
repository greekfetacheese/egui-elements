//! [`MenuSection`]: a small uppercase header for a group of menu rows.

use egui::{Pos2, Response, Role, Sense, Ui, Vec2, Widget, WidgetInfo, WidgetText};

use super::shared::galley;
use crate::theme::Theme;

/// A small uppercase header used to label a group of items inside a
/// [`Menu`](super::Menu).
///
/// Add with `ui.add(MenuSection::new("Edit"))` between groups of related items.
/// The header is non-interactive and renders in the muted text tone with extra
/// top padding so it visually separates from the item above without needing a
/// separator.
///
/// ```no_run
/// # use egui::Ui;
/// # use egui_elements::widgets::{Menu, MenuItem, MenuSection};
/// # fn menus(ui: &mut Ui) {
/// # let trigger = ui.button("⋯");
/// Menu::new("row_actions").show_below(&trigger, |ui| {
///     ui.add(MenuItem::new("Copy").shortcut("⌘C"));
///     ui.separator();
///     ui.add(MenuSection::new("Selection"));
///     ui.add(MenuItem::new("Highlight matches").checked(true));
///     ui.add(MenuItem::new("Show whitespace").checked(false));
/// });
/// # }
/// ```
#[must_use = "Add with `ui.add(...)`."]
pub struct MenuSection {
   label: WidgetText,
}

impl std::fmt::Debug for MenuSection {
   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      f.debug_struct("MenuSection").field("label", &self.label.text()).finish()
   }
}

impl MenuSection {
   /// Create a section header with the given label. The text is uppercased at
   /// render time.
   pub fn new(label: impl Into<WidgetText>) -> Self {
      Self {
         label: label.into(),
      }
   }
}

impl Widget for MenuSection {
   fn ui(self, ui: &mut Ui) -> Response {
      let theme = Theme::current(ui.ctx());
      let colors = &theme.colors;
      let typography = &theme.typography;

      let pad_x = 10.0;
      let pad_top = 6.0;
      let pad_bottom = 2.0;

      let text = self.label.text().to_uppercase();
      let galley = galley(ui, &text, typography.small, false);

      let desired = Vec2::new(
         galley.size().x + pad_x * 2.0,
         galley.size().y + pad_top + pad_bottom,
      );
      // Use `allocate_at_least` to inherit the parent menu's
      // top-down-justified width so the row spans the popup's interior.
      let (rect, response) = ui.allocate_at_least(desired, Sense::hover());

      if ui.is_rect_visible(rect) {
         let pos = Pos2::new(rect.min.x + pad_x, rect.min.y + pad_top);

         ui.painter().galley(pos, galley, colors.text_muted);
      }

      response.widget_info(|| WidgetInfo::labeled(Role::Label, true, &text));

      response
   }
}
