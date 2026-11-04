//! The live theme audit: what palette this demo is actually drawing with.
//!
//! One panel, five sections: a summary of the installed palette, a swatch for
//! every crate ThemeColors slot, the crate's own contrast report
//! (ThemeColors::audit) as a table, the same mini gallery on bg, frame1 and
//! frame2, and a map from each slot to the place this demo spends it.
//!
//! The panel reports the palette installed in this window right now, so it
//! moves with the theme switcher and the editor instead of describing a
//! hard-coded default.

use eframe::egui::scroll_area::ScrollBarVisibility;
use eframe::egui::*;
use egui_elements::egui_lucide::Lucide;
use egui_elements::layout::{Column, CrossAlign, Flex, Item, Table};
use egui_elements::theme::{ContrastCheck, Theme};
use egui_elements::widgets::{
   Button, Checkbox, Chip, ChipRole, ComboBox, Divider, Label, Radio, Scroll, Switch, Window,
};

use crate::TelegramApp;
use crate::style;

/// Every ThemeColors slot, in struct order, with the job this demo gives it.
const SLOT_USAGE: [(&str, &str); 13] = [
   ("title_bar", "title strip"),
   ("bg", "panes, pane headers, composer band"),
   (
      "widget_bg",
      "incoming bubbles, search pill, composer field, day pill",
   ),
   ("hover", "chat rows, icon buttons, settings rows"),
   ("text", "titles, message bodies, form labels"),
   ("text_muted", "times, hints, previews, meta lines"),
   ("highlight", "the selected chat row"),
   (
      "border",
      "hairlines, field outlines, bubble edges",
   ),
   (
      "accent",
      "unread badge, CREATE, active tab, quote bars",
   ),
   ("error", "failed-message marker, draft preview"),
   ("warning", "pinned chat, storage value"),
   ("success", "online presence, read ticks"),
   ("info", "Export Telegram data link"),
];

/// The demo's theme choices for the probe combo boxes.
const PROBE_OPTIONS: [&str; 3] = ["Default", "Compact", "Large"];

/// Interactive state one surface's mini gallery keeps between frames.
#[derive(Clone, Copy, Debug, Default)]
struct ProbeState {
   /// Checkbox value.
   checked: bool,
   /// Switch value.
   switched: bool,
   /// Selected index of the two-radio group.
   radio: usize,
   /// Selected index of the ComboBox.
   combo: usize,
}

/// Draw the audit window when it is open.
pub fn show(app: &mut TelegramApp, ctx: &Context) {
   if !app.audit_open {
      return;
   }

   let theme = app.theme.clone();
   let space = theme.spacing;
   let mut open = app.audit_open;

   Window::new("Theme audit")
      .open(&mut open)
      .frame(theme.window_frame)
      .title_frame(theme.window_frame.stroke(Stroke::NONE))
      .default_width(style::AUDIT_W)
      .max_height(style::AUDIT_H)
      // The title strip owns the top edge; tucking the window under it keeps
      // both the strip and the palette controls reachable.
      .anchor(
         Align2::RIGHT_TOP,
         vec2(-space.lg, style::TITLE_H + space.sm),
      )
      .resizable([true, true])
      .collapsible(false)
      .show(ctx, |ui| {
         ui.set_width(ui.available_width());
         ui.set_max_height(style::AUDIT_BODY_H);
         ui.spacing_mut().item_spacing.y = space.md;
         Scroll::vertical("theme_audit")
            .bars(ScrollBarVisibility::AlwaysVisible)
            .auto_shrink([false, true])
            .show(ui, |ui| {
               ui.spacing_mut().item_spacing.y = space.md;
               summary(ui, &theme);
               swatches(ui, &theme);
               contrast_report(ui, &theme);
               surfaces(ui, &theme);
               slot_map(ui, &theme);
            });
      });

   app.audit_open = open;
}

