//! Orb proof: the size ladder, the staggered label and the colour override on a
//! filled surface.
//!
//! Run with: `cargo run --example orb`

use eframe::egui::*;
use egui_elements::utils;
use egui_elements::widgets::{Frame as Card, Orb};
use egui_elements::{Theme, ThemeKind};

fn main() -> eframe::Result {
   let options = eframe::NativeOptions {
      viewport: ViewportBuilder::default()
         .with_title("egui-elements orb")
         .with_inner_size([820.0, 620.0])
         .with_resizable(true),
      ..Default::default()
   };

   eframe::run_native(
      "egui-elements orb",
      options,
      Box::new(|cc| Ok(Box::new(OrbApp::new(cc)))),
   )
}

struct OrbApp {
   theme: Theme,
   size: f32,
   period: f32,
   letter_delay: f32,
   text: String,
}

impl OrbApp {
   fn new(cc: &eframe::CreationContext<'_>) -> Self {
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&cc.egui_ctx);

      Self {
         theme,
         size: 180.0,
         period: 2.0,
         letter_delay: 0.1,
         text: "Generating".to_owned(),
      }
   }

   /// The knob-driven orb: the theme accent, in the size the sliders ask for.
   fn orbed(&self) -> Orb {
      Orb::new().size(self.size).period(self.period).letter_delay(self.letter_delay)
   }

   fn header(&mut self, ui: &mut Ui) {
      ui.horizontal(|ui| {
         ui.spacing_mut().button_padding = self.theme.button_padding;

         ui.label(
            RichText::new("egui-elements orb")
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
      ui.add(Slider::new(&mut self.size, 32.0..=220.0).text("size"));
      ui.add(Slider::new(&mut self.period, 0.4..=4.0).text("period"));
      ui.add(Slider::new(&mut self.letter_delay, 0.0..=0.5).text("letter delay"));
      ui.text_edit_singleline(&mut self.text);
   }
}

impl eframe::App for OrbApp {
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
               for size in [40.0, 64.0, 96.0, 140.0] {
                  ui.add(Orb::new().size(size));
               }
            });

            ui.add_space(space.sm);
            self.section(ui, "Knobs");
            self.knobs(ui);

            ui.add_space(space.sm);
            ui.horizontal(|ui| {
               ui.spacing_mut().item_spacing.x = space.lg;

               ui.add(self.orbed().text(self.text.clone()));

               let accent = self.theme.colors.accent;
               let ink = self.theme.colors.ink_on(accent);
               Card::new()
                  .fill(accent)
                  .corner_radius(CornerRadius::same(12))
                  .inner_margin(Margin::same(24))
                  .show(ui, |ui| {
                     ui.allocate_ui_with_layout(
                        vec2(150.0, 40.0),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                           ui.spacing_mut().item_spacing.x = space.sm;
                           // a neutral ball, so the orb reads on a saturated fill
                           ui.add(Orb::new().size(28.0).color(ink));
                           ui.label(
                              RichText::new("Saving").size(self.theme.typography.normal).color(ink),
                           );
                        },
                     );
                  });

               // No label and no size: the ball alone, at the default square.
               ui.add(Orb::new());
            });
         });
   }
}
