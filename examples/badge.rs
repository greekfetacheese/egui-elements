//! Badge proof: the reference's straddling badges, both motions, the three hop triggers, the live knobs and a
//! standalone badge with no host.
//!
//! Run with: `cargo run --example badge`

use eframe::egui::*;
use egui_elements::motion::Motion;
use egui_elements::widgets::{
   Badge, BadgeBounce, BadgeBounceTrigger, BadgePulse, Button, Frame as Card,
};
use egui_elements::{Theme, ThemeKind, utils};

fn main() -> eframe::Result {
   let options = eframe::NativeOptions {
      viewport: ViewportBuilder::default()
         .with_title("egui-elements badge")
         .with_inner_size([920.0, 720.0])
         .with_resizable(true),
      ..Default::default()
   };

   eframe::run_native(
      "egui-elements badge",
      options,
      Box::new(|cc| Ok(Box::new(BadgeApp::new(cc)))),
   )
}

struct BadgeApp {
   theme: Theme,
   /// Counts for the reference row and the trigger row; changing one hops its badge.
   counts: [u32; 4],
   /// Count for the `Manual` demo badge.
   manual_count: u32,
   pulse_period: f32,
   pulse_scale: f32,
   hop_amplitude: f32,
   hop_duration: f32,
   reduce_motion: bool,
}

impl BadgeApp {
   fn new(cc: &eframe::CreationContext<'_>) -> Self {
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&cc.egui_ctx);

