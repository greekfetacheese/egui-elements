//! Interactive hold-to-confirm demo for `egui-elements`: [`Button::hold_settings`]
//! with a configurable progress fill and a [`HoldSettings`] completion flourish.
//!
//! Run with: `cargo run --example hold_to_confirm`

use eframe::egui::*;
use egui_elements::theme::{Theme, ThemeKind};
use egui_elements::utils;
use egui_elements::widgets::{Button, HoldSettings};

/// Uniform button size, so the two hold buttons and the plain one line up.
const BUTTON_SIZE: Vec2 = Vec2::new(250.0, 45.0);

fn main() -> eframe::Result {
   let options = eframe::NativeOptions {
      viewport: ViewportBuilder::default()
         .with_title("egui-elements hold-to-confirm")
         .with_inner_size([900.0, 650.0])
         .with_resizable(true),
      ..Default::default()
   };

   eframe::run_native(
      "egui-elements hold-to-confirm",
      options,
      Box::new(|cc| Ok(Box::new(HoldApp::new(cc)))),
   )
}

struct HoldApp {
   theme: Theme,
   settings: HoldSettings,
   confirmed: u32,
   normal_clicks: u32,
}

impl HoldApp {
   fn new(cc: &eframe::CreationContext<'_>) -> Self {
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&cc.egui_ctx);

      Self {
         theme,
         settings: HoldSettings {
            open: true,
            ..Default::default()
         },
         confirmed: 0,
         normal_clicks: 0,
      }
   }

   fn header(&mut self, ui: &mut Ui) {
      ui.horizontal(|ui| {
         ui.spacing_mut().button_padding = self.theme.button_padding;

         ui.label(
            RichText::new("egui-elements hold-to-confirm")
               .size(self.theme.typography.very_large)
               .color(self.theme.colors.accent)
               .strong(),
         );

         ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if let Some(new_theme) = utils::theme_switcher(&self.theme, ui) {
               self.theme = new_theme;
            }
            ui.checkbox(&mut self.settings.open, "Settings");
         });
      });
   }
}

impl eframe::App for HoldApp {
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
               RichText::new("Hold to confirm")
                  .size(self.theme.typography.large)
                  .color(self.theme.colors.text),
            );
            ui.label(
               RichText::new(
                  "clicked() only reports once the sweep fills; a plain click never confirms. \
                   Both buttons share the settings window.",
               )
               .size(self.theme.typography.normal)
               .color(self.theme.colors.text_muted),
            );

            ui.add_space(space.md);

            ui.horizontal(|ui| {
               ui.spacing_mut().item_spacing.x = space.md;

               if ui
                  .add(
                     Button::new("Hold to delete")
                        .hold_settings(&self.settings)
                        .min_size(BUTTON_SIZE),
                  )
                  .clicked()
               {
                  self.confirmed += 1;
               }

               if ui
                  .add(
                     Button::new("Hold to strike")
                        .hold_settings(&self.settings)
                        .min_size(BUTTON_SIZE),
                  )
                  .clicked()
               {
                  self.confirmed += 1;
               }
            });

            ui.add_space(space.md);
            ui.label(
               RichText::new(format!("Confirmed actions: {}", self.confirmed))
                  .size(self.theme.typography.normal)
                  .color(self.theme.colors.success),
            );

            ui.add_space(space.xl);
            ui.label(
               RichText::new("An ordinary button, for comparison")
                  .size(self.theme.typography.normal)
                  .color(self.theme.colors.text_muted),
            );
            if ui.add(Button::new("Normal button").min_size(BUTTON_SIZE)).clicked() {
               self.normal_clicks += 1;
            }
            ui.label(
               RichText::new(format!("Normal clicks: {}", self.normal_clicks))
                  .size(self.theme.typography.normal)
                  .color(self.theme.colors.text),
            );
         });

      self.settings.show_window(ui.ctx());
   }
}
