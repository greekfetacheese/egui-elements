//! The Settings dialog and its Advanced sub-page.
//!
//! One `Modal` hosts both pages, so the chrome and the footer stay put while
//! the body swaps. Every control is a public egui-elements widget: `ComboBox`
//! for the language and connection pickers, `Switch`, `Checkbox` and `Radio`
//! for the toggles, `Chip` for the Advanced beta tag.

use eframe::egui::*;
use egui_elements::egui_lucide::Lucide;
use egui_elements::layout::{CrossAlign, Flex, Item};
use egui_elements::theme::Theme;
use egui_elements::widgets::{
   Avatar, AvatarGroup, AvatarVisuals, Button, Checkbox, Chip, ChipRole, ComboBox, Divider, Label,
   Modal, Radio, Scroll, Switch,
};

use crate::TelegramApp;
use crate::paint;
use crate::style;

/// Languages offered by the demo's language picker.
const LANGUAGES: [&str; 4] = ["English", "Deutsch", "Ελληνικά", "Français"];
/// Connection choices on the Advanced page.
const CONNECTIONS: [&str; 3] = ["Default (TCP used)", "Proxy", "IPv6-only"];
/// Interface-scale stops, visual only.
const SCALES: [&str; 6] = ["100", "125", "150", "200", "250", "300"];

/// Which page the Settings modal shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SettingsPage {
   /// Profile, navigation rows and interface scale.
   #[default]
   Settings,
   /// Network, storage and media downloads.
   Advanced,
}

impl SettingsPage {
   /// Modal heading for the page.
   fn heading(self) -> &'static str {
      match self {
         SettingsPage::Settings => "Settings",
         SettingsPage::Advanced => "Advanced",
      }
   }
}

/// Automatic media download policy in private chats.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PrivateMedia {
   /// Wi-Fi only.
   #[default]
   Wifi,
   /// Mobile data.
   Mobile,
   /// Off.
   Off,
}

impl PrivateMedia {
   /// Radio choices in display order.
   const ALL: [PrivateMedia; 3] = [PrivateMedia::Wifi, PrivateMedia::Mobile, PrivateMedia::Off];

   /// Radio label.
   fn label(self) -> &'static str {
      match self {
         PrivateMedia::Wifi => "Wi-Fi only",
         PrivateMedia::Mobile => "Mobile data",
         PrivateMedia::Off => "Off",
      }
   }
}

/// Draw the Settings modal when it is open.
pub fn show(app: &mut TelegramApp, ctx: &Context) {
   if !app.settings_open {
      return;
   }

   let page = app.settings_page;
   let heading = RichText::new(page.heading()).size(20.0).color(app.theme.colors.text);
   let mut open = app.settings_open;
   let mut close = false;

   Modal::new("settings", &mut open)
      .heading(heading)
      .width(style::MODAL_W)
      .footer(|ui| {
         let button = Button::new("Close").small().min_size(vec2(88.0, 32.0));
         if ui.add(button).clicked() {
            close = true;
         }
      })
      .show(ctx, |ui| {
         ui.set_max_height(style::SETTINGS_BODY_H);
         Scroll::vertical("settings_body").auto_shrink([false, true]).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            match page {
               SettingsPage::Settings => settings_page(app, ui),
               SettingsPage::Advanced => advanced_page(app, ui),
            }
         });
      });

   if close {
      open = false;
   }
   if !open {
      // Reopening starts at the root page, never on a stale sub-page.
      app.settings_page = SettingsPage::Settings;
   }
   app.settings_open = open;
}

// ---------------------------------------------------------------------------
// Settings page
// ---------------------------------------------------------------------------

