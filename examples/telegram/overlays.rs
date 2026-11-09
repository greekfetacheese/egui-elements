//! Overlay surfaces: the New Contact modal, the emoji / sticker / GIF picker
//! and the attachment menu.
//!
//! Every colour comes from the installed theme and every gap from
//! `Theme::spacing`; the floating panels are `Popover`s on the theme's overlay
//! surface, so they read as the same material as the modal card and inherit its
//! outside-click and Escape dismissal. State the panels need across frames lives
//! on `TelegramApp`.

use eframe::egui::*;
use egui_elements::components::InputField;
use egui_elements::egui_lucide::Lucide;
use egui_elements::layout::{CrossAlign, Flex, Item, Justify};
use egui_elements::theme::{Surface, Theme};
use egui_elements::utils::mix;
use egui_elements::widgets::{
   Button, Frame, Modal, Popover, PopoverSide, Spinner, Tabs, TabsStyle,
};

use crate::TelegramApp;
use crate::paint;
use crate::style;

/// How long the mock "Creating…" state runs before the modal closes.
const CREATE_SECONDS: f64 = 0.8;
/// The 42 emoji the picker grid offers, in six rows of seven: one row each
/// for faces, hands, hearts, animals, food, and objects / symbols.
const EMOJI: [&str; 42] = [
   "😀", "😂", "🥰", "😎", "🤔", "😴", "🤯", "👍", "👎", "👏", "🙌", "🤝", "👋", "✌️", "❤️", "🧡",
   "💛", "💚", "💙", "💜", "🖤", "🐶", "🐱", "🦊", "🐼", "🐸", "🦉", "🐢", "🍎", "🍕", "🍔", "🍩",
   "🍓", "☕", "🍺", "⚽", "🚀", "🎸", "💡", "📷", "🎁", "✨",
];
/// Bottom category strip; the first entry is the active one.
const CATEGORIES: [Lucide; 8] = [
   Lucide::Clock,
   Lucide::FaceGrinning,
   Lucide::Cat,
   Lucide::Apple,
   Lucide::Volleyball,
   Lucide::Car,
   Lucide::Lightbulb,
   Lucide::Heart,
];
/// Sticker tiles: glyph and caption.
const STICKERS: [(&str, &str); 3] = [
   ("🦊", "Cheering fox"),
   ("🐼", "Panda wave"),
   ("🚀", "Rocket"),
];
/// Attachment menu entries: icon and label.
const ATTACH: [(Lucide, &str); 4] = [
   (Lucide::Image, "Photo"),
   (Lucide::FileText, "File"),
   (Lucide::ListChecks, "Poll"),
   (Lucide::MapPin, "Location"),
];

/// Which picker tab is showing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EmojiTab {
   /// The glyph grid.
   #[default]
   Emoji,
   /// The sticker tiles.
   Stickers,
   /// The GIF tiles.
   Gifs,
}

impl EmojiTab {
   /// Tabs in display order.
   const ALL: [EmojiTab; 3] = [EmojiTab::Emoji, EmojiTab::Stickers, EmojiTab::Gifs];
}

/// Draw every overlay surface on top of the shell.
pub fn show(app: &mut TelegramApp, ctx: &Context) {
   new_contact(app, ctx);
   emoji_panel(app, ctx);
   attach_menu(app, ctx);
}

// ---------------------------------------------------------------------------
// New Contact
// ---------------------------------------------------------------------------