/// Installed palette, its kind and how many contrast checks it fails.
fn summary(ui: &mut Ui, theme: &Theme) {
   let colors = theme.colors;
   let space = theme.spacing;
   let checks = theme.colors.audit();
   let failing = checks.iter().filter(|check| !check.passes()).count();

   section_heading(ui, theme, "Installed palette", None);
   Flex::row("audit_summary")
      .gap(space.sm)
      .align_items(CrossAlign::Center)
      .show(ui, |f| {
         f.item(Item::new(), |ui| {
            ui.add(
               Label::new(
                  RichText::new(theme.kind.to_str()).size(18.0).color(colors.text),
                  None,
               )
               .interactive(false),
            );
         });
         f.item(Item::new(), |ui| {
            let kind = if theme.dark { "Dark" } else { "Light" };
            ui.add(Chip::new(kind).role(ChipRole::Neutral));
         });
         f.item(Item::new(), |ui| {
            ui.add(
               Label::sections(
                  [
                     RichText::new(format!("{} checks · ", checks.len()))
                        .size(12.5)
                        .color(colors.text_muted),
                     RichText::new(format!("{failing} failing")).size(12.5).color(colors.error),
                  ],
                  None,
               )
               .interactive(false),
            );
         });
      });
   ui.add_space(space.xs);
   ui.add(
      Label::new(
         RichText::new(
            "Reads the palette installed in this window: the switcher and the \
             editor update it live.",
         )
         .size(12.0)
         .color(colors.text_muted),
         None,
      )
      .wrap()
      .interactive(false),
   );
}

/// One tile per ThemeColors slot, three to a row, in struct order.
fn swatches(ui: &mut Ui, theme: &Theme) {
   let colors = theme.colors;
   let space = theme.spacing;
   let slots: [(&str, Color32); 13] = [
      ("title_bar", colors.title_bar),
      ("bg", colors.bg),
      ("widget_bg", colors.widget_bg),
      ("hover", colors.hover),
      ("text", colors.text),
      ("text_muted", colors.text_muted),
      ("highlight", colors.highlight),
      ("border", colors.border),
      ("accent", colors.accent),
      ("error", colors.error),
      ("warning", colors.warning),
      ("success", colors.success),
      ("info", colors.info),
   ];

   section_heading(
      ui,
      theme,
      "Colour slots",
      Some("Fill is the slot; ink is colors.ink_on(slot)."),
   );

   let rows = slots.len().div_ceil(4);
   for (row, chunk) in slots.chunks(4).enumerate() {
      Flex::row(("audit_swatch_row", row))
         .gap(space.sm)
         .align_items(CrossAlign::Stretch)
         .show(ui, |f| {
            for (name, color) in chunk {
               f.item(Item::grow(1.0), |ui| {
                  swatch(ui, theme, name, *color)
               });
            }
            for _ in chunk.len()..4 {
               f.item(Item::grow(1.0), |_ui| {});
            }
         });
      if row + 1 < rows {
         ui.add_space(space.sm);
      }
   }
}

/// One slot tile: the fill, its name and its hex, all in ink_on of the fill.
fn swatch(ui: &mut Ui, theme: &Theme, name: &str, color: Color32) {
   let colors = theme.colors;
   let space = theme.spacing;
   let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 50.0), Sense::hover());
   if !ui.is_rect_visible(rect) {
      return;
   }

   let ink = colors.ink_on(color);
   let painter = ui.painter();
   painter.rect_filled(rect, CornerRadius::same(style::ROW_RADIUS), color);
   painter.rect_stroke(
      rect,
      CornerRadius::same(style::ROW_RADIUS),
      Stroke::new(1.0, colors.border),
      StrokeKind::Inside,
   );
   painter.text(
      pos2(rect.left() + space.sm, rect.top() + space.sm),
      Align2::LEFT_TOP,
      name,
      FontId::proportional(11.0),
      ink,
   );
   painter.text(
      pos2(rect.left() + space.sm, rect.bottom() - space.sm),
      Align2::LEFT_BOTTOM,
      hex(color),
      FontId::monospace(10.5),
      ink,
   );
}

