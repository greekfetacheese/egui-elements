//! Secure credentials proof: a `CredentialsForm` a screen reader can drive.
//!
//! Run with: `cargo run --example secure_credentials`
//!
//! The dev-dependency enables eframe's `accesskit` feature; without it the window never
//! reaches the accessibility bus and a screen reader falls back to speaking raw keystrokes.
//!
//! The form announces its own password length, so the host does no accessibility plumbing.

use eframe::egui::*;
use egui_elements::widgets::{Frame as Card, Radio, SecureSpeech};
use egui_elements::{CredentialsForm, Theme, ThemeKind, utils};

fn main() -> eframe::Result {
   let options = eframe::NativeOptions {
      viewport: ViewportBuilder::default()
         .with_title("egui-elements secure credentials")
         .with_inner_size([640.0, 640.0])
         .with_resizable(true),
      ..Default::default()
   };

   eframe::run_native(
      "egui-elements secure credentials",
      options,
      Box::new(|cc| Ok(Box::new(SecureCredentialsApp::new(cc)))),
   )
}

struct SecureCredentialsApp {
   theme: Theme,
   form: CredentialsForm,
   speech: SecureSpeech,
}

impl SecureCredentialsApp {
   fn new(cc: &eframe::CreationContext<'_>) -> Self {
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&cc.egui_ctx);

      let form = CredentialsForm::new().with_open(true).with_confirm_password(true);

      Self {
         theme,
         form,
         speech: SecureSpeech::Masked,
      }
   }

   fn header(&mut self, ui: &mut Ui) {
      let space = self.theme.spacing;
      let colors = self.theme.colors;
      let title = self.theme.typography.large;
      let normal = self.theme.typography.normal;
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
               ui.label(RichText::new("Secure credentials").size(title).color(colors.text).strong());
               ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                  utils::theme_switcher(theme, ui);
               });
            });

            ui.label(
               RichText::new(
                  "Attach Orca or NVDA before you start: this window is the eframe root viewport, so AccessKit is available to it. Use headphones when Plaintext is on.",
               )
               .size(normal)
               .color(colors.text_muted),
            );
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
               RichText::new("Accessibility check")
                  .size(normal)
                  .color(colors.text)
                  .strong(),
            );
            ui.label(
               RichText::new(
                  "The form announces the focused password field's length, never its characters: each keystroke says the count. In Masked mode the field itself stays silent.",
               )
               .size(normal)
               .color(colors.text_muted),
            );
            ui.add_space(space.lg);

            self.form.show(ui);

            ui.add_space(space.lg);
            ui.label(RichText::new("Speech mode").size(normal).color(colors.text_muted));
            ui.horizontal(|ui| {
               ui.add(Radio::new(&mut self.speech, SecureSpeech::Masked, "Masked"));
               ui.add(Radio::new(&mut self.speech, SecureSpeech::Plaintext, "Plaintext"));
            });
            ui.label(
               RichText::new("Ctrl+Shift+S toggles Masked / Plaintext from the keyboard.")
                  .size(normal)
                  .color(colors.text_muted),
            );

            if self.speech == SecureSpeech::Plaintext {
               ui.add_space(space.sm);
               ui.label(
                  RichText::new(
                     "Plaintext is on: the password will be read aloud and can be captured by the screen reader's speech history. Use headphones.",
                  )
                  .size(normal)
                  .color(colors.warning),
               );
            }
         });
   }
}

impl eframe::App for SecureCredentialsApp {
   fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
      if ui.input_mut(|input| input.consume_key(Modifiers::CTRL | Modifiers::SHIFT, Key::S)) {
         self.speech = match self.speech {
            SecureSpeech::Masked => SecureSpeech::Plaintext,
            SecureSpeech::Plaintext => SecureSpeech::Masked,
         };
      }

      self.form.set_speech(self.speech);

      self.header(ui);
      self.theme.clone().install(ui.ctx());

      let space = self.theme.spacing;
      let bg = self.theme.colors.bg;

      CentralPanel::default()
         .frame(Frame::new().fill(bg).inner_margin(space.margin(space.lg)))
         .show(ui, |ui| {
            ScrollArea::vertical().show(ui, |ui| self.card(ui));
         });
   }
}