/// The New Contact dialog, with its mock creating state driven from the footer.
fn new_contact(app: &mut TelegramApp, ctx: &Context) {
   if !app.new_contact_open {
      app.creating_since = None;
      return;
   }

   // The mock create runs for a fixed beat, then closes and clears the form.
   if let Some(started) = app.creating_since {
      if ctx.input(|i| i.time) - started >= CREATE_SECONDS {
         app.new_contact_open = false;
         app.creating_since = None;
         reset_contact(app);
         ctx.request_repaint();
         return;
      }
      ctx.request_repaint();
   }

   let theme = app.theme.clone();
   let colors = theme.colors;
   let space = theme.spacing;
   let pending = app.creating_since.is_some();
   let can_create = !app.contact_first.text().trim().is_empty() && !pending;
   // The disabled CREATE is a painted-down accent, not just reduced alpha:
   // an alpha-dimmed solid accent still read as enabled on the dark palettes.
   let enabled_fill = colors.accent;
   let enabled_ink = colors.ink_on(enabled_fill);
   let fill = if can_create {
      enabled_fill
   } else {
      mix(enabled_fill, colors.bg, 0.55)
   };
   let ink = if can_create {
      enabled_ink
   } else {
      mix(enabled_ink, fill, 0.35)
   };
   let primary = theme.button_visuals().filled(fill, colors);

   let mut open = app.new_contact_open;
   let mut cancel = false;
   let mut create = false;

   let heading = RichText::new("New Contact").size(20.0).color(colors.text);
   Modal::new("new_contact", &mut open)
      .heading(heading)
      .width(style::MODAL_W)
      .footer(|ui| {
         Flex::row("contact_footer")
            .justify(Justify::End)
            .gap(space.sm)
            .align_items(CrossAlign::Center)
            .show(ui, |f| {
               f.item(Item::new(), |ui| {
                  let button = Button::new("CANCEL").small().min_size(vec2(96.0, 32.0));
                  if ui.add(button).clicked() {
                     cancel = true;
                  }
               });
               f.item(Item::new(), |ui| {
                  if pending {
                     let (rect, _) = ui.allocate_exact_size(vec2(104.0, 32.0), Sense::hover());
                     let spinner = Rect::from_center_size(
                        pos2(rect.left() + 7.0, rect.center().y),
                        Vec2::splat(14.0),
                     );
                     Spinner::new().size(14.0).paint_at(ui, spinner);
                     ui.painter().text(
                        pos2(spinner.right() + space.xs, rect.center().y),
                        Align2::LEFT_CENTER,
                        "Creating…",
                        FontId::proportional(13.0),
                        colors.text_muted,
                     );
                  } else {
                     let button = Button::new(RichText::new("CREATE").size(13.0).color(ink))
                        .visuals(primary)
                        .min_size(vec2(104.0, 32.0))
                        .sense(if can_create {
                           Sense::click()
                        } else {
                           Sense::hover()
                        });
                     if ui.add(button).clicked() {
                        create = true;
                     }
                  }
               });
            });
      })
      .show(ctx, |ui| contact_body(app, ui));

   if create {
      app.creating_since = Some(ctx.input(|i| i.time));
      ctx.request_repaint();
   }
   if cancel {
      open = false;
   }
   if !open {
      reset_contact(app);
   }
   app.new_contact_open = open;
}

/// Avatar slot on the left, the three fields stacked on the right.
fn contact_body(app: &mut TelegramApp, ui: &mut Ui) {
   let theme = app.theme.clone();
   let space = theme.spacing;

   Flex::row("contact_body")
      .gap(space.lg)
      .align_items(CrossAlign::Start)
      .show(ui, |f| {
         f.item(Item::fixed(style::AVATAR_PROFILE), |ui| {
            avatar_slot(ui, &theme)
         });
         f.item(Item::grow(1.0).stack(), |ui| {
            ui.spacing_mut().item_spacing.y = space.md;
            field(ui, &theme, &mut app.contact_first);
            field(ui, &theme, &mut app.contact_last);
            field(ui, &theme, &mut app.contact_phone);
         });
      });
}

/// A plain circular avatar placeholder with a `User` glyph.
fn avatar_slot(ui: &mut Ui, theme: &Theme) {
   let colors = theme.colors;
   let (rect, _) = ui.allocate_exact_size(Vec2::splat(style::AVATAR_PROFILE), Sense::hover());
   let painter = ui.painter();
   let center = rect.center();
   let radius = style::AVATAR_PROFILE * 0.5;
   painter.circle_filled(center, radius, colors.widget_bg);
   painter.circle_stroke(
      center,
      radius - 0.5,
      Stroke::new(1.0, colors.border),
   );
   Lucide::User.size(32.0).color(colors.text_muted).image().paint_at(
      ui,
      Rect::from_center_size(center, Vec2::splat(32.0)),
   );
}