/// The crate's ThemeColors::audit report as a data-driven table.
fn contrast_report(ui: &mut Ui, theme: &Theme) {
   let colors = theme.colors;
   let space = theme.spacing;
   let mut checks: Vec<ContrastCheck> = theme.colors.audit();
   // Failing pairs first; otherwise the order audit documents.
   checks.sort_by_key(|check| check.passes());

   section_heading(
      ui,
      theme,
      "Contrast report",
      Some("WCAG 2.2: text needs 4.5:1, and every pair carries its own minimum."),
   );

   Table::new("theme_audit_contrast", &checks)
      .columns([
         Column::Fr(3.0),
         Column::Fixed(56.0),
         Column::Fixed(48.0),
         Column::Fixed(64.0),
      ])
      .gap(space.md)
      .row_gap(space.sm)
      .header_rule(true)
      .key(theme.kind.to_str())
      .show(ui, |t| {
         t.header(|h| {
            for title in ["Pair", "Ratio", "Needs", "Verdict"] {
               h.cell(|ui| {
                  ui.add(
                     Label::new(
                        RichText::new(title).size(12.0).strong().color(colors.text),
                        None,
                     )
                     .interactive(false),
                  );
               });
            }
         });
         t.rows(|r, check| {
            let pass = check.passes();
            let verdict = if pass { colors.success } else { colors.error };
            r.cell(|ui| {
               ui.add(
                  Label::new(
                     RichText::new(check.label).size(12.0).color(colors.text),
                     None,
                  )
                  .interactive(false)
                  .wrap_mode(TextWrapMode::Truncate),
               );
            });
            r.cell(|ui| {
               ui.add(
                  Label::new(
                     RichText::new(format!("{:.2}:1", check.ratio)).size(12.0).color(verdict),
                     None,
                  )
                  .interactive(false),
               );
            });
            r.cell(|ui| {
               ui.add(
                  Label::new(
                     RichText::new(format!("{:.1}", check.required))
                        .size(12.0)
                        .color(colors.text_muted),
                     None,
                  )
                  .interactive(false),
               );
            });
            r.cell(|ui| {
               let chip = if pass {
                  Chip::new("AA").role(ChipRole::Success)
               } else {
                  Chip::new("FAIL").role(ChipRole::Error)
               };
               ui.add(chip.padding(vec2(space.xs, 1.0)));
            });
         });
      });
}

/// The same mini gallery on the canvas, a frame1 panel and a nested frame2 well.
fn surfaces(ui: &mut Ui, theme: &Theme) {
   let colors = theme.colors;
   let space = theme.spacing;

   section_heading(
      ui,
      theme,
      "Surface probes",
      Some("One gallery, three fills: bg, then frame1, then frame2 inside frame1."),
   );

   Frame::new()
      .fill(colors.bg)
      .stroke(Stroke::new(1.0, colors.border))
      .corner_radius(CornerRadius::same(style::PANEL_RADIUS as u8))
      .inner_margin(space.margin(space.md))
      .show(ui, |ui| {
         ui.set_width(ui.available_width());
         gallery(ui, theme, "bg");
      });

   ui.add_space(space.sm);

   theme.frame1.show(ui, |ui| {
      ui.set_width(ui.available_width());
      gallery(ui, theme, "frame1");
      ui.add_space(space.md);
      theme.frame2.show(ui, |ui| {
         ui.set_width(ui.available_width());
         gallery(ui, theme, "frame2");
      });
   });
}

/// One surface's mini gallery; salt keeps its widget ids apart per surface.
fn gallery(ui: &mut Ui, theme: &Theme, salt: &'static str) {
   let colors = theme.colors;
   let space = theme.spacing;
   let state_id = Id::unique(("theme_audit_probe", salt));
   let mut state = ui.ctx().data(|data| data.get_temp::<ProbeState>(state_id)).unwrap_or_default();

   ui.push_id(("theme_audit_surface", salt), |ui| {
      ui.spacing_mut().item_spacing.y = space.sm;
      surface_caption(ui, theme, salt);

      Flex::row(("audit_probe_actions", salt))
         .gap(space.sm)
         .align_items(CrossAlign::Center)
         .show(ui, |f| {
            f.item(Item::new(), |ui| {
               ui.add(Button::new("Button"));
            });
            f.item(Item::new(), |ui| {
               ui.add(Button::new("Selected").selected(true));
            });
            f.item(Item::new(), |ui| {
               ui.add(Checkbox::new(&mut state.checked, "Checkbox"));
            });
            f.item(Item::new(), |ui| {
               ui.add(Switch::new(&mut state.switched, "Switch"));
            });
         });

      Flex::row(("audit_probe_choices", salt))
         .gap(space.md)
         .align_items(CrossAlign::Center)
         .show(ui, |f| {
            for (index, label) in ["Radio", "Other"].into_iter().enumerate() {
               f.item(Item::new(), |ui| {
                  ui.add(
                     Radio::new(
                        &mut state.radio,
                        index,
                        RichText::new(label).size(13.0).color(colors.text),
                     )
                     .size(16.0),
                  );
               });
            }
            f.item(Item::new(), |ui| {
               probe_combo(ui, theme, salt, &mut state.combo)
            });
            f.item(Item::new(), |ui| {
               ui.add(Chip::new("Chip").role(ChipRole::Accent));
            });
         });

      ui.add(Divider::new());
      ui.add(
         Label::new(
            RichText::new("Muted label on this surface").size(12.0).color(colors.text_muted),
            None,
         )
         .interactive(false),
      );
   });

   ui.ctx().data_mut(|data| data.insert_temp(state_id, state));
}

