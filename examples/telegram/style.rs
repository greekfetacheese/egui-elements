//! Size constants and colour helpers for the Telegram demo.
//!
//! One source of truth: every pane, overlay and audit panel reads its geometry
//! and its derived colours from here. Gaps and padding come from
//! `Theme::spacing`; the constants below are component sizes, not step values.

use eframe::egui::{Color32, Stroke};
use egui_elements::theme::{ColorRole, DerivedColor, Theme};
use egui_elements::utils::mix;

/// Sidebar width in points.
pub const SIDEBAR_W: f32 = 380.0;
/// Title strip height in points.
pub const TITLE_H: f32 = 34.0;
/// Conversation header height in points.
pub const HEADER_H: f32 = 60.0;
/// Sidebar header height in points.
pub const SIDEBAR_HEAD_H: f32 = 52.0;
/// Chat row height in points.
pub const ROW_H: f32 = 72.0;
/// Avatar diameter in a chat row.
pub const AVATAR_ROW: f32 = 48.0;
/// Avatar diameter in the conversation header.
pub const AVATAR_HEADER: f32 = 40.0;
/// Avatar diameter on a profile overlay.
pub const AVATAR_PROFILE: f32 = 72.0;
/// Widest a chat bubble may get, in points.
pub const BUBBLE_MAX_W: f32 = 520.0;
/// Corner radius of a chat bubble.
pub const BUBBLE_RADIUS: f32 = 12.0;
/// Composer height in points.
pub const COMPOSER_H: f32 = 64.0;
/// Corner radius of a floating panel.
pub const PANEL_RADIUS: f32 = 12.0;
/// Width of a modal dialog in points.
pub const MODAL_W: f32 = 632.0;
/// Width of the theme-audit window.
pub const AUDIT_W: f32 = 736.0;
/// Tallest the theme-audit window may grow.
pub const AUDIT_H: f32 = 760.0;
/// Tallest the audit body scrolls before it clips (window chrome is the rest).
pub const AUDIT_BODY_H: f32 = 700.0;
/// Emoji picker column count.
pub const EMOJI_COLS: usize = 7;

/// Search pill height in points.
pub const SEARCH_H: f32 = 40.0;
/// Search pill corner radius.
pub const SEARCH_RADIUS: u8 = 9;
/// Chat row corner radius.
pub const ROW_RADIUS: u8 = 8;
/// Width of the right-hand time / status column in a chat row.
pub const META_W: f32 = 64.0;

/// Avatar plate for a chat, cycled deterministically from the palette.
///
/// The role colour is pulled toward the canvas so the plate reads as a pastel
/// swatch rather than a full-strength fill; `index` alternates the pull so two
/// neighbours on the same role stay distinguishable.
pub fn chat_plate(theme: &Theme, index: usize) -> Color32 {
   let colors = theme.colors;
   let roles = [colors.accent, colors.info, colors.success, colors.warning];
   let role = roles[index % roles.len()];
   let pull = 0.30 + 0.06 * ((index / roles.len()) % 3) as f32;
   mix(role, colors.bg, pull)
}

/// Outgoing bubble fill: a 32 % accent wash over `widget_bg`.
///
/// The strength is deliberately past a light tint: on the dark palettes
/// `widget_bg` and a 0.18 accent mix read as the same grey, so the two sides of
/// a conversation lose their shape. Naming the wash lets the audit panel check
/// it like a palette slot.
pub const BUBBLE_OUT: DerivedColor = DerivedColor::new("bubble.out", ColorRole::Accent, 0.32);

/// Bubble fill: incoming `widget_bg`, outgoing the accent wash.
pub fn bubble_fill(theme: &Theme, outgoing: bool) -> Color32 {
   let colors = theme.colors;
   if outgoing {
      BUBBLE_OUT.fill(colors)
   } else {
      colors.widget_bg
   }
}

/// The one-point edge that reads a bubble as a card on the wallpaper.
pub fn bubble_stroke(theme: &Theme, outgoing: bool) -> Stroke {
   let colors = theme.colors;
   let tint = if outgoing {
      colors.accent
   } else {
      colors.text_muted
   };
   Stroke::new(1.0, mix(colors.border, tint, 0.35))
}

/// `color` at `alpha`, keeping the palette's hue (shadows, vignettes, hairlines).
pub fn translucent(color: Color32, alpha: u8) -> Color32 {
   Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}

