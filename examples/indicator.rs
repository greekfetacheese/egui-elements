//! Indicator proof: the three states on the size ladder and the badge's pulse
//! loop behind them, with the loop's knobs live.
//!
//! Run with: `cargo run --example indicator`

use eframe::egui::*;
use egui_elements::utils;
use egui_elements::widgets::{BadgePulse, Frame as Card, Indicator, IndicatorState};
use egui_elements::{Theme, ThemeKind};

fn main() -> eframe::Result {
   let options = eframe::NativeOptions {
      viewport: ViewportBuilder::default()
         .with_title("egui-elements indicator")
         .with_inner_size([820.0, 620.0])
         .with_resizable(true),
      ..Default::default()
   };

   eframe::run_native(
      "egui-elements indicator",
      options,
      Box::new(|cc| Ok(Box::new(IndicatorApp::new(cc)))),
   )
}

struct IndicatorApp {
   theme: Theme,
   pulse_period: f32,
   pulse_scale: f32,
   pulse_alpha: f32,
   reduce_motion: bool,
}

impl IndicatorApp {
   fn new(cc: &eframe::CreationContext<'_>) -> Self {
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&cc.egui_ctx);

      Self {
         theme,
         pulse_period: 1.5,
         pulse_scale: 1.9,
         pulse_alpha: 0.5,
         reduce_motion: false,
      }
   }

   fn pulse(&self) -> Option<BadgePulse> {
      if self.reduce_motion {
         None
      } else {
         Some(
            BadgePulse::new()
               .period(self.pulse_period)
               .scale(self.pulse_scale)
               .alpha(self.pulse_alpha),
         )
      }
   }

   fn header(&mut self, ui: &mut Ui) {
      ui.horizontal(|ui| {
         ui.spacing_mut().button_padding = self.theme.button_padding;

         ui.label(
            RichText::new("egui-elements indicator")
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

      self.theme.clone().install(ui.ctx());
   }

   /// The size ladder: every size is a ratio of `.size()`, so the glow, the ring
   /// stroke and the bar scale with the light.
   fn states_card(&mut self, ui: &mut Ui) {
      let space = self.theme.spacing;
      let colors = self.theme.colors;
      let normal = self.theme.typography.normal;

      Card::new()
         .fill(colors.widget_bg)
         .corner_radius(CornerRadius::same(12))
         .inner_margin(space.margin(space.xl))
         .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = space.sm;

            ui.label(RichText::new("States").size(normal).color(colors.text).strong());

            ui.add_space(space.md);

            for state in [
               IndicatorState::On,
               IndicatorState::Connecting,
               IndicatorState::Off,
            ] {
               ui.horizontal(|ui| {
                  ui.spacing_mut().item_spacing.x = space.md;

                  ui.label(
                     RichText::new(match state {
                        IndicatorState::On => "On",
                        IndicatorState::Connecting => "Connecting",
                        IndicatorState::Off => "Off",
                     })
                     .size(normal)
                     .color(colors.text_muted),
                  );

                  for size in [10.0, 14.0, 20.0, 28.0] {
                     ui.add(Indicator::new(state).size(size));
                  }
               });
            }

            ui.horizontal(|ui| {
               ui.spacing_mut().item_spacing.x = space.md;

               ui.label(RichText::new("Ring").size(normal).color(colors.text_muted));

               for size in [14.0, 20.0, 28.0] {
                  ui.add(
                     Indicator::new(IndicatorState::On)
                        .size(size)
                        .ring(Stroke::new(2.0, colors.bg)),
                  );
               }
            });

            ui.add_space(space.sm);

            ui.label(
               RichText::new(
                  "A light reads at 10 pt; the ring's stroke and the Off bar scale with it.",
               )
               .size(self.theme.typography.small)
               .color(colors.text_muted),
            );
         });
   }

   /// The badge's loop, reused verbatim: a copy of the light grows and fades behind it.
   fn pulse_card(&mut self, ui: &mut Ui) {
      let space = self.theme.spacing;
      let colors = self.theme.colors;
      let normal = self.theme.typography.normal;
      let pulse = self.pulse();

      Card::new()
         .fill(colors.widget_bg)
         .corner_radius(CornerRadius::same(12))
         .inner_margin(space.margin(space.xl))
         .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = space.sm;

            ui.label(RichText::new("Pulse (BadgePulse)").size(normal).color(colors.text).strong());

            ui.add_space(space.md);

            ui.horizontal(|ui| {
               ui.spacing_mut().item_spacing.x = space.xl;

               for state in [
                  IndicatorState::On,
                  IndicatorState::Connecting,
                  IndicatorState::Off,
               ] {
                  let mut indicator = Indicator::new(state).size(16.0);
                  if let Some(pulse) = pulse {
                     indicator = indicator.pulse(pulse);
                  }
                  ui.add(indicator);
               }

               ui.add_space(space.md);

               // Three lights, three phases: the phase comes from each widget's id, so
               // siblings do not beat in lockstep.
               for _ in 0..3 {
                  let mut indicator = Indicator::new(IndicatorState::On).size(12.0);
                  if let Some(pulse) = pulse {
                     indicator = indicator.pulse(pulse);
                  }
                  ui.add(indicator);
               }
            });

            ui.add_space(space.lg);

            ui.add(
               Slider::new(&mut self.pulse_period, 0.6..=3.0)
                  .text("period (s)")
                  .fixed_decimals(2),
            );
            ui.add(
               Slider::new(&mut self.pulse_scale, 1.2..=3.0)
                  .text("scale (x d)")
                  .fixed_decimals(2),
            );
            ui.add(Slider::new(&mut self.pulse_alpha, 0.1..=1.0).text("alpha").fixed_decimals(2));
            ui.add(Checkbox::new(
               &mut self.reduce_motion,
               "Reduce motion",
            ));
         });
   }
}

impl eframe::App for IndicatorApp {
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
            ui.spacing_mut().item_spacing = vec2(space.lg, space.lg);
            ui.spacing_mut().button_padding = self.theme.button_padding;

            ScrollArea::vertical().show(ui, |ui| {
               ui.spacing_mut().item_spacing = vec2(space.lg, space.lg);
               self.states_card(ui);
               self.pulse_card(ui);
            });
         });
   }
}
