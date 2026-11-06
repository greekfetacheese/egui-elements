//! The conversation pane: header, wallpaper-backed message list and composer.
//!
//! Rows are laid out by a `Flex` row (a content-sized bubble cell plus a growing
//! spacer) rather than a right-to-left `Ui`: a frame placed in an RTL row is
//! anchored at the row's left edge and stretches to the full width, so the
//! bubble could neither hug the right edge nor size to its text.

use eframe::egui::*;
use egui_elements::egui_lucide::Lucide;
use egui_elements::layout::{CrossAlign, Flex, Item};
use egui_elements::theme::{Theme, ThemeColors};
use egui_elements::utils::mix;
use egui_elements::widgets::{Avatar, AvatarGroup, AvatarVisuals, Frame, Label, SecureTextEdit};

use crate::TelegramApp;
use crate::data::{self, Chat, Msg, MsgBody, MsgStatus, Quote};
use crate::paint;
use crate::style;

/// Paint the conversation pane for the selected chat.
pub fn show(app: &mut TelegramApp, ui: &mut Ui) {
   // Reserving the whole cell is what makes the shell's conversation pane
   // measure full height.
   let cell = ui.max_rect();
   let (rect, _) = ui.allocate_exact_size(cell.size(), Sense::hover());
   paint::wallpaper(ui, rect, &app.theme);

   let head_rect = Rect::from_min_size(rect.min, vec2(rect.width(), style::HEADER_H));
   let compose_rect = Rect::from_min_max(
      pos2(rect.left(), rect.bottom() - style::COMPOSER_H),
      rect.max,
   );
   let list_rect = Rect::from_min_max(
      pos2(rect.left(), head_rect.bottom()),
      pos2(rect.right(), compose_rect.top()),
   );

   let Some(chat) = data::chats().iter().find(|chat| chat.id == app.selected_id) else {
      empty(ui, list_rect, &app.theme);
      return;
   };

   ui.scope_builder(UiBuilder::new().max_rect(head_rect), |ui| {
      header(&app.theme, ui, chat)
   });
   ui.scope_builder(UiBuilder::new().max_rect(list_rect), |ui| {
      messages(&app.theme, ui, chat)
   });
   ui.scope_builder(UiBuilder::new().max_rect(compose_rect), |ui| {
      composer(app, ui)
   });
}

/// 60 pt conversation header: opaque canvas, avatar, title over status and the
/// two chrome buttons on the trailing edge.
fn header(theme: &Theme, ui: &mut Ui, chat: &Chat) {
   let colors = theme.colors;
   let space = theme.spacing;
   let rect = ui.max_rect();

   {
      let painter = ui.painter();
      painter.rect_filled(rect, CornerRadius::ZERO, colors.bg);
      painter.hline(
         rect.x_range(),
         rect.bottom() - 0.5,
         style::hairline(theme),
      );
   }

   let (status, status_ink) = presence(chat, colors);
   let inner = rect.shrink2(vec2(space.md, 0.0));
   ui.scope_builder(
      UiBuilder::new().max_rect(inner).layout(Layout::left_to_right(Align::Center)),
      |ui| {
         Flex::row(("chat_header", chat.id))
            .gap(space.md)
            .align_items(CrossAlign::Center)
            .show(ui, |f| {
               f.item(Item::fixed(style::AVATAR_HEADER), |ui| {
                  header_avatar(ui, chat, theme)
               });
               f.item(Item::grow(1.0).stack(), |ui| {
                  ui.spacing_mut().item_spacing.y = 0.0;
                  ui.add(
                     Label::new(
                        RichText::new(chat.title).size(15.5).color(colors.text),
                        None,
                     )
                     .interactive(false)
                     .wrap_mode(TextWrapMode::Truncate),
                  );
                  ui.add(
                     Label::new(
                        RichText::new(status).size(13.0).color(status_ink),
                        None,
                     )
                     .interactive(false)
                     .wrap_mode(TextWrapMode::Truncate),
                  );
               });
               f.item(Item::new(), |ui| header_actions(ui, theme));
            });
      },
   );
}

