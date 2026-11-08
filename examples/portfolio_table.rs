//! The `zeus portfolio.rs` case: a header plus one row per token, with the rows in the table's own
//! scroll area so the header stays put.
//!
//! It is the layout that exposed the row-offset bug: a row's cells were placed one slot below its
//! band, so each band looked like an empty row and the content under it belonged to the row above.
//! The divider marks each band's bottom and each row removes *itself*, so a drift shows up as a gap
//! and a mis-hit as the wrong token disappearing.
//!
//! Sixty rows, so the table also has to virtualise — scroll, and only the rows in view are laid out
//! while the header does not move.
//!
//! Run with: `cargo run --example portfolio_table`

use eframe::egui::*;
use egui_elements::layout::{Column, Table, TableScroll};
use egui_elements::theme::{Theme, ThemeKind};
use egui_elements::widgets::{Button, Label};

const TINTS: [Color32; 5] = [
   Color32::from_rgb(0x62, 0x72, 0xa4),
   Color32::from_rgb(0x27, 0x7a, 0xc4),
   Color32::from_rgb(0xf5, 0xac, 0x37),
   Color32::from_rgb(0x82, 0x47, 0xe2),
   Color32::from_rgb(0x2e, 0xb8, 0x7a),
];

struct Token {
   symbol: String,
   price: String,
   balance: String,
   value: String,
   tint: Color32,
}

fn tokens() -> Vec<Token> {
   const SYMBOLS: [&str; 8] = ["ETH", "WETH", "USDC", "DAI", "LINK", "UNI", "AAVE", "ARB"];

   (0..60)
      .map(|i| Token {
         symbol: format!(
            "{} {}",
            SYMBOLS[i % SYMBOLS.len()],
            i / SYMBOLS.len()
         ),
         price: format!("${}.{:02}", 1 + i * 7 % 2400, i % 100),
         balance: format!("{}.{:04}", i % 12, i * 37 % 10_000),
         value: format!("${}.{:02}", i * 3 % 900, i * 11 % 100),
         tint: TINTS[i % TINTS.len()],
      })
      .collect()
}

fn main() -> eframe::Result {
   let options = eframe::NativeOptions {
      viewport: ViewportBuilder::default()
         .with_title("egui-elements portfolio table")
         .with_inner_size([980.0, 720.0])
         .with_resizable(true),
      ..Default::default()
   };

   eframe::run_native(
      "egui-elements portfolio table",
      options,
      Box::new(|cc| Ok(Box::new(PortfolioApp::new(cc)))),
   )
}

struct PortfolioApp {
   theme: Theme,
   tokens: Vec<Token>,
   removed: usize,
}

impl PortfolioApp {
   fn new(cc: &eframe::CreationContext<'_>) -> Self {
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&cc.egui_ctx);

      Self {
         theme,
         tokens: tokens(),
         removed: 0,
      }
   }
}

impl eframe::App for PortfolioApp {
   fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
      let theme = &self.theme;
      let bg = theme.colors.bg;
      let mut remove: Option<usize> = None;

      CentralPanel::default()
         .frame(Frame::new().fill(bg).inner_margin(Margin::same(16)))
         .show(ui, |ui| {
            ui.label(
               RichText::new("PortfolioUi — uniform rows, a divider under each, a pinned header")
                  .size(theme.typography.large)
                  .color(theme.colors.text),
            );
            ui.label(
               RichText::new(format!(
                  "{} tokens, {} removed · each X removes only its own row",
                  self.tokens.len(),
                  self.removed
               ))
               .size(theme.typography.small)
               .color(theme.colors.text_muted),
            );

            ui.add_space(theme.spacing.md);

            Table::new("portfolio_tokens", &self.tokens)
               .columns([
                  Column::Fr(2.2), // Asset
                  Column::Fr(1.8), // Price
                  Column::Fr(1.8), // Balance
                  Column::Fr(1.8), // Value
                  Column::Auto,    // Remove
               ])
               .gap(20.0)
               .row_gap(theme.spacing.md)
               .header_rule(true)
               .row_divider(true)
               .key(self.tokens.len())
               // The table owns the scroll area, so the header above it cannot scroll away.
               .scroll(TableScroll::new())
               .show(ui, |table| {
                  table.header(|header| {
                     for text in ["Asset", "Price", "Balance", "Value", ""] {
                        header.cell(|ui| {
                           if !text.is_empty() {
                              ui.label(
                                 RichText::new(text)
                                    .strong()
                                    .size(theme.typography.large)
                                    .color(theme.colors.text),
                              );
                           }
                        });
                     }
                  });

                  table.rows(|row, token| {
                     let index = row.index();

                     row.cell(|ui| {
                        let (rect, _) = ui.allocate_exact_size(vec2(28.0, 28.0), Sense::hover());
                        ui.painter().circle_filled(rect.center(), 13.0, token.tint);
                        ui.add(Label::new(
                           RichText::new(&token.symbol)
                              .size(theme.typography.normal)
                              .color(theme.colors.text),
                           None,
                        ));
                     });

                     for text in [&token.price, &token.balance, &token.value] {
                        row.cell(|ui| {
                           ui.label(
                              RichText::new(text)
                                 .size(theme.typography.normal)
                                 .color(theme.colors.text),
                           );
                        });
                     }

                     row.cell(|ui| {
                        let button = Button::new(RichText::new("X").size(theme.typography.normal));
                        if ui.add(button).clicked() {
                           remove = Some(index);
                        }
                     });
                  });
               });
         });

      if let Some(index) = remove {
         self.tokens.remove(index);
         self.removed += 1;
      }
   }
}
