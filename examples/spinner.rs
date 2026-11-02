//! Spinner proof: the size ladder, the live knobs and the colour override on a
//! filled surface.
//!
//! Run with: `cargo run --example spinner`

use eframe::egui::*;
use egui_elements::utils;
use egui_elements::widgets::{Frame as Card, Spinner, SpinnerShadow};
use egui_elements::{Theme, ThemeKind};

fn main() -> eframe::Result {
   let options = eframe::NativeOptions {
      viewport: ViewportBuilder::default()
         .with_title("egui-elements spinner")
         .with_inner_size([820.0, 620.0])
         .with_resizable(true),
      ..Default::default()
   };

   eframe::run_native(
      "egui-elements spinner",
      options,
      Box::new(|cc| Ok(Box::new(SpinnerApp::new(cc)))),
   )
}

struct SpinnerApp {
   theme: Theme,
   size: f32,
   comet: f32,
   blur_ratio: f32,
   strength: f32,
   track: f32,
}

impl SpinnerApp {
   fn new(cc: &eframe::CreationContext<'_>) -> Self {
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&cc.egui_ctx);

      Self {
         theme,
         size: 40.0,
         comet: 1.0,
         blur_ratio: 0.22,
         strength: 0.30,
         track: 0.16,
      }
   }

   /// The knob-driven spinner: same colour as the theme accent, every effect
   /// exposed on the sliders.
   fn spinning(&self) -> Spinner {
      Spinner::new()
         .size(self.size)
         .comet(self.comet)
         .track_strength(self.track)
         .shadow(
            SpinnerShadow::new(self.theme.colors.accent)
               .blur_ratio(self.blur_ratio)
               .strength(self.strength),
         )
   }

   fn header(&mut self, ui: &mut Ui) {
      ui.horizontal(|ui| {
         ui.spacing_mut().button_padding = self.theme.button_padding;

         ui.label(
            RichText::new("egui-elements spinner")
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

   fn section(&self, ui: &mut Ui, title: &str) {
      ui.label(
         RichText::new(title)
            .size(self.theme.typography.large)
            .color(self.theme.colors.text),
      );
   }

   fn knobs(&mut self, ui: &mut Ui) {
      ui.spacing_mut().slider_width = 260.0;
      ui.add(Slider::new(&mut self.size, 16.0..=96.0).text("size"));
      ui.add(Slider::new(&mut self.comet, 0.0..=1.0).text("comet"));
      ui.add(Slider::new(&mut self.blur_ratio, 0.0..=0.6).text("glow reach"));
      ui.add(Slider::new(&mut self.strength, 0.0..=1.0).text("glow strength"));
      ui.add(Slider::new(&mut self.track, 0.0..=1.0).text("track strength"));
   }
}

impl eframe::App for SpinnerApp {
   fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
      let space = self.theme.spacing;
      let bg = self.theme.colors.bg;

      Panel::top("header")
         .frame(
            Frame::new()
               .fill(bg)
               .inner_margin(space.margin_xy(space.lg, space.md))
               .stroke(Stroke::new(1.0, self.theme.colors.border)),
         )
         .show(ui, |ui| self.header(ui));

      CentralPanel::default()
         .frame(Frame::new().fill(bg).inner_margin(space.margin(space.lg)))
         .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = space.md;

            self.section(ui, "Sizes");
            ui.horizontal(|ui| {
               ui.spacing_mut().item_spacing.x = space.lg;
               for size in [16.0, 20.0, 28.0, 40.0, 56.0] {
                  // A fixed box per sample: the glow spills past the square, and a
                  // fixed allocation keeps the row's rhythm.
                  let (rect, _) =
                     ui.allocate_exact_size(vec2(size * 1.6, size * 1.6), Sense::hover());
                  Spinner::new().size(size).paint_at(
                     ui,
                     Rect::from_center_size(rect.center(), vec2(size, size)),
                  );
               }
            });

            ui.add_space(space.sm);
            self.section(ui, "Knobs");
            self.knobs(ui);

            ui.add_space(space.sm);
            ui.horizontal(|ui| {
               ui.spacing_mut().item_spacing.x = space.lg;

               let knob = self.size;
               let (rect, _) = ui.allocate_exact_size(vec2(160.0, 160.0), Sense::hover());
               self.spinning().paint_at(
                  ui,
                  Rect::from_center_size(rect.center(), vec2(knob, knob)),
               );

               let accent = self.theme.colors.accent;
               let bg = self.theme.colors.bg;
               Card::new()
                  .fill(accent)
                  .corner_radius(CornerRadius::same(12))
                  .inner_margin(Margin::same(24))
                  .show(ui, |ui| {
                     ui.allocate_ui_with_layout(
                        vec2(140.0, 32.0),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                           ui.spacing_mut().item_spacing.x = space.sm;
                           // the colour override carries the track and the glow with it
                           let (rect, _) = ui.allocate_exact_size(vec2(24.0, 24.0), Sense::hover());
                           Spinner::new().size(24.0).color(bg).paint_at(ui, rect);
                           ui.label(
                              RichText::new("Saving").size(self.theme.typography.normal).color(bg),
                           );
                        },
                     );
                  });

               let (rect, _) = ui.allocate_exact_size(vec2(160.0, 160.0), Sense::hover());
               self.spinning().shadow(SpinnerShadow::NONE).paint_at(
                  ui,
                  Rect::from_center_size(rect.center(), vec2(knob, knob)),
               );
            });
         });
   }
}