/// Round avatar ringed in the header canvas so the plate sits into the bar.
fn header_avatar(ui: &mut Ui, chat: &Chat, theme: &Theme) {
   let colors = theme.colors;
   let plate = style::chat_plate(theme, chat.id);
   let mut visuals = AvatarVisuals::from_theme(colors);
   visuals.ring = colors.bg;
   let group = [Avatar::initials(chat.initials).plate(plate).ink(colors.ink_on(plate))];
   let label = if chat.group { "Group avatar" } else { "Avatar" };

   ui.add(
      AvatarGroup::new(&group)
         .size(style::AVATAR_HEADER)
         .expand(false)
         .visuals(visuals)
         .label(label),
   );
}

/// Search and the message menu, both chrome-less icon buttons.
///
/// Added straight into the header row's cell: a nested container here would
/// measure against the header's width, then be placed against its own cell, and
/// a `Flex` never leaves its measuring pass when that space changes.
fn header_actions(ui: &mut Ui, theme: &Theme) {
   let colors = theme.colors;
   let space = theme.spacing;

   ui.spacing_mut().item_spacing.x = space.xs;
   let _ = paint::icon_button(
      ui,
      Lucide::Search,
      18.0,
      colors.text_muted,
      colors.hover,
   );
   let _ = paint::icon_button(
      ui,
      Lucide::EllipsisVertical,
      18.0,
      colors.text_muted,
      colors.hover,
   );
}

/// Status line: "online" for a live private chat, a member count for a group,
/// "last seen recently" otherwise.
fn presence(chat: &Chat, colors: ThemeColors) -> (String, Color32) {
   if chat.group {
      (
         format!("{} members", members(chat.id)),
         colors.text_muted,
      )
   } else if chat.online {
      ("online".to_owned(), colors.success)
   } else {
      ("last seen recently".to_owned(), colors.text_muted)
   }
}

/// Group size, derived from the distinct authors in the conversation plus us.
fn members(chat_id: usize) -> usize {
   let mut authors: Vec<&str> =
      data::messages(chat_id).iter().filter_map(|message| message.author).collect();
   authors.sort_unstable();
   authors.dedup();
   authors.len() + 1
}

/// The scrolling conversation. Opens at the newest message once per chat change.
fn messages(theme: &Theme, ui: &mut Ui, chat: &Chat) {
   let space = theme.spacing;
   let rect = ui.max_rect();
   let inner = Rect::from_min_max(
      pos2(rect.left() + space.md, rect.top()),
      pos2(rect.right() - space.md, rect.bottom()),
   );
   let messages = data::messages(chat.id);

   if messages.is_empty() {
      empty(ui, inner, theme);
      return;
   }

   // Telegram shows the latest message when a chat opens, then leaves the
   // scroll where the reader put it: the flag is per chat, not per frame.
   let marker = Id::unique("telegram_first_show");
   let pending = ui.ctx().data(|data| data.get_temp::<usize>(marker)) != Some(chat.id);

   let out = ui.scope_builder(
      UiBuilder::new().max_rect(inner).layout(Layout::top_down(Align::Min)),
      |ui| {
         ScrollArea::vertical()
            .id_salt("messages")
            .auto_shrink([false, false])
            .show(ui, |ui| {
               ui.spacing_mut().item_spacing.y = space.sm;
               ui.add_space(space.lg);
               for (index, message) in messages.iter().enumerate() {
                  message_row(ui, message, theme, chat.id, index);
               }
               ui.add_space(space.lg);
               if pending {
                  ui.scroll_to_cursor(Some(Align::BOTTOM));
               }
            })
      },
   );

   // Only claim the chat as shown once the newest message is actually in view:
   // a discarded first frame would otherwise consume the flag without moving.
   if pending {
      let scroll = &out.inner;
      let bottom = scroll.state.clamped_offset().y + scroll.inner_rect.height();
      if bottom + 1.0 >= scroll.content_size.y {
         ui.ctx().data_mut(|data| data.insert_temp(marker, chat.id));
      }
   }
}

