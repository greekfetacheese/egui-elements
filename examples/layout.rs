//! Proof for the `layout` module: five sections, each a case a host used to
//! hand-roll with `ui.horizontal`. Sections B–E contain no `ui.horizontal` at
//! all — the flex containers do the work.
//!
//! Run with: `cargo run --example layout`

use eframe::egui::*;
use egui_elements::layout::{CrossAlign, Flex, Item, Justify};
use egui_elements::theme::{Theme, ThemeKind};
use egui_elements::widgets::{Button, Label};

fn main() -> eframe::Result {
   let options = eframe::NativeOptions {
      viewport: ViewportBuilder::default()
         .with_title("egui-elements layout")
         .with_inner_size([900.0, 760.0])
         .with_resizable(true),
      ..Default::default()
   };

   eframe::run_native(
      "egui-elements layout",
      options,
      Box::new(|cc| Ok(Box::new(LayoutApp::new(cc)))),
   )
}

struct LayoutApp {
   theme: Theme,
   has_seed: bool,
}

impl LayoutApp {
   fn new(cc: &eframe::CreationContext<'_>) -> Self {
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&cc.egui_ctx);

      Self {
         theme,
         has_seed: true,
      }
   }
}

impl eframe::App for LayoutApp {
   fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
      let mut has_seed = self.has_seed;
      let theme = &self.theme;
      let bg = theme.colors.bg;

      CentralPanel::default()
         .frame(Frame::new().fill(bg).inner_margin(Margin::same(10)))
         .show(ui, |ui| {
            ScrollArea::vertical()
               .id_salt("layout_scroll")
               .auto_shrink([false, false])
               .show(ui, |ui| {
                  ui.spacing_mut().button_padding = vec2(6.0, 8.0);
                  ui.spacing_mut().item_spacing.y = theme.spacing.md;

                  section_centering(ui, theme);
                  section_equal_actions(ui, theme, &mut has_seed);
                  section_distribution(ui, theme);
                  section_nested(ui, theme);
                  section_alignment(ui, theme);
               });
         });

      self.has_seed = has_seed;
   }
}

/// A section caption: what the reader should look at.
fn caption(ui: &mut Ui, theme: &Theme, text: &str) {
   ui.label(RichText::new(text).size(theme.typography.small).color(theme.colors.text_muted));
}

/// A mini heading above one comparison row.
fn heading(ui: &mut Ui, theme: &Theme, text: &str) {
   ui.label(RichText::new(text).size(theme.typography.normal).color(theme.colors.text));
}

/// A. The `demo2` comparison: `ui.horizontal` seeds the row's height from
/// egui's guess, so the taller button is not centred against the label. The
/// flex row measures both items first.
fn section_centering(ui: &mut Ui, theme: &Theme) {
   caption(
      ui,
      theme,
      "A. Centering — the horizontal row starts at egui's guessed height; the flex row measures the button first.",
   );

   heading(ui, theme, "Bad — ui.horizontal");
   ui.horizontal(|ui| {
      ui.label("Normal text");
      ui.add(Button::new("Click me"));
   });

   ui.add_space(theme.spacing.sm);

   heading(ui, theme, "Good — Flex::row + CrossAlign::Center");
   Flex::row("demo_good")
      .gap(theme.spacing.sm)
      .align_items(CrossAlign::Center)
      .show(ui, |f| {
         f.item(Item::new(), |ui| {
            ui.add(Label::new("Normal text", None));
         });
         f.item(Item::new(), |ui| {
            ui.add(Button::new("Click me"));
         });
      });
}

/// B. The `zeus export.rs` case: every action is `flex: 1`, so the captions
/// never decide the widths and no `(w - gap * (n - 1)) / n` line exists.
fn section_equal_actions(ui: &mut Ui, theme: &Theme, has_seed: &mut bool) {
   caption(
      ui,
      theme,
      "B. Equal-width actions — grow items share the row equally, whatever the caption length.",
   );

   ui.checkbox(has_seed, "Has seed phrase");

   let mut captions: Vec<&str> = vec!["Copy Key"];
   if *has_seed {
      captions.push("Copy Seed Phrase");
   }
   captions.push("Show QR Code");

   Flex::column("export_actions")
      .gap(theme.spacing.sm)
      .align_items(CrossAlign::Stretch)
      .show(ui, |f| {
         f.item(Item::new(), |ui| {
            Flex::row("export_row").gap(theme.spacing.sm).show(ui, |f| {
               for text in &captions {
                  f.item(Item::grow(1.0), |ui| {
                     let width = ui.available_width();
                     ui.add(
                        Button::new(*text).min_size(vec2(width, 0.0)).label_align(Align::Center),
                     );
                  });
               }
            });
         });
         f.item(Item::new(), |ui| {
            Flex::row("export_clear").show(ui, |f| {
               f.item(Item::grow(1.0), |ui| {
                  let width = ui.available_width();
                  ui.add(
                     Button::new("Clear Clipboard")
                        .min_size(vec2(width, 0.0))
                        .label_align(Align::Center),
                  );
               });
            });
         });
      });
}

