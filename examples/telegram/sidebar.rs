//! Sidebar: the search pill and the chat list.

use eframe::egui::*;
use egui_elements::egui_lucide::Lucide;
use egui_elements::layout::{CrossAlign, Flex, Item};
use egui_elements::theme::{ThemeColors, ThemeSpacing};
use egui_elements::utils::mix;
use egui_elements::widgets::SecureTextEdit;
use egui_elements::widgets::{Avatar, AvatarGroup, AvatarVisuals, Badge, Label, Scroll};

use crate::TelegramApp;
use crate::data::{self, Chat};
use crate::paint;
use crate::style;

/// Sidebar column: header over the scrolling chat list.
pub fn show(app: &mut TelegramApp, ui: &mut Ui) {
   let colors = app.theme.colors;
   // Pin the content width. The shell's measuring pass hands every item the
   // full shell width, so a pane that measured at 1280 and placed at 380
   // would make every container inside it re-measure on the next pass.
   let cell = ui.max_rect();
   let rect = Rect::from_min_size(
      cell.min,
      vec2(style::SIDEBAR_W.min(cell.width()), cell.height()),
   );

   {
      let painter = ui.painter();
      painter.rect_filled(rect, CornerRadius::ZERO, colors.bg);
      painter.vline(
         rect.right() - 0.5,
         rect.y_range(),
         style::hairline(&app.theme),
      );
   }

   // Exact rects with no cursor gaps: the two scopes tile the pane exactly.
   ui.spacing_mut().item_spacing.y = 0.0;
   let head = Rect::from_min_size(
      rect.min,
      vec2(rect.width(), style::SIDEBAR_HEAD_H),
   );
   let list_rect = Rect::from_min_max(pos2(rect.left(), head.bottom()), rect.max);

   ui.scope_builder(UiBuilder::new().max_rect(head), |ui| {
      header(app, ui)
   });
   // The cell starts left-to-right, and a scroll area inherits its parent's
   // layout, so the list must opt into a vertical layout or its rows lay out
   // side by side behind the clip.
   ui.scope_builder(
      UiBuilder::new().max_rect(list_rect).layout(Layout::top_down(Align::Min)),
      |ui| list(app, ui),
   );
}

/// Header: the menu button and the search pill, vertically centred in 52 pt.
fn header(app: &mut TelegramApp, ui: &mut Ui) {
   let space = app.theme.spacing;
   let inner = ui.max_rect().shrink2(vec2(space.md, 0.0));

   ui.scope_builder(
      UiBuilder::new().max_rect(inner).layout(Layout::left_to_right(Align::Center)),
      |ui| {
         Flex::row("sidebar_header")
            .gap(space.sm)
            .align_items(CrossAlign::Center)
            .show(ui, |f| {
               f.item(Item::fixed(36.0), |ui| {
                  if paint::icon_button(
                     ui,
                     Lucide::UserPlus,
                     20.0,
                     app.theme.colors.text_muted,
                     app.theme.colors.hover,
                  )
                  .clicked()
                  {
                     app.new_contact_open = true;
                  }
               });
               f.item(Item::fixed(36.0), |ui| {
                  let _ = paint::icon_button(
                     ui,
                     Lucide::Menu,
                     20.0,
                     app.theme.colors.text_muted,
                     app.theme.colors.hover,
                  );
               });
               f.item(Item::grow(1.0), |ui| search(app, ui));
            });
      },
   );
}

/// A single pill: one fill, one hairline, the lens icon, and a chrome-less edit.
fn search(app: &mut TelegramApp, ui: &mut Ui) {
   let colors = app.theme.colors;
   let space = app.theme.spacing;
   let (rect, response) = ui.allocate_exact_size(
      vec2(ui.available_width(), style::SEARCH_H),
      Sense::hover(),
   );

   let fill = if response.hovered() {
      colors.hover
   } else {
      colors.widget_bg
   };
   {
      let painter = ui.painter();
      painter.rect_filled(
         rect,
         CornerRadius::same(style::SEARCH_RADIUS),
         fill,
      );
      painter.rect_stroke(
         rect,
         CornerRadius::same(style::SEARCH_RADIUS),
         style::hairline(&app.theme),
         StrokeKind::Inside,
      );
   }

   let inner = rect.shrink2(vec2(space.md, space.xs));
   ui.scope_builder(
      UiBuilder::new().max_rect(inner).layout(Layout::left_to_right(Align::Center)),
      |ui| {
         Flex::row("search_row")
            .gap(space.sm)
            .align_items(CrossAlign::Center)
            .show(ui, |f| {
               f.item(Item::fixed(16.0), |ui| {
                  ui.add(Lucide::Search.size(16.0).color(colors.text_muted).image());
               });
               f.item(Item::grow(1.0), |ui| edit(app, ui));
            });
      },
   );
}