/// One row: a day pill, a sticker, or a bubble aligned to its sender.
fn message_row(ui: &mut Ui, message: &Msg, theme: &Theme, chat_id: usize, index: usize) {
   match message.body {
      MsgBody::Day(label) => day_pill(ui, label, theme),
      MsgBody::Sticker(name) => sticker(ui, message, name, theme),
      _ => {
         Flex::row(("message", chat_id, index))
            .gap(0.0)
            .align_items(CrossAlign::Center)
            .show(ui, |f| {
               if message.from_me {
                  f.item(Item::grow(1.0), |_ui| {});
                  f.item(Item::new().stack(), |ui| {
                     bubble(ui, message, theme)
                  });
               } else {
                  f.item(Item::new().stack(), |ui| {
                     bubble(ui, message, theme)
                  });
                  f.item(Item::grow(1.0), |_ui| {});
               }
            });
      }
   }
}

/// The message bubble. Sizes to its content; the flex cell places it.
fn bubble(ui: &mut Ui, message: &Msg, theme: &Theme) {
   let colors = theme.colors;
   let space = theme.spacing;
   let radius = style::BUBBLE_RADIUS as u8;
   let tail = style::BUBBLE_TAIL;
   let corners = CornerRadius {
      nw: radius,
      ne: radius,
      sw: if message.from_me { radius } else { tail },
      se: if message.from_me { tail } else { radius },
   };
   let photo = matches!(message.body, MsgBody::Photo { .. });
   let margin = if photo {
      // The image flushes with the top and side edges; the caption band under
      // it carries the padding.
      Margin {
         left: 0,
         right: 0,
         top: 0,
         bottom: 0,
      }
   } else {
      Margin::symmetric(style::BUBBLE_PAD_X, style::BUBBLE_PAD_Y)
   };

   let frame = Frame::new()
      .fill(style::bubble_fill(theme, message.from_me))
      .stroke(style::bubble_stroke(theme, message.from_me))
      .inner_margin(margin)
      .corner_radius(corners)
      .shadow(Shadow {
         offset: (0, 1).into(),
         blur: 6,
         spread: 0,
         color: style::translucent(Color32::BLACK, 40),
      });

   frame.show(ui, |ui| {
      // Cap the wrap width. `set_max_width` only shrinks, so the placing pass
      // keeps the width the flex row measured for this bubble.
      let cap = style::BUBBLE_MAX_W - (style::BUBBLE_PAD_X * 2) as f32;
      if ui.available_width() > cap {
         ui.set_max_width(cap);
      }

      if !photo {
         if let Some(author) = message.author {
            ui.add(
               Label::new(
                  RichText::new(author).size(13.0).color(colors.accent),
                  None,
               )
               .interactive(false)
               .wrap_mode(TextWrapMode::Truncate),
            );
         }
         if let Some(quote) = message.quote {
            ui.add_space(space.xs);
            quote_band(ui, quote, theme);
            ui.add_space(space.xs);
         }
      }

      match message.body {
         MsgBody::Text(text) => {
            ui.add(
               Label::new(
                  RichText::new(text).size(14.5).color(colors.text),
                  None,
               )
               .wrap()
               .interactive(false),
            );
            ui.add_space(space.xs);
            meta(ui, message, theme, None);
         }
         MsgBody::Photo { caption, seed } => photo_body(ui, message, &caption, seed, theme),
         MsgBody::Voice { duration, seed } => voice_body(ui, message, &duration, seed, theme),
         MsgBody::File { name, size } => file_body(ui, message, &name, &size, theme),
         MsgBody::Day(_) | MsgBody::Sticker(_) => {}
      }
   });
}

