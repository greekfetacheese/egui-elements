//! Tabs proof: the sliding indicator, the lift, the hover wash and the knobs that shape them.
//!
//! Run with: `cargo run --example tabs`

use eframe::egui::*;
use egui_elements::motion::Motion;
use egui_elements::widgets::{Frame as Card, Tabs, TabsStyle, TabsVisuals, ToggleShadow};
use egui_elements::{Theme, ThemeKind, utils};

fn main() -> eframe::Result {
   let options = eframe::NativeOptions {
      viewport: ViewportBuilder::default()
         .with_title("egui-elements tabs")
         .with_inner_size([720.0, 780.0])
         .with_resizable(true),
      ..Default::default()
   };

   eframe::run_native(
      "egui-elements tabs",
      options,
      Box::new(|cc| Ok(Box::new(TabsApp::new(cc)))),
   )
}

struct TabsApp {
   theme: Theme,
   filter: usize,
   range: usize,
   left: usize,
   right: usize,
   accent: usize,
   page: usize,
}

impl TabsApp {
   fn new(cc: &eframe::CreationContext<'_>) -> Self {
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&cc.egui_ctx);

      Self {
         theme,
         filter: 0,
         range: 1,
         left: 0,
         right: 2,
         accent: 0,
         page: 0,
      }
   }

   fn header(&mut self, ui: &mut Ui) {
      let space = self.theme.spacing;
      let colors = self.theme.colors;
      let title = self.theme.typography.large;
      let theme = self.theme.clone();

      Panel::top("header")
         .frame(
            Frame::new()
               .fill(colors.bg)
               .inner_margin(space.margin_xy(space.lg, space.md))
               .stroke(Stroke::new(1.0, colors.border)),
         )
         .show(ui, |ui| {
            ui.horizontal(|ui| {
               ui.label(RichText::new("Tabs").size(title).color(colors.text).strong());
               ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                  if let Some(new_theme) = utils::theme_switcher(&theme, ui) {
                     self.theme = new_theme;
                  }
               });
            });
         });
   }

   fn caption(&self, ui: &mut Ui, text: &str) {
      ui.label(
         RichText::new(text)
            .size(self.theme.typography.normal)
            .color(self.theme.colors.text_muted),
      );
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
            ui.label(RichText::new("Segmented tabs").size(normal).color(colors.text).strong());
            ui.label(
               RichText::new("The pill slides to the segment you pick and every state eases.")
                  .size(normal)
                  .color(colors.text_muted),
            );
            ui.add_space(space.lg);

            self.caption(ui, "Filled to the row — the reference strip");
            Tabs::new(
               "filter",
               &mut self.filter,
               ["All", "Media", "Links", "Files"],
            )
            .show(ui);
            ui.add_space(space.lg);

            self.caption(ui, "Fitted to its labels");
            Tabs::new("range", &mut self.range, ["Day", "Week", "Month"])
               .fill_width(false)
               .show(ui);
            ui.add_space(space.lg);

            self.caption(
               ui,
               "Taller strip — the inset and the pill scale with .height()",
            );
            Tabs::new("tall", &mut self.range, ["Day", "Week", "Month"])
               .height(44.0)
               .fill_width(false)
               .show(ui);
            ui.add_space(space.lg);

            self.caption(
               ui,
               "Compact labels — the strip height follows .text_size()",
            );
            Tabs::new(
               "compact",
               &mut self.range,
               ["Day", "Week", "Month"],
            )
            .text_size(12.0)
            .fill_width(false)
            .show(ui);
            ui.add_space(space.lg);

            self.caption(
               ui,
               "Accent indicator — TabsVisuals over the palette",
            );
            let accent = colors.accent;
            Tabs::new(
               "accent",
               &mut self.accent,
               ["Bold", "Italic", "Mono"],
            )
            .fill_width(false)
            .visuals(TabsVisuals {
               indicator: accent,
               text_selected: colors.ink_on(accent),
               shadow: ToggleShadow::new(accent),
               ..TabsVisuals::from_theme(colors)
            })
            .show(ui);
            ui.add_space(space.lg);

            self.caption(
               ui,
               "Flat (ToggleShadow::NONE) and instant (Motion::instant())",
            );
            ui.horizontal(|ui| {
               Tabs::new("flat", &mut self.left, ["A", "B"])
                  .fill_width(false)
                  .shadow(ToggleShadow::NONE)
                  .show(ui);
               Tabs::new("instant", &mut self.right, ["x", "y", "z"])
                  .fill_width(false)
                  .motion(Motion::instant())
                  .show(ui);
            });

            ui.add_space(space.lg);

            self.caption(
               ui,
               "Underline tabs — the page-tab reference, baseline across the row",
            );
            Tabs::new(
               "page",
               &mut self.page,
               ["Banners", "Posts", "Members", "Settings"],
            )
            .style(TabsStyle::Underline)
            .show(ui);
            ui.add_space(space.md);

            self.caption(
               ui,
               "Fitted to its labels — the baseline stops at the last tab",
            );
            Tabs::new(
               "page_fitted",
               &mut self.page,
               ["Overview", "Activity"],
            )
            .style(TabsStyle::Underline)
            .fill_width(false)
            .show(ui);

            ui.add_space(space.md);
            self.caption(
               ui,
               &format!(
                  "filter {} · range {} · left {} · right {} · accent {} · page {}",
                  self.filter, self.range, self.left, self.right, self.accent, self.page
               ),
            );
         });
   }
}

impl eframe::App for TabsApp {
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
