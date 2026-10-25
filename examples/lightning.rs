//! Interactive lightning demo for `egui-elements`: the [`Lightning`] effect
//! driven by the public [`LightningSettings`] window.
//!
//! Run with: `cargo run --example lightning`

use eframe::egui::*;
use egui_elements::particles::{Lightning, LightningSettings};
use egui_elements::theme::{Theme, ThemeKind};
use egui_elements::utils;

fn main() -> eframe::Result {
   let options = eframe::NativeOptions {
      viewport: ViewportBuilder::default()
         .with_title("egui-elements lightning")
         .with_inner_size([900.0, 650.0])
         .with_resizable(true),
      ..Default::default()
   };

   eframe::run_native(
      "egui-elements lightning",
      options,
      Box::new(|cc| Ok(Box::new(LightningApp::new(cc)))),
   )
}

struct LightningApp {
   theme: Theme,
   settings: LightningSettings,
   draw_in_window: bool,
}

impl LightningApp {
   fn new(cc: &eframe::CreationContext<'_>) -> Self {
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&cc.egui_ctx);

      Self {
         theme,
         settings: LightningSettings {
            open: true,
            ..Default::default()
         },
         draw_in_window: false,
      }
   }

   fn header(&mut self, ui: &mut Ui) {
      ui.horizontal(|ui| {
         ui.spacing_mut().button_padding = self.theme.button_padding;

         ui.label(
            RichText::new("egui-elements lightning")
               .size(self.theme.typography.very_large)
               .color(self.theme.colors.accent)
               .strong(),
         );

         ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if let Some(new_theme) = utils::theme_switcher(&self.theme, ui) {
               self.theme = new_theme;
            }
         });
      });
   }
}

impl eframe::App for LightningApp {
   fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
      let space = self.theme.spacing;

      Panel::top("header")
         .frame(
            Frame::new()
               .fill(self.theme.colors.bg)
               .inner_margin(space.margin_xy(space.lg, space.md))
               .stroke(Stroke::new(1.0, self.theme.colors.border)),
         )
         .show(ui, |ui| self.header(ui));

      let bg = self.theme.colors.bg;
      CentralPanel::default()
         .frame(Frame::new().fill(bg).inner_margin(space.margin(space.lg)))
         .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = space.md;

            ui.label(
               RichText::new("Episodic lightning for egui.")
                  .size(self.theme.typography.large)
                  .color(self.theme.colors.text),
            );
            ui.label(
               RichText::new("A jagged bolt with a full-area flash, on a layer above the app.")
                  .size(self.theme.typography.normal)
                  .color(self.theme.colors.text),
            );

            ui.add_space(space.md);

            ui.label(
               RichText::new(
                  "Strike it yourself, or set an ambient interval in the settings window and let it fire on its own.",
               )
               .size(self.theme.typography.normal)
               .color(self.theme.colors.text_muted),
            );

            ui.add_space(space.md);
            if ui.button("Strike now").clicked() {
               Lightning::from_settings(LightningSettings::STRIKE_ID, &self.settings).strike(ui.ctx());
            }
            ui.checkbox(&mut self.draw_in_window, "Draw in this window only");
         });

      let win_rect = self.settings.show_window(ui.ctx()).map(|r| r.rect);

      if self.settings.active {
         let mut lightning = Lightning::from_settings(LightningSettings::STRIKE_ID, &self.settings);
         if self.draw_in_window {
            if let Some(rect) = win_rect {
               lightning = lightning.area(rect);
            }
         }
         lightning.show(ui); // `Ui` derefs to `Context` in egui 0.36 — keep the call style
      }
   }
}
