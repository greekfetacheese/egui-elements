//! Slider proof: the reference card (label, references, hint, markers, tooltip), the size ladder,
//! a vertical column, the custom formatter and the value cell aligned beside the rail.
//!
//! Run with: `cargo run --example slider`

use eframe::egui::*;
use egui_elements::motion::Motion;
use egui_elements::widgets::{Frame as Card, Slider, SliderVisuals};
use egui_elements::{Theme, ThemeKind, utils};

fn main() -> eframe::Result {
   let options = eframe::NativeOptions {
      viewport: ViewportBuilder::default()
         .with_title("egui-elements slider")
         .with_inner_size([920.0, 760.0])
         .with_resizable(true),
      ..Default::default()
   };

   eframe::run_native(
      "egui-elements slider",
      options,
      Box::new(|cc| Ok(Box::new(SliderApp::new(cc)))),
   )
}

struct SliderApp {
   theme: Theme,
   users: f32,
   volume: f32,
   quality: f32,
   price: f64,
   speed: f32,
   duration: f32,
   reduce_motion: bool,
   disabled: f32,
   sizes: [f32; 4],
   columns: [f32; 3],
}

impl SliderApp {
   fn new(cc: &eframe::CreationContext<'_>) -> Self {
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&cc.egui_ctx);
      let duration = theme.style().animation_time;

