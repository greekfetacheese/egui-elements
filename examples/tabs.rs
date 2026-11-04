//! Tabs proof: a filled strip, a fitted strip and two independent strips.
//!
//! Run with: `cargo run --example tabs`

use eframe::egui::*;
use egui_elements::widgets::{Frame as Card, Tabs};
use egui_elements::{Theme, ThemeKind, utils};

fn main() -> eframe::Result {
   let options = eframe::NativeOptions {
      viewport: ViewportBuilder::default()
         .with_title("egui-elements tabs")
         .with_inner_size([680.0, 480.0])
         .with_resizable(true),
      ..Default::default()
   };

   eframe::run_native(
      "egui-elements tabs",
      options,
      Box::new(|cc| Ok(Box::new(TabsApp::new(cc)))),
   )
}

struct TabsApp {
   theme: Theme,
   filter: usize,
   range: usize,
   left: usize,
   right: usize,
}

impl TabsApp {
   fn new(cc: &eframe::CreationContext<'_>) -> Self {
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&cc.egui_ctx);

      Self {
         theme,
         filter: 0,
         range: 1,
         left: 0,
         right: 2,
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
               ui.label(RichText::new("Tabs").size(title).color(colors.text).strong());
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
            ui.label(RichText::new("Segmented tabs").size(normal).color(colors.text).strong());
            ui.label(
               RichText::new("The index is written in place and each strip owns its own.")
                  .size(normal)
                  .color(colors.text_muted),
            );
            ui.add_space(space.lg);

            ui.label(RichText::new("Filled to the row").size(normal).color(colors.text_muted));
            Tabs::new(
               "filter",
               &mut self.filter,
               ["All", "Media", "Links", "Files"],
            )
            .show(ui);
            ui.add_space(space.lg);

            ui.label(RichText::new("Fitted to its labels").size(normal).color(colors.text_muted));
            Tabs::new("range", &mut self.range, ["Day", "Week", "Month"])
               .fill_width(false)
               .show(ui);
            ui.add_space(space.lg);

            ui.label(
               RichText::new("Two strips, independent indices")
                  .size(normal)
                  .color(colors.text_muted),
            );
            ui.horizontal(|ui| {
               Tabs::new("left", &mut self.left, ["A", "B"]).fill_width(false).show(ui);
               Tabs::new("right", &mut self.right, ["x", "y", "z"]).fill_width(false).show(ui);
            });

            ui.add_space(space.md);
            ui.label(
               RichText::new(format!(
                  "filter {} · range {} · left {} · right {}",
                  self.filter, self.range, self.left, self.right
               ))
               .size(normal)
               .color(colors.text_muted),
            );
         });
   }
}

impl eframe::App for TabsApp {
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
