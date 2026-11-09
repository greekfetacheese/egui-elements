//! A Telegram Desktop-shaped demo for `egui-elements`: a title strip, a chat
//! sidebar and a conversation pane, built from public egui-elements / egui
//! APIs only — no binary assets, every colour from the installed `Theme`.
//!
//! The conversation pane lives in `chat::show`; overlays and settings hang
//! over it, and the title strip opens a live theme audit of the installed
//! palette.
//!
//! Run with: `cargo run --features full --example telegram`

mod audit;
mod chat;
mod data;
mod overlays;
mod paint;
mod settings;
mod sidebar;
mod style;

use eframe::egui::*;
use egui_elements::components::InputField;
use egui_elements::editor::ThemeEditor;
use egui_elements::egui_lucide::Lucide;
use egui_elements::layout::{CrossAlign, Flex, Item, Justify};
use egui_elements::theme::{Theme, ThemeKind};
use egui_elements::utils;

/// Window title shared by the viewport and `run_native`.
const TITLE: &str = "Telegram Desktop — egui-elements";

/// Demo state: the installed theme, its editor, the sidebar filter, the
/// selected chat, and the overlays / settings a user can open.
pub struct TelegramApp {
   /// Palette installed into the context every frame.
   pub(crate) theme: Theme,
   /// Live palette editor, toggled from the title strip.
   pub(crate) editor: ThemeEditor,
   /// Sidebar query; the chat list filters on title and preview.
   pub(crate) search: String,
   /// `Chat::id` of the row selected in the list.
   pub(crate) selected_id: usize,
   /// Draft typed into the conversation composer.
   pub(crate) composer: String,
   /// Emoji picker open; the composer's smiley flips this flag.
   pub(crate) emoji_open: bool,
   /// Which picker tab is showing (emoji, stickers or GIFs).
   pub(crate) emoji_tab: overlays::EmojiTab,
   /// Attachment menu open; the composer's paperclip flips this flag.
   pub(crate) attach_open: bool,
   /// New Contact modal open, from the sidebar header's add button.
   pub(crate) new_contact_open: bool,
   /// When the mock create started, or `None` while CREATE is idle.
   pub(crate) creating_since: Option<f64>,
   /// First-name field in the New Contact form; it owns its text.
   pub(crate) contact_first: InputField,
   /// Last-name field in the New Contact form.
   pub(crate) contact_last: InputField,
   /// Phone field in the New Contact form; starts on the `+49` prefix.
   pub(crate) contact_phone: InputField,
   /// Settings modal open, from the title strip's gear.
   pub(crate) settings_open: bool,
   /// Theme-audit window open, from the title strip's palette button.
   pub(crate) audit_open: bool,
   /// Composer button the emoji picker hangs off, recorded each frame.
   pub(crate) emoji_anchor: Rect,
   /// Composer button the attachment menu hangs off.
   pub(crate) attach_anchor: Rect,
   /// Settings or its Advanced sub-page.
   pub(crate) settings_page: settings::SettingsPage,
   /// Selected language index in the settings picker.
   pub(crate) language: usize,
   /// Selected connection index on the Advanced page.
   pub(crate) connection: usize,
   /// "Ask download path for each file" switch.
   pub(crate) download_path_ask: bool,
   /// "Save to gallery" checkbox.
   pub(crate) save_gallery: bool,
   /// Automatic media download policy for private chats.
   pub(crate) private_media: settings::PrivateMedia,
   /// Download media in groups automatically.
   pub(crate) groups_media: bool,
   /// Download media in channels automatically.
   pub(crate) channels_media: bool,
   /// Interface-scale switch; the strip below it is visual only.
   pub(crate) scale_enabled: bool,
}

/// One New Contact field: it owns its text, and the label doubles as its id.
fn contact_field(label: &'static str, text: &str) -> InputField {
   let mut field = InputField::new(label, true).qr_enabled(false);
   field.set_text(text);
   field
}

fn main() -> eframe::Result {
   let options = eframe::NativeOptions {
      viewport: ViewportBuilder::default()
         .with_title(TITLE)
         .with_inner_size([1280.0, 820.0])
         .with_min_inner_size([1024.0, 640.0])
         .with_resizable(true),
      ..Default::default()
   };

   eframe::run_native(
      TITLE,
      options,
      Box::new(|cc| Ok(Box::new(TelegramApp::new(cc)))),
   )
}

impl TelegramApp {
   fn new(cc: &eframe::CreationContext<'_>) -> Self {
      egui_extras::install_image_loaders(&cc.egui_ctx);

      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&cc.egui_ctx);

