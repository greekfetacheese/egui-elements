//! FPS meter proof: the themed window and the chrome-free body over a
//! workload whose cost you can dial in.
//!
//! Run with: `cargo run --example fps_meter`

use std::time::{Duration, Instant};

use eframe::egui::*;
use egui_elements::utils;
use egui_elements::widgets::{Checkbox, FpsMeter, Frame as Card, Slider};
use egui_elements::{Theme, ThemeKind};

fn main() -> eframe::Result {
   let options = eframe::NativeOptions {
      viewport: ViewportBuilder::default()
         .with_title("egui-elements FPS meter")
         .with_inner_size([760.0, 520.0])
         .with_resizable(true),
      ..Default::default()
   };

   eframe::run_native(
      "egui-elements FPS meter",
      options,
      Box::new(|cc| Ok(Box::new(MeterApp::new(cc)))),
   )
}

struct MeterApp {
   theme: Theme,
   meter: FpsMeter,
   work_ms: f32,
}

impl MeterApp {
   fn new(cc: &eframe::CreationContext<'_>) -> Self {
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&cc.egui_ctx);

      Self {
         theme,
         meter: FpsMeter::new(),
         work_ms: 3.0,
      }
   }

   fn header(&mut self, ui: &mut Ui) {
      ui.horizontal(|ui| {
         ui.spacing_mut().button_padding = self.theme.button_padding;

         ui.label(
            RichText::new("egui-elements FPS meter")
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

   /// Spend `work_ms` on the CPU so the meter has something to report. A real
   /// app measures its own update work with the same `Instant`.
   fn burn(&self) {
      let budget = Duration::from_secs_f32(self.work_ms / 1000.0);
      if budget.is_zero() {
         return;
      }

      let start = Instant::now();
      let mut state = 0x2545_f491_4f6c_dd1du64;
      while start.elapsed() < budget {
         state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
      }
      std::hint::black_box(state);
   }

   fn controls_card(&mut self, ui: &mut Ui) {
      let space = self.theme.spacing;
      let colors = self.theme.colors;
      let typography = self.theme.typography.clone();

      Card::new()
         .fill(colors.widget_bg)
         .corner_radius(CornerRadius::same(12))
         .inner_margin(space.margin(space.lg))
         .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = space.sm;

            ui.label(RichText::new("Workload").size(typography.normal).color(colors.text).strong());
            ui.label(
               RichText::new(
                  "The meter only reports what the host measures: raise the busy loop and \
                   the CPU time climbs while FPS falls.",
               )
               .size(typography.small)
               .color(colors.text_muted),
            );

            ui.add(
               Slider::new(&mut self.work_ms, 0.0..=12.0)
                  .text("busy work (ms)")
                  .fixed_decimals(1),
            );
            let mut open = self.meter.is_open();
            if ui.add(Checkbox::new(&mut open, "Floating window")).changed() {
               self.meter.set_open(open);
            }
         });
   }

   fn body_card(&mut self, ui: &mut Ui) {
      let space = self.theme.spacing;
      let colors = self.theme.colors;
      let typography = self.theme.typography.clone();

      Card::new()
         .fill(colors.widget_bg)
         .corner_radius(CornerRadius::same(12))
         .inner_margin(space.margin(space.lg))
         .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = space.sm;

            ui.label(
               RichText::new("body_ui — chrome-free")
                  .size(typography.normal)
                  .color(colors.text)
                  .strong(),
            );
            ui.label(
               RichText::new("The same body, hosted in a card instead of the window.")
                  .size(typography.small)
                  .color(colors.text_muted),
            );

            ui.add_space(space.xs);
            self.meter.body_ui(ui);
         });
   }
}

impl eframe::App for MeterApp {
   fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
      let space = self.theme.spacing;
      let bg = self.theme.colors.bg;

      // The host clock: `frame` diffs consecutive calls for FPS, and its
      // argument is the CPU time this frame spent.
      let start = Instant::now();
      self.burn();

      self.theme.clone().install(ui.ctx());

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
            ui.spacing_mut().item_spacing = vec2(space.lg, space.lg);
            ui.spacing_mut().button_padding = self.theme.button_padding;

            ScrollArea::vertical().show(ui, |ui| {
               ui.spacing_mut().item_spacing = vec2(space.lg, space.lg);

               self.controls_card(ui);

               // One call per frame, at the end of the measured work: FPS plus
               // this frame's CPU time, then paint the current numbers.
               self.meter.frame(start.elapsed());
               self.body_card(ui);
            });
         });

      self.meter.show_window(ui.ctx());
   }
}
