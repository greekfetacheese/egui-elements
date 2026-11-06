//! Overlay surfaces: the New Contact modal, the emoji / sticker / GIF picker
//! and the attachment menu.
//!
//! Every colour comes from the installed theme and every gap from
//! `Theme::spacing`; the floating panels are `Area`s with the window frame's
//! fill, a hairline and a soft shadow, so they read as the same material as
//! the modal card. State the panels need across frames lives on
//! `TelegramApp`.

use eframe::egui::*;
use egui_elements::components::InputField;
use egui_elements::egui_lucide::Lucide;
use egui_elements::layout::{CrossAlign, Flex, Item, Justify};
use egui_elements::theme::Theme;
use egui_elements::utils::mix;
use egui_elements::widgets::{Button, Modal, Spinner};

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

   /// Uppercase tab label.
   fn label(self) -> &'static str {
      match self {
         EmojiTab::Emoji => "EMOJI",
         EmojiTab::Stickers => "STICKERS",
         EmojiTab::Gifs => "GIFS",
      }
   }
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
   let can_create = !app.contact_first.trim().is_empty() && !pending;
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
            field(ui, &theme, "First Name", &mut app.contact_first);
            field(ui, &theme, "Last Name", &mut app.contact_last);
            field(ui, &theme, "Phone Number", &mut app.contact_phone);
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

/// One owning `InputField` bound to a host `String`.
fn field(ui: &mut Ui, theme: &Theme, name: &'static str, value: &mut String) {
   let width = ui.available_width();
   let mut field = InputField::new(name, true)
      .with_min_size(vec2(width, style::CONTACT_FIELD_H))
      .inner_margin(theme.spacing.margin(theme.spacing.sm))
      .qr_enabled(false);
   field.set_text(value);
   let _ = field.show(ui);
   *value = field.text().to_owned();
}

/// Close the form and restore its starting state.
fn reset_contact(app: &mut TelegramApp) {
   app.contact_first.clear();
   app.contact_last.clear();
   app.contact_phone = "+49 ".to_owned();
   app.creating_since = None;
}

// ---------------------------------------------------------------------------
// Emoji / stickers / GIFs
// ---------------------------------------------------------------------------

/// The picker, anchored above the composer on the conversation side.
fn emoji_panel(app: &mut TelegramApp, ctx: &Context) {
   let was_open = sync_open(ctx, "emoji_panel_open", app.emoji_open);
   if !app.emoji_open {
      return;
   }

   let theme = app.theme.clone();
   let space = theme.spacing;
   let screen = ctx.content_rect();
   let origin = pos2(
      screen.right() - space.md - style::EMOJI_PANEL_W,
      screen.bottom() - style::COMPOSER_H - space.sm - style::EMOJI_PANEL_H,
   );

   let area = Area::new(Id::unique("emoji_panel"))
      .order(Order::Foreground)
      .fixed_pos(origin)
      .show(ctx, |ui| {
         panel_frame(&theme, style::PANEL_RADIUS).show(ui, |ui| {
            ui.set_width(style::EMOJI_PANEL_W);
            ui.set_height(style::EMOJI_PANEL_H);
            ui.spacing_mut().item_spacing = Vec2::ZERO;

            // The category strip owns the bottom edge; the grid gets exactly
            // the band between the tabs and the strip, so no dead row opens up
            // between them.
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

            tabs(app, ui, &theme);
            match app.emoji_tab {
               EmojiTab::Emoji => emoji_grid(app, ui, body, &theme),
               EmojiTab::Stickers => stickers(ui, body, &theme),
               EmojiTab::Gifs => gifs(ui, body, &theme),
            }
            categories(ui, &theme, strip);
         });
      });

   // The click that opened the panel is still the frame's click position, so
   // it must not also read as a click outside it.
   if was_open {
      let escape = ctx.input(|i| i.key_pressed(Key::Escape));
      if escape || area.response.clicked_elsewhere() {
         app.emoji_open = false;
      }
   }
}

