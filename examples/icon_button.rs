//! IconButton proof: themed square icon buttons, tooltips, selected and disabled states.
//!
//! Run with: `cargo run --example icon_button --features lucide`

use eframe::egui::*;
use egui_elements::egui_lucide::Lucide;
use egui_elements::widgets::{Frame as Card, IconButton};
use egui_elements::{Theme, ThemeKind, utils};

fn main() -> eframe::Result {
   let options = eframe::NativeOptions {
      viewport: ViewportBuilder::default()
         .with_title("egui-elements icon button")
         .with_inner_size([640.0, 420.0])
         .with_resizable(true),
      ..Default::default()
   };

   eframe::run_native(
      "egui-elements icon button",
      options,
      Box::new(|cc| Ok(Box::new(IconApp::new(cc)))),
   )
}

struct IconApp {
   theme: Theme,
   pinned: bool,
   last: &'static str,
}

impl IconApp {
   fn new(cc: &eframe::CreationContext<'_>) -> Self {
      egui_extras::install_image_loaders(&cc.egui_ctx);
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&cc.egui_ctx);

      Self {
         theme,
         pinned: false,
         last: "none",
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
               ui.label(RichText::new("IconButton").size(title).color(colors.text).strong());
               ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                  utils::theme_switcher(theme, ui);
               });
            });
         });
   }

   fn card(&mut self, ui: &mut Ui) {
      let space = self.theme.spacing;
      let colors = self.theme.colors;
      let normal = self.theme.typography.normal;

      Card::new()
         .fill(colors.widget_bg)
         .corner_radius(CornerRadius::same(12))
         .inner_margin(space.margin(space.xl))
         .show(ui, |ui| {
            ui.label(RichText::new("Toolbar").size(normal).color(colors.text).strong());
            ui.label(
               RichText::new("Button chrome, image instead of text; the tooltip is the label.")
                  .size(normal)
                  .color(colors.text_muted),
            );
            ui.add_space(space.lg);

            ui.horizontal(|ui| {
               ui.spacing_mut().item_spacing.x = space.sm;

               for (icon, name) in [
                  (Lucide::Settings, "Settings"),
                  (Lucide::SlidersHorizontal, "Advanced"),
                  (Lucide::UserPlus, "Add contact"),
                  (Lucide::Menu, "Menu"),
               ] {
                  let image = icon.size(18.0).color(colors.text).image();
                  let button = IconButton::new(image).tooltip(name);
                  if ui.add(button).clicked() {
                     self.last = name;
                  }
               }
            });

            ui.add_space(space.lg);
            ui.label(RichText::new("States").size(normal).color(colors.text_muted));
            ui.horizontal(|ui| {
               ui.spacing_mut().item_spacing.x = space.sm;

               let image = Lucide::Settings.size(18.0).color(colors.text).image();
               if ui.add(IconButton::new(image).tooltip("Selected").selected(true)).clicked() {
                  self.last = "selected";
               }

               let image = Lucide::Menu.size(18.0).color(colors.text).image();
               if ui.add(IconButton::new(image).tooltip("Disabled").enabled(false)).clicked() {
                  self.last = "disabled";
               }

               let image = Lucide::Settings.size(22.0).color(colors.accent).image();
               if ui.add(IconButton::new(image).size(44.0).tooltip("Large accent")).clicked() {
                  self.last = "large accent";
               }

               let image = Lucide::Menu.size(18.0).color(colors.text).image();
               if ui.add(IconButton::new(image).tooltip("Pin").selected(self.pinned)).clicked() {
                  self.pinned = !self.pinned;
                  self.last = "pin toggled";
               }
            });

            ui.add_space(space.md);
            ui.label(
               RichText::new(format!("Last: {}", self.last))
                  .size(normal)
                  .color(colors.text_muted),
            );
         });
   }
}

impl eframe::App for IconApp {
   fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
      let space = self.theme.spacing;
      let bg = self.theme.colors.bg;

      self.header(ui);
      self.theme.clone().install(ui.ctx());

      CentralPanel::default()
         .frame(Frame::new().fill(bg).inner_margin(space.margin(space.lg)))
         .show(ui, |ui| {
            ScrollArea::vertical().show(ui, |ui| self.card(ui));
         });
   }
}
