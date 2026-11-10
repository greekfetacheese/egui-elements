//! Gradient directions: one widget set painted four ways.
//!
//! Each card clones the theme, sets [`Theme::gradient_dir`] and asks for its visuals — the whole
//! configuration a host needs. Comparing them side by side is the point of the example; the demo's
//! Fill section is the same field with one direction at a time.
//!
//! Run with: `cargo run --example gradient`

use eframe::egui::*;
use egui_elements::gradient::{Fill, Gradient, GradientDir};
use egui_elements::utils;
use egui_elements::widgets::{Button, Frame as Card, Label, TextEdit};
use egui_elements::{Theme, ThemeKind, ThemeVariant};

/// The four axes, in the order the rack shows them.
const DIRECTIONS: [GradientDir; 4] = GradientDir::ALL;

fn main() -> eframe::Result {
   let options = eframe::NativeOptions {
      viewport: ViewportBuilder::default()
         .with_title("egui-elements gradient directions")
         .with_inner_size([980.0, 840.0])
         .with_resizable(true),
      ..Default::default()
   };

   eframe::run_native(
      "egui-elements gradient directions",
      options,
      Box::new(|cc| Ok(Box::new(GradientApp::new(cc)))),
   )
}

struct GradientApp {
   theme: Theme,
   /// Share of the way to white / black each ramp may travel, before contrast clamps it.
   strength: f32,
   /// One field per card, so typing in one does not write into another.
   texts: [String; 4],
}

impl GradientApp {
   fn new(cc: &eframe::CreationContext<'_>) -> Self {
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&cc.egui_ctx);

      Self {
         theme,
         strength: 0.1,
         texts: std::array::from_fn(|_| String::new()),
      }
   }

   fn header(&mut self, ui: &mut Ui) {
      let colors = self.theme.colors;
      let space = self.theme.spacing;
      let normal = self.theme.typography.normal;

      ui.horizontal(|ui| {
         ui.spacing_mut().button_padding = self.theme.button_padding;

         ui.label(
            RichText::new("Gradient directions")
               .size(self.theme.typography.very_large)
               .color(colors.accent)
               .strong(),
         );

         ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if let Some(new_theme) = utils::theme_switcher(&self.theme, ui) {
               self.theme = new_theme;
            }
         });
      });

      ui.add_space(space.xs);

      ui.horizontal(|ui| {
         ui.label(RichText::new("Bevel depth").size(normal).color(colors.text_muted));
         ui.add(Slider::new(&mut self.strength, 0.0..=1.0).max_decimals(2));

         let note = "Each card sets Theme::gradient_dir and asks the theme for its visuals.";
         ui.label(RichText::new(note).size(normal).color(colors.text_muted));
      });
   }

   fn rack(&mut self, ui: &mut Ui) {
      let space = self.theme.spacing;

      ui.add_space(space.sm);

      ui.columns(2, |columns| {
         for (index, dir) in DIRECTIONS.into_iter().enumerate() {
            let column = &mut columns[index % 2];

            if index >= 2 {
               column.add_space(space.lg);
            }

            self.card(column, dir, index);
         }
      });

      ui.add_space(space.xl);
      self.converted(ui);
   }

   fn card(&mut self, ui: &mut Ui, dir: GradientDir, index: usize) {
      let colors = self.theme.colors;
      let space = self.theme.spacing;
      let normal = self.theme.typography.normal;
      let heading = self.theme.typography.large;
      let strength = self.strength;

      // One theme, re-aimed. This is the whole configuration: the variant paints the ramps, the
      // direction aims them, the strength sets their depth (and the ink clamp trims it).
      let mut themed = self.theme.clone();
      themed.variant = ThemeVariant::Gradient;
      themed.gradient_strength = strength;
      themed.gradient_dir = dir;

      let button = themed.button_visuals();
      let label = themed.label_visuals();
      let edit = themed.text_edit_visuals();
      let ramp = Gradient::bevel(colors.widget_bg, colors.text, strength).with_dir(dir);
      let ramp = Fill::Gradient(ramp);
      let text = &mut self.texts[index];

      Card::new()
         .fill(colors.widget_bg)
         .corner_radius(CornerRadius::same(12))
         .inner_margin(space.margin(space.lg))
         .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = space.sm;

            ui.horizontal(|ui| {
               ui.label(RichText::new(dir.to_str()).size(heading).color(colors.text).strong());

               ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                  ui.label(RichText::new(axis_note(dir)).size(normal).color(colors.text_muted));
               });
            });

            // The bare ramp, so the axis reads before the chrome does.
            ramp.show_rounded(ui, vec2(ui.available_width(), 26.0), 6);

            ui.add_space(space.xs);

            ui.horizontal(|ui| {
               ui.add(Button::new("Button").visuals(button));
               ui.add(Button::new("Selected").selected(true).visuals(button));
            });

            ui.add(Label::new("Interactive label", None).interactive(true).visuals(label));

            ui.add(
               TextEdit::singleline(text)
                  .hint_text("Gradient field")
                  .visuals(edit)
                  .desired_width(f32::INFINITY),
            );
         });
   }

   /// A card the theme graded, handed out through `Frame::to_egui`. egui paints one colour, so the
   /// ramp lands on its midpoint.
   fn converted(&self, ui: &mut Ui) {
      let colors = self.theme.colors;
      let space = self.theme.spacing;

      let mut themed = self.theme.clone();
      themed.variant = ThemeVariant::Gradient;

      Card::new()
         .visuals(themed.frame1_visuals())
         .corner_radius(CornerRadius::same(12))
         .inner_margin(space.margin(space.lg))
         .to_egui()
         .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
               RichText::new("Frame::to_egui()")
                  .size(self.theme.typography.large)
                  .color(colors.text)
                  .strong(),
            );
            ui.label(
               RichText::new("egui paints one colour, so the ramp lands on its midpoint.")
                  .size(self.theme.typography.normal)
                  .color(colors.text_muted),
            );
         });
   }
}

impl eframe::App for GradientApp {
   fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
      self.theme.clone().install(ui.ctx());

      let space = self.theme.spacing;
      let bg = self.theme.colors.bg;
      let border = self.theme.colors.border;

      Panel::top("header")
         .frame(
            Frame::new()
               .fill(bg)
               .inner_margin(space.margin_xy(space.lg, space.md))
               .stroke(Stroke::new(1.0, border)),
         )
         .show(ui, |ui| self.header(ui));

      CentralPanel::default()
         .frame(Frame::new().fill(bg).inner_margin(space.margin(space.lg)))
         .show(ui, |ui| {
            ScrollArea::vertical().show(ui, |ui| self.rack(ui));
         });
   }
}

fn axis_note(dir: GradientDir) -> &'static str {
   match dir {
      GradientDir::TopDown => "light at the top",
      GradientDir::BottomUp => "light at the bottom",
      GradientDir::LeftToRight => "light at the left",
      GradientDir::RightToLeft => "light at the right",
   }
}
