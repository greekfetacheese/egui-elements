//! Action menus — a popup list of items attached to a trigger [`Response`].
//!
//! [`Menu`] opens a popup below a trigger widget when the trigger is clicked.
//! [`MenuItem`] is the styled leaf inside the popup: label on the left, optional
//! keyboard-shortcut hint on the right, optional `danger` tint for destructive
//! actions. [`MenuSection`] labels a group of rows, and [`SubMenuItem`] opens a
//! flyout child menu. Separators between groups use the stock `ui.separator()`.
//!
//! ```no_run
//! # use egui::Ui;
//! # use egui_elements::widgets::{Button, Menu, MenuItem};
//! # fn menus(ui: &mut Ui) {
//! let trigger = ui.add(Button::new("⋯").small());
//! Menu::new("row_actions").show_below(&trigger, |ui| {
//!    if ui.add(MenuItem::new("Edit").shortcut("⌘ E")).clicked() {
//!       // …
//!    }
//!    ui.separator();
//!    if ui.add(MenuItem::new("Delete").danger()).clicked() { /* … */ }
//! });
//! # }
//! ```
//!
//! The popup is dismissed by clicking any item, clicking outside, or pressing
//! `Esc`. Keyboard navigation (arrows + Enter) is not implemented in this
//! version.
//!
//! Forked from [`egui-elegance`](https://github.com/matrix-research-inc/egui-elegance)
//! 0.18.1 `src/menu.rs` (MIT OR Apache-2.0, © Stephen Berry), adapted to this
//! crate's [`Theme`](crate::theme::Theme) slots and spacing tokens.

mod item;
mod menu;
mod section;
mod shared;
mod submenu;

pub use item::MenuItem;
pub use menu::Menu;
pub use section::MenuSection;
pub use submenu::SubMenuItem;

#[cfg(test)]
mod tests {
   use super::*;
   use crate::theme::{Theme, ThemeKind};
   use egui::{
      Context, Event, FullOutput, Modifiers, PointerButton, Pos2, RawInput, Rect, Sense, Ui, pos2,
      vec2,
   };