/// One-point rule in the theme's border colour.
pub fn hairline(theme: &Theme) -> Stroke {
   Stroke::new(1.0, theme.colors.border)
}

/// Bubble inner padding: text sits 10 pt from the sides and 7 pt from the top
/// and bottom. Telegram's own numbers, not a spacing step.
pub const BUBBLE_PAD_X: i8 = 10;
/// Bubble vertical inner padding.
pub const BUBBLE_PAD_Y: i8 = 7;
/// Corner radius where the bubble points at its sender.
pub const BUBBLE_TAIL: u8 = 4;
/// Quote band corner radius inside a bubble.
pub const QUOTE_RADIUS: u8 = 6;
/// Width of the accent bar on a quote band.
pub const QUOTE_BAR_W: f32 = 3.0;
/// Day separator pill corner radius.
pub const DAY_RADIUS: u8 = 10;
/// Day separator horizontal padding.
pub const DAY_PAD_X: i8 = 10;
/// Photo bubble width in points (clamped to the bubble's inner width).
pub const PHOTO_W: f32 = 380.0;
/// Photo bubble image height in points.
pub const PHOTO_H: f32 = 200.0;
/// Diameter of the play / download disc in a voice or file message.
pub const VOICE_PLAY: f32 = 44.0;
/// Waveform width in a voice message.
pub const WAVEFORM_W: f32 = 170.0;
/// Waveform height in a voice message.
pub const WAVEFORM_H: f32 = 26.0;
/// Sticker glyph size in points.
pub const STICKER_SIZE: f32 = 64.0;
/// Composer field height in points.
pub const COMPOSER_FIELD_H: f32 = 40.0;
/// Composer field corner radius.
pub const COMPOSER_FIELD_RADIUS: u8 = 10;
/// Height of the meta line under a bubble body.
pub const META_H: f32 = 14.0;
/// Width reserved for the delivery ticks.
pub const TICK_W: f32 = 14.0;
/// Size of the failed-message marker.
pub const FAIL_ICON: f32 = 13.0;

/// Width of the emoji / sticker / GIF picker panel.
pub const EMOJI_PANEL_W: f32 = 548.0;
/// Height of the picker panel.
pub const EMOJI_PANEL_H: f32 = 452.0;
/// Height of the picker's tab row.
pub const EMOJI_TAB_H: f32 = 44.0;
/// Height of the picker's category strip.
pub const EMOJI_STRIP_H: f32 = 44.0;
/// Width of one emoji hit cell.
pub const EMOJI_CELL_W: f32 = 46.0;
/// Height of one emoji hit cell.
pub const EMOJI_CELL_H: f32 = 38.0;
/// Emoji glyph size in the picker grid.
pub const EMOJI_GLYPH: f32 = 26.0;
/// Corner radius of an emoji hover plate.
pub const EMOJI_RADIUS: u8 = 6;
/// Sticker tile width.
pub const STICKER_TILE_W: f32 = 150.0;
/// Sticker tile height.
pub const STICKER_TILE_H: f32 = 130.0;
/// Sticker tile corner radius.
pub const STICKER_TILE_RADIUS: u8 = 10;
/// GIF tile width.
pub const GIF_TILE_W: f32 = 120.0;
/// GIF tile height.
pub const GIF_TILE_H: f32 = 90.0;
/// GIF tile corner radius.
pub const GIF_TILE_RADIUS: u8 = 8;
/// Width of the attachment menu.
pub const ATTACH_W: f32 = 220.0;
/// Height of a row in the attachment menu.
pub const ATTACH_ROW_H: f32 = 38.0;
/// Corner radius of the attachment menu.
pub const ATTACH_RADIUS: u8 = 10;
/// Height of one field in the New Contact form.
pub const CONTACT_FIELD_H: f32 = 40.0;
/// Tallest a settings page body scrolls before it clips. Both pages are under
/// this cap, so the modal sizes to each page's content instead of a fixed box.
pub const SETTINGS_BODY_H: f32 = 520.0;
/// Height of a settings-page row.
pub const SETTINGS_ROW_H: f32 = 48.0;
/// Height of an advanced-page row.
pub const SETTINGS_ROW_SM: f32 = 40.0;
/// Height of the interface-scale strip.
pub const SCALE_STRIP_H: f32 = 30.0;
/// Width of the language picker.
pub const SETTINGS_COMBO_W: f32 = 140.0;
/// Width of the connection picker; its labels are longer.
pub const SETTINGS_COMBO_WIDE: f32 = 180.0;