/// Three equal tabs, the active one underlined in the accent.
fn tabs(app: &mut TelegramApp, ui: &mut Ui, theme: &Theme) {
   let colors = theme.colors;
   let (rect, _) = ui.allocate_exact_size(
      vec2(ui.available_width(), style::EMOJI_TAB_H),
      Sense::hover(),
   );
   let column = rect.width() / EmojiTab::ALL.len() as f32;
   let painter = ui.painter();

   for (index, tab) in EmojiTab::ALL.iter().enumerate() {
      let tab_rect = Rect::from_min_max(
         pos2(rect.left() + column * index as f32, rect.top()),
         pos2(
            rect.left() + column * (index + 1) as f32,
            rect.bottom(),
         ),
      );
      let response = ui.interact(
         tab_rect,
         ui.scope_id().with(("emoji_tab", index)),
         Sense::click(),
      );
      let active = *tab == app.emoji_tab;
      if response.hovered() && !active {
         painter.rect_filled(
            tab_rect.shrink2(vec2(theme.spacing.sm, theme.spacing.xs)),
            CornerRadius::same(style::EMOJI_RADIUS),
            colors.hover,
         );
      }
      painter.text(
         tab_rect.center(),
         Align2::CENTER_CENTER,
         tab.label(),
         FontId::proportional(13.0),
         if active {
            colors.accent
         } else {
            colors.text_muted
         },
      );
      if active {
         let underline = Rect::from_min_max(
            pos2(
               tab_rect.center().x - 26.0,
               tab_rect.bottom() - 2.0,
            ),
            pos2(tab_rect.center().x + 26.0, tab_rect.bottom()),
         );
         painter.rect_filled(underline, CornerRadius::ZERO, colors.accent);
      }
      if response.clicked() {
         app.emoji_tab = *tab;
      }
   }
}

/// The glyph grid, seven columns wide, filling the band above the strip.
fn emoji_grid(app: &mut TelegramApp, ui: &mut Ui, body: Rect, theme: &Theme) {
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
                        emoji_cell(app, ui, emoji, cell_w, cell_h, theme);
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
   app: &mut TelegramApp,
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
      app.composer.push_str(emoji);
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
         let ink = if index == 0 {
            colors.accent
         } else {
            colors.text_muted
         };
         let _ = paint::icon_button(ui, *icon, 20.0, ink, colors.hover);
      });
      x += size + gap;
   }
}

// ---------------------------------------------------------------------------
// Attachment menu
// ---------------------------------------------------------------------------

/// The paperclip menu, anchored above the paperclip in the composer.
fn attach_menu(app: &mut TelegramApp, ctx: &Context) {
   let was_open = sync_open(ctx, "attach_menu_open", app.attach_open);
   if !app.attach_open {
      return;
   }

   let theme = app.theme.clone();
   let colors = theme.colors;
   let space = theme.spacing;
   let screen = ctx.content_rect();
   let height = ATTACH.len() as f32 * style::ATTACH_ROW_H + space.sm * 2.0;
   let origin = pos2(
      style::SIDEBAR_W + space.md,
      screen.bottom() - style::COMPOSER_H - space.sm - height,
   );

   let area = Area::new(Id::unique("attach_menu"))
      .order(Order::Foreground)
      .fixed_pos(origin)
      .show(ctx, |ui| {
         panel_frame(&theme, style::ATTACH_RADIUS as f32)
            .inner_margin(space.margin(space.sm))
            .show(ui, |ui| {
               ui.set_width(style::ATTACH_W - space.sm * 2.0);
               ui.spacing_mut().item_spacing = Vec2::ZERO;
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
                     app.attach_open = false;
                  }
               }
            });
      });

   if was_open {
      let escape = ctx.input(|i| i.key_pressed(Key::Escape));
      if escape || area.response.clicked_elsewhere() {
         app.attach_open = false;
      }
   }
}

/// The shared floating-panel chrome: window fill, hairline, soft shadow.
fn panel_frame(theme: &Theme, radius: f32) -> Frame {
   Frame::new()
      .fill(theme.window_frame.fill)
      .stroke(Stroke::new(1.0, theme.colors.border))
      .corner_radius(CornerRadius::same(radius as u8))
      .shadow(Shadow {
         offset: (0, 6).into(),
         blur: 24,
         spread: 0,
         color: style::translucent(Color32::BLACK, 90),
      })
}

/// Record whether a panel was open last frame; returns that value.
///
/// Both pickers are toggled in the frame the click that opens them lands, so
/// the same click would read as a click outside the panel. `was_open` lets the
/// outside-click check skip that first frame.
fn sync_open(ctx: &Context, key: &'static str, open: bool) -> bool {
   let id = Id::unique(key);
   let previous = ctx.data(|data| data.get_temp::<bool>(id)).unwrap_or(false);
   ctx.data_mut(|data| data.insert_temp(id, open));
   previous
}
