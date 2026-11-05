//! Gradient directions: one widget set painted four ways.
//!
//! Each card builds its visuals from the theme's stored solids and re-aims the bevel at one
//! [`GradientDir`] — the same `Fill` a host hands to `.visuals(...)`. The theme's own
//! [`ThemeVariant`] always paints TopDown, so this example is the side-by-side of the alternatives.
//!
//! Run with: `cargo run --example gradient`

use eframe::egui::*;
use egui_elements::gradient::{Fill, Gradient, GradientDir};
use egui_elements::utils;
use egui_elements::visuals::{ButtonVisuals, TextEditVisuals};
use egui_elements::widgets::{Button, Frame as Card, Label, TextEdit};
use egui_elements::{Theme, ThemeKind};

/// The four axes, in the order the rack shows them.
const DIRECTIONS: [GradientDir; 4] = [
   GradientDir::TopDown,
   GradientDir::BottomUp,
   GradientDir::LeftToRight,
   GradientDir::RightToLeft,
];

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

         let note =
            "The variant pins TopDown; these cards pass an explicit Fill::Gradient instead.";
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
   }

   fn card(&mut self, ui: &mut Ui, dir: GradientDir, index: usize) {
      let colors = self.theme.colors;
      let space = self.theme.spacing;
      let normal = self.theme.typography.normal;
      let heading = self.theme.typography.large;
      let strength = self.strength;

      let button = self.button_visuals(dir, strength);
      let label = self.label_visuals(dir, strength);
      let edit = self.text_edit_visuals(dir, strength);
      let ramp =
         Fill::Gradient(Gradient::bevel(colors.widget_bg, colors.text, strength).with_dir(dir));
      let text = &mut self.texts[index];

      Card::new()
         .fill(colors.widget_bg)
         .corner_radius(CornerRadius::same(12))
         .inner_margin(space.margin(space.lg))
         .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = space.sm;

            ui.horizontal(|ui| {
               ui.label(RichText::new(dir_name(dir)).size(heading).color(colors.text).strong());

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

   fn button_visuals(&self, dir: GradientDir, strength: f32) -> ButtonVisuals {
      let visuals = self.theme.visuals.button_visuals;
      let ink = visuals.text;

      ButtonVisuals {
         bg: bevel(visuals.bg, ink, dir, strength),
         bg_hover: bevel(visuals.bg_hover, ink, dir, strength),
         bg_click: bevel(visuals.bg_click, ink, dir, strength),
         bg_selected: bevel(visuals.bg_selected, ink, dir, strength),
         ..visuals
      }
   }

   fn label_visuals(&self, dir: GradientDir, strength: f32) -> ButtonVisuals {
      let visuals = self.theme.visuals.label_visuals;
      let ink = visuals.text;

      ButtonVisuals {
         bg: bevel(visuals.bg, ink, dir, strength),
         bg_hover: bevel(visuals.bg_hover, ink, dir, strength),
         bg_click: bevel(visuals.bg_click, ink, dir, strength),
         bg_selected: bevel(visuals.bg_selected, ink, dir, strength),
         ..visuals
      }
   }

   fn text_edit_visuals(&self, dir: GradientDir, strength: f32) -> TextEditVisuals {
      let visuals = self.theme.visuals.text_edit_visuals;
      let ink = visuals.text;

      TextEditVisuals {
         bg: bevel(visuals.bg, ink, dir, strength),
         ..visuals
      }
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

/// A themed fill as a bevel along `dir` — the derivation the theme variant uses, re-aimed.
fn bevel(fill: Fill, ink: Color32, dir: GradientDir, strength: f32) -> Fill {
   match fill.graded(ink, strength) {
      Fill::Gradient(gradient) => Fill::Gradient(gradient.with_dir(dir)),
      solid => solid,
   }
}

fn dir_name(dir: GradientDir) -> &'static str {
   match dir {
      GradientDir::TopDown => "TopDown",
      GradientDir::BottomUp => "BottomUp",
      GradientDir::LeftToRight => "LeftToRight",
      GradientDir::RightToLeft => "RightToLeft",
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
