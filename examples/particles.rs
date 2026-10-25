//! Interactive snow demo for `egui-elements`: the [`Snow`] effect driven by the
//! public [`SnowSettings`] window.
//!
//! Run with: `cargo run --example particles`

use eframe::egui::*;
use egui_elements::particles::{Snow, SnowSettings};
use egui_elements::theme::{Theme, ThemeKind};
use egui_elements::utils;

fn main() -> eframe::Result {
   let options = eframe::NativeOptions {
      viewport: ViewportBuilder::default()
         .with_title("egui-elements particles")
         .with_inner_size([900.0, 650.0])
         .with_resizable(true),
      ..Default::default()
   };

   eframe::run_native(
      "egui-elements particles",
      options,
      Box::new(|cc| Ok(Box::new(ParticlesApp::new(cc)))),
   )
}

struct ParticlesApp {
   theme: Theme,
   settings: SnowSettings,
   draw_in_window: bool,
}

impl ParticlesApp {
   fn new(cc: &eframe::CreationContext<'_>) -> Self {
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&cc.egui_ctx);

      Self {
         theme,
         settings: SnowSettings {
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
            RichText::new("egui-elements particles")
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

impl eframe::App for ParticlesApp {
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
               RichText::new("High performance particle system for egui.")
                  .size(self.theme.typography.large)
                  .color(self.theme.colors.text),
            );
            ui.label(
               RichText::new("This application demonstrates the snowfall effect widget.")
                  .size(self.theme.typography.normal)
                  .color(self.theme.colors.text),
            );

            ui.add_space(space.md);

            ui.label(
               RichText::new(
                  "Try moving the Settings window around to see how snow particles interact with layers.",
               )
               .size(self.theme.typography.normal)
               .color(self.theme.colors.text_muted),
            );
            ui.label(
               RichText::new("Content inside the CentralPanel is usually on the Background layer.")
                  .size(self.theme.typography.small)
                  .color(self.theme.colors.text_muted),
            );

            ui.add_space(space.md);
            ui.checkbox(&mut self.draw_in_window, "Draw in this window only");
         });

      let win_rect = self.settings.show_window(ui.ctx()).map(|r| r.rect);

      if self.settings.active {
         let mut snow = Snow::from_settings("particles_demo", &self.settings);
         if self.draw_in_window {
            if let Some(rect) = win_rect {
               snow = snow.area(rect);
            }
         }
         snow.show(ui); // `Ui` derefs to `Context` in egui 0.36 — keep the upstream call style
      }
   }
}
