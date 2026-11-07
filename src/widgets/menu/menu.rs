//! [`Menu`]: a click-to-open popup list anchored below a trigger.

use std::hash::Hash;

use egui::{IdSalt, Popup, PopupCloseBehavior, Response, Ui};

use super::shared::menu_frame;
use crate::theme::Theme;

/// A click-to-open popup menu anchored below a trigger [`Response`].
///
/// Call [`Menu::show_below`] after painting the trigger; it opens on trigger
/// clicks and closes on item click, outside-click, or `Esc`.
#[derive(Debug, Clone)]
#[must_use = "Call `.show_below(&trigger, |ui| ...)` to render the menu."]
pub struct Menu {
   id_salt: IdSalt,
   min_width: f32,
}

impl Menu {
   /// Create a menu keyed by `id_salt`.
   ///
   /// The salt is combined with the trigger's id to persist the open/closed
   /// state across frames, so it only needs to be unique among the menus on one
   /// trigger.
   pub fn new(id_salt: impl Hash + std::fmt::Debug) -> Self {
      Self {
         id_salt: IdSalt::new(("egui_elements::menu", id_salt)),
         min_width: 180.0,
      }
   }

   /// Minimum width of the popup in points. Default: 180.
   pub fn min_width(mut self, min_width: f32) -> Self {
      self.min_width = min_width;
      self
   }

   /// Render the menu below `trigger`. Returns `Some(R)` with the body
   /// closure's return value while the menu is open, `None` while closed.
   pub fn show_below<R>(
      self,
      trigger: &Response,
      add_contents: impl FnOnce(&mut Ui) -> R,
   ) -> Option<R> {
      let theme = Theme::current(&trigger.ctx);
      let popup_id = trigger.id.with_salt(self.id_salt);

      Popup::menu(trigger)
         .id(popup_id)
         .frame(menu_frame(&theme))
         // One spacing step of air under the trigger, so the panel reads as its
         // own surface instead of being welded to the widget that opened it.
         // Same default as `ComboBox::popup_gap`.
         .gap(theme.spacing.xs)
         .close_behavior(PopupCloseBehavior::CloseOnClick)
         .show(|ui| {
            ui.set_min_width(self.min_width);
            // Tight stacking — `MenuItem` has its own interior padding.
            ui.spacing_mut().item_spacing.y = 2.0;
            add_contents(ui)
         })
         .map(|r| r.inner)
   }
}
