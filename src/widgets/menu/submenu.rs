//! [`SubMenuItem`]: a menu row that opens a flyout child menu.

use egui::{
   Layout, PointerButton, Popup, PopupCloseBehavior, Rect, Response, Ui, UiKind, UiStackInfo,
   WidgetText,
   emath::{Align, RectAlign},
   pos2,
};

use super::MenuItem;
use super::shared::menu_frame;
use crate::theme::Theme;

/// A menu row that opens a flyout submenu when hovered.
///
/// Visually a [`MenuItem`] with a right-pointing chevron; pair the trigger with a
/// body closure that fills the child menu. Use inside any [`Menu`](super::Menu)
/// popup. The submenu opens to the right and stays open while the pointer
/// remains over either the trigger or the flyout panel.
///
/// ```no_run
/// # use egui::Ui;
/// # use egui_elements::widgets::{Menu, MenuItem, SubMenuItem};
/// # fn menus(ui: &mut Ui) {
/// # let trigger = ui.button("File");
/// Menu::new("app").show_below(&trigger, |ui| {
///     ui.add(MenuItem::new("New"));
///     SubMenuItem::new("Open Recent").show(ui, |ui| {
///         ui.add(MenuItem::new("theme.rs"));
///         ui.add(MenuItem::new("README.md"));
///     });
///     ui.add(MenuItem::new("Save"));
/// });
/// # }
/// ```
///
/// Hover-to-flyout, click-to-pin, and proper "stay open while child is open"
/// behavior come from `egui`'s built-in submenu machinery; this type just wires
/// our [`MenuItem`] visual into that pipeline.
#[must_use = "Call `.show(ui, |ui| ...)` to render the submenu trigger and flyout."]
pub struct SubMenuItem {
   label: WidgetText,
   icon: Option<WidgetText>,
   enabled: bool,
}

impl std::fmt::Debug for SubMenuItem {
   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      f.debug_struct("SubMenuItem")
         .field("label", &self.label.text())
         .field("icon", &self.icon.is_some())
         .field("enabled", &self.enabled)
         .finish()
   }
}

impl SubMenuItem {
   /// Create a new submenu trigger with the given label.
   pub fn new(label: impl Into<WidgetText>) -> Self {
      Self {
         label: label.into(),
         icon: None,
         enabled: true,
      }
   }

   /// Show a leading icon (any text, typically a unicode glyph) in the gutter to
   /// the left of the label.
   pub fn icon(mut self, icon: impl Into<WidgetText>) -> Self {
      self.icon = Some(icon.into());
      self
   }

   /// Disable the submenu trigger. Disabled triggers do not open the flyout and
   /// render with muted text.
   pub fn enabled(mut self, enabled: bool) -> Self {
      self.enabled = enabled;
      self
   }

   /// Render the trigger row and attach the flyout submenu. The body closure
   /// populates the child menu and is invoked while the flyout is open. Returns
   /// `Some(R)` with the body's return value while the submenu is open, `None`
   /// while closed.
   pub fn show<R>(self, ui: &mut Ui, body: impl FnOnce(&mut Ui) -> R) -> Option<R> {
      let mut item = MenuItem::new(self.label).enabled(self.enabled).with_submenu_arrow();

      if let Some(icon) = self.icon {
         item = item.icon(icon);
      }

      let response = ui.add(item);

      sub_menu_show(ui, &response, body)
   }
}

