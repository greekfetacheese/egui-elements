//! Menu proof: action rows, shortcuts, toggle rows, sections and a flyout submenu.
//!
//! Run with: `cargo run --example menu`

use eframe::egui::*;
use egui_elements::widgets::{Button, Menu, MenuItem, MenuSection, SubMenuItem};
use egui_elements::{Theme, ThemeKind, utils};

fn main() -> eframe::Result {
   let options = eframe::NativeOptions {
      viewport: ViewportBuilder::default()
         .with_title("egui-elements menu")
         .with_inner_size([680.0, 460.0])
         .with_resizable(true),
      ..Default::default()
   };

   eframe::run_native(
      "egui-elements menu",
      options,
      Box::new(|cc| Ok(Box::new(MenuApp::new(cc)))),
   )
}

struct MenuApp {
   theme: Theme,
   last: String,
   auto_save: bool,
   word_wrap: bool,
   zoom: u8,
}

impl MenuApp {
   fn new(cc: &eframe::CreationContext<'_>) -> Self {
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&cc.egui_ctx);

      Self {
         theme,
         last: "none".to_owned(),
         auto_save: true,
         word_wrap: false,
         zoom: 100,
      }
   }

   fn header(&mut self, ui: &mut Ui) {
      let space = self.theme.spacing;
      let colors = self.theme.colors;
      let title = self.theme.typography.large;
      let theme = &self.theme;

      Panel::top("header")
         .frame(
            Frame::new()
               .fill(colors.bg)
               .inner_margin(space.margin_xy(space.lg, space.md))
               .stroke(Stroke::new(1.0, colors.border)),
         )
         .show(ui, |ui| {
            ui.horizontal(|ui| {
               ui.label(RichText::new("Menu").size(title).color(colors.text).strong());

               ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                  utils::theme_switcher(theme, ui);
               });
            });
         });
   }

   fn body(&mut self, ui: &mut Ui) {
      let space = self.theme.spacing;
      let colors = self.theme.colors;
      let normal = self.theme.typography.normal;

      ui.label(
         RichText::new(
            "Click a trigger, then an item. An item click, an outside click or Esc dismisses the menu.",
         )
         .size(normal)
         .color(colors.text_muted),
      );

      ui.add_space(space.lg);

      ui.horizontal_wrapped(|ui| {
         ui.spacing_mut().item_spacing.x = space.md;

         let trigger = ui.add(Button::new("Actions").min_size(vec2(132.0, 36.0)));

         let _ = Menu::new("actions").show_below(&trigger, |ui| {
            if ui.add(MenuItem::new("Edit").shortcut("Ctrl+E")).clicked() {
               self.last = "edit".to_owned();
            }

            if ui.add(MenuItem::new("Duplicate").icon("\u{29C9}")).clicked() {
               self.last = "duplicate".to_owned();
            }

            ui.add(MenuItem::new("Copy link").enabled(false));

            ui.separator();
            ui.add(MenuSection::new("Options"));

            if ui.add(MenuItem::new("Auto save").checked(self.auto_save)).clicked() {
               self.auto_save = !self.auto_save;
               self.last = format!("auto save {}", on_off(self.auto_save));
            }

            if ui.add(MenuItem::new("Word wrap").checked(self.word_wrap)).clicked() {
               self.word_wrap = !self.word_wrap;
               self.last = format!("word wrap {}", on_off(self.word_wrap));
            }

            ui.separator();

            if ui.add(MenuItem::new("Delete").danger()).clicked() {
               self.last = "delete".to_owned();
            }
         });

         let trigger = ui.add(Button::new("View").min_size(vec2(132.0, 36.0)));

         let _ = Menu::new("view").show_below(&trigger, |ui| {
            for (zoom, label) in [(80_u8, "80 %"), (100, "100 %"), (125, "125 %")] {
               if ui.add(MenuItem::new(label).radio(self.zoom == zoom)).clicked() {
                  self.zoom = zoom;
                  self.last = format!("zoom {zoom} %");
               }
            }

            ui.separator();
            ui.add(MenuSection::new("Navigate"));

            let _ = SubMenuItem::new("Open recent").show(ui, |ui| {
               for file in ["theme.rs", "menu.rs", "README.md"] {
                  if ui.add(MenuItem::new(file)).clicked() {
                     self.last = file.to_owned();
                  }
               }
            });
         });
      });

      ui.add_space(space.md);

      ui.label(
         RichText::new(format!("Last action: {}", self.last))
            .size(normal)
            .color(colors.text_muted),
      );
   }
}

impl eframe::App for MenuApp {
   fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
      let space = self.theme.spacing;
      let bg = self.theme.colors.bg;

      self.header(ui);
      self.theme.clone().install(ui.ctx());

      CentralPanel::default()
         .frame(Frame::new().fill(bg).inner_margin(space.margin(space.lg)))
         .show(ui, |ui| self.body(ui));
   }
}

fn on_off(value: bool) -> &'static str {
   if value { "on" } else { "off" }
}
