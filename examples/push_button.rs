//! Interactive demo for [`Button::push`] and [`PushStyle`]: a button raised on a
//! solid base that collapses while it is pressed. The last row reproduces the
//! reference look the base was measured from.
//!
//! Run with: `cargo run --example push_button`

use eframe::egui::*;
use egui_elements::theme::{Theme, ThemeKind};
use egui_elements::utils;
use egui_elements::visuals::ButtonVisuals;
use egui_elements::widgets::{Button, PushStyle};

/// Uniform face size, so the rows line up; `min_size` sizes the face only.
const BUTTON_SIZE: Vec2 = Vec2::new(200.0, 44.0);
/// The reference's 8 px base under a 61 px face, applied to a 44 pt button.
const DEPTH: f32 = 6.0;

fn main() -> eframe::Result {
   let options = eframe::NativeOptions {
      viewport: ViewportBuilder::default()
         .with_title("egui-elements push buttons")
         .with_inner_size([900.0, 620.0])
         .with_resizable(true),
      ..Default::default()
   };

   eframe::run_native(
      "egui-elements push buttons",
      options,
      Box::new(|cc| Ok(Box::new(PushApp::new(cc)))),
   )
}

struct PushApp {
   theme: Theme,
   clicks: u32,
   pill_clicks: u32,
}

/// The palette of the reference button: a light face, a periwinkle border and a
/// deep blue base, on a full pill.
fn reference_visuals() -> ButtonVisuals {
   ButtonVisuals {
      text: Color32::from_rgb(0x00, 0x11, 0xff),
      bg: Color32::from_rgb(0xf8, 0xf8, 0xfd),
      bg_hover: Color32::from_rgb(0xf8, 0xf8, 0xfd),
      bg_click: Color32::from_rgb(0xf8, 0xf8, 0xfd),
      bg_selected: Color32::from_rgb(0xdf, 0xe3, 0xff),
      border: Stroke::new(3.0, Color32::from_rgb(0x8b, 0x93, 0xf8)),
      border_hover: Stroke::new(3.0, Color32::from_rgb(0x8b, 0x93, 0xf8)),
      border_click: Stroke::new(3.0, Color32::from_rgb(0x64, 0x6f, 0xff)),
      corner_radius: CornerRadius::same(u8::MAX),
      shadow: Shadow::NONE,
   }
}

impl PushApp {
   fn new(cc: &eframe::CreationContext<'_>) -> Self {
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&cc.egui_ctx);

      Self {
         theme,
         clicks: 0,
         pill_clicks: 0,
      }
   }

   fn header(&mut self, ui: &mut Ui) {
      ui.horizontal(|ui| {
         ui.spacing_mut().button_padding = self.theme.button_padding;

         ui.label(
            RichText::new("egui-elements push buttons")
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
}

impl eframe::App for PushApp {
   fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
      let space = self.theme.spacing;

      Panel::top("header")
         .frame(
            Frame::new()
               .fill(self.theme.colors.bg)
               .inner_margin(space.margin_xy(space.lg, space.md))
               .stroke(Stroke::new(1.0, self.theme.colors.border)),
         )
         .show(ui, |ui| self.header(ui));

      CentralPanel::default()
         .frame(Frame::new().fill(self.theme.colors.bg).inner_margin(space.margin(space.lg)))
         .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = space.md;

            ui.label(
               RichText::new("Push buttons")
                  .size(self.theme.typography.large)
                  .color(self.theme.colors.text),
            );
            ui.label(
               RichText::new(
                  "The face sits on a solid base and travels down one depth while the \
                   pointer is down, so the button looks pushed flush. The base is part of \
                   the layout and of the click target, not an overflowing shadow.",
               )
               .size(self.theme.typography.normal)
               .color(self.theme.colors.text_muted),
            );

            ui.add_space(space.md);

            ui.horizontal(|ui| {
               ui.spacing_mut().item_spacing.x = space.md;

               if ui.add(Button::new("Push me").push(DEPTH).min_size(BUTTON_SIZE)).clicked() {
                  self.clicks += 1;
               }

               if ui
                  .add(Button::new("Selected").push(DEPTH).selected(true).min_size(BUTTON_SIZE))
                  .clicked()
               {
                  self.clicks += 1;
               }
            });

            ui.add_space(space.sm);

            ui.horizontal(|ui| {
               ui.spacing_mut().item_spacing.x = space.md;

               for depth in [2.0, 4.0, 8.0] {
                  ui.add(
                     Button::new(format!("{depth:.0} pt base"))
                        .push(depth)
                        .min_size(Vec2::new(120.0, 40.0)),
                  );
               }
            });

            ui.add_space(space.md);
            ui.label(
               RichText::new(format!("Clicks: {}", self.clicks))
                  .size(self.theme.typography.normal)
                  .color(self.theme.colors.success),
            );

            ui.add_space(space.xl);

            ui.label(
               RichText::new("The reference look")
                  .size(self.theme.typography.large)
                  .color(self.theme.colors.text),
            );
            ui.label(
               RichText::new(
                  "The button from uiverse.io/KhelVers/orange-lionfish-62: a periwinkle \
                   border, a deep blue base and a full pill. The numbers are the \
                   reference's own, measured off the screenshot.",
               )
               .size(self.theme.typography.normal)
               .color(self.theme.colors.text_muted),
            );

            ui.add_space(space.md);

            let visuals = reference_visuals();
            let base = Color32::from_rgb(0x1f, 0x35, 0xff);

            ui.horizontal(|ui| {
               ui.spacing_mut().item_spacing.x = space.md;

               if ui
                  .add(
                     Button::new(RichText::new("Winter").size(20.0))
                        .visuals(visuals)
                        .push(6.0)
                        .push_color(base)
                        .min_size(Vec2::new(160.0, 44.0)),
                  )
                  .clicked()
               {
                  self.pill_clicks += 1;
               }

               // The same look through the struct, at double the travel.
               if ui
                  .add(
                     Button::new(RichText::new("Winter").size(20.0))
                        .visuals(visuals)
                        .push_style(&PushStyle {
                           depth: 12.0,
                           color: Some(base),
                        })
                        .min_size(Vec2::new(160.0, 44.0)),
                  )
                  .clicked()
               {
                  self.pill_clicks += 1;
               }
            });

            ui.add_space(space.md);
            ui.label(
               RichText::new(format!("Reference clicks: {}", self.pill_clicks))
                  .size(self.theme.typography.normal)
                  .color(self.theme.colors.text_muted),
            );
         });
   }
}