/// Open a flyout submenu below `button_response`, with the parent panel's own
/// margin plus one spacing step between the two panels.
///
/// This vendors egui 0.37's `SubMenu::show` logic so we keep its
/// hover/click/close-behavior semantics intact, but anchors the popup to a strip
/// along the trigger row's bottom and indents it, so the flyout reads as a
/// distinct panel rather than a continuation of the parent menu.
fn sub_menu_show<R>(
   ui: &Ui,
   button_response: &Response,
   content: impl FnOnce(&mut Ui) -> R,
) -> Option<R> {
   use egui::containers::menu::{MenuConfig, MenuState};

   let theme = Theme::current(ui.ctx());
   // Tighten the submenu's padding to the menu's own margin so the first row
   // sits close to the panel's top edge instead of dropping below it.
   let frame = menu_frame(&theme);
   // The anchor is the trigger row, which ends one frame margin above the parent
   // panel's bottom border. Clear that margin, then leave one spacing step, so
   // the flyout reads as a separate panel instead of cutting into the parent's
   // edge.
   let gap = frame.inner_margin.bottom as f32 + theme.spacing.xs;
   let id = button_response.id.with("submenu");

   let (open_item, menu_id) = MenuState::from_ui(ui, |state, stack| {
      (state.open_item, stack.unique_id)
   });
   // `MenuConfig::find` walks the stack from the parent menu (we haven't shown
   // the submenu yet, so the current ui's stack tail is the parent). The submenu
   // inherits the parent's close behavior and style.
   let menu_config = MenuConfig::find(ui);

   let menu_root_response = ui.ctx().read_response(menu_id).expect("submenu must be inside a menu");
   let hover_pos = ui.ctx().pointer_hover_pos();
   let menu_rect = menu_root_response.rect - frame.total_margin();
   let is_hovering_menu = hover_pos.is_some_and(|pos| {
      ui.ctx().layer_id_at(pos) == Some(menu_root_response.layer_id) && menu_rect.contains(pos)
   });

   let is_any_open = open_item.is_some();
   let mut is_open = open_item == Some(id);
   let was_open = is_open;
   let mut set_open: Option<bool> = None;

   let button_rect = button_response.rect.expand2(ui.style().spacing.item_spacing / 2.0);
   let is_hovered = hover_pos.is_some_and(|pos| button_rect.contains(pos));

   let clicked = button_response.clicked();
   let clicked_by_pointer = button_response.clicked_by(PointerButton::Primary);
   let clicked_by_keyboard_or_access = clicked && !clicked_by_pointer;

   // egui's original reads `ui.is_enabled()`, which is only the parent's state.
   // The trigger's response is disabled when either the parent or
   // `SubMenuItem::enabled(false)` disables it.
   let enabled = button_response.enabled();

   if enabled && is_open && clicked_by_keyboard_or_access {
      set_open = Some(false);
      is_open = false;
   }

   let should_open = enabled && ((!was_open && clicked) || (is_hovered && !is_any_open));

   if should_open {
      set_open = Some(true);
      is_open = true;
      MenuState::from_id(ui.ctx(), menu_id, |state| {
         state.open_item = None;
      });
   }

   // Anchor the popup to a zero-height strip along the trigger row's bottom,
   // indented from the row's left edge by `LEFT_INSET` so a `BOTTOM_START` align
   // drops the submenu down with its left edge *inside* the trigger row (rather
   // than column-flush with the parent menu). 24pt aligns roughly with where the
   // trigger's label text starts past the leading icon gutter, so the submenu
   // reads as continuing from the trigger's content rather than from the panel's
   // edge. We only touch `interact_rect` (what `Popup::from_response` uses for
   // anchoring); hover/click detection further down still consults the original
   // `rect`.
   const LEFT_INSET: f32 = 24.0;

   let mut response = button_response.clone();
   let bottom = button_response.rect.bottom();
   let left = (button_response.rect.left() + LEFT_INSET).min(button_response.rect.right());

   response.interact_rect = Rect::from_min_max(
      pos2(left, bottom),
      pos2(button_response.rect.right(), bottom),
   );

   let popup_response = Popup::from_response(&response)
      .id(id)
      .open(is_open)
      // Drop straight down from the trigger row: popup top-left aligns with the
      // anchor strip's left-bottom, so the submenu shares a column with the
      // trigger and grows to the right only as wide as its contents need.
      .align(RectAlign::BOTTOM_START)
      // Pin the alignment — without this, egui's `find_best_align` tries
      // `RectAlign::MENU_ALIGNS` as fallbacks if it thinks our requested side
      // doesn't fit, and silently picks a different alignment, which is what
      // makes our `gap` look like it has no effect when the parent menu sits on
      // a side of the viewport.
      .align_alternatives(&[])
      .layout(Layout::top_down_justified(Align::Min))
      .gap(gap)
      .style(menu_config.style.clone())
      .frame(frame)
      .close_behavior(PopupCloseBehavior::IgnoreClicks)
      .info(
         UiStackInfo::new(UiKind::Menu)
            .with_tag_value(MenuConfig::MENU_CONFIG_TAG, menu_config.clone()),
      )
      .show(|ui| {
         if button_response.clicked() || button_response.is_pointer_button_down_on() {
            ui.ctx().move_to_top(ui.layer_id());
         }

         content(ui)
      });

   if let Some(popup_response) = &popup_response {
      let is_deepest_submenu = MenuState::is_deepest_open_sub_menu(ui.ctx(), id);
      let clicked_outside = is_deepest_submenu
         && popup_response.response.clicked_elsewhere()
         && menu_root_response.clicked_elsewhere();
      let submenu_button_clicked = button_response.clicked();
      let clicked_inside = is_deepest_submenu
         && !submenu_button_clicked
         && response.ctx.input(|i| i.pointer.any_click())
         && hover_pos.is_some_and(|pos| popup_response.response.interact_rect.contains(pos));

      let click_close = match menu_config.close_behavior {
         PopupCloseBehavior::CloseOnClick => clicked_outside || clicked_inside,
         PopupCloseBehavior::CloseOnClickOutside => clicked_outside,
         PopupCloseBehavior::IgnoreClicks => false,
      };

      if click_close {
         set_open = Some(false);
      }

      let is_moving_towards_rect =
         ui.input(|i| i.pointer.is_moving_towards_rect(&popup_response.response.rect));

      if is_moving_towards_rect {
         ui.ctx().request_repaint();
      }

      let hovering_other_menu_entry = is_open
         && !is_hovered
         && !popup_response.response.contains_pointer()
         && !is_moving_towards_rect
         && is_hovering_menu;

      if hovering_other_menu_entry {
         set_open = Some(false);
      }
   }

   if let Some(open) = set_open {
      MenuState::from_id(ui.ctx(), menu_id, |state| {
         state.open_item = open.then_some(id);
      });
   }

   if is_open {
      MenuState::mark_shown(ui.ctx(), id);
   }

   popup_response.map(|r| r.inner)
}