/// A quoted reply: a tinted band with an accent bar on its leading edge.
fn quote_band(ui: &mut Ui, quote: Quote, theme: &Theme) {
   let colors = theme.colors;
   let space = theme.spacing;
   let radius = style::QUOTE_RADIUS;
   let width = ui.available_width();

   let band = Frame::new()
      .fill(style::translucent(colors.accent, 26))
      .corner_radius(CornerRadius::same(radius))
      .inner_margin(Margin::symmetric(space.sm as i8, space.xs as i8))
      .show(ui, |ui| {
         // The band spans the bubble, so its own width is the bubble width.
         ui.set_min_width((width - (space.sm * 2.0)).max(0.0));
         ui.spacing_mut().item_spacing.y = 0.0;
         ui.add(
            Label::new(
               RichText::new(quote.author).size(13.0).color(colors.accent),
               None,
            )
            .interactive(false)
            .wrap_mode(TextWrapMode::Truncate),
         );
         ui.add(
            Label::new(
               RichText::new(quote.text).size(13.0).color(colors.text_muted),
               None,
            )
            .interactive(false)
            .wrap_mode(TextWrapMode::Truncate),
         );
      });

   let rect = band.response.rect;
   let bar = Rect::from_min_max(
      pos2(rect.left() + 1.0, rect.top() + space.xs),
      pos2(
         rect.left() + 1.0 + style::QUOTE_BAR_W,
         rect.bottom() - space.xs,
      ),
   );
   ui.painter().rect_filled(bar, CornerRadius::same(2), colors.accent);
}

/// A photo bubble: the image flushes with the top and side edges, the caption
/// and meta sit in the padded band under it.
fn photo_body(ui: &mut Ui, message: &Msg, caption: &str, seed: u32, theme: &Theme) {
   let colors = theme.colors;
   let space = theme.spacing;
   let width = style::PHOTO_W.min(ui.available_width());
   let (rect, _) = ui.allocate_exact_size(vec2(width, style::PHOTO_H), Sense::hover());
   paint::photo(ui, rect, seed, theme);

   ui.set_max_width(width);
   Frame::new()
      .inner_margin(Margin::symmetric(
         style::BUBBLE_PAD_X,
         style::BUBBLE_PAD_Y,
      ))
      .show(ui, |ui| {
         ui.add(
            Label::new(
               RichText::new(caption).size(14.5).color(colors.text),
               None,
            )
            .wrap()
            .interactive(false),
         );
         ui.add_space(space.xs);
         meta(ui, message, theme, None);
      });
}

/// A voice message: play disc, waveform, duration and meta.
fn voice_body(ui: &mut Ui, message: &Msg, duration: &str, seed: u32, theme: &Theme) {
   let colors = theme.colors;
   let space = theme.spacing;
   let width = style::VOICE_PLAY + space.sm + style::WAVEFORM_W;
   let (rect, _) = ui.allocate_exact_size(vec2(width, style::VOICE_PLAY), Sense::hover());
   let play = Rect::from_min_size(rect.min, Vec2::splat(style::VOICE_PLAY));
   let painter = ui.painter();

   painter.circle_filled(
      play.center(),
      style::VOICE_PLAY * 0.5,
      colors.accent,
   );
   Lucide::Play.size(18.0).color(colors.ink_on(colors.accent)).image().paint_at(
      ui,
      Rect::from_center_size(play.center(), Vec2::splat(18.0)),
   );

   let wave = Rect::from_min_size(
      pos2(
         play.right() + space.sm,
         rect.center().y - style::WAVEFORM_H * 0.5,
      ),
      vec2(style::WAVEFORM_W, style::WAVEFORM_H),
   );
   paint::waveform(
      painter,
      wave,
      seed,
      0.42,
      colors.accent,
      style::translucent(colors.text_muted, 70),
   );

   ui.add_space(space.xs);
   meta(ui, message, theme, Some(duration));
}