/// The field inside the pill, with its own chrome removed.
fn edit(app: &mut TelegramApp, ui: &mut Ui) {
   let colors = app.theme.colors;
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
      SecureTextEdit::singleline(&mut app.search)
         .id_salt("telegram_search")
         .hint_text(RichText::new("Search").size(13.5).color(colors.text_muted))
         .margin(Margin::ZERO)
         .desired_width(width)
         .min_size(vec2(width, height))
         .visuals(visuals),
   );
}

/// The chat list, filtered live on title and preview.
fn list(app: &mut TelegramApp, ui: &mut Ui) {
   let colors = app.theme.colors;
   let space = app.theme.spacing;
   let query = app.search.trim().to_ascii_lowercase();
   let chats: Vec<&'static Chat> =
      data::chats().iter().filter(|chat| matches(chat, &query)).collect();

   Scroll::vertical("chat_list").auto_shrink([false, false]).show(ui, |ui| {
      ui.spacing_mut().item_spacing.y = 0.0;
      ui.add_space(space.xs);

      if chats.is_empty() {
         ui.add_space(space.xl);
         ui.vertical_centered(|ui| {
            ui.label(RichText::new("No chats found").size(13.5).color(colors.text_muted));
         });
         return;
      }

      for chat in &chats {
         row(app, ui, chat);
      }
      ui.add_space(space.sm);
   });
}

/// One 72 pt chat row, with a floating hover / selection fill.
fn row(app: &mut TelegramApp, ui: &mut Ui, chat: &Chat) {
   let colors = app.theme.colors;
   let space = app.theme.spacing;
   let plate = style::chat_plate(&app.theme, chat.id);

   let (rect, response) = ui.allocate_exact_size(
      vec2(ui.available_width(), style::ROW_H),
      Sense::click(),
   );
   if response.clicked() {
      app.selected_id = chat.id;
   }
   let selected = app.selected_id == chat.id;

   let fill = if selected {
      colors.highlight
   } else if response.hovered() {
      colors.hover
   } else {
      Color32::TRANSPARENT
   };
   let row_rect = Rect::from_min_max(
      pos2(rect.left() + space.xs, rect.top()),
      pos2(rect.right() - space.xs, rect.bottom()),
   );
   if fill != Color32::TRANSPARENT {
      ui.painter().rect_filled(
         row_rect,
         CornerRadius::same(style::ROW_RADIUS),
         fill,
      );
   }

   let ring = if fill == Color32::TRANSPARENT {
      colors.bg
   } else {
      fill
   };
   let title_ink = if selected {
      colors.ink_on(colors.highlight)
   } else {
      colors.text
   };
   let dim_ink = if selected {
      mix(title_ink, colors.highlight, 0.25)
   } else {
      colors.text_muted
   };
   let preview_ink = if !selected && chat.preview.is_draft() {
      colors.error
   } else {
      dim_ink
   };

   let content = row_rect.shrink2(vec2(space.sm, 0.0));
   ui.scope_builder(
      UiBuilder::new().max_rect(content).layout(Layout::left_to_right(Align::Center)),
      |ui| {
         Flex::row(("chat_row", chat.id))
            .gap(space.md)
            .align_items(CrossAlign::Center)
            .show(ui, |f| {
               f.item(Item::fixed(style::AVATAR_ROW), |ui| {
                  avatar(ui, chat, colors, plate, ring);
               });
               f.item(Item::grow(1.0).stack(), |ui| {
                  text(ui, chat, title_ink, preview_ink, space);
               });
               f.item(Item::fixed(style::META_W), |ui| {
                  meta(ui, chat, dim_ink, colors, space);
               });
            });
      },
   );
}

