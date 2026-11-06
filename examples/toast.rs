//! Toast proof: every tone, a persistent toast, the stack cap, the four anchor
//! corners and the "Clear all" pill.
//!
//! Run with: `cargo run --example toast`

use eframe::egui::*;
use egui_elements::utils;
use egui_elements::widgets::{Button, Checkbox, ChipRole, Frame as Card, Toast, Toasts};
use egui_elements::{Theme, ThemeKind};

fn main() -> eframe::Result {
   let options = eframe::NativeOptions {
      viewport: ViewportBuilder::default()
         .with_title("egui-elements toast")
         .with_inner_size([880.0, 640.0])
         .with_resizable(true),
      ..Default::default()
   };

   eframe::run_native(
      "egui-elements toast",
      options,
      Box::new(|cc| Ok(Box::new(ToastApp::new(cc)))),
   )
}

/// Corner the stack hangs from.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Corner {
   TopLeft,
   TopRight,
   BottomLeft,
   BottomRight,
}

impl Corner {
   const ALL: [Corner; 4] = [
      Corner::TopLeft,
      Corner::TopRight,
      Corner::BottomLeft,
      Corner::BottomRight,
   ];

   fn align2(self) -> Align2 {
      match self {
         Corner::TopLeft => Align2::LEFT_TOP,
         Corner::TopRight => Align2::RIGHT_TOP,
         Corner::BottomLeft => Align2::LEFT_BOTTOM,
         Corner::BottomRight => Align2::RIGHT_BOTTOM,
      }
   }

   fn to_str(self) -> &'static str {
      match self {
         Corner::TopLeft => "Top left",
         Corner::TopRight => "Top right",
         Corner::BottomLeft => "Bottom left",
         Corner::BottomRight => "Bottom right",
      }
   }
}

struct ToastApp {
   theme: Theme,
   corner: Corner,
   width: f32,
   max_visible: usize,
   clear_all: bool,
   offset: f32,
}

impl ToastApp {
   fn new(cc: &eframe::CreationContext<'_>) -> Self {
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&cc.egui_ctx);

      Self {
         theme,
         corner: Corner::BottomRight,
         width: 320.0,
         max_visible: 5,
         clear_all: true,
         offset: 12.0,
      }
   }

   fn toasts(&self) -> Toasts {
      Toasts::new()
         .anchor(self.corner.align2())
         .offset(vec2(self.offset, self.offset))
         .width(self.width)
         .max_visible(self.max_visible)
         .clear_all_button(self.clear_all)
   }

   fn header(&mut self, ui: &mut Ui) {
      ui.horizontal(|ui| {
         ui.spacing_mut().button_padding = self.theme.button_padding;

         ui.label(
            RichText::new("egui-elements toast")
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

   fn triggers_card(&mut self, ui: &mut Ui) {
      let space = self.theme.spacing;
      let colors = self.theme.colors;

      Card::new()
         .fill(colors.widget_bg)
         .corner_radius(CornerRadius::same(12))
         .inner_margin(space.margin(space.xl))
         .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = space.sm;

            ui.label(
               RichText::new("Fire a toast")
                  .size(self.theme.typography.normal)
                  .color(colors.text)
                  .strong(),
            );

            ui.add_space(space.md);

            let tones = [
               ("Success", ChipRole::Success, "Backup finished", "42 files, 3.1 GB"),
               ("Error", ChipRole::Error, "Sync failed", "The remote rejected the push"),
               ("Warning", ChipRole::Warning, "Low disk space", "1.2 GB left on /"),
               ("Info", ChipRole::Info, "Update available", "Version 1.5.0 is ready"),
               ("Neutral", ChipRole::Neutral, "Session saved", "Reconnect to resume"),
            ];

            ui.horizontal_wrapped(|ui| {
               ui.spacing_mut().item_spacing = vec2(space.sm, space.sm);

               for (label, tone, title, description) in tones {
                  let button = Button::new(label).min_size(vec2(96.0, 30.0));

                  if ui.add(button).clicked() {
                     Toast::new(title)
                        .tone(tone)
                        .description(description)
                        .show(ui.ctx());
                  }
               }
            });

            ui.add_space(space.md);

            ui.horizontal_wrapped(|ui| {
               ui.spacing_mut().item_spacing = vec2(space.sm, space.sm);

               let persistent = Button::new("Persistent").min_size(vec2(120.0, 30.0));
               if ui.add(persistent).clicked() {
                  Toast::new("Waiting for review")
                     .tone(ChipRole::Info)
                     .description("Stays until you dismiss it")
                     .persistent()
                     .show(ui.ctx());
               }

               let long = Button::new("Long body").min_size(vec2(120.0, 30.0));
               if ui.add(long).clicked() {
                  Toast::new("This toast has a title long enough to wrap onto a second row")
                     .tone(ChipRole::Warning)
                     .description(
                        "And a description that keeps going, so the card height is measured                          from the wrapped galleys rather than assumed.",
                     )
                     .show(ui.ctx());
               }

               let burst = Button::new("Burst of 6").min_size(vec2(120.0, 30.0));
               if ui.add(burst).clicked() {
                  for index in 0..6 {
                     Toast::new(format!("Queued #{index}"))
                        .tone(ChipRole::Neutral)
                        .show(ui.ctx());
                  }
               }
            });
         });
   }

   fn knobs_card(&mut self, ui: &mut Ui) {
      let space = self.theme.spacing;
      let colors = self.theme.colors;
      let normal = self.theme.typography.normal;

      Card::new()
         .fill(colors.widget_bg)
         .corner_radius(CornerRadius::same(12))
         .inner_margin(space.margin(space.xl))
         .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = space.sm;

            ui.label(RichText::new("Toasts::new() knobs").size(normal).color(colors.text).strong());

            ui.add_space(space.md);

            ui.horizontal_wrapped(|ui| {
               ui.spacing_mut().item_spacing = vec2(space.sm, space.sm);

               for corner in Corner::ALL {
                  let button = Button::new(corner.to_str())
                     .selected(self.corner == corner)
                     .min_size(vec2(112.0, 34.0));

                  if ui.add(button).clicked() {
                     self.corner = corner;
                  }
               }
            });

            ui.add_space(space.md);

            ui.add(
               Slider::new(&mut self.width, 200.0..=480.0).text("width (pt)").fixed_decimals(0),
            );
            ui.add(Slider::new(&mut self.offset, 0.0..=48.0).text("offset (pt)").fixed_decimals(0));
            ui.add(Slider::new(&mut self.max_visible, 1..=8).text("max visible"));
            ui.add(Checkbox::new(
               &mut self.clear_all,
               "Clear all pill",
            ));
         });
   }
}

impl eframe::App for ToastApp {
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
            ui.spacing_mut().item_spacing.y = space.lg;
            ui.spacing_mut().button_padding = self.theme.button_padding;

            ScrollArea::vertical().show(ui, |ui| {
               ui.spacing_mut().item_spacing.y = space.lg;
               self.triggers_card(ui);
               self.knobs_card(ui);
            });
         });

      // The stack is an overlay: one render call per frame, after every panel.
      self.toasts().render(ui.ctx());
   }
}