      Self {
         theme,
         counts: [0, 2, 1, 6],
         manual_count: 1,
         pulse_period: 1.5,
         pulse_scale: 1.9,
         hop_amplitude: 0.5,
         hop_duration: 0.42,
         reduce_motion: false,
      }
   }

   fn pulse(&self) -> Option<BadgePulse> {
      if self.reduce_motion {
         None
      } else {
         Some(BadgePulse::new().period(self.pulse_period).scale(self.pulse_scale))
      }
   }

   fn bounce(&self) -> BadgeBounce {
      let motion = if self.reduce_motion {
         Motion::instant()
      } else {
         Motion::new(self.hop_duration)
      };

      BadgeBounce::new().amplitude(self.hop_amplitude).motion(motion)
   }

   fn header(&mut self, ui: &mut Ui) {
      ui.horizontal(|ui| {
         ui.spacing_mut().button_padding = self.theme.button_padding;

         ui.label(
            RichText::new("egui-elements badge")
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

   /// The reference capture: four chips, every badge straddling its top-right corner.
   fn reference_card(&mut self, ui: &mut Ui) {
      let space = self.theme.spacing;
      let colors = self.theme.colors;
      let normal = self.theme.typography.normal;
      let pulse = self.pulse();
      let bounce = self.bounce();

      Card::new()
         .fill(colors.widget_bg)
         .corner_radius(CornerRadius::same(12))
         .inner_margin(space.margin(space.xl))
         .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = space.sm;

            ui.label(RichText::new("The reference").size(normal).color(colors.text).strong());
            ui.label(
               RichText::new(
                  "18 pt badges, centred on each chip's top-right corner. The second one pulses.",
               )
               .size(normal)
               .color(colors.text_muted),
            );

            ui.add_space(space.md);

            let fills = [colors.success, colors.error, colors.info, colors.warning];
            let labels = ["Jedi", "Sith", "Chosen", "Episodes"];

            ui.horizontal_wrapped(|ui| {
               ui.spacing_mut().item_spacing.x = space.lg;

               for i in 0..labels.len() {
                  let mut badge = Badge::new(self.counts[i].to_string())
                     .color(fills[i])
                     .min_diameter(18.0)
                     .straddle()
                     .bounce(bounce);

                  if let Some(pulse) = pulse.filter(|_| i == 1) {
                     badge = badge.pulse(pulse);
                  }

                  if ui
                     .add(Button::new(labels[i]).min_size(vec2(96.0, 38.0)).badge(badge))
                     .clicked()
                  {
                     self.counts[i] = self.counts[i].saturating_add(1);
                  }
               }
            });
         });
   }

   /// The three hop triggers, side by side.
   fn trigger_card(&mut self, ui: &mut Ui) {
      let space = self.theme.spacing;
      let colors = self.theme.colors;
      let normal = self.theme.typography.normal;
      let bounce = self.bounce();

      Card::new()
         .fill(colors.widget_bg)
         .corner_radius(CornerRadius::same(12))
         .inner_margin(space.margin(space.xl))
         .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = space.sm;

            ui.label(RichText::new("Triggers").size(normal).color(colors.text).strong());
            ui.label(
               RichText::new("OnChange hops when the count moves, Manual only when the host asks, Loop repeats.")
                  .size(normal)
                  .color(colors.text_muted),
            );

            ui.add_space(space.md);

            ui.horizontal_wrapped(|ui| {
               ui.spacing_mut().item_spacing.x = space.lg;

               // OnChange: the default.
               if ui
                  .add(
                     Button::new("+1 count").min_size(vec2(96.0, 38.0)).badge(
                        Badge::new(self.counts[0].to_string())
                           .color(colors.success)
                           .straddle()
                           .bounce(bounce),
                     ),
                  )
                  .clicked()
               {
                  self.counts[0] = self.counts[0].saturating_add(1);
               }

               // Manual: the badge hops only when `bounce_now` is called.
               let manual = Badge::new(self.manual_count.to_string())
                  .color(colors.error)
                  .straddle()
                  .id("manual-badge")
                  .bounce(bounce.trigger(BadgeBounceTrigger::Manual));

               if ui
                  .add(Button::new("Manual").min_size(vec2(96.0, 38.0)).badge(manual))
                  .clicked()
               {
                  self.manual_count = self.manual_count.saturating_add(1);
               }

               if ui.button("Hop it").clicked() {
                  Badge::bounce_now(ui.ctx(), "manual-badge");
               }

               // Loop: the hop repeats on a period.
               let looping = Badge::new("7")
                  .color(colors.info)
                  .straddle()
                  .bounce(bounce.trigger(BadgeBounceTrigger::Loop).period(2.2));

               ui.add(Button::new("Loop").min_size(vec2(96.0, 38.0)).badge(looping));
            });
         });
   }

   /// The measured ratios, live.
   fn knobs_card(&mut self, ui: &mut Ui) {
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

            ui.label(RichText::new("Knobs").size(normal).color(colors.text).strong());

            ui.add_space(space.md);

            ui.add(
               Slider::new(&mut self.pulse_period, 0.6..=2.6)
                  .text("pulse period (s)")
                  .fixed_decimals(2),
            );
            ui.add(
               Slider::new(&mut self.pulse_scale, 1.2..=2.4)
                  .text("pulse scale (x d)")
                  .fixed_decimals(2),
            );
            ui.add(
               Slider::new(&mut self.hop_amplitude, 0.1..=0.7)
                  .text("hop amplitude (x d)")
                  .fixed_decimals(2),
            );
            ui.add(
               Slider::new(&mut self.hop_duration, 0.2..=1.0)
                  .text("hop duration (s)")
                  .fixed_decimals(2),
            );
            ui.checkbox(&mut self.reduce_motion, "Reduce motion");

            ui.add_space(space.md);

            let mut sample = Badge::new("12").straddle().bounce(self.bounce());
            if let Some(pulse) = pulse {
               sample = sample.pulse(pulse);
            }

            ui.horizontal(|ui| {
               ui.add(Button::new("Live sample").min_size(vec2(140.0, 38.0)).badge(sample));
            });
         });
   }

   /// Badges with no host widget at all.
   fn standalone_card(&mut self, ui: &mut Ui) {
      let space = self.theme.spacing;
      let colors = self.theme.colors;
      let normal = self.theme.typography.normal;

      Card::new()
         .fill(colors.widget_bg)
         .corner_radius(CornerRadius::same(12))
         .inner_margin(space.margin(space.xl))
         .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = space.sm;

            ui.label(RichText::new("Standalone").size(normal).color(colors.text).strong());
            ui.label(
               RichText::new("`ui.add(Badge::new(\"3\"))` allocates its own square; the halo paints outside it.")
                  .size(normal)
                  .color(colors.text_muted),
            );

            ui.add_space(space.md);

            ui.horizontal(|ui| {
               ui.spacing_mut().item_spacing.x = space.md;
               ui.add(Badge::dot().color(colors.error));
               ui.add(Badge::new("12").color(colors.info));

               let mut pulsing = Badge::new("99+").color(colors.success);
               if let Some(pulse) = self.pulse() {
                  pulsing = pulsing.pulse(pulse);
               }
               ui.add(pulsing);
            });
         });
   }
}

impl eframe::App for BadgeApp {
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
            ScrollArea::vertical().show(ui, |ui| {
               ui.spacing_mut().item_spacing.y = space.lg;
               self.reference_card(ui);
               self.trigger_card(ui);
               self.knobs_card(ui);
               self.standalone_card(ui);
            });
         });
   }
}
