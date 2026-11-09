//! ComboBox popup chrome and virtualized rows.
//!
//! Run with: `cargo run --example combo_box`

use eframe::egui::*;
use egui_elements::theme::{Theme, ThemeKind};
use egui_elements::utils;
use egui_elements::widgets::{ComboBox, Label};

/// The height every virtualized row must occupy, without `item_spacing.y`.
const ROW_H: f32 = 24.0;
/// Rows that exist in the data set — far more than are ever laid out.
const ROWS: usize = 10_000;
const FRUIT: [&str; 8] = [
   "Apple",
   "Banana",
   "Cherry",
   "Date",
   "Elderberry",
   "Fig",
   "Grape",
   "Kiwi",
];

fn main() -> eframe::Result {
   let options = eframe::NativeOptions {
      viewport: ViewportBuilder::default()
         .with_title("egui-elements combo box")
         .with_inner_size([600.0, 660.0])
         .with_resizable(true),
      ..Default::default()
   };

   eframe::run_native(
      "egui-elements combo box",
      options,
      Box::new(|cc| Ok(Box::new(App::new(cc)))),
   )
}

struct App {
   theme: Theme,
   gap: f32,
   margin: f32,
   fruit: usize,
   row: usize,
   built: usize,
   plus_open: bool,
}

impl App {
   fn new(cc: &eframe::CreationContext<'_>) -> Self {
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&cc.egui_ctx);
      Self {
         theme,
         gap: 4.0,
         margin: 5.0,
         fruit: 0,
         row: 0,
         built: 0,
         plus_open: false,
      }
   }
}

impl eframe::App for App {
   fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
      self.theme.clone().install(ui.ctx());

      let space = self.theme.spacing;
      let bg = self.theme.colors.bg;
      let small = self.theme.typography.small;

      CentralPanel::default()
         .frame(Frame::new().fill(bg).inner_margin(space.margin(space.lg)))
         .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = space.md;

            ui.horizontal(|ui| {
               ui.heading("ComboBox");
               ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                  if let Some(new_theme) = utils::theme_switcher(&self.theme, ui) {
                     self.theme = new_theme;
                  }
               });
            });

            // 1. A plain list, with the two spacing knobs on sliders.
            ui.label(RichText::new("Popup spacing").size(small).weak());
            ui.add(Slider::new(&mut self.gap, 0.0..=24.0).text("popup_gap"));
            ui.add(Slider::new(&mut self.margin, 0.0..=24.0).text("popup_content_margin"));

            let selected = Label::new(FRUIT[self.fruit], None);
            ComboBox::new("fruit", selected)
               .width(200.0)
               .popup_gap(self.gap)
               .popup_content_margin(self.margin)
               .show_ui(ui, |ui| {
                  for (index, fruit) in FRUIT.iter().enumerate() {
                     if ComboBox::option(ui, *fruit, index == self.fruit).clicked() {
                        self.fruit = index;
                     }
                  }
               });

            ui.separator();

            // 2. 10 000 rows, of which only the viewport is ever built. Every row occupies
            //    exactly ROW_H, which is the contract `show_rows_ui` takes.
            ui.label(RichText::new("Virtualized rows").size(small).weak());
            self.built = 0;
            let selected = Label::new(format!("Row {}", self.row + 1), None);
            ComboBox::new("virtual", selected)
               .width(200.0)
               .popup_gap(self.gap)
               .popup_content_margin(self.margin)
               .show_rows_ui(ui, ROW_H, ROWS, |ui, range| {
                  self.built += range.len();

                  for index in range {
                     let picked = ui
                        .allocate_ui_with_layout(
                           vec2(ui.available_width(), ROW_H),
                           Layout::left_to_right(Align::Center),
                           |ui| {
                              ui.add(
                                 Label::new(format!("Row {}", index + 1), None)
                                    .expand(Some(4.0))
                                    .selected(index == self.row)
                                    .sense(Sense::click())
                                    .fill_width(true),
                              )
                              .clicked()
                           },
                        )
                        .inner;

                     if picked {
                        self.row = index;
                     }
                  }
               });

            ui.label(
               RichText::new(format!(
                  "built over the last frame: {} of {ROWS}",
                  self.built
               ))
               .size(small)
               .weak(),
            );

            ui.separator();

            // 3. A custom glyph: the painter still receives the live open flag.
            ui.label(RichText::new("Custom icon").size(small).weak());
            let selected = Label::new(
               if self.plus_open {
                  "expanded"
               } else {
                  "collapsed"
               },
               None,
            );
            ComboBox::new("custom_icon", selected)
               .width(200.0)
               .popup_gap(self.gap)
               .popup_content_margin(self.margin)
               .icon(|ui, rect, visuals, open| {
                  let stroke = Stroke::new(1.5, visuals.fg_stroke.color);
                  ui.painter().line_segment([rect.left_center(), rect.right_center()], stroke);

                  if !open {
                     ui.painter().line_segment([rect.center_top(), rect.center_bottom()], stroke);
                  }
               })
               .show_ui(ui, |ui| {
                  for state in [false, true] {
                     let row = Label::new(if state { "expanded" } else { "collapsed" }, None)
                        .expand(Some(4.0))
                        .selected(state == self.plus_open)
                        .sense(Sense::click())
                        .fill_width(true);

                     if ui.add(row).clicked() {
                        self.plus_open = state;
                     }
                  }
               });
         });

      self.theme.install(ui.ctx());
   }
}
