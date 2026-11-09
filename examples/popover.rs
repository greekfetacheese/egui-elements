//! Popover proof: a picker, a menu and an above-side inspector.
//!
//! Run with: `cargo run --example popover`

use eframe::egui::*;
use egui_elements::widgets::{Button, Frame as Card, Popover, PopoverSide};
use egui_elements::{Theme, ThemeKind, utils};

fn main() -> eframe::Result {
   let options = eframe::NativeOptions {
      viewport: ViewportBuilder::default()
         .with_title("egui-elements popover")
         .with_inner_size([760.0, 560.0])
         .with_resizable(true),
      ..Default::default()
   };

   eframe::run_native(
      "egui-elements popover",
      options,
      Box::new(|cc| Ok(Box::new(PopoverApp::new(cc)))),
   )
}

struct PopoverApp {
   theme: Theme,
   picker_open: bool,
   menu_open: bool,
   above_open: bool,
   symbol: &'static str,
   action: &'static str,
}

impl PopoverApp {
   fn new(cc: &eframe::CreationContext<'_>) -> Self {
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&cc.egui_ctx);

      Self {
         theme,
         picker_open: false,
         menu_open: false,
         above_open: false,
         symbol: "★",
         action: "none",
      }
   }

   fn header(&mut self, ui: &mut Ui) {
      let space = self.theme.spacing;
      let colors = self.theme.colors;
      let title = self.theme.typography.large;
      let theme = &self.theme;

      Panel::top("header")
         .frame(
            Frame::new()
               .fill(colors.bg)
               .inner_margin(space.margin_xy(space.lg, space.md))
               .stroke(Stroke::new(1.0, colors.border)),
         )
         .show(ui, |ui| {
            ui.horizontal(|ui| {
               ui.label(RichText::new("Popover").size(title).color(colors.text).strong());

               ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                  utils::theme_switcher(theme, ui);
               });
            });
         });
   }

   fn card(&mut self, ui: &mut Ui) {
      let space = self.theme.spacing;
      let colors = self.theme.colors;
      let normal = self.theme.typography.normal;

      Card::new()
         .fill(colors.widget_bg)
         .corner_radius(CornerRadius::same(12))
         .inner_margin(space.margin(space.xl))
         .show(ui, |ui| {
            ui.label(
               RichText::new("Anchored panels")
                  .size(normal)
                  .color(colors.text)
                  .strong(),
            );
            ui.label(
               RichText::new(
                  "Each popover owns its card, shadow, outside click and Esc. Open one while another is open: they are independent.",
               )
               .size(normal)
               .color(colors.text_muted),
            );

            ui.add_space(space.lg);

            ui.horizontal_wrapped(|ui| {
               ui.spacing_mut().item_spacing.x = space.md;

               // Picker: a symbol grid that closes on selection.
               let button = ui.add(
                  Button::new(format!("Pick a symbol: {}", self.symbol))
                     .min_size(vec2(200.0, 36.0)),
               );
               let mut open = self.picker_open;
               if button.clicked() {
                  open = !open;
               }
               let mut picked = None;
               let _ = Popover::new("picker", &mut open)
                  .width(220.0)
                  .show(ui.ctx(), button.rect, |ui| {
                     ui.spacing_mut().item_spacing = vec2(space.sm, space.sm);
                     for symbol in ["★", "◆", "●", "▲", "■", "✦"] {
                        if ui.add(Button::new(symbol).small()).clicked() {
                           picked = Some(symbol);
                        }
                     }
                  });
               self.picker_open = open;
               if let Some(symbol) = picked {
                  self.symbol = symbol;
                  self.picker_open = false;
               }

               // Menu: full-width rows.
               let button = ui.add(Button::new("Actions").min_size(vec2(120.0, 36.0)));
               let mut open = self.menu_open;
               if button.clicked() {
                  open = !open;
               }
               let mut action = None;
               let _ = Popover::new("menu", &mut open)
                  .width(200.0)
                  .show(ui.ctx(), button.rect, |ui| {
                     for item in ["Pin", "Mute", "Archive", "Delete"] {
                        let row = Button::new(item).min_size(vec2(ui.available_width(), 32.0));
                        if ui.add(row).clicked() {
                           action = Some(item);
                        }
                     }
                  });
               self.menu_open = open;
               if let Some(action) = action {
                  self.action = action;
                  self.menu_open = false;
               }

               // Above: bottom-left pivot on the anchor's top edge.
               let button = ui.add(Button::new("About").min_size(vec2(110.0, 36.0)));
               let mut open = self.above_open;
               if button.clicked() {
                  open = !open;
               }
               let _ = Popover::new("above", &mut open)
                  .side(PopoverSide::Above)
                  .align(Align2::RIGHT_BOTTOM)
                  .screen_margin(space.md)
                  .width(280.0)
                  .show(ui.ctx(), button.rect, |ui| {
                     ui.label(
                        RichText::new("Opened above the anchor.")
                           .size(normal)
                           .color(colors.text),
                     );
                     ui.label(
                        RichText::new("Esc, an outside click, or the button's own toggle closes it.")
                           .size(normal)
                           .color(colors.text_muted),
                     );
                  });
               self.above_open = open;
            });

            ui.add_space(space.md);

            ui.label(
               RichText::new(format!("Last symbol: {} · last action: {}", self.symbol, self.action))
                  .size(normal)
                  .color(colors.text_muted),
            );
         });
   }
}

impl eframe::App for PopoverApp {
   fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
      let space = self.theme.spacing;
      let bg = self.theme.colors.bg;

      self.header(ui);
      self.theme.clone().install(ui.ctx());

      CentralPanel::default()
         .frame(Frame::new().fill(bg).inner_margin(space.margin(space.lg)))
         .show(ui, |ui| {
            ScrollArea::vertical().show(ui, |ui| self.card(ui));
         });
   }
}