/// One stored `InputField` drawn at the form's width.
fn field(ui: &mut Ui, theme: &Theme, value: &mut InputField) {
   value.set_min_size(vec2(ui.available_width(), style::CONTACT_FIELD_H));
   value.set_inner_margin(theme.spacing.margin(theme.spacing.sm));
   let _ = value.show(ui);
}

/// Close the form and restore its starting state.
fn reset_contact(app: &mut TelegramApp) {
   app.contact_first.set_text("");
   app.contact_last.set_text("");
   app.contact_phone.set_text("+49 ");
   app.creating_since = None;
}

// ---------------------------------------------------------------------------
// Emoji / stickers / GIFs
// ---------------------------------------------------------------------------

/// The picker, anchored above the composer's smiley.
fn emoji_panel(app: &mut TelegramApp, ctx: &Context) {
   let theme = app.theme.clone();
   let space = theme.spacing;
   let anchor = app.emoji_anchor;
   let mut open = app.emoji_open;
   let mut tab = app.emoji_tab;
   let composer = &mut app.composer;

   let _ = Popover::new("emoji_panel", &mut open)
      .side(PopoverSide::Above)
      .gap(space.sm)
      .frame(panel_frame(&theme, style::PANEL_RADIUS))
      .show(ctx, anchor, |ui| {
         ui.set_width(style::EMOJI_PANEL_W);
         ui.set_height(style::EMOJI_PANEL_H);
         ui.spacing_mut().item_spacing = Vec2::ZERO;

         // The category strip owns the bottom edge; the grid gets exactly the
         // band between the tabs and the strip, so no dead row opens up between
         // them.
         let content = ui.max_rect();
         let strip = Rect::from_min_max(
            pos2(
               content.left(),
               content.bottom() - style::EMOJI_STRIP_H,
            ),
            content.max,
         );
         let body = Rect::from_min_max(
            pos2(content.left(), content.top() + style::EMOJI_TAB_H),
            pos2(content.right(), strip.top()),
         );

         tabs(&mut tab, ui);
         match tab {
            EmojiTab::Emoji => emoji_grid(composer, ui, body, &theme),
            EmojiTab::Stickers => stickers(ui, body, &theme),
            EmojiTab::Gifs => gifs(ui, body, &theme),
         }
         categories(ui, &theme, strip);
      });

   app.emoji_tab = tab;
   app.emoji_open = open;
}

/// The three page tabs, the active one marked by the crate's travelling underline.
fn tabs(selected: &mut EmojiTab, ui: &mut Ui) {
   let mut index = *selected as usize;
   let response = Tabs::new(
      "emoji_tabs",
      &mut index,
      ["EMOJI", "STICKERS", "GIFS"],
   )
   .style(TabsStyle::Underline)
   .height(style::EMOJI_TAB_H)
   .show(ui);

   if response.changed() {
      *selected = EmojiTab::ALL[index];
   }
}

/// The glyph grid, seven columns wide, filling the band above the strip.
fn emoji_grid(composer: &mut String, ui: &mut Ui, body: Rect, theme: &Theme) {
   let space = theme.spacing;
   let rows = EMOJI.len().div_ceil(style::EMOJI_COLS).max(1);

   ui.scope_builder(
      UiBuilder::new().max_rect(body).layout(Layout::top_down(Align::Min)),
      |ui| {
         ScrollArea::vertical()
            .id_salt("emoji_grid")
            .auto_shrink([false, true])
            .show(ui, |ui| {
               ui.spacing_mut().item_spacing = Vec2::ZERO;
               let pad = space.md;
               let width = ui.available_width();
               let cell_w = ((width - pad * 2.0) / style::EMOJI_COLS as f32).max(1.0);
               // Spread the rows over the whole band: the grid reaches the strip
               // instead of stopping two rows early.
               let cell_h =
                  ((body.height() - space.sm * 2.0) / rows as f32).max(style::EMOJI_CELL_H);

               ui.add_space(space.sm);
               for row in EMOJI.chunks(style::EMOJI_COLS) {
                  ui.horizontal(|ui| {
                     ui.add_space(pad);
                     for emoji in row {
                        emoji_cell(composer, ui, emoji, cell_w, cell_h, theme);
                     }
                  });
               }
               ui.add_space(space.sm);
            });
      },
   );
}