/// Profile block, the navigation rows, language and the interface scale.
fn settings_page(app: &mut TelegramApp, ui: &mut Ui) {
   let theme = app.theme.clone();
   let colors = theme.colors;
   let space = theme.spacing;

   profile(ui, &theme);
   ui.add_space(space.md);
   ui.add(Divider::new());
   ui.add_space(space.sm);

   let _ = nav_row(ui, &theme, Lucide::Pencil, "Edit profile");
   let _ = nav_row(ui, &theme, Lucide::Bell, "Notifications");
   let _ = nav_row(
      ui,
      &theme,
      Lucide::ShieldCheck,
      "Privacy and Security",
   );
   let _ = nav_row(
      ui,
      &theme,
      Lucide::MessagesSquare,
      "Chat Settings",
   );

   let advanced = nav_row_with_chip(ui, &theme);
   if advanced.clicked() {
      app.settings_page = SettingsPage::Advanced;
   }

   let (rect, _) = row(
      ui,
      &theme,
      Lucide::Languages,
      "Language",
      style::SETTINGS_ROW_H,
      Sense::hover(),
      colors.text,
   );
   let combo = Rect::from_center_size(
      pos2(
         rect.right() - space.md - style::SETTINGS_COMBO_W * 0.5,
         rect.center().y,
      ),
      vec2(style::SETTINGS_COMBO_W, 34.0),
   );
   if let Some(index) = options_combo(
      ui,
      &theme,
      combo,
      "settings_language",
      &LANGUAGES,
      app.language,
   ) {
      app.language = index;
   }

   ui.add_space(space.sm);
   ui.add(Divider::new());
   ui.add_space(space.sm);

   let (rect, _) = row(
      ui,
      &theme,
      Lucide::Monitor,
      "Default interface scale",
      style::SETTINGS_ROW_H,
      Sense::hover(),
      colors.text,
   );
   let switch = Rect::from_center_size(
      pos2(rect.right() - space.md - 19.0, rect.center().y),
      vec2(38.0, 20.0),
   );
   ui.put(
      switch,
      Switch::without_text(&mut app.scale_enabled).size(20.0),
   );

   scale_strip(ui, &theme);
   ui.add_space(space.sm);
}

/// Own-profile block: 72 pt avatar, display name and presence.
fn profile(ui: &mut Ui, theme: &Theme) {
   let colors = theme.colors;
   let space = theme.spacing;
   let plate = style::chat_plate(theme, 2);
   let mut visuals = AvatarVisuals::from_theme(colors);
   visuals.ring = colors.bg;
   let group = [Avatar::initials("CW").plate(plate).ink(colors.ink_on(plate))];

   Flex::row("settings_profile")
      .gap(space.md)
      .align_items(CrossAlign::Center)
      .show(ui, |f| {
         f.item(Item::fixed(style::AVATAR_PROFILE), |ui| {
            ui.add(
               AvatarGroup::new(&group)
                  .size(style::AVATAR_PROFILE)
                  .expand(false)
                  .visuals(visuals)
                  .label("Profile avatar"),
            );
         });
         f.item(Item::grow(1.0).stack(), |ui| {
            ui.spacing_mut().item_spacing.y = space.xs;
            ui.add(
               Label::new(
                  RichText::new("Christopher W.").size(20.0).color(colors.text),
                  None,
               )
               .interactive(false),
            );
            ui.add(
               Label::new(
                  RichText::new("online").size(13.0).color(colors.success),
                  None,
               )
               .interactive(false),
            );
         });
      });
}

/// A navigation row with a trailing chevron.
fn nav_row(ui: &mut Ui, theme: &Theme, icon: Lucide, label: &str) -> Response {
   let (rect, response) = row(
      ui,
      theme,
      icon,
      label,
      style::SETTINGS_ROW_H,
      Sense::click(),
      theme.colors.text,
   );
   chevron(ui, theme, rect);
   response
}

