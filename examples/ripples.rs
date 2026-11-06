//! Ripple proof: the size ladder, the live knobs and the flat-colour override on a
//! filled surface.
//!
//! Run with: `cargo run --example ripples`

use eframe::egui::*;
use egui_elements::utils;
use egui_elements::widgets::{Frame as Card, Ripple};
use egui_elements::{Theme, ThemeKind};

fn main() -> eframe::Result {
   let options = eframe::NativeOptions {
      viewport: ViewportBuilder::default()
         .with_title("egui-elements ripples")
         .with_inner_size([820.0, 620.0])
         .with_resizable(true),
      ..Default::default()
   };

   eframe::run_native(
      "egui-elements ripples",
      options,
      Box::new(|cc| Ok(Box::new(RippleApp::new(cc)))),
   )
}

struct RippleApp {
   theme: Theme,
   size: f32,
   period: f32,
   delay: f32,
}

impl RippleApp {
   fn new(cc: &eframe::CreationContext<'_>) -> Self {
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&cc.egui_ctx);

      Self {
         theme,
         size: 104.0,
         period: 1.5,
         delay: 0.1,
      }
   }

   /// The knob-driven loader: the theme ramp, in the size the sliders ask for.
   fn rippling(&self) -> Ripple {
      Ripple::new().size(self.size).period(self.period).delay(self.delay)
   }

   fn header(&mut self, ui: &mut Ui) {
      ui.horizontal(|ui| {
         ui.spacing_mut().button_padding = self.theme.button_padding;

         ui.label(
            RichText::new("egui-elements ripples")
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
      ui.add(Slider::new(&mut self.size, 32.0..=200.0).text("size"));
      ui.add(Slider::new(&mut self.period, 0.4..=4.0).text("period"));
      ui.add(Slider::new(&mut self.delay, 0.0..=0.5).text("cell delay"));
   }
}

impl eframe::App for RippleApp {
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
               for size in [32.0, 52.0, 80.0, 120.0] {
                  ui.add(Ripple::new().size(size));
               }
            });

            ui.add_space(space.sm);
            self.section(ui, "Knobs");
            self.knobs(ui);

            ui.add_space(space.sm);
            ui.horizontal(|ui| {
               ui.spacing_mut().item_spacing.x = space.lg;

               // The knob size is capped so the row still fits the panel.
               let knob = self.size.min(160.0);
               let (rect, _) = ui.allocate_exact_size(vec2(knob, knob), Sense::hover());
               self.rippling().size(knob).paint_at(ui, rect);

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
                           // one flat colour, so the loader reads on a saturated fill
                           let (rect, _) = ui.allocate_exact_size(vec2(28.0, 28.0), Sense::hover());
                           Ripple::new().size(28.0).color(ink).paint_at(ui, rect);
                           ui.label(
                              RichText::new("Saving").size(self.theme.typography.normal).color(ink),
                           );
                        },
                     );
                  });

               // No delay: all nine cells light together, a single square pulse.
               let (rect, _) = ui.allocate_exact_size(vec2(160.0, 160.0), Sense::hover());
               Ripple::new().size(120.0).delay(0.0).color(self.theme.colors.info).paint_at(
                  ui,
                  Rect::from_center_size(rect.center(), vec2(120.0, 120.0)),
               );
            });
         });
   }
}
