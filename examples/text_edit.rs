//! TextEdit proof: a search field, a composer and a character limit.
//!
//! Run with: `cargo run --example text_edit`

use eframe::egui::*;
use egui_elements::widgets::{Frame as Card, TextEdit};
use egui_elements::{Theme, ThemeKind, utils};

fn main() -> eframe::Result {
   let options = eframe::NativeOptions {
      viewport: ViewportBuilder::default()
         .with_title("egui-elements text edit")
         .with_inner_size([720.0, 560.0])
         .with_resizable(true),
      ..Default::default()
   };

   eframe::run_native(
      "egui-elements text edit",
      options,
      Box::new(|cc| Ok(Box::new(TextEditApp::new(cc)))),
   )
}

struct TextEditApp {
   theme: Theme,
   search: String,
   message: String,
   handle: String,
   sent: Option<String>,
}

impl TextEditApp {
   fn new(cc: &eframe::CreationContext<'_>) -> Self {
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&cc.egui_ctx);

      Self {
         theme,
         search: String::new(),
         message: String::new(),
         handle: String::new(),
         sent: None,
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
               ui.label(RichText::new("TextEdit").size(title).color(colors.text).strong());
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
            ui.label(RichText::new("Themed text edits").size(normal).color(colors.text).strong());
            ui.label(
               RichText::new("Same widget as the secure field, named for the common case.")
                  .size(normal)
                  .color(colors.text_muted),
            );
            ui.add_space(space.lg);

            ui.label(RichText::new("Search").size(normal).color(colors.text_muted));
            ui.add(
               TextEdit::singleline(&mut self.search)
                  .hint_text("Search chats")
                  .desired_width(320.0),
            );
            ui.add_space(space.lg);

            ui.label(
               RichText::new("Composer (3 rows, Ctrl+Enter sends)")
                  .size(normal)
                  .color(colors.text_muted),
            );
            let composer = ui.add(
               TextEdit::multiline(&mut self.message)
                  .hint_text("Message")
                  .desired_width(400.0)
                  .desired_rows(3)
                  .return_key(KeyboardShortcut::new(Modifiers::CTRL, Key::Enter)),
            );
            let submit = composer.has_focus()
               && ui.input(|input| input.modifiers.ctrl && input.key_pressed(Key::Enter));
            if submit {
               self.sent = Some(self.message.clone());
            }
            ui.add_space(space.lg);

            ui.label(RichText::new("Handle (16 characters)").size(normal).color(colors.text_muted));
            ui.add(
               TextEdit::singleline(&mut self.handle)
                  .hint_text("@handle")
                  .char_limit(16)
                  .desired_width(220.0),
            );
            ui.label(
               RichText::new(format!("{}/16", self.handle.chars().count()))
                  .size(normal)
                  .color(colors.text_muted),
            );

            if let Some(sent) = &self.sent {
               ui.add_space(space.md);
               ui.label(RichText::new(format!("Sent: {sent}")).size(normal).color(colors.success));
            }
         });
   }
}

impl eframe::App for TextEditApp {
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
