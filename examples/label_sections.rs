//! Mixed-style wrapping labels (`Label::sections`).
//!
//! Drag the width slider to see a sentence wrap across bold/italic/underline
//! spans. Run with: `cargo run --example label_sections`

use eframe::egui::*;
use egui_elements::theme::{Theme, ThemeKind};
use egui_elements::utils;
use egui_elements::widgets::Label;

fn main() -> eframe::Result {
   let options = eframe::NativeOptions {
      viewport: ViewportBuilder::default()
         .with_title("egui-elements label sections")
         .with_inner_size([720.0, 780.0])
         .with_resizable(true),
      ..Default::default()
   };

   eframe::run_native(
      "egui-elements label sections",
      options,
      Box::new(|cc| Ok(Box::new(App::new(cc)))),
   )
}

struct App {
   theme: Theme,
   column_width: f32,
}

impl App {
   fn new(cc: &eframe::CreationContext<'_>) -> Self {
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&cc.egui_ctx);
      Self {
         theme,
         column_width: 420.0,
      }
   }

   fn paragraph(ui: &mut Ui, size: f32, parts: impl IntoIterator<Item = RichText>) {
      ui.add(
         Label::sections(parts, None)
            .size(size)
            .wrap()
            .fill_width(true)
            .interactive(false),
      );
   }
}

impl eframe::App for App {
   fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
      self.theme.clone().install(ui.ctx());

      let space = self.theme.spacing;
      let bg = self.theme.colors.bg;
      CentralPanel::default()
         .frame(Frame::new().fill(bg).inner_margin(space.margin(space.lg)))
         .show(ui, |ui| {
         ui.spacing_mut().item_spacing.y = space.md;

         ui.horizontal(|ui| {
            ui.heading("Label::sections");
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
               if let Some(new_theme) = utils::theme_switcher(&self.theme, ui) {
                  self.theme = new_theme;
               }
            });
         });

         ui.label(
            RichText::new(
               "One wrapping galley, several RichText spans. RichText::strong is a stronger color, not a bold font.",
            )
            .small()
            .weak(),
         );

         ui.add(
            Slider::new(&mut self.column_width, 160.0..=640.0).text("column width"),
         );

         let size = self.theme.typography.large;
         ui.set_max_width(self.column_width);
         ui.vertical(|ui| {
            ui.set_width(self.column_width);
            ui.spacing_mut().item_spacing.y = space.md;

            Self::paragraph(
               ui,
               size,
               [
                  "Zeus can download token icons from ".into(),
                  RichText::new("app.com").strong(),
                  " so unknown tokens show an image instead of a placeholder.".into(),
               ],
            );
            Self::paragraph(
               ui,
               size,
               [
                  "Zeus can also look up verified contract names on ".into(),
                  RichText::new("app.dev").strong(),
                  " when you sign a transaction or message.".into(),
               ],
            );
            Self::paragraph(
               ui,
               size,
               [
                  "Zeus can check ".into(),
                  RichText::new("GitHub").strong(),
                  " for a newer release on startup.".into(),
               ],
            );
            Self::paragraph(
               ui,
               size,
               [
                  "These are optional and just do http calls to third-party servers. No telemetry or data collection involved.".into(),
               ],
            );
            Self::paragraph(
               ui,
               size,
               ["You can change this later in Settings/General.".into()],
            );

            ui.separator();

            Self::paragraph(
               ui,
               size,
               [
                  "Also ".into(),
                  RichText::new("italics").italics(),
                  ", ".into(),
                  RichText::new("underline").underline(),
                  ", ".into(),
                  RichText::new("strikethrough").strikethrough(),
                  ", and ".into(),
                  RichText::new("code").code(),
                  " in the same wrapping sentence.".into(),
               ],
            );
         });
      });

      self.theme.install(ui.ctx());
   }
}