/// A document: download disc, file name over its size, meta.
///
/// The disc is 44 pt, taller than a text row's baseline band, so the row is one
/// allocated rect painted from measured galleys instead of a nested container.
fn file_body(ui: &mut Ui, message: &Msg, name: &str, size: &str, theme: &Theme) {
   let colors = theme.colors;
   let space = theme.spacing;
   let name_font = FontId::proportional(14.0);
   let size_font = FontId::proportional(12.0);
   let name_galley = ui.painter().layout_no_wrap(name.to_owned(), name_font, colors.text);
   let size_galley = ui.painter().layout_no_wrap(size.to_owned(), size_font, colors.text_muted);
   let name_height = name_galley.size().y;
   let text_width = name_galley.size().x.max(size_galley.size().x);
   let width = style::VOICE_PLAY + space.md + text_width;

   let (rect, _) = ui.allocate_exact_size(vec2(width, style::VOICE_PLAY), Sense::hover());
   if !ui.is_rect_visible(rect) {
      return;
   }

   let painter = ui.painter();
   let disc = Rect::from_min_size(rect.min, Vec2::splat(style::VOICE_PLAY));
   painter.circle_filled(
      disc.center(),
      style::VOICE_PLAY * 0.5,
      colors.accent,
   );
   Lucide::Download
      .size(18.0)
      .color(colors.ink_on(colors.accent))
      .image()
      .paint_at(
         ui,
         Rect::from_center_size(disc.center(), Vec2::splat(18.0)),
      );

   let left = disc.right() + space.md;
   let top = rect.center().y - (name_height + size_galley.size().y) * 0.5;
   let text = painter.with_clip_rect(rect);
   text.galley(pos2(left, top), name_galley, colors.text);
   text.galley(
      pos2(left, top + name_height),
      size_galley,
      colors.text_muted,
   );

   ui.add_space(space.xs);
   meta(ui, message, theme, None);
}

/// Time, delivery ticks and the failed marker, right-aligned inside the bubble.
///
/// Painted rather than laid out: the row is already sized, and a right-aligned
/// layout inside a content-sized bubble would stretch the bubble to the pane.
fn meta(ui: &mut Ui, message: &Msg, theme: &Theme, leading: Option<&str>) {
   let colors = theme.colors;
   let space = theme.spacing;
   let font = FontId::proportional(11.0);
   let duration_font = FontId::proportional(12.0);
   // The outgoing fill is chromatic: the palette's muted ink drops under 2.5:1
   // on it, so its meta uses body ink pulled a little toward the bubble.
   let muted = if message.from_me {
      mix(colors.text, style::bubble_fill(theme, true), 0.10)
   } else {
      colors.text_muted
   };

   let time_width = ui
      .painter()
      .layout_no_wrap(message.time.to_owned(), font.clone(), muted)
      .size()
      .x;
   let leading_width = leading
      .map(|text| {
         ui.painter()
            .layout_no_wrap(text.to_owned(), duration_font.clone(), muted)
            .size()
            .x
      })
      .unwrap_or(0.0);

   let mut width = time_width;
   if message.from_me {
      width += space.xs + style::TICK_W;
   }
   if message.failed {
      width += space.xs + style::FAIL_ICON;
   }
   if leading.is_some() {
      width += space.xs + leading_width;
   }

   let (rect, _) = ui.allocate_exact_size(vec2(width, style::META_H), Sense::hover());
   if !ui.is_rect_visible(rect) {
      return;
   }

   let painter = ui.painter();
   let center_y = rect.center().y;
   let mut right = ui.max_rect().right();

   if message.failed {
      right -= style::FAIL_ICON;
      Lucide::CircleAlert.size(style::FAIL_ICON).color(colors.error).image().paint_at(
         ui,
         Rect::from_min_size(
            pos2(right, center_y - style::FAIL_ICON * 0.5),
            Vec2::splat(style::FAIL_ICON),
         ),
      );
      right -= space.xs;
   }

   if message.from_me {
      right -= style::TICK_W;
      let read = matches!(message.status, MsgStatus::Read);
      let ink = if read { colors.success } else { muted };
      let tick_rect = Rect::from_min_size(
         pos2(right, center_y - 5.0),
         vec2(style::TICK_W, 10.0),
      );
      paint::ticks(painter, tick_rect, read, ink);
      right -= space.xs;
   }

   painter.text(
      pos2(right, center_y),
      Align2::RIGHT_CENTER,
      message.time,
      font,
      muted,
   );
   if let Some(text) = leading {
      painter.text(
         pos2(rect.left(), center_y),
         Align2::LEFT_CENTER,
         text,
         duration_font,
         muted,
      );
   }
}

