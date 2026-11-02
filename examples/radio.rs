//! Radio proof: the reference's settings card, the live knobs (size, duration, shadow, reduce motion),
//! a size ladder and a colour override on a filled surface.
//!
//! Two independent groups prove the group semantics: each radio is bound to the same `&mut value` as its
//! siblings, so selecting one clears the rest.
//!
//! Run with: `cargo run --example radio`

use eframe::egui::*;
use egui_elements::motion::Motion;
use egui_elements::widgets::{Frame as Card, Radio, ToggleShadow, ToggleVisuals};
use egui_elements::{Theme, ThemeKind, utils};

fn main() -> eframe::Result {
   let options = eframe::NativeOptions {
      viewport: ViewportBuilder::default()
         .with_title("egui-elements radio")
         .with_inner_size([880.0, 680.0])
         .with_resizable(true),
      ..Default::default()
   };

   eframe::run_native(
      "egui-elements radio",
      options,
      Box::new(|cc| Ok(Box::new(RadioApp::new(cc)))),
   )
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Flavor {
   Sms,
   Authenticator,
   Hardware,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Backup {
   Cloud,
   Local,
   None,
}

struct RadioApp {
   theme: Theme,
   size: f32,
   duration: f32,
   shadow: bool,
   reduce_motion: bool,
   channel: Flavor,
   backup: Backup,
   /// One group of one per size in the ladder, so each sample keeps its own idle/selected state.
   ladder: [Flavor; 5],
}

impl RadioApp {
   fn new(cc: &eframe::CreationContext<'_>) -> Self {
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&cc.egui_ctx);
      // The widget reads the style's clock; the slider opens on the value it really uses.
      let duration = theme.style().animation_time;

      Self {
         theme,
         size: 18.0,
         duration,
         shadow: true,
         reduce_motion: false,
         channel: Flavor::Sms,
         backup: Backup::Cloud,
         ladder: [
            Flavor::Sms,
            Flavor::Authenticator,
            Flavor::Hardware,
            Flavor::Sms,
            Flavor::Authenticator,
         ],
      }
   }

   fn motion(&self) -> Motion {
      if self.reduce_motion {
         Motion::instant()
      } else {
         Motion::new(self.duration)
      }
   }

   fn lift(&self) -> ToggleShadow {
      if self.shadow {
         ToggleShadow::derived()
      } else {
         ToggleShadow::NONE
      }
   }

   fn header(&mut self, ui: &mut Ui) {
      ui.horizontal(|ui| {
         ui.spacing_mut().button_padding = self.theme.button_padding;

         ui.label(
            RichText::new("egui-elements radio")
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

   /// The reference's settings card: heading, subtitle, rules and two independent radio groups.
   fn settings_card(&mut self, ui: &mut Ui) {
      let space = self.theme.spacing;
      let colors = self.theme.colors;
      let normal = self.theme.typography.normal;
      let heading = self.theme.typography.large;
      let motion = self.motion();
      let lift = self.lift();
      let size = self.size;

      Card::new()
         .fill(colors.widget_bg)
         .corner_radius(CornerRadius::same(12))
         .inner_margin(space.margin(space.xl))
         .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = space.sm;

            ui.label(
               RichText::new("Authentication settings")
                  .size(heading)
                  .color(colors.text)
                  .strong(),
            );
            ui.label(
               RichText::new("Edit your preferences for authentication settings.")
                  .size(normal)
                  .color(colors.text_muted),
            );

            ui.add_space(space.xs);
            ui.separator();
            ui.add_space(space.xs);

            ui.label(
               RichText::new("Two-factor authentication")
                  .size(normal)
                  .color(colors.text)
                  .strong(),
            );

            ui.add(
               Radio::new(
                  &mut self.channel,
                  Flavor::Sms,
                  RichText::new("SMS verification").size(normal).color(colors.text),
               )
               .size(size)
               .motion(motion)
               .shadow(lift),
            );
            ui.add(
               Radio::new(
                  &mut self.channel,
                  Flavor::Authenticator,
                  RichText::new("Authenticator app").size(normal).color(colors.text),
               )
               .size(size)
               .motion(motion)
               .shadow(lift),
            );
            ui.add(
               Radio::new(
                  &mut self.channel,
                  Flavor::Hardware,
                  RichText::new("Hardware key").size(normal).color(colors.text),
               )
               .size(size)
               .motion(motion)
               .shadow(lift),
            );

            ui.add_space(space.xs);
            ui.separator();
            ui.add_space(space.xs);

            ui.label(RichText::new("Backup codes").size(normal).color(colors.text).strong());
            ui.label(
               RichText::new(
                  "A second, independent group: it shares the palette and clock, not the value.",
               )
               .size(normal)
               .color(colors.text_muted),
            );
            ui.add(
               Radio::new(
                  &mut self.backup,
                  Backup::Cloud,
                  RichText::new("Cloud sync").size(normal).color(colors.text),
               )
               .size(size)
               .motion(motion)
               .shadow(lift),
            );
            ui.add(
               Radio::new(
                  &mut self.backup,
                  Backup::Local,
                  RichText::new("Local file").size(normal).color(colors.text),
               )
               .size(size)
               .motion(motion)
               .shadow(lift),
            );
            ui.add_enabled(
               false,
               Radio::new(
                  &mut self.backup,
                  Backup::None,
                  RichText::new("None (disabled)").size(normal).color(colors.text),
               )
               .size(size)
               .motion(motion),
            );
         });
   }

   /// Knobs, the size ladder and the colour override on a filled surface.
   fn proof_card(&mut self, ui: &mut Ui) {
      let space = self.theme.spacing;
      let colors = self.theme.colors;
      let normal = self.theme.typography.normal;
      let motion = self.motion();
      let lift = self.lift();
      let size = self.size;

      Card::new()
         .fill(colors.widget_bg)
         .corner_radius(CornerRadius::same(12))
         .inner_margin(space.margin(space.lg))
         .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = space.sm;
            ui.spacing_mut().slider_width = 260.0;

            ui.add(Slider::new(&mut self.size, 12.0..=40.0).text("size"));
            ui.add(Slider::new(&mut self.duration, 0.05..=1.0).text("duration (s)"));
            ui.checkbox(
               &mut self.shadow,
               RichText::new("shadow").size(normal).color(colors.text),
            );
            ui.checkbox(
               &mut self.reduce_motion,
               RichText::new("reduce motion").size(normal).color(colors.text),
            );
            ui.label(
               RichText::new(format!(
                  "state.animation_time = {:.3} s · slider = {:.3} s",
                  self.theme.style().animation_time,
                  self.duration
               ))
               .size(normal)
               .color(colors.text_muted),
            );

            ui.add_space(space.sm);
            ui.label(RichText::new("Size ladder").size(normal).color(colors.text).strong());
            ui.horizontal_wrapped(|ui| {
               ui.spacing_mut().item_spacing.x = space.lg;
               for (index, sample) in [12.0, 18.0, 24.0, 32.0, 40.0].into_iter().enumerate() {
                  ui.add(
                     Radio::without_text(&mut self.ladder[index], Flavor::Sms)
                        .size(sample)
                        .motion(motion)
                        .shadow(lift),
                  );
               }
            });

            ui.add_space(space.sm);
            ui.label(RichText::new("On a filled surface").size(normal).color(colors.text).strong());
            let on_accent = ToggleVisuals {
               bg_hover: colors.accent.gamma_multiply(0.85),
               bg_checked: colors.bg,
               border: colors.bg,
               border_hover: colors.bg,
               border_checked: colors.bg,
               mark: colors.accent,
               shadow: ToggleShadow::new(colors.bg),
               ..ToggleVisuals::from_theme(colors)
            };
            Card::new()
               .fill(colors.accent)
               .corner_radius(CornerRadius::same(10))
               .inner_margin(space.margin(space.md))
               .show(ui, |ui| {
                  ui.add(
                     Radio::new(
                        &mut self.channel,
                        Flavor::Sms,
                        RichText::new("Readable on an accent fill").size(normal).color(colors.bg),
                     )
                     .size(size)
                     .visuals(on_accent)
                     .motion(motion),
                  );
               });
         });
   }
}

impl eframe::App for RadioApp {
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
               self.settings_card(ui);
               self.proof_card(ui);
            });
         });
   }
}