/// The Advanced row, with the "Beta" chip beside its title.
fn nav_row_with_chip(ui: &mut Ui, theme: &Theme) -> Response {
   let colors = theme.colors;
   let space = theme.spacing;
   let label = "Advanced";
   let (rect, response) = row(
      ui,
      theme,
      Lucide::Settings,
      label,
      style::SETTINGS_ROW_H,
      Sense::click(),
      colors.text,
   );
   if ui.is_rect_visible(rect) {
      let label_width = ui
         .painter()
         .layout_no_wrap(
            label.to_owned(),
            FontId::proportional(14.5),
            colors.text,
         )
         .size()
         .x;
      let chip = Rect::from_min_size(
         pos2(
            rect.left() + space.md + 20.0 + space.md + label_width + space.sm,
            rect.center().y - 10.0,
         ),
         vec2(64.0, 20.0),
      );
      ui.put(
         chip,
         Chip::new("Beta").role(ChipRole::Accent).padding(vec2(space.sm, 1.0)),
      );
   }
   chevron(ui, theme, rect);
   response
}

/// The six-stop interface-scale strip; the active stop is underlined.
///
/// The strip owns a full-width row of its own, below the label and its switch,
/// so the last stop can never run under the switch. Its ends sit one pad step
/// inside the card, and the labels are centred so the outer halves stay inside
/// that pad.
fn scale_strip(ui: &mut Ui, theme: &Theme) {
   let colors = theme.colors;
   let space = theme.spacing;
   let (rect, _) = ui.allocate_exact_size(
      vec2(ui.available_width(), style::SCALE_STRIP_H),
      Sense::hover(),
   );
   if !ui.is_rect_visible(rect) {
      return;
   }
   let left = rect.left() + space.md;
   let right = rect.right() - space.md;
   let step = (right - left) / (SCALES.len() as f32 - 1.0);
   let y = rect.center().y - 1.0;

   for (index, label) in SCALES.iter().enumerate() {
      let x = left + step * index as f32;
      let active = index == 0;
      ui.painter().text(
         pos2(x, y),
         Align2::CENTER_CENTER,
         *label,
         FontId::proportional(12.0),
         if active {
            colors.accent
         } else {
            colors.text_muted
         },
      );
      if active {
         let underline = Rect::from_min_max(pos2(x - 11.0, y + 9.0), pos2(x + 11.0, y + 11.0));
         ui.painter().rect_filled(underline, CornerRadius::ZERO, colors.accent);
      }
   }
}

// ---------------------------------------------------------------------------
// Advanced page
// ---------------------------------------------------------------------------

/// Back row, then the network, storage and media sections.
fn advanced_page(app: &mut TelegramApp, ui: &mut Ui) {
   let theme = app.theme.clone();
   let colors = theme.colors;
   let space = theme.spacing;

   back_row(app, ui, &theme);
   ui.add_space(space.md);

   section(ui, &theme, "Network and proxy");
   let (rect, _) = row(
      ui,
      &theme,
      Lucide::Globe,
      "Connection type",
      style::SETTINGS_ROW_SM,
      Sense::hover(),
      colors.text,
   );
   let combo = Rect::from_center_size(
      pos2(
         rect.right() - space.md - style::SETTINGS_COMBO_WIDE * 0.5,
         rect.center().y,
      ),
      vec2(style::SETTINGS_COMBO_WIDE, 34.0),
   );
   if let Some(index) = options_combo(
      ui,
      &theme,
      combo,
      "settings_connection",
      &CONNECTIONS,
      app.connection,
   ) {
      app.connection = index;
   }

   section(ui, &theme, "Data and storage");
   switch_row(
      ui,
      &theme,
      Lucide::FolderOpen,
      "Ask download path for each file",
      &mut app.download_path_ask,
   );
   let (rect, _) = row(
      ui,
      &theme,
      Lucide::FolderOpen,
      "Download path",
      style::SETTINGS_ROW_SM,
      Sense::hover(),
      colors.text,
   );
   value(ui, &theme, rect, "Default folder", colors.accent);

   let (rect, _) = row(
      ui,
      &theme,
      Lucide::HardDrive,
      "Manage local storage",
      style::SETTINGS_ROW_SM,
      Sense::hover(),
      colors.text,
   );
   value(ui, &theme, rect, "1.2 GB", colors.warning);

   let (rect, _) = row(
      ui,
      &theme,
      Lucide::Download,
      "Export Telegram data",
      style::SETTINGS_ROW_SM,
      Sense::click(),
      colors.info,
   );
   if ui.is_rect_visible(rect) {
      paint_link_hint(ui, &theme, rect);
   }

   let (rect, _) = row(
      ui,
      &theme,
      Lucide::Images,
      "Save to gallery",
      style::SETTINGS_ROW_SM,
      Sense::hover(),
      colors.text,
   );
   let check = Rect::from_center_size(
      pos2(rect.right() - space.md - 9.0, rect.center().y),
      Vec2::splat(18.0),
   );
   ui.put(
      check,
      Checkbox::without_text(&mut app.save_gallery).size(18.0),
   );

   section(ui, &theme, "Automatic media download");
   let (rect, _) = row(
      ui,
      &theme,
      Lucide::MessageSquare,
      "In private chats",
      style::SETTINGS_ROW_SM,
      Sense::hover(),
      colors.text,
   );
   media_radios(app, ui, &theme, rect);

   switch_row(
      ui,
      &theme,
      Lucide::Users,
      "In groups",
      &mut app.groups_media,
   );
   switch_row(
      ui,
      &theme,
      Lucide::Megaphone,
      "In channels",
      &mut app.channels_media,
   );
   ui.add_space(space.sm);
}