      Self {
         theme,
         editor: ThemeEditor::new(),
         search: String::new(),
         selected_id: data::EVA_ID,
         composer: String::new(),
         emoji_open: false,
         emoji_tab: overlays::EmojiTab::default(),
         attach_open: false,
         new_contact_open: false,
         creating_since: None,
         contact_first: contact_field("First Name", ""),
         contact_last: contact_field("Last Name", ""),
         contact_phone: contact_field("Phone Number", "+49 "),
         settings_open: false,
         audit_open: false,
         emoji_anchor: Rect::NOTHING,
         attach_anchor: Rect::NOTHING,
         settings_page: settings::SettingsPage::default(),
         language: 0,
         connection: 0,
         download_path_ask: true,
         save_gallery: false,
         private_media: settings::PrivateMedia::default(),
         groups_media: true,
         channels_media: false,
         scale_enabled: true,
      }
   }

   /// The 34 pt chrome strip: brand on the left, switcher and editor on the right.
   fn title_strip(&mut self, ui: &mut Ui) {
      let colors = self.theme.colors;
      let space = self.theme.spacing;
      let (rect, _) = ui.allocate_exact_size(
         vec2(ui.available_width(), style::TITLE_H),
         Sense::hover(),
      );

      {
         let painter = ui.painter();
         painter.rect_filled(rect, CornerRadius::ZERO, colors.title_bar);
         painter.hline(
            rect.x_range(),
            rect.bottom() - 0.5,
            style::hairline(&self.theme),
         );
      }

      let inner = Rect::from_min_max(
         pos2(rect.left() + space.lg, rect.top()),
         pos2(rect.right() - space.lg, rect.bottom()),
      );
      ui.scope_builder(
         UiBuilder::new().max_rect(inner).layout(Layout::left_to_right(Align::Center)),
         |ui| {
            ui.spacing_mut().button_padding = vec2(space.sm, space.xs);
            Flex::row("title_strip")
               .justify(Justify::SpaceBetween)
               .align_items(CrossAlign::Center)
               .gap(space.sm)
               .show(ui, |f| {
                  f.item(Item::new(), |ui| self.brand(ui));
                  f.item(Item::new(), |ui| self.strip_actions(ui));
               });
         },
      );
   }

   fn brand(&self, ui: &mut Ui) {
      let colors = self.theme.colors;
      let space = self.theme.spacing;

      Flex::row_auto("title_brand")
         .gap(space.sm)
         .align_items(CrossAlign::Center)
         .show(ui, |f| {
            f.item(Item::new(), |ui| {
               ui.add(Lucide::Send.size(15.0).color(colors.accent).image());
            });
            f.item(Item::new(), |ui| {
               ui.label(RichText::new("Telegram Desktop").size(13.5).color(colors.text));
            });
            f.item(Item::new(), |ui| {
               ui.label(RichText::new("·").size(12.0).color(colors.text_muted));
            });
            f.item(Item::new(), |ui| {
               ui.label(RichText::new("egui-elements demo").size(12.0).color(colors.text_muted));
            });
         });
   }

   fn strip_actions(&mut self, ui: &mut Ui) {
      let colors = self.theme.colors;
      let space = self.theme.spacing;

      Flex::row_auto("title_actions")
         .gap(space.sm)
         .align_items(CrossAlign::Center)
         .show(ui, |f| {
            f.item(Item::new(), |ui| {
               if let Some(new_theme) = utils::theme_switcher(&self.theme, ui) {
                  self.theme = new_theme;
               }
            });
            f.item(Item::new(), |ui| {
               if paint::icon_button(ui, Lucide::Settings, 16.0, colors.text_muted).clicked() {
                  self.settings_open = true;
               }
            });
            f.item(Item::new(), |ui| {
               let audit = Lucide::Palette;
               if paint::icon_button(ui, audit, 16.0, colors.text_muted).clicked() {
                  self.audit_open = !self.audit_open;
               }
            });
            f.item(Item::new(), |ui| {
               let editor = Lucide::SlidersHorizontal;
               if paint::icon_button(ui, editor, 16.0, colors.text_muted).clicked() {
                  self.editor.open = !self.editor.open;
               }
            });
         });
   }

   /// Sidebar and conversation side by side, both stretched to the pane height.
   fn shell(&mut self, ui: &mut Ui) {
      Flex::row("shell").gap(0.0).align_items(CrossAlign::Stretch).show(ui, |f| {
         f.item(Item::fixed(style::SIDEBAR_W), |ui| {
            sidebar::show(self, ui)
         });
         f.item(Item::grow(1.0), |ui| chat::show(self, ui));
      });
   }
}

impl eframe::App for TelegramApp {
   fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
      let bg = self.theme.colors.bg;

      CentralPanel::default().frame(Frame::NONE.fill(bg)).show(ui, |ui| {
         self.title_strip(ui);

         if let Some(new_theme) = self.editor.show(&mut self.theme, ui) {
            self.theme = new_theme;
         }
         // Install after the switcher and editor every frame, never only on `Some`.
         self.theme.install(ui.ctx());

         self.shell(ui);

         // Overlays hang off the context, above the shell's own layers; the
         // audit window sits on the same layer and the modals stay on top.
         audit::show(self, ui.ctx());
         overlays::show(self, ui.ctx());
         settings::show(self, ui.ctx());
      });
   }
}
