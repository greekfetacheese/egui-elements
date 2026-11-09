//! Row proof: navigation rows, a subtitle row, selection, and trailing controls.
//!
//! Run with: `cargo run --example row --features lucide`

use eframe::egui::*;
use egui_elements::egui_lucide::Lucide;
use egui_elements::widgets::{Checkbox, Frame as Card, Label, Row, Switch};
use egui_elements::{Theme, ThemeKind, utils};

fn main() -> eframe::Result {
   let options = eframe::NativeOptions {
      viewport: ViewportBuilder::default()
         .with_title("egui-elements row")
         .with_inner_size([760.0, 620.0])
         .with_resizable(true),
      ..Default::default()
   };

   eframe::run_native(
      "egui-elements row",
      options,
      Box::new(|cc| {
         egui_extras::install_image_loaders(&cc.egui_ctx);
         Ok(Box::new(RowApp::new(cc)))
      }),
   )
}

struct RowApp {
   theme: Theme,
   page: usize,
   notifications: bool,
   sounds: bool,
   last: &'static str,
}

impl RowApp {
   fn new(cc: &eframe::CreationContext<'_>) -> Self {
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&cc.egui_ctx);

      Self {
         theme,
         page: 0,
         notifications: true,
         sounds: false,
         last: "none",
      }
   }

   fn rows(&mut self, ui: &mut Ui) {
      let space = self.theme.spacing;
      let colors = self.theme.colors;
      let size = self.theme.typography.normal;
      let heading = self.theme.typography.large;

      ui.label(RichText::new("Rows").size(heading).color(colors.text).strong());
      ui.label(
         RichText::new(
            "Icon + title (+ subtitle) + optional trailing control, on the themed hover and selection plate.",
         )
         .size(size)
         .color(colors.text_muted),
      );

      ui.add_space(space.lg);

      Card::new()
         .fill(colors.widget_bg)
         .corner_radius(CornerRadius::same(12))
         .inner_margin(space.margin(space.md))
         .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;

            for (index, (icon, label)) in [
               (Lucide::Pencil, "Edit profile"),
               (Lucide::Bell, "Notifications"),
               (Lucide::ShieldCheck, "Privacy and security"),
            ]
            .into_iter()
            .enumerate()
            {
               let row = Row::new(
                  icon.image(),
                  RichText::new(label).size(size).color(colors.text),
               )
               .chevron(true)
               .selected(self.page == index)
               .id_salt(("nav", index));

               if ui.add(row).clicked() {
                  self.page = index;
                  self.last = label;
               }
            }

            ui.add_space(space.sm);

            ui.add(
               Row::new(
                  Lucide::MessageSquare.image(),
                  RichText::new("Sounds and popups").size(size).color(colors.text),
               )
               .subtitle(
                  RichText::new("Play a sound and show a banner for new messages")
                     .size(self.theme.typography.small)
                     .color(colors.text_muted),
               )
               .trailing(|ui| {
                  ui.add(Switch::without_text(&mut self.notifications));
               })
               .id_salt("sounds"),
            );

            ui.add(
               Row::new(
                  Lucide::Download.image(),
                  RichText::new("Download path").size(size).color(colors.text),
               )
               .trailing(|ui| {
                  ui.add(
                     Label::new(
                        RichText::new("Default folder")
                           .size(self.theme.typography.small)
                           .color(colors.accent),
                        None,
                     )
                     .interactive(false),
                  );
               })
               .id_salt("path"),
            );

            ui.add(
               Row::new(
                  Lucide::Images.image(),
                  RichText::new("Save to gallery").size(size).color(colors.text),
               )
               .trailing(|ui| {
                  ui.add(Checkbox::without_text(&mut self.sounds));
               })
               .id_salt("gallery"),
            );

            ui.add(
               Row::new(
                  Lucide::Trash.image(),
                  RichText::new("Clear cache").size(size).color(colors.error),
               )
               .chevron(true)
               .id_salt("clear"),
            );
         });

      ui.add_space(space.md);

      ui.label(
         RichText::new(format!(
            "Selected page: {} · notifications {} · last clicked: {}",
            self.page, self.notifications, self.last
         ))
         .size(self.theme.typography.small)
         .color(colors.text_muted),
      );
   }
}

impl eframe::App for RowApp {
   fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
      let space = self.theme.spacing;
      let colors = self.theme.colors;
      let heading = self.theme.typography.large;

      Panel::top("header")
         .frame(
            Frame::new()
               .fill(colors.bg)
               .inner_margin(space.margin_xy(space.lg, space.md))
               .stroke(Stroke::new(1.0, colors.border)),
         )
         .show(ui, |ui| {
            ui.horizontal(|ui| {
               ui.label(RichText::new("Row").size(heading).color(colors.text).strong());

               ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                  let _ = utils::theme_switcher(&self.theme, ui);
               });
            });
         });

      self.theme.clone().install(ui.ctx());

      CentralPanel::default()
         .frame(Frame::new().fill(colors.bg).inner_margin(space.margin(space.lg)))
         .show(ui, |ui| {
            ScrollArea::vertical().show(ui, |ui| self.rows(ui));
         });
   }
}