/// A centred day separator spanning the row.
fn day_pill(ui: &mut Ui, label: &str, theme: &Theme) {
   let colors = theme.colors;
   let space = theme.spacing;

   // Painted from a measured galley instead of a Frame: a Frame in a centred
   // layout takes the available width, which turns the pill into a full-width bar.
   let galley = ui.painter().layout_no_wrap(
      label.to_owned(),
      FontId::proportional(11.5),
      colors.text_muted,
   );
   let size = vec2(
      galley.size().x + style::DAY_PAD_X as f32 * 2.0,
      galley.size().y + space.xs * 2.0,
   );
   ui.vertical_centered(|ui| {
      let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
      let painter = ui.painter();
      painter.rect_filled(
         rect,
         CornerRadius::same(style::DAY_RADIUS),
         colors.widget_bg,
      );
      painter.galley(
         rect.center() - galley.size() * 0.5,
         galley,
         colors.text_muted,
      );
   });
}

/// A sticker: a big glyph and its time, no bubble.
fn sticker(ui: &mut Ui, message: &Msg, name: &str, theme: &Theme) {
   let colors = theme.colors;
   let space = theme.spacing;

   ui.vertical_centered(|ui| {
      ui.add(
         Label::new(
            RichText::new(sticker_glyph(name)).size(style::STICKER_SIZE).color(colors.text),
            None,
         )
         .interactive(false),
      );
      ui.add_space(space.xs);
      ui.add(
         Label::new(
            RichText::new(message.time).size(11.0).color(colors.text_muted),
            None,
         )
         .interactive(false),
      );
   });
}

/// Sticker art is a glyph, not an asset: named stickers map to one emoji.
fn sticker_glyph(name: &str) -> &'static str {
   match name {
      "Cheering fox" => "🦊",
      _ => "🖼",
   }
}

/// The empty-conversation card.
fn empty(ui: &Ui, rect: Rect, theme: &Theme) {
   let colors = theme.colors;
   let space = theme.spacing;
   let width = (rect.width() * 0.42).clamp(240.0, 320.0);
   let card = Rect::from_center_size(rect.center(), vec2(width, 124.0));
   let radius = style::PANEL_RADIUS as u8;
   let painter = ui.painter();

   paint::soft_shadow(
      painter,
      card.translate(vec2(0.0, space.xs)),
      style::PANEL_RADIUS,
      style::translucent(theme.frame1.shadow.color, 90),
   );
   painter.rect_filled(
      card,
      CornerRadius::same(radius),
      theme.frame1.fill,
   );
   painter.rect_stroke(
      card,
      CornerRadius::same(radius),
      style::hairline(theme),
      StrokeKind::Inside,
   );
   painter.text(
      card.center() - vec2(0.0, space.sm),
      Align2::CENTER_CENTER,
      "No messages yet",
      FontId::proportional(15.5),
      colors.text,
   );
   painter.text(
      card.center() + vec2(0.0, space.lg),
      Align2::CENTER_CENTER,
      "Say hello",
      FontId::proportional(13.0),
      colors.text_muted,
   );
}