   fn themed_ctx() -> Context {
      let ctx = Context::default();
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&ctx);
      ctx
   }

   /// One frame at an explicit `input.time` (no sleeps, no wall clock).
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

   /// A click-sensing stand-in for a trigger button, at a stable id.
   fn trigger(ui: &mut Ui) -> egui::Response {
      ui.allocate_response(vec2(96.0, 30.0), Sense::click())
   }

   #[test]
   fn a_closed_menu_runs_no_body() {
      let ctx = themed_ctx();
      let mut drawn = false;
      let mut shown = true;

      frame(&ctx, 0.0, vec![], |ui| {
         let trigger = trigger(ui);
         shown = Menu::new("closed").show_below(&trigger, |_ui| drawn = true).is_some();
      });

      assert!(
         !shown,
         "an untouched trigger must keep the menu closed"
      );
      assert!(!drawn, "a closed menu must not run its body");
   }

   #[test]
   fn a_trigger_click_opens_the_menu() {
      let ctx = themed_ctx();
      let mut rect = Rect::NOTHING;

      frame(&ctx, 0.0, vec![], |ui| {
         rect = trigger(ui).rect;
      });

      let at = rect.center();

      frame(&ctx, 0.1, vec![press(at)], |ui| {
         let trigger = trigger(ui);
         let _ = Menu::new("open").show_below(&trigger, |_ui| {});
      });
      frame(&ctx, 0.2, vec![release(at)], |ui| {
         let trigger = trigger(ui);
         let _ = Menu::new("open").show_below(&trigger, |_ui| {});
      });

      let mut drawn = false;
      let mut shown = false;
      let mut trigger_bottom = f32::NAN;
      let mut content_top = f32::NAN;

      frame(&ctx, 0.3, vec![], |ui| {
         let trigger = trigger(ui);
         trigger_bottom = trigger.rect.bottom();
         shown = Menu::new("open")
            .show_below(&trigger, |ui| {
               drawn = true;
               content_top = ui.max_rect().top();
               ui.add(MenuItem::new("Edit"));
               let _ = SubMenuItem::new("Open recent").show(ui, |ui| {
                  ui.add(MenuItem::new("theme.rs"));
               });
            })
            .is_some();
      });

      assert!(shown, "a click on the trigger must open the menu");
      assert!(drawn, "an open menu must run its body");

      // The panel used to be welded to the button: the popup gap and the frame
      // margin both sit between the trigger's bottom edge and the body.
      let spacing = Theme::spacing_from_ctx(&ctx).xs;
      assert!(
         content_top - trigger_bottom >= spacing * 2.0 - 0.01,
         "the panel body must clear the trigger: {content_top} vs {trigger_bottom}"
      );
   }

   #[test]
   fn an_enabled_item_reports_a_click() {
      let ctx = themed_ctx();
      let mut rect = Rect::NOTHING;

      frame(&ctx, 0.0, vec![], |ui| {
         rect = ui.add(MenuItem::new("Edit")).rect;
      });

      let at = rect.center();

      frame(&ctx, 0.1, vec![press(at)], |ui| {
         ui.add(MenuItem::new("Edit"));
      });

      let mut clicked = false;

      frame(&ctx, 0.2, vec![release(at)], |ui| {
         clicked = ui.add(MenuItem::new("Edit")).clicked();
      });

      assert!(clicked, "an enabled row must fire on click");
   }

   #[test]
   fn a_disabled_item_ignores_clicks_and_reports_disabled() {
      let ctx = themed_ctx();
      let mut rect = Rect::NOTHING;

      frame(&ctx, 0.0, vec![], |ui| {
         rect = ui.add(MenuItem::new("Publish").enabled(false)).rect;
      });

      let at = rect.center();

      frame(&ctx, 0.1, vec![press(at)], |ui| {
         ui.add(MenuItem::new("Publish").enabled(false));
      });

      let mut clicked = false;
      let mut enabled = true;

      frame(&ctx, 0.2, vec![release(at)], |ui| {
         let response = ui.add(MenuItem::new("Publish").enabled(false));
         clicked = response.clicked();
         enabled = response.enabled();
      });

      assert!(!clicked, "a disabled row must not fire");
      assert!(!enabled, "egui must see the row as disabled");
   }

   #[test]
   fn a_section_header_does_not_take_clicks() {
      let ctx = themed_ctx();
      let mut rect = Rect::NOTHING;

      frame(&ctx, 0.0, vec![], |ui| {
         rect = ui.add(MenuSection::new("View")).rect;
      });

      let at = rect.center();

      frame(&ctx, 0.1, vec![press(at)], |ui| {
         ui.add(MenuSection::new("View"));
      });

      let mut clicked = false;

      frame(&ctx, 0.2, vec![release(at)], |ui| {
         clicked = ui.add(MenuSection::new("View")).clicked();
      });

      assert!(!clicked, "a section header is not interactive");
   }

   #[test]
   fn a_leading_gutter_aligns_plain_and_toggled_rows() {
      let ctx = themed_ctx();
      let mut widths = [0.0; 3];

      frame(&ctx, 0.0, vec![], |ui| {
         widths[0] = ui.add(MenuItem::new("Same")).rect.width();
         widths[1] = ui.add(MenuItem::new("Same").checked(false)).rect.width();
         widths[2] = ui.add(MenuItem::new("Same").checked(true)).rect.width();
      });

      assert!(
         (widths[1] - widths[0] - 24.0).abs() < 0.01,
         "a leading gutter must add 16 pt plus the 8 pt gap: {widths:?}"
      );
      assert!(
         (widths[1] - widths[2]).abs() < 0.01,
         "off and on rows must keep the same width: {widths:?}"
      );
   }
}