/// C. `SpaceBetween` against a full-width row: the disabled first page pushes
/// the label into the middle and the action to the right edge.
fn section_distribution(ui: &mut Ui, theme: &Theme) {
   caption(
      ui,
      theme,
      "C. Distribution — Justify::SpaceBetween spreads the free space between the items; Previous is disabled.",
   );

   Flex::row("pagination")
      .gap(theme.spacing.sm)
      .justify(Justify::SpaceBetween)
      .align_items(CrossAlign::Center)
      .show(ui, |f| {
         f.item(Item::new(), |ui| {
            ui.add_enabled(false, Button::new("Previous"));
         });
         f.item(Item::new(), |ui| {
            ui.add(Label::new("Page 1 of 25", None));
         });
         f.item(Item::new(), |ui| {
            ui.add(Button::new("Next"));
         });
      });
}

/// D. A card whose body is a column; the column's first item is an
/// 80 / grow / 80 row. Flex containers nest.
fn section_nested(ui: &mut Ui, theme: &Theme) {
   caption(
      ui,
      theme,
      "D. Nested flex — a column card whose first row is fixed 80 / flex 1 / fixed 80.",
   );

   theme.frame1.show(ui, |ui| {
      ui.set_width(ui.available_width());
      Flex::column("card_body").gap(theme.spacing.sm).fill_main(false).show(ui, |f| {
         f.item(Item::new(), |ui| {
            Flex::row("card_row").gap(theme.spacing.sm).show(ui, |f| {
               f.item(Item::fixed(80.0), |ui| {
                  let width = ui.available_width();
                  ui.add(Button::new("Back").min_size(vec2(width, 0.0)));
               });
               f.item(Item::grow(1.0), |ui| {
                  ui.add(
                     Label::new(
                        RichText::new("A growing middle item between two fixed ones")
                           .color(theme.colors.text_muted),
                        None,
                     )
                     .wrap(),
                  );
               });
               f.item(Item::fixed(80.0), |ui| {
                  let width = ui.available_width();
                  ui.add(Button::new("Next").min_size(vec2(width, 0.0)));
               });
            });
         });
         f.item(Item::new(), |ui| {
            ui.add(
               Label::new(
                  "The row above is one item of this column, so a card can mix both axes.",
                  None,
               )
               .wrap(),
            );
         });
      });
   });
}

/// Tints the item's cell behind its content, so `Stretch` is visible even when
/// the widget itself does not fill the cell.
fn cell(ui: &mut Ui, theme: &Theme, add: impl FnOnce(&mut Ui)) {
   let rect = ui.max_rect();
   ui.painter().rect_filled(rect, theme.spacing.xs, theme.colors.widget_bg);
   add(ui);
}

/// E. The same three items under each cross-axis mode, one row per mode. The
/// tinted box is the item's cell.
fn section_alignment(ui: &mut Ui, theme: &Theme) {
   caption(
      ui,
      theme,
      "E. Alignment modes — the same label, button and tall button under each align_items; the tint is the cell.",
   );

   let modes = [
      ("Start", CrossAlign::Start),
      ("Center", CrossAlign::Center),
      ("End", CrossAlign::End),
      ("Stretch", CrossAlign::Stretch),
   ];

   for (name, mode) in modes {
      ui.label(
         RichText::new(format!("align_items: {name}"))
            .size(theme.typography.small)
            .color(theme.colors.text_muted),
      );
      Flex::row(("align", name))
         .gap(theme.spacing.sm)
         .align_items(mode)
         .show(ui, |f| {
            f.item(Item::new(), |ui| {
               cell(ui, theme, |ui| {
                  ui.add(Label::new("Label", None));
               });
            });
            f.item(Item::new(), |ui| {
               cell(ui, theme, |ui| {
                  ui.add(Button::new("Button").min_size(vec2(96.0, 28.0)));
               });
            });
            f.item(Item::new(), |ui| {
               cell(ui, theme, |ui| {
                  ui.add(Button::new("Tall").min_size(vec2(96.0, 44.0)));
               });
            });
         });
      ui.add_space(theme.spacing.sm);
   }
}