/// The 64 pt composer: attach, the message field, emoji and send / mic.
fn composer(app: &mut TelegramApp, ui: &mut Ui) {
   let colors = app.theme.colors;
   let space = app.theme.spacing;
   let rect = ui.max_rect();

   {
      let painter = ui.painter();
      painter.rect_filled(rect, CornerRadius::ZERO, colors.bg);
      painter.hline(
         rect.x_range(),
         rect.top() + 0.5,
         style::hairline(&app.theme),
      );
   }

   let inner = rect.shrink2(vec2(space.md, 0.0));
   ui.scope_builder(
      UiBuilder::new().max_rect(inner).layout(Layout::left_to_right(Align::Center)),
      |ui| {
         Flex::row("composer")
            .gap(space.sm)
            .align_items(CrossAlign::Center)
            .show(ui, |f| {
               f.item(Item::new(), |ui| {
                  if paint::icon_button(
                     ui,
                     Lucide::Paperclip,
                     18.0,
                     colors.text_muted,
                     colors.hover,
                  )
                  .clicked()
                  {
                     app.attach_open = !app.attach_open;
                  }
               });
               f.item(Item::grow(1.0), |ui| composer_field(app, ui));
               f.item(Item::new(), |ui| {
                  if paint::icon_button(
                     ui,
                     Lucide::FaceGrinning,
                     18.0,
                     colors.text_muted,
                     colors.hover,
                  )
                  .clicked()
                  {
                     app.emoji_open = !app.emoji_open;
                  }
               });
               f.item(Item::new(), |ui| {
                  let empty = app.composer.trim().is_empty();
                  let (icon, ink) = if empty {
                     (Lucide::Mic, colors.text_muted)
                  } else {
                     (Lucide::SendHorizontal, colors.accent)
                  };
                  // Mock: the send state is visual only.
                  let _ = paint::icon_button(ui, icon, 18.0, ink, colors.hover);
               });
            });
      },
   );
}

/// The rounded message field: one fill, one hairline, a chrome-less edit.
fn composer_field(app: &mut TelegramApp, ui: &mut Ui) {
   let colors = app.theme.colors;
   let space = app.theme.spacing;
   let (rect, response) = ui.allocate_exact_size(
      vec2(ui.available_width(), style::COMPOSER_FIELD_H),
      Sense::hover(),
   );
   let corners = CornerRadius::same(style::COMPOSER_FIELD_RADIUS);
   let fill = if response.hovered() {
      colors.hover
   } else {
      colors.widget_bg
   };
   {
      let painter = ui.painter();
      painter.rect_filled(rect, corners, fill);
      painter.rect_stroke(
         rect,
         corners,
         style::hairline(&app.theme),
         StrokeKind::Inside,
      );
   }

   let inner = rect.shrink2(vec2(space.md, 0.0));
   ui.scope_builder(
      UiBuilder::new().max_rect(inner).layout(Layout::left_to_right(Align::Center)),
      |ui| {
         let width = ui.available_width();
         let height = ui.available_height();

         let mut visuals = app.theme.text_edit_visuals();
         visuals.bg = Color32::TRANSPARENT.into();
         visuals.border = Stroke::NONE;
         visuals.border_hover = Stroke::NONE;
         visuals.border_open = Stroke::NONE;
         visuals.shadow = Shadow::NONE;
         visuals.corner_radius = CornerRadius::ZERO;

         ui.add(
            SecureTextEdit::singleline(&mut app.composer)
               .id_salt("telegram_composer")
               .hint_text(RichText::new("Write a message…").size(13.5).color(colors.text_muted))
               .margin(Margin::ZERO)
               .desired_width(width)
               .min_size(vec2(width, height))
               .visuals(visuals),
         );
      },
   );
}