/// One grid cell: a rounded hover plate under the glyph, click to append.
fn emoji_cell(
   composer: &mut String,
   ui: &mut Ui,
   emoji: &'static str,
   width: f32,
   height: f32,
   theme: &Theme,
) {
   let colors = theme.colors;
   let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::click());
   if !ui.is_rect_visible(rect) {
      return;
   }

   let plate = Rect::from_center_size(
      rect.center(),
      vec2(style::EMOJI_CELL_W, style::EMOJI_CELL_H),
   );
   if response.hovered() {
      ui.painter().rect_filled(
         plate,
         CornerRadius::same(style::EMOJI_RADIUS),
         colors.hover,
      );
      ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
   }
   ui.painter().text(
      rect.center(),
      Align2::CENTER_CENTER,
      emoji,
      FontId::proportional(style::EMOJI_GLYPH),
      colors.text,
   );
   if response.clicked() {
      composer.push_str(emoji);
   }
}

/// Sticker tiles: a big glyph and a muted caption.
fn stickers(ui: &mut Ui, area: Rect, theme: &Theme) {
   let colors = theme.colors;
   let space = theme.spacing;
   let gap = space.md;
   let total = STICKERS.len() as f32 * style::STICKER_TILE_W + (STICKERS.len() as f32 - 1.0) * gap;
   let left = area.center().x - total * 0.5;
   let top = area.center().y - style::STICKER_TILE_H * 0.5;

   for (index, (glyph, caption)) in STICKERS.iter().enumerate() {
      let rect = Rect::from_min_size(
         pos2(
            left + index as f32 * (style::STICKER_TILE_W + gap),
            top,
         ),
         vec2(style::STICKER_TILE_W, style::STICKER_TILE_H),
      );
      let response = ui.interact(
         rect,
         ui.scope_id().with(("sticker_tile", index)),
         Sense::click(),
      );
      if !ui.is_rect_visible(rect) {
         continue;
      }
      let fill = if response.hovered() {
         colors.hover
      } else {
         colors.widget_bg
      };
      ui.painter().rect_filled(
         rect,
         CornerRadius::same(style::STICKER_TILE_RADIUS),
         fill,
      );
      ui.painter().text(
         pos2(rect.center().x, rect.center().y - space.sm),
         Align2::CENTER_CENTER,
         *glyph,
         FontId::proportional(style::STICKER_SIZE),
         colors.text,
      );
      ui.painter().text(
         pos2(rect.center().x, rect.bottom() - space.lg),
         Align2::CENTER_CENTER,
         *caption,
         FontId::proportional(12.0),
         colors.text_muted,
      );
   }
}

/// Four GIF tiles with a play glyph; clicking is a no-op in the demo.
fn gifs(ui: &mut Ui, area: Rect, theme: &Theme) {
   let colors = theme.colors;
   let space = theme.spacing;
   let gap = space.md;
   let total = 4.0 * style::GIF_TILE_W + 3.0 * gap;
   let left = area.center().x - total * 0.5;
   let top = area.center().y - style::GIF_TILE_H * 0.5;

   for index in 0..4usize {
      let rect = Rect::from_min_size(
         pos2(
            left + index as f32 * (style::GIF_TILE_W + gap),
            top,
         ),
         vec2(style::GIF_TILE_W, style::GIF_TILE_H),
      );
      let response = ui.interact(
         rect,
         ui.scope_id().with(("gif_tile", index)),
         Sense::click(),
      );
      if !ui.is_rect_visible(rect) {
         continue;
      }
      let fill = if response.hovered() {
         colors.hover
      } else {
         colors.widget_bg
      };
      ui.painter().rect_filled(
         rect,
         CornerRadius::same(style::GIF_TILE_RADIUS),
         fill,
      );
      Lucide::Play.size(20.0).color(colors.text_muted).image().paint_at(
         ui,
         Rect::from_center_size(
            pos2(rect.center().x, rect.center().y - space.xs),
            Vec2::splat(20.0),
         ),
      );
      ui.painter().text(
         pos2(rect.center().x, rect.bottom() - space.md),
         Align2::CENTER_CENTER,
         "GIF",
         FontId::proportional(12.0),
         colors.text_muted,
      );
   }
}