      Self {
         theme,
         users: 2.0,
         volume: 45.0,
         quality: 70.0,
         price: 40.0,
         speed: 3.0,
         duration,
         reduce_motion: false,
         disabled: 30.0,
         sizes: [50.0, 50.0, 35.0, 80.0],
         columns: [35.0, 60.0, 45.0],
      }
   }

   fn motion(&self) -> Motion {
      if self.reduce_motion {
         Motion::instant()
      } else {
         Motion::new(self.duration)
      }
   }

   fn header(&mut self, ui: &mut Ui) {
      ui.horizontal(|ui| {
         ui.spacing_mut().button_padding = self.theme.button_padding;
         ui.label(
            RichText::new("egui-elements slider")
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

   /// The card the Web Awesome docs open with: label, markers, references, hint, tooltip.
   fn reference_card(&mut self, ui: &mut Ui) {
      let motion = self.motion();
      let colors = self.theme.colors;
      card(ui, &self.theme, |ui| {
         caption(ui, &self.theme, "Reference");
         ui.add(
            Slider::new(&mut self.users, 0.0..=6.0)
               .text("Number of users")
               .step_by(1.0)
               .markers(true)
               .tooltip(true)
               .references(["Less", "More"])
               .hint("Limit six per team")
               .motion(motion),
         );

         caption(
            ui,
            &self.theme,
            "Label, tooltip and markers at a larger size",
         );
         ui.add(
            Slider::new(&mut self.quality, 0.0..=100.0)
               .text("Quality")
               .size(24.0)
               .markers(true)
               .tooltip(true)
               .motion(motion),
         );

         caption(
            ui,
            &self.theme,
            "References spread with space-between",
         );
         ui.add(
            Slider::new(&mut self.speed, 1.0..=5.0)
               .text("Speed")
               .step_by(1.0)
               .markers(true)
               .tooltip(true)
               .references(["Slow", "Medium", "Fast"])
               .hint("Controls the speed of the thing you're currently doing.")
               .motion(motion),
         );

         let on_accent = SliderVisuals {
            rail: colors.accent.gamma_multiply(0.45),
            fill: colors.bg,
            thumb: colors.bg,
            thumb_ring: colors.accent,
            marker: colors.accent,
            text: colors.bg,
            text_muted: colors.bg,
         };
         Card::new()
            .fill(colors.accent)
            .corner_radius(CornerRadius::same(10))
            .inner_margin(self.theme.spacing.margin(self.theme.spacing.md))
            .show(ui, |ui| {
               ui.add(
                  Slider::new(&mut self.volume, 0.0..=100.0)
                     .text("On an accent fill")
                     .visuals(on_accent)
                     .motion(motion),
               );
            });
      });
   }

   /// The formatter, the value cell and the stock-number parity.
   fn value_card(&mut self, ui: &mut Ui) {
      let motion = self.motion();
      card(ui, &self.theme, |ui| {
         caption(
            ui,
            &self.theme,
            "Custom formatter, step markers and a five-dollar step",
         );
         ui.add(
            Slider::new(&mut self.price, 0.0..=100.0)
               .text("Price")
               .step_by(5.0)
               .markers(true)
               .tooltip(true)
               .custom_formatter(|value, _| format!("\u{24}{value:.0}"))
               .references(["$0", "$50", "$100"])
               .motion(motion),
         );

         caption(
            ui,
            &self.theme,
            "One fixed decimal, value beside the rail",
         );
         ui.add(
            Slider::new(&mut self.volume, 0.0..=100.0)
               .text("Volume")
               .fixed_decimals(1)
               .tooltip(true)
               .motion(motion),
         );

         caption(
            ui,
            &self.theme,
            "No value cell, references still span the rail",
         );
         ui.add(
            Slider::new(&mut self.quality, 0.0..=100.0)
               .text("Quality")
               .show_value(false)
               .references(["Low", "High"])
               .motion(motion),
         );

         caption(ui, &self.theme, "Disabled");
         ui.add_enabled(
            false,
            Slider::new(&mut self.disabled, 0.0..=100.0)
               .text("Disabled")
               .references(["Off", "On"])
               .motion(motion),
         );
      });
   }

   /// A row of vertical sliders, each with its own formatted value.
   fn vertical_card(&mut self, ui: &mut Ui) {
      let space = self.theme.spacing;
      let motion = self.motion();
      card(ui, &self.theme, |ui| {
         caption(ui, &self.theme, "Vertical");
         ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = space.xl;
            for (index, label) in ["Volume", "Bass", "Treble"].into_iter().enumerate() {
               ui.add(
                  Slider::new(&mut self.columns[index], 0.0..=100.0)
                     .text(label)
                     .vertical()
                     .height(180.0)
                     .width(100.0)
                     .tooltip(true)
                     .motion(motion),
               );
            }
         });
      });
   }

   /// The same slider at four sizes; every measurement is a ratio of `size`.
   fn size_card(&mut self, ui: &mut Ui) {
      let motion = self.motion();
      card(ui, &self.theme, |ui| {
         caption(ui, &self.theme, "Size ladder: 12, 18, 24 and 32");
         ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = self.theme.spacing.lg;
            for (index, size) in [12.0, 18.0, 24.0, 32.0].into_iter().enumerate() {
               ui.add(
                  Slider::new(&mut self.sizes[index], 0.0..=100.0)
                     .size(size)
                     .width(150.0)
                     .step_by(10.0)
                     .markers(true)
                     .motion(motion),
               );
            }
         });
      });
   }

   fn knobs_card(&mut self, ui: &mut Ui) {
      let motion = self.motion();
      card(ui, &self.theme, |ui| {
         caption(ui, &self.theme, "Knobs");
         ui.add(
            egui::Slider::new(&mut self.duration, 0.05..=1.0)
               .text("animation_time (s)")
               .fixed_decimals(2),
         );
         ui.checkbox(&mut self.reduce_motion, "reduce motion");
         ui.add(
            Slider::new(&mut self.volume, 0.0..=100.0)
               .text("Live slider")
               .tooltip(true)
               .motion(motion),
         );
      });
   }
}

fn card(ui: &mut Ui, theme: &Theme, add: impl FnOnce(&mut Ui)) {
   let space = theme.spacing;
   Card::new()
      .fill(theme.colors.widget_bg)
      .corner_radius(CornerRadius::same(12))
      .inner_margin(space.margin(space.lg))
      .show(ui, |ui| {
         ui.spacing_mut().item_spacing.y = space.md;
         add(ui);
      });
}

fn caption(ui: &mut Ui, theme: &Theme, text: &str) {
   ui.add_space(theme.spacing.xs);
   ui.label(
      RichText::new(text)
         .size(theme.typography.small)
         .color(theme.colors.text_muted)
         .strong(),
   );
}

impl eframe::App for SliderApp {
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
            ScrollArea::vertical().show(ui, |ui| {
               ui.spacing_mut().item_spacing.y = space.lg;
               self.reference_card(ui);
               self.value_card(ui);
               self.vertical_card(ui);
               self.size_card(ui);
               self.knobs_card(ui);
            });
         });
   }
}