/// The surface name and its fill, so the probe says which rung it is testing.
fn surface_caption(ui: &mut Ui, theme: &Theme, salt: &str) {
   let colors = theme.colors;
   let (name, fill) = match salt {
      "bg" => ("bg · Canvas", theme.colors.bg),
      "frame1" => ("frame1 · Panel", theme.frame1.fill),
      _ => ("frame2 · Well", theme.frame2.fill),
   };
   Flex::row(("audit_probe_caption", salt))
      .gap(theme.spacing.sm)
      .align_items(CrossAlign::Center)
      .show(ui, |f| {
         f.item(Item::new(), |ui| {
            ui.add(
               Label::new(
                  RichText::new(name).size(11.0).color(colors.text_muted),
                  None,
               )
               .interactive(false),
            );
         });
         f.item(Item::new(), |ui| {
            ui.add(
               Label::new(
                  RichText::new(hex(fill)).size(11.0).color(colors.text_muted),
                  None,
               )
               .interactive(false),
            );
         });
      });
}

/// A compact probe combo box; its own id_salt keeps each surface's popup apart.
fn probe_combo(ui: &mut Ui, theme: &Theme, salt: &'static str, selected: &mut usize) {
   let colors = theme.colors;
   let space = theme.spacing;
   let label = Label::new(
      RichText::new(PROBE_OPTIONS[*selected]).size(13.0).color(colors.text),
      None,
   );

   let _ = ComboBox::new(("probe_combo", salt), label).width(132.0).show_ui(ui, |ui| {
      for (index, name) in PROBE_OPTIONS.iter().enumerate() {
         let row = Label::new(
            RichText::new(*name).size(13.0).color(colors.text),
            None,
         )
         .expand(Some(space.xs))
         .selected(*selected == index)
         .sense(Sense::click())
         .fill_width(true);
         if ui.add(row).clicked() {
            *selected = index;
         }
      }
   });
}

/// The 13 slots and the one place this demo spends each of them.
fn slot_map(ui: &mut Ui, theme: &Theme) {
   let colors = theme.colors;
   let space = theme.spacing;

   section_heading(
      ui,
      theme,
      "Slot usage map",
      Some("Every slot below has a real call site in this demo."),
   );

   for (name, usage) in SLOT_USAGE {
      Flex::row(("audit_slot", name))
         .gap(space.sm)
         .align_items(CrossAlign::Center)
         .show(ui, |f| {
            f.item(Item::fixed(14.0), |ui| {
               ui.add(Lucide::Check.size(12.0).color(colors.success).image());
            });
            f.item(Item::fixed(96.0), |ui| {
               ui.add(
                  Label::new(
                     RichText::new(name).size(12.0).color(colors.text),
                     None,
                  )
                  .interactive(false),
               );
            });
            f.item(Item::grow(1.0), |ui| {
               ui.add(
                  Label::new(
                     RichText::new(usage).size(12.0).color(colors.text_muted),
                     None,
                  )
                  .interactive(false)
                  .wrap_mode(TextWrapMode::Truncate),
               );
            });
         });
   }
}

/// A section title plus its one-line note.
fn section_heading(ui: &mut Ui, theme: &Theme, title: &str, note: Option<&str>) {
   let colors = theme.colors;
   let space = theme.spacing;

   ui.add_space(space.lg);
   ui.add(
      Label::new(
         RichText::new(title).size(14.0).strong().color(colors.text),
         None,
      )
      .interactive(false),
   );
   if let Some(note) = note {
      ui.add_space(space.xs);
      ui.add(
         Label::new(
            RichText::new(note).size(11.5).color(colors.text_muted),
            None,
         )
         .wrap()
         .interactive(false),
      );
   }
   ui.add_space(space.sm);
}

/// #rrggbb for a palette slot.
fn hex(color: Color32) -> String {
   format!(
      "#{:02x}{:02x}{:02x}",
      color.r(),
      color.g(),
      color.b()
   )
}