/// The category strip: eight icon buttons pinned to the panel's bottom edge.
fn categories(ui: &mut Ui, theme: &Theme, rect: Rect) {
   let colors = theme.colors;
   let space = theme.spacing;
   let size = 20.0 + space.sm * 2.0;
   let gap = space.xs;
   let total = CATEGORIES.len() as f32 * size + (CATEGORIES.len() as f32 - 1.0) * gap;
   let mut x = rect.center().x - total * 0.5;
   let y = rect.center().y - size * 0.5;

   for (index, icon) in CATEGORIES.iter().enumerate() {
      let cell = Rect::from_min_size(pos2(x, y), Vec2::splat(size));
      ui.scope_builder(UiBuilder::new().max_rect(cell), |ui| {
         // The crate button adds its padding to the icon; match the 36 pt cell.
         ui.spacing_mut().button_padding = Vec2::splat(space.sm);
         let ink = if index == 0 {
            colors.accent
         } else {
            colors.text_muted
         };
         let _ = paint::icon_button(ui, *icon, 20.0, ink);
      });
      x += size + gap;
   }
}

// ---------------------------------------------------------------------------
// Attachment menu
// ---------------------------------------------------------------------------

/// The paperclip menu, anchored above the paperclip in the composer.
fn attach_menu(app: &mut TelegramApp, ctx: &Context) {
   let theme = app.theme.clone();
   let colors = theme.colors;
   let space = theme.spacing;
   let anchor = app.attach_anchor;
   let mut open = app.attach_open;

   let picked = Popover::new("attach_menu", &mut open)
      .side(PopoverSide::Above)
      .gap(space.sm)
      .frame(panel_frame(&theme, style::ATTACH_RADIUS as f32).inner_margin(space.margin(space.sm)))
      .show(ctx, anchor, |ui| {
         ui.set_width(style::ATTACH_W - space.sm * 2.0);
         ui.spacing_mut().item_spacing = Vec2::ZERO;
         let mut picked = false;

         for (icon, label) in ATTACH {
            let (rect, response) = ui.allocate_exact_size(
               vec2(ui.available_width(), style::ATTACH_ROW_H),
               Sense::click(),
            );
            if response.hovered() {
               ui.painter().rect_filled(
                  rect,
                  CornerRadius::same(style::EMOJI_RADIUS),
                  colors.hover,
               );
            }
            let icon_rect = Rect::from_center_size(
               pos2(rect.left() + space.sm + 9.0, rect.center().y),
               Vec2::splat(18.0),
            );
            icon.size(18.0).color(colors.text_muted).image().paint_at(ui, icon_rect);
            ui.painter().text(
               pos2(icon_rect.right() + space.sm, rect.center().y),
               Align2::LEFT_CENTER,
               label,
               FontId::proportional(14.0),
               colors.text,
            );
            if response.clicked() {
               picked = true;
            }
         }

         picked
      })
      .map(|shown| shown.inner)
      .unwrap_or(false);

   app.attach_open = open && !picked;
}

/// The shared floating-panel chrome: the theme's overlay surface on the demo's
/// own corner radius.
fn panel_frame(theme: &Theme, radius: f32) -> Frame {
   Frame::from_egui(theme.surface_frame(Surface::Overlay))
      .inner_margin(Margin::ZERO)
      .corner_radius(CornerRadius::same(radius as u8))
}