/// Round avatar with initials, the presence dot for an online contact, and a
/// ring that matches the row fill so the circle sits into the selection.
fn avatar(ui: &mut Ui, chat: &Chat, colors: ThemeColors, plate: Color32, ring: Color32) {
   let mut visuals = AvatarVisuals::from_theme(colors);
   visuals.ring = ring;
   let group = [Avatar::initials(chat.initials).plate(plate).ink(colors.ink_on(plate))];
   let label = if chat.group { "Group avatar" } else { "Avatar" };

   let response = ui.add(
      AvatarGroup::new(&group)
         .size(style::AVATAR_ROW)
         .expand(false)
         .visuals(visuals)
         .label(label),
   );

   if chat.online {
      let center = response.rect.right_bottom() - vec2(7.0, 7.0);
      let painter = ui.painter();
      painter.circle_filled(center, 7.0, ring);
      painter.circle_filled(center, 5.0, colors.success);
   }
}

/// Title over the last-message line, both truncated to the middle column.
fn text(ui: &mut Ui, chat: &Chat, title_ink: Color32, preview_ink: Color32, space: ThemeSpacing) {
   ui.spacing_mut().item_spacing.y = space.xs;
   ui.add(
      Label::new(
         RichText::new(chat.title).size(15.0).color(title_ink),
         None,
      )
      .wrap_mode(TextWrapMode::Truncate),
   );
   ui.add(
      Label::new(
         RichText::new(chat.preview.line()).size(13.5).color(preview_ink),
         None,
      )
      .wrap_mode(TextWrapMode::Truncate),
   );
}

/// Time over the status glyph or unread badge, right-aligned and hand-placed.
///
/// Painted rather than laid out: a container here would re-measure when the
/// row's fixed meta column changes width, and the row has to paint whole on
/// its first placed frame.
fn meta(ui: &mut Ui, chat: &Chat, ink: Color32, colors: ThemeColors, space: ThemeSpacing) {
   let (rect, _) = ui.allocate_exact_size(ui.available_size(), Sense::hover());
   if !ui.is_rect_visible(rect) {
      return;
   }

   let line_y = rect.center().y - space.md;
   let mut right = rect.right();
   // The pin is the warning slot's call site: a pinned chat is a caution
   // against the row's muted meta ink, so it reads at a glance.
   for (icon, shown, tint) in [
      (Lucide::BellOff, chat.muted, ink),
      (Lucide::Pin, chat.pinned, colors.warning),
   ] {
      if !shown {
         continue;
      }
      right -= 14.0;
      let icon_rect = Rect::from_min_size(pos2(right, line_y - 7.0), Vec2::splat(14.0));
      icon.size(14.0).color(tint).image().paint_at(ui, icon_rect);
      right -= space.xs;
   }
   ui.painter().text(
      pos2(right, line_y),
      Align2::RIGHT_CENTER,
      chat.time,
      FontId::proportional(12.0),
      ink,
   );

   let status_y = rect.center().y + space.md;
   if chat.unread > 0 {
      let fill = if chat.muted {
         colors.text_muted
      } else {
         colors.accent
      };
      let badge_rect = Rect::from_center_size(
         pos2(rect.right() - 10.0, status_y),
         Vec2::splat(20.0),
      );
      let _ = ui.put(
         badge_rect,
         Badge::new(chat.unread.to_string())
            .color(fill)
            .text_color(colors.ink_on(fill))
            .min_diameter(20.0)
            .padding(0.0),
      );
   } else if chat.preview_read {
      let icon_rect = Rect::from_center_size(
         pos2(rect.right() - 7.5, status_y),
         Vec2::splat(15.0),
      );
      Lucide::CheckCheck
         .size(15.0)
         .color(colors.success)
         .image()
         .paint_at(ui, icon_rect);
   }
}

/// Case-insensitive filter on a chat's title and preview.
fn matches(chat: &Chat, query: &str) -> bool {
   if query.is_empty() {
      return true;
   }
   chat.title.to_lowercase().contains(query) || chat.preview.line().to_lowercase().contains(query)
}