/// Chevron-left back row that returns to the Settings page.
fn back_row(app: &mut TelegramApp, ui: &mut Ui, theme: &Theme) {
   let colors = theme.colors;
   let space = theme.spacing;
   let (row_rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 34.0), Sense::hover());
   let inner = Rect::from_min_max(
      pos2(row_rect.left(), row_rect.top()),
      pos2(row_rect.right(), row_rect.bottom()),
   );
   ui.scope_builder(
      UiBuilder::new().max_rect(inner).layout(Layout::left_to_right(Align::Center)),
      |ui| {
         if paint::icon_button(
            ui,
            Lucide::ChevronLeft,
            18.0,
            colors.text_muted,
            colors.hover,
         )
         .clicked()
         {
            app.settings_page = SettingsPage::Settings;
         }
         ui.add_space(space.xs);
         ui.label(RichText::new("Advanced").size(15.5).color(colors.text));
      },
   );
}

/// A section heading in the accent.
fn section(ui: &mut Ui, theme: &Theme, title: &str) {
   ui.add_space(theme.spacing.md);
   ui.label(RichText::new(title).size(13.0).color(theme.colors.accent));
   ui.add_space(theme.spacing.xs);
}

/// A row whose trailing control is an on/off switch.
fn switch_row(ui: &mut Ui, theme: &Theme, icon: Lucide, label: &str, value: &mut bool) {
   let (rect, _) = row(
      ui,
      theme,
      icon,
      label,
      style::SETTINGS_ROW_SM,
      Sense::hover(),
      theme.colors.text,
   );
   let switch = Rect::from_center_size(
      pos2(
         rect.right() - theme.spacing.md - 19.0,
         rect.center().y,
      ),
      vec2(38.0, 22.0),
   );
   ui.put(switch, Switch::without_text(value).size(18.0));
}

/// The three private-chat media radios, laid out in one row.
fn media_radios(app: &mut TelegramApp, ui: &mut Ui, theme: &Theme, rect: Rect) {
   let colors = theme.colors;
   let space = theme.spacing;
   let group = Rect::from_min_max(
      pos2(rect.right() - space.md - 292.0, rect.top()),
      pos2(rect.right() - space.md, rect.bottom()),
   );

   ui.scope_builder(
      UiBuilder::new().max_rect(group).layout(Layout::left_to_right(Align::Center)),
      |ui| {
         ui.spacing_mut().item_spacing.x = space.sm;
         for choice in PrivateMedia::ALL {
            ui.add(
               Radio::new(
                  &mut app.private_media,
                  choice,
                  RichText::new(choice.label()).size(13.0).color(colors.text),
               )
               .size(16.0),
            );
         }
      },
   );
}

/// A combo box owning its own open state; returns the chosen index.
fn options_combo(
   ui: &mut Ui,
   theme: &Theme,
   rect: Rect,
   id: &'static str,
   options: &[&str],
   selected: usize,
) -> Option<usize> {
   let colors = theme.colors;
   let space = theme.spacing;
   let label = Label::new(
      RichText::new(options[selected]).size(13.5).color(colors.text),
      None,
   );
   let mut chosen = None;

   ui.scope_builder(
      UiBuilder::new().max_rect(rect).layout(Layout::left_to_right(Align::Center)),
      |ui| {
         let _ = ComboBox::new(id, label).width(rect.width()).show_ui(ui, |ui| {
            for (index, name) in options.iter().enumerate() {
               let row = Label::new(
                  RichText::new(*name).size(13.5).color(colors.text),
                  None,
               )
               .expand(Some(space.xs))
               .selected(selected == index)
               .sense(Sense::click())
               .fill_width(true);
               if ui.add(row).clicked() {
                  chosen = Some(index);
               }
            }
         });
      },
   );

   chosen
}

// ---------------------------------------------------------------------------
// Shared row geometry
// ---------------------------------------------------------------------------

/// One settings row: a 20 pt leading icon, a 14.5 pt label, an optional hover
/// fill. The returned rect is where the caller puts its trailing control.
fn row(
   ui: &mut Ui,
   theme: &Theme,
   icon: Lucide,
   label: &str,
   height: f32,
   sense: Sense,
   ink: Color32,
) -> (Rect, Response) {
   let colors = theme.colors;
   let space = theme.spacing;
   let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), height), sense);
   if !ui.is_rect_visible(rect) {
      return (rect, response);
   }

   if response.hovered() {
      ui.painter().rect_filled(
         rect,
         CornerRadius::same(style::ROW_RADIUS),
         colors.hover,
      );
   }
   let icon_rect = Rect::from_center_size(
      pos2(rect.left() + space.md + 10.0, rect.center().y),
      Vec2::splat(20.0),
   );
   icon.size(20.0).color(colors.text_muted).image().paint_at(ui, icon_rect);
   ui.painter().text(
      pos2(icon_rect.right() + space.md, rect.center().y),
      Align2::LEFT_CENTER,
      label,
      FontId::proportional(14.5),
      ink,
   );
   (rect, response)
}

/// A trailing chevron on a navigation row.
fn chevron(ui: &Ui, theme: &Theme, rect: Rect) {
   if !ui.is_rect_visible(rect) {
      return;
   }
   Lucide::ChevronRight.size(16.0).color(theme.colors.text_muted).image().paint_at(
      ui,
      Rect::from_center_size(
         pos2(
            rect.right() - theme.spacing.md - 8.0,
            rect.center().y,
         ),
         Vec2::splat(16.0),
      ),
   );
}

/// A right-aligned trailing value.
fn value(ui: &Ui, theme: &Theme, rect: Rect, text: &str, ink: Color32) {
   ui.painter().text(
      pos2(rect.right() - theme.spacing.md, rect.center().y),
      Align2::RIGHT_CENTER,
      text,
      FontId::proportional(13.5),
      ink,
   );
}

/// A short underline under the export row, so it reads as a link.
fn paint_link_hint(ui: &Ui, theme: &Theme, rect: Rect) {
   let colors = theme.colors;
   let width = ui
      .painter()
      .layout_no_wrap(
         "Export Telegram data".to_owned(),
         FontId::proportional(14.5),
         colors.info,
      )
      .size()
      .x;
   let left = rect.left() + theme.spacing.md + 20.0 + theme.spacing.md;
   let underline = Rect::from_min_max(
      pos2(left, rect.center().y + 11.0),
      pos2(left + width, rect.center().y + 12.0),
   );
   ui.painter().rect_filled(
      underline,
      CornerRadius::ZERO,
      colors.info.gamma_multiply(0.6),
   );
}
