//! Product-framed reference client for egui-elements: a mail client that
//! exercises the layout containers, the widget set, and every palette role
//! on the surfaces a real product uses. Nothing here talks to a mail backend.
//!
//! The checks live in the composition, not the inbox data:
//! - Flex distributes every header, row, and footer.
//! - Table lays out the attachment list in the details rail.
//! - frame1 sits on bg, frame2 sits on frame1, and frame1 nests on frame2
//!   in the quoted reply.
//! - Every ThemeColors slot appears in a job: title_bar chrome, canvas,
//!   elevated cards, hover rows, selection, muted metadata, borders, the five
//!   semantic colours, and accent as the primary action.
//!
//! Run with: cargo run --example email

use eframe::egui::scroll_area::ScrollBarVisibility;
use eframe::egui::*;
use egui_elements::editor::ThemeEditor;
use egui_elements::layout::{Column, CrossAlign, Flex, Item, Justify, Table};
use egui_elements::theme::{Theme, ThemeColors, ThemeKind};
use egui_elements::utils;
use egui_elements::visuals::ButtonVisuals;
use egui_elements::widgets::{
   Avatar, AvatarGroup, AvatarVisuals, Badge, Button, Checkbox, ComboBox, Frame as Card, HoldBurst,
   Label, Modal, MultiLabel, Radio, SecureTextEdit, Spinner, SpinnerShadow, Switch, Window,
};

fn main() -> eframe::Result {
   let options = eframe::NativeOptions {
      viewport: ViewportBuilder::default()
         .with_title("egui mail")
         .with_inner_size([1360.0, 900.0])
         .with_min_inner_size([1120.0, 700.0])
         .with_resizable(true),
      ..Default::default()
   };

   eframe::run_native(
      "egui mail",
      options,
      Box::new(|cc| Ok(Box::new(MailApp::new(cc)))),
   )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Folder {
   Inbox,
   Starred,
   Sent,
   Drafts,
   Archive,
   Trash,
}

impl Folder {
   const ALL: [Folder; 6] = [
      Folder::Inbox,
      Folder::Starred,
      Folder::Sent,
      Folder::Drafts,
      Folder::Archive,
      Folder::Trash,
   ];

   fn label(self) -> &'static str {
      match self {
         Folder::Inbox => "Inbox",
         Folder::Starred => "Starred",
         Folder::Sent => "Sent",
         Folder::Drafts => "Drafts",
         Folder::Archive => "Archive",
         Folder::Trash => "Trash",
      }
   }

   fn icon(self) -> Icon {
      match self {
         Folder::Inbox => Icon::Inbox,
         Folder::Starred => Icon::Star,
         Folder::Sent => Icon::Sent,
         Folder::Drafts => Icon::Draft,
         Folder::Archive => Icon::Archive,
         Folder::Trash => Icon::Trash,
      }
   }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum InboxFilter {
   All,
   Unread,
   Attachments,
}

impl InboxFilter {
   const ALL: [InboxFilter; 3] = [
      InboxFilter::All,
      InboxFilter::Unread,
      InboxFilter::Attachments,
   ];

   fn label(self) -> &'static str {
      match self {
         InboxFilter::All => "All",
         InboxFilter::Unread => "Unread",
         InboxFilter::Attachments => "Files",
      }
   }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SortOrder {
   Newest,
   Unread,
   Sender,
}

impl SortOrder {
   fn label(self) -> &'static str {
      match self {
         SortOrder::Newest => "Newest",
         SortOrder::Unread => "Unread first",
         SortOrder::Sender => "Sender",
      }
   }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Priority {
   Low,
   Normal,
   High,
}

#[derive(Clone, Copy)]
enum TagRole {
   Accent,
   Info,
   Warning,
   Success,
   Error,
}

impl TagRole {
   fn color(self, colors: ThemeColors) -> Color32 {
      match self {
         TagRole::Accent => colors.accent,
         TagRole::Info => colors.info,
         TagRole::Warning => colors.warning,
         TagRole::Success => colors.success,
         TagRole::Error => colors.error,
      }
   }
}

#[derive(Clone, Copy)]
struct Tag {
   text: &'static str,
   role: TagRole,
}

#[derive(Clone, Copy)]
struct Attachment {
   name: &'static str,
   size: &'static str,
   kind: &'static str,
}

struct Message {
   id: u32,
   folder: Folder,
   starred: bool,
   unread: bool,
   from: &'static str,
   address: &'static str,
   to: &'static str,
   subject: &'static str,
   preview: &'static str,
   time: &'static str,
   thread_count: u8,
   body: &'static str,
   tags: &'static [Tag],
   attachments: &'static [Attachment],
}

const NO_ATTACHMENTS: &[Attachment] = &[];

const MESSAGES: &[Message] = &[
   Message {
      id: 1,
      folder: Folder::Inbox,
      starred: true,
      unread: true,
      from: "Alice Chen",
      address: "alice@figma.com",
      to: "you@egui.dev",
      subject: "Q3 design review - reading pane density",
      preview: "The sidebar density feels right. Two notes on the reading pane before we lock the cut.",
      time: "10:24",
      thread_count: 3,
      body: "Hi Mike,\n\nI went through the latest mockups this morning. The sidebar density feels right now - the folder labels stay quiet and the unread counts carry the emphasis.\n\nA few notes before we lock the next cut:\n- Keep the reading pane on the same canvas as the thread list.\n- Compose should stay modal so the inbox does not lose context.\n- Check frame1 on frame2, and the reverse, against every palette.",
      tags: &[
         Tag {
            text: "Design",
            role: TagRole::Info,
         },
         Tag {
            text: "Needs reply",
            role: TagRole::Warning,
         },
      ],
      attachments: &[
         Attachment {
            name: "q3-review.fig",
            size: "8.4 MB",
            kind: "Figma",
         },
         Attachment {
            name: "reading-pane.png",
            size: "1.2 MB",
            kind: "Preview",
         },
      ],
   },
   Message {
      id: 2,
      folder: Folder::Inbox,
      starred: false,
      unread: true,
      from: "GitHub",
      address: "notifications@github.com",
      to: "you@egui.dev",
      subject: "[egui-elements] Review requested: #42",
      preview: "mike requested your review on pull request #42: theme tokens.",
      time: "9:02",
      thread_count: 2,
      body: "mike requested your review on pull request #42: theme tokens.\n\nhighlight must stay distinct from hover. text_muted is not a selection fill.\n\nhttps://github.com/greekfetacheese/egui-elements/pull/42",
      tags: &[Tag {
         text: "Engineering",
         role: TagRole::Accent,
      }],
      attachments: &[Attachment {
         name: "review.diff",
         size: "24 KB",
         kind: "Patch",
      }],
   },
   Message {
      id: 3,
      folder: Folder::Inbox,
      starred: false,
      unread: false,
      from: "Nora Patel",
      address: "nora@northstar.studio",
      to: "you@egui.dev",
      subject: "Invoice for March",
      preview: "Attached the PDF. Let me know if the line items look off.",
      time: "Yesterday",
      thread_count: 1,
      body: "Attached the PDF. Let me know if the line items look off.\n\nTotal: $1,240.00\nDue: April 12\n\nThanks,\nNora",
      tags: &[Tag {
         text: "Invoice",
         role: TagRole::Warning,
      }],
      attachments: &[Attachment {
         name: "invoice-march.pdf",
         size: "184 KB",
         kind: "PDF",
      }],
   },
   Message {
      id: 4,
      folder: Folder::Inbox,
      starred: false,
      unread: false,
      from: "This Week in Rust",
      address: "read@this-week-in-rust.org",
      to: "you@egui.dev",
      subject: "This Week in Rust #592",
      preview: "Cranelift news, a new egui release, and interesting blog posts.",
      time: "Mon",
      thread_count: 1,
      body: "Cranelift news, a new egui release, and interesting blog posts.\n\nQuote of the week:\n\nMake illegal states unrepresentable.\n\nThat is all for this week.",
      tags: &[Tag {
         text: "Reading",
         role: TagRole::Info,
      }],
      attachments: NO_ATTACHMENTS,
   },
   Message {
      id: 5,
      folder: Folder::Inbox,
      starred: true,
      unread: false,
      from: "Mike",
      address: "mike@egui.dev",
      to: "you@egui.dev",
      subject: "Theme tokens before the next cut",
      preview: "Can we try the mail layout against frame1 and frame2 before the next cut?",
      time: "Sun",
      thread_count: 1,
      body: "Can we try the mail layout against frame1 and frame2 before the next cut?\n\nI want to see buttons, labels, combos, and text edits in a product chrome - not just the gallery.\n\nHeader stays the demo chrome: theme switcher and editor only.",
      tags: &[Tag {
         text: "Engineering",
         role: TagRole::Accent,
      }],
      attachments: NO_ATTACHMENTS,
   },
   Message {
      id: 6,
      folder: Folder::Inbox,
      starred: false,
      unread: true,
      from: "Figma",
      address: "comments@figma.com",
      to: "you@egui.dev",
      subject: "3 new comments on Mail client",
      preview: "Alice and Nora left comments on the reading pane and details rail.",
      time: "Sat",
      thread_count: 4,
      body: "Alice and Nora left comments on the reading pane and details rail.\n\nOpen the file to reply in context. You can resolve comments from the right rail.",
      tags: &[Tag {
         text: "Design",
         role: TagRole::Info,
      }],
      attachments: &[Attachment {
         name: "mail-client.fig",
         size: "11.8 MB",
         kind: "Figma",
      }],
   },
   Message {
      id: 7,
      folder: Folder::Inbox,
      starred: false,
      unread: true,
      from: "Linear",
      address: "updates@linear.app",
      to: "you@egui.dev",
      subject: "Sprint 14 digest",
      preview: "12 issues shipped, 3 moved to review, and one palette regression caught.",
      time: "Fri",
      thread_count: 1,
      body: "12 issues shipped, 3 moved to review, and one palette regression caught.\n\nThe regression was a hover fill that collapsed into highlight. The new mail reference made it obvious in one pass.",
      tags: &[Tag {
         text: "Engineering",
         role: TagRole::Info,
      }],
      attachments: NO_ATTACHMENTS,
   },
   Message {
      id: 8,
      folder: Folder::Sent,
      starred: false,
      unread: false,
      from: "you@egui.dev",
      address: "you@egui.dev",
      to: "Alice Chen",
      subject: "Re: Q3 design review",
      preview: "Agreed on the sidebar. I will mock the reading pane tonight.",
      time: "11:03",
      thread_count: 2,
      body: "Agreed on the sidebar. I will mock the reading pane tonight.\n\nAlso dropping a compose modal so we can see SecureTextEdit stacked in a real flow.",
      tags: &[Tag {
         text: "Design",
         role: TagRole::Info,
      }],
      attachments: NO_ATTACHMENTS,
   },
   Message {
      id: 9,
      folder: Folder::Drafts,
      starred: false,
      unread: false,
      from: "you@egui.dev",
      address: "you@egui.dev",
      to: "",
      subject: "Weekend plans",
      preview: "Draft - still missing a recipient.",
      time: "Sat",
      thread_count: 1,
      body: "Draft - still missing a recipient.\n\nThinking of heading north if the weather holds.",
      tags: &[Tag {
         text: "Personal",
         role: TagRole::Success,
      }],
      attachments: NO_ATTACHMENTS,
   },
   Message {
      id: 10,
      folder: Folder::Archive,
      starred: false,
      unread: false,
      from: "Stripe",
      address: "receipts@stripe.com",
      to: "you@egui.dev",
      subject: "Your payout is on the way",
      preview: "$2,480.00 will arrive in your account on Friday.",
      time: "Mar 2",
      thread_count: 1,
      body: "$2,480.00 will arrive in your account on Friday.\n\nNo action is needed. You can download the receipt from the details rail.",
      tags: &[Tag {
         text: "Finance",
         role: TagRole::Success,
      }],
      attachments: &[Attachment {
         name: "payout-receipt.pdf",
         size: "96 KB",
         kind: "PDF",
      }],
   },
   Message {
      id: 11,
      folder: Folder::Trash,
      starred: false,
      unread: false,
      from: "Prize Desk",
      address: "winner@example.invalid",
      to: "you@egui.dev",
      subject: "You have won",
      preview: "Definitely not a prize.",
      time: "Mar 1",
      thread_count: 1,
      body: "Definitely not a prize.",
      tags: &[Tag {
         text: "Spam",
         role: TagRole::Error,
      }],
      attachments: NO_ATTACHMENTS,
   },
];

struct MailApp {
   theme: Theme,
   editor: ThemeEditor,
   folder: Folder,
   selected_id: u32,
   search: String,
   account: usize,
   filter: InboxFilter,
   sort: SortOrder,
   select_all: bool,
   compose_open: bool,
   delete_open: bool,
   shortcuts_open: bool,
   show_details: bool,
   compose_to: String,
   compose_subject: String,
   compose_body: String,
   compose_cc: String,
   compose_receipt: bool,
   reply: String,
   reply_all: bool,
   mute_thread: bool,
   notify_on_reply: bool,
   priority: Priority,
}

impl MailApp {
   fn new(cc: &eframe::CreationContext<'_>) -> Self {
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&cc.egui_ctx);

      Self {
         theme,
         editor: ThemeEditor::new(),
         folder: Folder::Inbox,
         selected_id: 1,
         search: String::new(),
         account: 0,
         filter: InboxFilter::All,
         sort: SortOrder::Newest,
         select_all: false,
         compose_open: false,
         delete_open: false,
         shortcuts_open: false,
         show_details: true,
         compose_to: String::new(),
         compose_subject: String::new(),
         compose_body: String::new(),
         compose_cc: String::new(),
         compose_receipt: true,
         reply: String::new(),
         reply_all: false,
         mute_thread: false,
         notify_on_reply: true,
         priority: Priority::Normal,
      }
   }

   fn visible_messages(&self) -> Vec<&'static Message> {
      let q = self.search.trim().to_ascii_lowercase();
      let mut messages: Vec<&'static Message> = MESSAGES
         .iter()
         .filter(|m| match self.folder {
            Folder::Starred => m.starred,
            other => m.folder == other,
         })
         .filter(|m| match self.filter {
            InboxFilter::All => true,
            InboxFilter::Unread => m.unread,
            InboxFilter::Attachments => !m.attachments.is_empty(),
         })
         .filter(|m| {
            if q.is_empty() {
               return true;
            }
            m.from.to_ascii_lowercase().contains(&q)
               || m.address.to_ascii_lowercase().contains(&q)
               || m.subject.to_ascii_lowercase().contains(&q)
               || m.preview.to_ascii_lowercase().contains(&q)
         })
         .collect();

      match self.sort {
         SortOrder::Newest => {}
         SortOrder::Unread => messages.sort_by_key(|m| !m.unread),
         SortOrder::Sender => messages.sort_by_key(|m| m.from),
      }
      messages
   }

   fn unread_count(folder: Folder) -> usize {
      MESSAGES
         .iter()
         .filter(|m| match folder {
            Folder::Starred => m.starred && m.unread,
            other => m.folder == other && m.unread,
         })
         .count()
   }

   fn folder_badge(folder: Folder) -> usize {
      match folder {
         Folder::Inbox => Self::unread_count(Folder::Inbox),
         Folder::Starred => MESSAGES.iter().filter(|m| m.starred).count(),
         Folder::Drafts => MESSAGES.iter().filter(|m| m.folder == Folder::Drafts).count(),
         _ => 0,
      }
   }

   fn selected_message(&self) -> Option<&'static Message> {
      MESSAGES.iter().find(|m| m.id == self.selected_id)
   }

   fn account_name(&self) -> &'static str {
      match self.account {
         0 => "Personal",
         _ => "Work",
      }
   }

   fn account_email(&self) -> &'static str {
      match self.account {
         0 => "you@egui.dev",
         _ => "mike@studio.dev",
      }
   }

   fn muted(&self, ui: &mut Ui, text: impl Into<String>) {
      ui.label(
         RichText::new(text)
            .size(self.theme.typography.small)
            .color(self.theme.colors.text_muted),
      );
   }

   fn caption(&self, ui: &mut Ui, text: impl Into<String>) {
      ui.label(
         RichText::new(text)
            .size(self.theme.typography.very_small)
            .color(self.theme.colors.text_muted),
      );
   }

   fn avatar_visuals(&self, ring: Color32) -> AvatarVisuals {
      AvatarVisuals {
         ring,
         ..AvatarVisuals::from_theme(self.theme.colors)
      }
   }

   fn row_frame(&self, selected: bool) -> Card {
      let mut visuals = self.theme.frame2_visuals();
      visuals.bg = Color32::TRANSPARENT;
      visuals.border = Stroke::NONE;
      visuals.shadow = Shadow::NONE;
      visuals.corner_radius = CornerRadius::same(8);
      Card::new()
         .visuals(visuals)
         .interactive(true)
         .selected(selected)
         .fill_width(true)
         .inner_margin(self.theme.spacing.margin_xy(self.theme.spacing.sm, self.theme.spacing.sm))
         .corner_radius(8)
   }

   fn filled_button_visuals(&self, fill: Color32) -> ButtonVisuals {
      let colors = self.theme.colors;
      ButtonVisuals {
         text: colors.bg,
         bg: fill,
         bg_hover: mix(fill, colors.bg, 0.16),
         bg_click: mix(fill, colors.bg, 0.28),
         bg_selected: fill,
         border: Stroke::NONE,
         border_hover: Stroke::NONE,
         border_click: Stroke::NONE,
         ..self.theme.button_visuals()
      }
   }

   fn card1(&self) -> Frame {
      self.theme.frame1.inner_margin(self.theme.spacing.margin(self.theme.spacing.md))
   }

   fn card2(&self) -> Frame {
      self.theme.frame2.inner_margin(self.theme.spacing.margin(self.theme.spacing.md))
   }
}

impl eframe::App for MailApp {
   fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
      let space = self.theme.spacing;

      Panel::top("app_title")
         .exact_size(50.0)
         .frame(
            Frame::new()
               .fill(self.theme.colors.title_bar)
               .inner_margin(space.margin_xy(space.lg, 6.0))
               .stroke(Stroke::new(1.0, self.theme.colors.border)),
         )
         .show(ui, |ui| self.title_bar(ui));

      Panel::top("app_toolbar")
         .exact_size(64.0)
         .frame(
            Frame::new()
               .fill(self.theme.colors.bg)
               .inner_margin(space.margin_xy(space.lg, space.sm))
               .stroke(Stroke::new(1.0, self.theme.colors.border)),
         )
         .show(ui, |ui| self.toolbar(ui));

      if let Some(new_theme) = self.editor.show(&mut self.theme, ui) {
         self.theme = new_theme;
      }
      self.theme.install(ui.ctx());

      Panel::bottom("app_status")
         .exact_size(34.0)
         .frame(
            Frame::new()
               .fill(self.theme.colors.bg)
               .inner_margin(space.margin_xy(space.lg, 6.0))
               .stroke(Stroke::new(1.0, self.theme.colors.border)),
         )
         .show(ui, |ui| self.status_bar(ui));

      Panel::left("folders")
         .resizable(false)
         .default_size(208.0)
         .frame(
            Frame::new()
               .fill(self.theme.colors.widget_bg)
               .inner_margin(space.margin(space.md))
               .stroke(Stroke::new(1.0, self.theme.colors.border)),
         )
         .show(ui, |ui| self.sidebar(ui));

      Panel::left("threads")
         .resizable(true)
         .default_size(340.0)
         .size_range(300.0..=430.0)
         .frame(Frame::new().fill(self.theme.colors.bg).inner_margin(space.margin(space.sm)))
         .show(ui, |ui| self.thread_panel(ui));

      if self.show_details {
         Panel::right("details")
            .resizable(true)
            .default_size(280.0)
            .size_range(252.0..=330.0)
            .frame(Frame::new().fill(self.theme.colors.bg).inner_margin(space.margin(space.sm)))
            .show(ui, |ui| self.details_panel(ui));
      }

      CentralPanel::default()
         .frame(Frame::new().fill(self.theme.colors.bg).inner_margin(space.margin(space.sm)))
         .show(ui, |ui| self.reading_pane(ui));

      self.compose_modal(ui);
      self.delete_modal(ui);
      self.shortcuts_window(ui);
   }
}

impl MailApp {
   fn title_bar(&mut self, ui: &mut Ui) {
      let colors = self.theme.colors;
      let space = self.theme.spacing;

      Flex::row("title_bar")
         .justify(Justify::SpaceBetween)
         .align_items(CrossAlign::Center)
         .gap(space.md)
         .show(ui, |f| {
            f.item(Item::new(), |ui| {
               Flex::row("brand")
                  .fill_main(false)
                  .gap(space.sm)
                  .align_items(CrossAlign::Center)
                  .show(ui, |f| {
                     f.item(Item::new(), |ui| brand_mark(ui, colors));
                     f.item(Item::new(), |ui| {
                        ui.label(
                           RichText::new("egui mail")
                              .size(self.theme.typography.large)
                              .color(colors.text)
                              .strong(),
                        );
                     });
                     f.item(Item::new(), |ui| {
                        tag_chip(ui, self, "Reference", colors.info);
                     });
                  });
            });

            f.item(Item::new(), |ui| {
               Flex::row("sync")
                  .fill_main(false)
                  .gap(space.sm)
                  .align_items(CrossAlign::Center)
                  .show(ui, |f| {
                     f.item(Item::new(), |ui| {
                        let spinner = Spinner::new()
                           .size(13.0)
                           .color(colors.info)
                           .shadow(SpinnerShadow::new(colors.info).strength(0.22));
                        ui.add(spinner);
                     });
                     f.item(Item::new(), |ui| {
                        ui.label(
                           RichText::new("Syncing 2 of 12")
                              .size(self.theme.typography.very_small)
                              .color(colors.text_muted),
                        );
                     });
                     f.item(Item::new(), |ui| {
                        Badge::dot().color(colors.success).min_diameter(7.0).show(ui);
                     });
                     f.item(Item::new(), |ui| {
                        ui.label(
                           RichText::new("Connected")
                              .size(self.theme.typography.very_small)
                              .color(colors.text_muted),
                        );
                     });
                  });
            });

            f.item(Item::new(), |ui| {
               Flex::row("title_actions")
                  .fill_main(false)
                  .gap(space.sm)
                  .align_items(CrossAlign::Center)
                  .show(ui, |f| {
                     f.item(Item::new(), |ui| {
                        let help = Button::new("?")
                           .small()
                           .min_size(vec2(30.0, 30.0))
                           .selected(self.shortcuts_open);
                        if ui.add(help).clicked() {
                           self.shortcuts_open = !self.shortcuts_open;
                        }
                     });
                     f.item(Item::new(), |ui| {
                        if let Some(new_theme) = utils::theme_switcher(&self.theme, ui) {
                           self.theme = new_theme;
                        }
                     });
                     f.item(Item::new(), |ui| {
                        let label = if self.editor.open {
                           "Close editor"
                        } else {
                           "Theme editor"
                        };
                        let editor = Button::new(label).small().min_size(vec2(112.0, 30.0));
                        if ui.add(editor).clicked() {
                           self.editor.open = !self.editor.open;
                        }
                     });
                  });
            });
         });
   }

   fn toolbar(&mut self, ui: &mut Ui) {
      let colors = self.theme.colors;
      let space = self.theme.spacing;
      let text_size = self.theme.typography.normal;
      let muted = colors.text_muted;

      Flex::row("toolbar")
         .gap(space.md)
         .align_items(CrossAlign::Center)
         .show(ui, |f| {
            f.item(Item::new(), |ui| {
               let compose = Button::new(RichText::new("Compose").size(text_size).color(colors.bg))
                  .visuals(self.filled_button_visuals(colors.accent))
                  .min_size(vec2(126.0, 36.0))
                  .label_align(Align::Center);
               if ui.add(compose).clicked() {
                  self.compose_open = true;
               }
            });

            f.item(Item::grow(1.0), |ui| {
               let width = ui.available_width();
               ui.add(
                  SecureTextEdit::singleline(&mut self.search)
                     .id_salt("mail_search")
                     .hint_text(RichText::new("Search mail  ( / )").color(muted))
                     .margin(space.margin(space.sm))
                     .desired_width(width)
                     .font(FontId::proportional(self.theme.typography.small)),
               );
            });

            f.item(Item::new(), |ui| {
               let alerts = Button::new("Alerts")
                  .small()
                  .badge(Badge::new("3").color(colors.error).text_color(colors.bg))
                  .min_size(vec2(72.0, 34.0));
               ui.add(alerts);
            });

            f.item(Item::new(), |ui| {
               let details = Button::new("Details")
                  .small()
                  .selected(self.show_details)
                  .min_size(vec2(72.0, 34.0));
               if ui.add(details).clicked() {
                  self.show_details = !self.show_details;
               }
            });

            f.item(Item::new(), |ui| {
               let people = [
                  account_avatar("P", colors.accent, colors.bg),
                  account_avatar("W", colors.info, colors.bg),
               ];
               ui.add(
                  AvatarGroup::new(&people)
                     .size(28.0)
                     .visuals(self.avatar_visuals(colors.bg))
                     .label("Accounts"),
               );
            });
         });
   }

   fn status_bar(&mut self, ui: &mut Ui) {
      let colors = self.theme.colors;
      let small = self.theme.typography.very_small;

      Flex::row("status_bar")
         .justify(Justify::SpaceBetween)
         .align_items(CrossAlign::Center)
         .gap(self.theme.spacing.md)
         .show(ui, |f| {
            f.item(Item::new(), |ui| {
               Flex::row("status_left")
                  .fill_main(false)
                  .gap(self.theme.spacing.sm)
                  .align_items(CrossAlign::Center)
                  .show(ui, |f| {
                     f.item(Item::new(), |ui| {
                        Badge::dot().color(colors.success).min_diameter(7.0).show(ui);
                     });
                     f.item(Item::new(), |ui| {
                        ui.label(
                           RichText::new("Connected via IMAP").size(small).color(colors.text_muted),
                        );
                     });
                     f.item(Item::new(), |ui| {
                        ui.label(RichText::new("·").size(small).color(colors.border));
                     });
                     f.item(Item::new(), |ui| {
                        ui.label(
                           RichText::new("Last sync 2 min ago")
                              .size(small)
                              .color(colors.text_muted),
                        );
                     });
                  });
            });

            f.item(Item::new(), |ui| {
               Flex::row("status_center")
                  .fill_main(false)
                  .gap(self.theme.spacing.sm)
                  .align_items(CrossAlign::Center)
                  .show(ui, |f| {
                     f.item(Item::new(), |ui| {
                        Badge::dot().color(colors.info).min_diameter(7.0).show(ui);
                     });
                     f.item(Item::new(), |ui| {
                        ui.label(
                           RichText::new("End-to-end encrypted")
                              .size(small)
                              .color(colors.text_muted),
                        );
                     });
                  });
            });

            f.item(Item::new(), |ui| {
               Flex::row("status_right")
                  .fill_main(false)
                  .gap(self.theme.spacing.md)
                  .align_items(CrossAlign::Center)
                  .show(ui, |f| {
                     f.item(Item::new(), |ui| {
                        ui.label(RichText::new("Storage 86%").size(small).color(colors.warning));
                     });
                     f.item(Item::new(), |ui| {
                        Badge::new("1")
                           .color(colors.error)
                           .text_color(colors.bg)
                           .min_diameter(16.0)
                           .show(ui);
                     });
                     f.item(Item::new(), |ui| {
                        ui.label(RichText::new("send failed").size(small).color(colors.text_muted));
                     });
                     f.item(Item::new(), |ui| {
                        ui.add(Button::new("Retry").small().min_size(vec2(58.0, 24.0)));
                     });
                  });
            });
         });
   }

   fn sidebar(&mut self, ui: &mut Ui) {
      let colors = self.theme.colors;
      let space = self.theme.spacing;

      ScrollArea::vertical()
         .id_salt("sidebar_scroll")
         .auto_shrink([false, false])
         .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = space.sm;

            self.account_card(ui);
            ui.add_space(space.sm);

            let compose = Button::new(
               RichText::new("+  Compose").size(self.theme.typography.normal).color(colors.bg),
            )
            .visuals(self.filled_button_visuals(colors.accent))
            .min_size(vec2(ui.available_width(), 38.0))
            .label_align(Align::Center);
            if ui.add(compose).clicked() {
               self.compose_open = true;
            }

            ui.add_space(space.md);
            self.folder_nav(ui);
            self.label_nav(ui);
            ui.add_space(space.md);
            self.storage_card(ui);
         });
   }

   fn account_card(&mut self, ui: &mut Ui) {
      let colors = self.theme.colors;
      let space = self.theme.spacing;
      let account = self.account;

      self.card2().show(ui, |ui| {
         ui.set_width(ui.available_width());
         Flex::row("account_head")
            .gap(space.sm)
            .align_items(CrossAlign::Center)
            .show(ui, |f| {
               f.item(Item::new(), |ui| {
                  let people = [account_avatar("M", colors.accent, colors.bg)];
                  ui.add(
                     AvatarGroup::new(&people)
                        .size(32.0)
                        .expand(false)
                        .visuals(self.avatar_visuals(colors.bg)),
                  );
               });
               f.item(Item::grow(1.0), |ui| {
                  ui.vertical(|ui| {
                     ui.label(
                        RichText::new("Mike Molfetas")
                           .size(self.theme.typography.small)
                           .color(colors.text)
                           .strong(),
                     );
                     ui.label(
                        RichText::new(self.account_email())
                           .size(self.theme.typography.very_small)
                           .color(colors.text_muted),
                     );
                  });
               });
               f.item(Item::new(), |ui| {
                  Badge::dot().color(colors.success).min_diameter(7.0).show(ui);
               });
            });

         ui.add_space(space.sm);

         let selected = Label::new(
            RichText::new(self.account_name())
               .size(self.theme.typography.small)
               .color(colors.text),
            None,
         );
         ComboBox::new("account_picker", selected)
            .width(ui.available_width())
            .show_ui(ui, |ui| {
               for (index, name) in ["Personal", "Work"].into_iter().enumerate() {
                  let row = Label::new(
                     RichText::new(name).size(self.theme.typography.small).color(colors.text),
                     None,
                  )
                  .expand(Some(space.xs))
                  .selected(account == index)
                  .interactive(true)
                  .sense(Sense::click())
                  .fill_width(true);
                  if ui.add(row).clicked() {
                     self.account = index;
                  }
               }
            });
      });
   }

   fn folder_nav(&mut self, ui: &mut Ui) {
      let colors = self.theme.colors;
      let space = self.theme.spacing;
      ui.spacing_mut().item_spacing.y = space.xs;
      let mut clicked = None;

      ui.label(
         RichText::new("MAILBOX")
            .size(self.theme.typography.very_small)
            .color(colors.text_muted),
      );
      ui.add_space(space.xs);

      for folder in Folder::ALL {
         let selected = self.folder == folder;
         let badge = Self::folder_badge(folder);
         let row = self.row_frame(selected).show(ui, |ui| {
            ui.set_width(ui.available_width());
            Flex::row(("folder", folder.label()))
               .gap(space.sm)
               .align_items(CrossAlign::Center)
               .show(ui, |f| {
                  f.item(Item::fixed(20.0), |ui| {
                     let color = if selected {
                        colors.accent
                     } else {
                        colors.text_muted
                     };
                     icon(ui, folder.icon(), color, 17.0);
                  });
                  f.item(Item::grow(1.0), |ui| {
                     ui.add(
                        Label::new(
                           RichText::new(folder.label())
                              .size(self.theme.typography.small)
                              .color(colors.text),
                           None,
                        )
                        .interactive(false)
                        .fill_width(true),
                     );
                  });
                  if badge > 0 {
                     f.item(Item::new(), |ui| {
                        Badge::new(badge.to_string())
                           .color(colors.accent)
                           .text_color(colors.bg)
                           .min_diameter(18.0)
                           .show(ui);
                     });
                  }
               });
         });
         if row.response.clicked() {
            clicked = Some(folder);
         }
      }

      if let Some(folder) = clicked {
         self.folder = folder;
         if let Some(first) = self.visible_messages().first() {
            self.selected_id = first.id;
         }
      }
   }

   fn label_nav(&mut self, ui: &mut Ui) {
      let colors = self.theme.colors;
      let space = self.theme.spacing;
      ui.spacing_mut().item_spacing.y = space.xs;
      let labels = [
         ("Design", colors.info),
         ("Engineering", colors.accent),
         ("Finance", colors.warning),
         ("Travel", colors.success),
      ];

      ui.add_space(space.sm);
      ui.label(
         RichText::new("LABELS")
            .size(self.theme.typography.very_small)
            .color(colors.text_muted),
      );
      ui.add_space(space.xs);

      for (name, color) in labels {
         let row = self.row_frame(false).show(ui, |ui| {
            ui.set_width(ui.available_width());
            Flex::row(("label", name)).gap(space.sm).align_items(CrossAlign::Center).show(
               ui,
               |f| {
                  f.item(Item::fixed(20.0), |ui| {
                     Badge::dot().color(color).min_diameter(8.0).show(ui);
                  });
                  f.item(Item::grow(1.0), |ui| {
                     ui.add(
                        Label::new(
                           RichText::new(name)
                              .size(self.theme.typography.small)
                              .color(colors.text_muted),
                           None,
                        )
                        .interactive(false)
                        .fill_width(true),
                     );
                  });
                  f.item(Item::new(), |ui| {
                     ui.label(
                        RichText::new(">").size(self.theme.typography.small).color(colors.border),
                     );
                  });
               },
            );
         });
         row.response.clicked();
      }
   }

   fn storage_card(&mut self, ui: &mut Ui) {
      let colors = self.theme.colors;
      let space = self.theme.spacing;

      self.card2().show(ui, |ui| {
         ui.set_width(ui.available_width());
         Flex::row("storage_head")
            .justify(Justify::SpaceBetween)
            .align_items(CrossAlign::Center)
            .show(ui, |f| {
               f.item(Item::new(), |ui| {
                  ui.label(
                     RichText::new("Storage").size(self.theme.typography.small).color(colors.text),
                  );
               });
               f.item(Item::new(), |ui| {
                  ui.label(
                     RichText::new("4.3 / 5 GB")
                        .size(self.theme.typography.very_small)
                        .color(colors.text_muted),
                  );
               });
            });
         ui.add_space(space.xs);
         let width = ui.available_width();
         let (rect, _) = ui.allocate_exact_size(vec2(width, 6.0), Sense::hover());
         ui.painter().rect_filled(rect, 3.0, colors.border.gamma_multiply(0.5));
         let fill = Rect::from_min_size(rect.min, vec2(rect.width() * 0.86, rect.height()));
         ui.painter().rect_filled(fill, 3.0, colors.warning);
         ui.add_space(space.xs);
         ui.label(
            RichText::new("Archive older threads to free space.")
               .size(self.theme.typography.very_small)
               .color(colors.text_muted),
         );
      });
   }
}

// ---------------------------------------------------------------------------
// Mail list
// ---------------------------------------------------------------------------

impl MailApp {
   fn thread_panel(&mut self, ui: &mut Ui) {
      let card = self.theme.frame1.inner_margin(self.theme.spacing.margin(self.theme.spacing.sm));
      card.show(ui, |ui| {
         ui.set_width(ui.available_width());
         Flex::column("thread_column")
            .fill_main(true)
            .gap(self.theme.spacing.sm)
            .align_items(CrossAlign::Stretch)
            .show(ui, |f| {
               f.item(Item::new(), |ui| self.thread_header(ui));
               f.item(Item::new(), |ui| self.thread_filters(ui));
               f.item(Item::grow(1.0), |ui| self.thread_rows(ui));
               f.item(Item::new(), |ui| self.thread_footer(ui));
            });
      });
   }

   fn thread_header(&mut self, ui: &mut Ui) {
      let colors = self.theme.colors;
      let space = self.theme.spacing;
      let messages = self.visible_messages();
      let unread = messages.iter().filter(|m| m.unread).count();
      let sort = self.sort;

      Flex::row("thread_head")
         .justify(Justify::SpaceBetween)
         .align_items(CrossAlign::Center)
         .gap(space.sm)
         .show(ui, |f| {
            f.item(Item::new(), |ui| {
               Flex::row("thread_title")
                  .fill_main(false)
                  .gap(space.sm)
                  .align_items(CrossAlign::Center)
                  .show(ui, |f| {
                     f.item(Item::new(), |ui| {
                        ui.label(
                           RichText::new(self.folder.label())
                              .size(self.theme.typography.heading)
                              .color(colors.text)
                              .strong(),
                        );
                     });
                     f.item(Item::new(), |ui| {
                        Badge::new(unread.to_string())
                           .color(colors.accent)
                           .text_color(colors.bg)
                           .min_diameter(20.0)
                           .show(ui);
                     });
                  });
            });
            f.item(Item::new(), |ui| {
               let selected = Label::new(
                  RichText::new(sort.label())
                     .size(self.theme.typography.very_small)
                     .color(colors.text_muted),
                  None,
               );
               ComboBox::new("thread_sort", selected).width(116.0).show_ui(ui, |ui| {
                  for order in [SortOrder::Newest, SortOrder::Unread, SortOrder::Sender] {
                     let row = Label::new(
                        RichText::new(order.label())
                           .size(self.theme.typography.small)
                           .color(colors.text),
                        None,
                     )
                     .expand(Some(space.xs))
                     .selected(sort == order)
                     .sense(Sense::click())
                     .fill_width(true);
                     if ui.add(row).clicked() {
                        self.sort = order;
                     }
                  }
               });
            });
         });
   }

   fn thread_filters(&mut self, ui: &mut Ui) {
      let colors = self.theme.colors;
      let space = self.theme.spacing;
      let current = self.filter;
      let count = self.visible_messages().len();

      Flex::row("thread_filters")
         .justify(Justify::SpaceBetween)
         .align_items(CrossAlign::Center)
         .gap(space.sm)
         .show(ui, |f| {
            f.item(Item::new(), |ui| {
               Flex::row("filter_buttons")
                  .fill_main(false)
                  .gap(space.xs)
                  .align_items(CrossAlign::Center)
                  .show(ui, |f| {
                     for filter in InboxFilter::ALL {
                        f.item(Item::new(), |ui| {
                           let button = Button::new(filter.label())
                              .small()
                              .selected(current == filter)
                              .min_size(vec2(46.0, 28.0));
                           if ui.add(button).clicked() {
                              self.filter = filter;
                           }
                        });
                     }
                  });
            });
            f.item(Item::new(), |ui| {
               Flex::row("filter_counts")
                  .fill_main(false)
                  .gap(space.xs)
                  .align_items(CrossAlign::Center)
                  .show(ui, |f| {
                     f.item(Item::new(), |ui| {
                        ui.label(
                           RichText::new(format!("{count} threads"))
                              .size(self.theme.typography.very_small)
                              .color(colors.text_muted),
                        );
                     });
                     f.item(Item::new(), |ui| {
                        ui.add(Checkbox::without_text(&mut self.select_all).size(16.0));
                     });
                  });
            });
         });
   }

   fn thread_rows(&mut self, ui: &mut Ui) {
      let space = self.theme.spacing;
      let messages = self.visible_messages();
      let selected_id = self.selected_id;
      let mut clicked = None;

      ScrollArea::vertical()
         .id_salt("thread_scroll")
         .auto_shrink([false, false])
         .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = space.xs;

            if messages.is_empty() {
               ui.add_space(space.xl);
               self.empty_state(ui);
               return;
            }

            for message in messages {
               let selected = message.id == selected_id;
               let row = self.row_frame(selected).show(ui, |ui| {
                  ui.set_width(ui.available_width());
                  self.thread_row(ui, message);
               });
               if row.response.clicked() {
                  clicked = Some(message.id);
               }
            }
         });

      if let Some(id) = clicked {
         self.selected_id = id;
      }
   }

   fn thread_row(&self, ui: &mut Ui, message: &Message) {
      let colors = self.theme.colors;
      let space = self.theme.spacing;
      let muted = colors.text_muted;
      let text = colors.text;

      Flex::row(("thread", message.id))
         .gap(space.sm)
         .align_items(CrossAlign::Start)
         .show(ui, |f| {
            f.item(Item::fixed(38.0), |ui| {
               let plate = match message.tags.first().map(|tag| tag.role) {
                  Some(TagRole::Accent) => colors.accent,
                  Some(TagRole::Info) => colors.info,
                  Some(TagRole::Warning) => colors.warning,
                  Some(TagRole::Success) => colors.success,
                  Some(TagRole::Error) => colors.error,
                  None => colors.accent,
               };
               let ink = if !self.theme.dark
                  && matches!(
                     message.tags.first().map(|tag| tag.role),
                     Some(TagRole::Warning)
                  ) {
                  colors.text
               } else {
                  colors.bg
               };
               let people = [account_avatar(&initial(message.from), plate, ink)];
               ui.add(
                  AvatarGroup::new(&people)
                     .size(34.0)
                     .expand(false)
                     .visuals(self.avatar_visuals(colors.widget_bg)),
               );
            });

            f.item(Item::grow(1.0), |ui| {
               ui.vertical(|ui| {
                  Flex::row(("thread_top", message.id))
                     .justify(Justify::SpaceBetween)
                     .align_items(CrossAlign::Center)
                     .gap(space.sm)
                     .show(ui, |f| {
                        f.item(Item::grow(1.0), |ui| {
                           let sender = if message.unread {
                              RichText::new(message.from)
                                 .size(self.theme.typography.small)
                                 .color(text)
                                 .strong()
                           } else {
                              RichText::new(message.from)
                                 .size(self.theme.typography.small)
                                 .color(text)
                           };
                           ui.add(Label::new(sender, None).interactive(false));
                        });
                        f.item(Item::new(), |ui| {
                           ui.label(
                              RichText::new(message.time)
                                 .size(self.theme.typography.very_small)
                                 .color(muted),
                           );
                        });
                     });

                  ui.add_space(space.xs);

                  ui.add(
                     Label::new(
                        RichText::new(message.subject)
                           .size(self.theme.typography.small)
                           .color(text),
                        None,
                     )
                     .interactive(false)
                     .wrap_mode(TextWrapMode::Truncate)
                     .fill_width(true),
                  );

                  ui.add_space(space.xs);

                  ui.add(
                     Label::new(
                        RichText::new(message.preview)
                           .size(self.theme.typography.very_small)
                           .color(muted),
                        None,
                     )
                     .interactive(false)
                     .wrap_mode(TextWrapMode::Truncate)
                     .fill_width(true),
                  );

                  if !message.tags.is_empty()
                     || !message.attachments.is_empty()
                     || message.thread_count > 1
                  {
                     ui.add_space(space.xs);
                     Flex::row(("thread_meta", message.id))
                        .fill_main(false)
                        .gap(space.xs)
                        .align_items(CrossAlign::Center)
                        .show(ui, |f| {
                           for tag in message.tags.iter().take(2) {
                              f.item(Item::new(), |ui| {
                                 tag_chip(ui, self, tag.text, tag.role.color(colors));
                              });
                           }
                           if !message.attachments.is_empty() {
                              f.item(Item::new(), |ui| {
                                 tag_chip(
                                    ui,
                                    self,
                                    &format!("{} files", message.attachments.len()),
                                    colors.info,
                                 );
                              });
                           }
                           if message.thread_count > 1 {
                              f.item(Item::new(), |ui| {
                                 Badge::new(message.thread_count.to_string())
                                    .color(colors.hover)
                                    .text_color(text)
                                    .min_diameter(16.0)
                                    .show(ui);
                              });
                           }
                        });
                  }
               });
            });

            f.item(Item::fixed(20.0), |ui| {
               if message.starred {
                  ui.label(
                     RichText::new("★").size(self.theme.typography.small).color(colors.warning),
                  );
               } else if message.unread {
                  Badge::dot().color(colors.accent).min_diameter(7.0).show(ui);
               }
            });
         });
   }

   fn empty_state(&mut self, ui: &mut Ui) {
      let colors = self.theme.colors;
      let space = self.theme.spacing;

      Flex::column("empty_threads")
         .gap(space.sm)
         .align_items(CrossAlign::Center)
         .show(ui, |f| {
            f.item(Item::new(), |ui| {
               Badge::dot().color(colors.info).min_diameter(10.0).show(ui);
            });
            f.item(Item::new(), |ui| {
               ui.label(
                  RichText::new("No threads match this view.")
                     .size(self.theme.typography.small)
                     .color(colors.text),
               );
            });
            f.item(Item::new(), |ui| {
               self.caption(ui, "Try All, or clear the search field.");
            });
         });
   }

   fn thread_footer(&mut self, ui: &mut Ui) {
      let colors = self.theme.colors;
      let space = self.theme.spacing;
      let count = self.visible_messages().len();

      Flex::row("thread_footer")
         .justify(Justify::SpaceBetween)
         .align_items(CrossAlign::Center)
         .gap(space.sm)
         .show(ui, |f| {
            f.item(Item::new(), |ui| {
               ui.label(
                  RichText::new(format!("1-{count} of {count}"))
                     .size(self.theme.typography.very_small)
                     .color(colors.text_muted),
               );
            });
            f.item(Item::new(), |ui| {
               Flex::row("pager")
                  .fill_main(false)
                  .gap(space.xs)
                  .align_items(CrossAlign::Center)
                  .show(ui, |f| {
                     f.item(Item::new(), |ui| {
                        ui.add(Button::new("Previous").small().min_size(vec2(72.0, 26.0)));
                     });
                     f.item(Item::new(), |ui| {
                        ui.add(Button::new("Next").small().min_size(vec2(54.0, 26.0)));
                     });
                  });
            });
         });
   }
}

// ---------------------------------------------------------------------------
// Reading pane
// ---------------------------------------------------------------------------

impl MailApp {
   fn reading_pane(&mut self, ui: &mut Ui) {
      let Some(message) = self.selected_message() else {
         self.muted(ui, "Select a thread.");
         return;
      };

      let card = self.theme.frame1.inner_margin(self.theme.spacing.margin(self.theme.spacing.md));
      card.show(ui, |ui| {
         ui.set_width(ui.available_width());
         Flex::column("reading_column")
            .fill_main(true)
            .gap(self.theme.spacing.md)
            .align_items(CrossAlign::Stretch)
            .show(ui, |f| {
               f.item(Item::new(), |ui| self.reading_header(ui, message));
               f.item(Item::new(), |ui| {
                  self.reading_actions(ui, message)
               });
               f.item(Item::grow(1.0), |ui| {
                  self.reading_body(ui, message)
               });
               f.item(Item::new(), |ui| self.reply_composer(ui, message));
            });
      });
   }

   fn reading_header(&mut self, ui: &mut Ui, message: &'static Message) {
      let colors = self.theme.colors;
      let space = self.theme.spacing;

      Flex::column("reading_header")
         .gap(space.sm)
         .align_items(CrossAlign::Stretch)
         .show(ui, |f| {
            f.item(Item::new(), |ui| {
               Flex::row("reading_kicker")
                  .justify(Justify::SpaceBetween)
                  .align_items(CrossAlign::Center)
                  .gap(space.sm)
                  .show(ui, |f| {
                     f.item(Item::new(), |ui| {
                        Flex::row("reading_tags")
                           .fill_main(false)
                           .gap(space.xs)
                           .align_items(CrossAlign::Center)
                           .show(ui, |f| {
                              f.item(Item::new(), |ui| {
                                 tag_chip(ui, self, "Inbox", colors.accent);
                              });
                              for tag in message.tags {
                                 f.item(Item::new(), |ui| {
                                    tag_chip(ui, self, tag.text, tag.role.color(colors));
                                 });
                              }
                           });
                     });
                     f.item(Item::new(), |ui| {
                        let (rect, _) = ui.allocate_exact_size(vec2(24.0, 24.0), Sense::hover());
                        paint_icon(ui, rect, Icon::Archive, colors.text_muted);
                     });
                  });
            });

            f.item(Item::new(), |ui| {
               ui.add(
                  Label::new(
                     RichText::new(message.subject)
                        .size(self.theme.typography.heading)
                        .color(colors.text)
                        .strong(),
                     None,
                  )
                  .interactive(false)
                  .wrap()
                  .fill_width(true),
               );
            });

            f.item(Item::new(), |ui| {
               Flex::row("reading_sender").gap(space.sm).align_items(CrossAlign::Center).show(
                  ui,
                  |f| {
                     f.item(Item::new(), |ui| {
                        let people = [account_avatar(
                           &initial(message.from),
                           colors.info,
                           colors.bg,
                        )];
                        ui.add(
                           AvatarGroup::new(&people)
                              .size(38.0)
                              .expand(false)
                              .visuals(self.avatar_visuals(colors.widget_bg)),
                        );
                     });

                     f.item(Item::grow(1.0), |ui| {
                        ui.vertical(|ui| {
                           ui.add(
                              Label::new(
                                 RichText::new(message.from)
                                    .size(self.theme.typography.small)
                                    .color(colors.text)
                                    .strong(),
                                 None,
                              )
                              .interactive(false)
                              .fill_width(true),
                           );
                           ui.add(
                              MultiLabel::new(vec![
                                 Label::new(
                                    RichText::new(message.address)
                                       .size(self.theme.typography.very_small)
                                       .color(colors.text_muted),
                                    None,
                                 )
                                 .interactive(false),
                                 Label::new(
                                    RichText::new("·")
                                       .size(self.theme.typography.very_small)
                                       .color(colors.border),
                                    None,
                                 )
                                 .interactive(false),
                                 Label::new(
                                    RichText::new(format!("to {}", message.to))
                                       .size(self.theme.typography.very_small)
                                       .color(colors.text_muted),
                                    None,
                                 )
                                 .interactive(false),
                                 Label::new(
                                    RichText::new("·")
                                       .size(self.theme.typography.very_small)
                                       .color(colors.border),
                                    None,
                                 )
                                 .interactive(false),
                                 Label::new(
                                    RichText::new(message.time)
                                       .size(self.theme.typography.very_small)
                                       .color(colors.text_muted),
                                    None,
                                 )
                                 .interactive(false),
                              ])
                              .inter_spacing(space.xs),
                           );
                        });
                     });

                     f.item(Item::new(), |ui| {
                        Flex::row("sender_meta")
                           .fill_main(false)
                           .gap(space.sm)
                           .align_items(CrossAlign::Center)
                           .show(ui, |f| {
                              f.item(Item::new(), |ui| {
                                 Badge::dot().color(colors.success).min_diameter(7.0).show(ui);
                              });
                              f.item(Item::new(), |ui| {
                                 ui.label(
                                    RichText::new("Delivered")
                                       .size(self.theme.typography.very_small)
                                       .color(colors.text_muted),
                                 );
                              });
                           });
                     });
                  },
               );
            });
         });
   }

   fn reading_actions(&mut self, ui: &mut Ui, message: &'static Message) {
      let colors = self.theme.colors;
      let space = self.theme.spacing;
      let size = self.theme.typography.small;

      Flex::row("reading_actions")
         .gap(space.xs)
         .align_items(CrossAlign::Center)
         .show(ui, |f| {
            f.item(Item::new(), |ui| {
               let reply = Button::new(RichText::new("Reply").size(size).color(colors.bg))
                  .visuals(self.filled_button_visuals(colors.accent))
                  .min_size(vec2(78.0, 30.0));
               if ui.add(reply).clicked() {
                  self.reply_all = false;
               }
            });
            for label in ["Reply all", "Forward"] {
               f.item(Item::new(), |ui| {
                  ui.add(
                     Button::new(RichText::new(label).size(size))
                        .small()
                        .min_size(vec2(84.0, 30.0)),
                  );
               });
            }
            f.item(Item::new(), |ui| {
               ui.add(
                  Button::new(RichText::new("Archive").size(size))
                     .small()
                     .min_size(vec2(76.0, 30.0)),
               );
            });
            f.item(Item::new(), |ui| {
               let delete = Button::new(RichText::new("Delete").size(size))
                  .small()
                  .min_size(vec2(70.0, 30.0));
               if ui.add(delete).clicked() {
                  self.delete_open = true;
               }
            });

            f.item(Item::grow(1.0), |ui| {
               ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                  if message.starred {
                     ui.label(
                        RichText::new("★").size(self.theme.typography.normal).color(colors.warning),
                     );
                  }
                  ui.add(Button::new("More").small().min_size(vec2(60.0, 30.0)));
               });
            });
         });
   }

   fn reading_body(&mut self, ui: &mut Ui, message: &'static Message) {
      let colors = self.theme.colors;
      let space = self.theme.spacing;

      self.card2().show(ui, |ui| {
         ui.set_width(ui.available_width());
         ScrollArea::vertical()
            .id_salt(("reading_scroll", message.id))
            .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
            .auto_shrink([false, false])
            .show(ui, |ui| {
               ui.set_width(ui.available_width());
               ui.spacing_mut().item_spacing.y = space.sm;

               for paragraph in message.body.split("\n\n") {
                  if paragraph.is_empty() {
                     continue;
                  }
                  ui.add(
                     Label::new(
                        RichText::new(paragraph)
                           .size(self.theme.typography.normal)
                           .color(colors.text),
                        None,
                     )
                     .interactive(false)
                     .wrap()
                     .fill_width(true),
                  );
                  ui.add_space(space.sm);
               }

               if !message.attachments.is_empty() {
                  Flex::row("attachment_strip")
                     .gap(space.sm)
                     .align_items(CrossAlign::Center)
                     .show(ui, |f| {
                        for file in message.attachments {
                           f.item(Item::grow(1.0), |ui| {
                              self.card1().inner_margin(space.margin(space.sm)).show(ui, |ui| {
                                 ui.set_width(ui.available_width());
                                 Flex::row(("file_chip", file.name))
                                    .gap(space.sm)
                                    .align_items(CrossAlign::Center)
                                    .show(ui, |f| {
                                       f.item(Item::fixed(34.0), |ui| {
                                          Badge::new(file.kind)
                                             .color(colors.info)
                                             .text_color(colors.bg)
                                             .min_diameter(26.0)
                                             .padding(4.0)
                                             .show(ui);
                                       });
                                       f.item(Item::grow(1.0), |ui| {
                                          ui.vertical(|ui| {
                                             ui.label(
                                                RichText::new(file.name)
                                                   .size(self.theme.typography.very_small)
                                                   .color(colors.text),
                                             );
                                             ui.label(
                                                RichText::new(file.size)
                                                   .size(self.theme.typography.very_small)
                                                   .color(colors.text_muted),
                                             );
                                          });
                                       });
                                    });
                              });
                           });
                        }
                     });
                  ui.add_space(space.sm);
               }
            });
      });
   }

   fn reply_composer(&mut self, ui: &mut Ui, message: &'static Message) {
      let colors = self.theme.colors;
      let space = self.theme.spacing;

      self.card2().show(ui, |ui| {
         ui.set_width(ui.available_width());
         ui.spacing_mut().item_spacing.y = space.sm;

         Flex::row("reply_head")
            .justify(Justify::SpaceBetween)
            .align_items(CrossAlign::Center)
            .show(ui, |f| {
               f.item(Item::new(), |ui| {
                  ui.label(
                     RichText::new(format!("Reply to {}", message.from))
                        .size(self.theme.typography.small)
                        .color(colors.text)
                        .strong(),
                  );
               });
               f.item(Item::new(), |ui| {
                  ui.label(
                     RichText::new("Ctrl+Enter to send")
                        .size(self.theme.typography.very_small)
                        .color(colors.text_muted),
                  );
               });
            });

         // frame1 on frame2: the quoted reply sits inside the inset composer.
         self
            .card1()
            .inner_margin(space.margin_xy(space.sm, space.xs))
            .stroke(Stroke::new(
               1.0,
               colors.border.gamma_multiply(0.65),
            ))
            .show(ui, |ui| {
               ui.set_width(ui.available_width());
               ui.add(
                  Label::new(
                     RichText::new(format!(
                        "On Tuesday, {} wrote:  \"The sidebar density feels right...\"",
                        message.from
                     ))
                     .size(self.theme.typography.very_small)
                     .color(colors.text_muted)
                     .italics(),
                     None,
                  )
                  .interactive(false)
                  .wrap()
                  .fill_width(true),
               );
            });

         ui.add(
            SecureTextEdit::multiline(&mut self.reply)
               .id_salt("reply_body")
               .hint_text(RichText::new("Write a reply...").color(colors.text_muted))
               .margin(space.margin(space.sm))
               .desired_width(f32::INFINITY)
               .desired_rows(2)
               .font(FontId::proportional(self.theme.typography.small)),
         );

         Flex::row("reply_actions")
            .justify(Justify::SpaceBetween)
            .align_items(CrossAlign::Center)
            .show(ui, |f| {
               f.item(Item::new(), |ui| {
                  ui.add(
                     Switch::new(
                        &mut self.reply_all,
                        RichText::new("Reply all")
                           .size(self.theme.typography.very_small)
                           .color(colors.text),
                     )
                     .size(16.0),
                  );
               });
               f.item(Item::new(), |ui| {
                  Flex::row("reply_send")
                     .fill_main(false)
                     .gap(space.xs)
                     .align_items(CrossAlign::Center)
                     .show(ui, |f| {
                        f.item(Item::new(), |ui| {
                           ui.add(Button::new("Attach").small().min_size(vec2(58.0, 26.0)));
                        });
                        f.item(Item::new(), |ui| {
                           let send = Button::new(
                              RichText::new("Send reply")
                                 .size(self.theme.typography.very_small)
                                 .color(colors.bg),
                           )
                           .visuals(self.filled_button_visuals(colors.accent))
                           .min_size(vec2(86.0, 26.0));
                           ui.add(send);
                        });
                     });
               });
            });
      });
   }
}

// ---------------------------------------------------------------------------
// Details rail
// ---------------------------------------------------------------------------

impl MailApp {
   fn details_panel(&mut self, ui: &mut Ui) {
      let card = self.theme.frame1.inner_margin(self.theme.spacing.margin(self.theme.spacing.sm));
      card.show(ui, |ui| {
         ui.set_width(ui.available_width());
         Flex::column("details_column")
            .fill_main(true)
            .gap(self.theme.spacing.sm)
            .align_items(CrossAlign::Stretch)
            .show(ui, |f| {
               f.item(Item::new(), |ui| self.details_header(ui));
               f.item(Item::grow(1.0), |ui| {
                  ScrollArea::vertical()
                     .id_salt("details_scroll")
                     .auto_shrink([false, false])
                     .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.spacing_mut().item_spacing.y = self.theme.spacing.sm;
                        self.details_content(ui);
                     });
               });
            });
      });
   }

   fn details_header(&mut self, ui: &mut Ui) {
      let colors = self.theme.colors;

      Flex::row("details_head")
         .justify(Justify::SpaceBetween)
         .align_items(CrossAlign::Center)
         .show(ui, |f| {
            f.item(Item::new(), |ui| {
               ui.label(
                  RichText::new("Thread details")
                     .size(self.theme.typography.large)
                     .color(colors.text)
                     .strong(),
               );
            });
            f.item(Item::new(), |ui| {
               let close = Button::new("x").small().min_size(vec2(28.0, 26.0));
               if ui.add(close).clicked() {
                  self.show_details = false;
               }
            });
         });
   }

   fn details_content(&mut self, ui: &mut Ui) {
      let Some(message) = self.selected_message() else {
         return;
      };
      let colors = self.theme.colors;
      let space = self.theme.spacing;

      self.card2().show(ui, |ui| {
         ui.set_width(ui.available_width());
         ui.spacing_mut().item_spacing.y = space.sm;
         Flex::row("people_head")
            .justify(Justify::SpaceBetween)
            .align_items(CrossAlign::Center)
            .show(ui, |f| {
               f.item(Item::new(), |ui| {
                  ui.label(
                     RichText::new("People")
                        .size(self.theme.typography.small)
                        .color(colors.text)
                        .strong(),
                  );
               });
               f.item(Item::new(), |ui| {
                  Badge::new("4")
                     .color(colors.highlight)
                     .text_color(colors.text)
                     .min_diameter(18.0)
                     .show(ui);
               });
            });

         let people = [
            account_avatar("A", colors.info, colors.bg),
            account_avatar("M", colors.accent, colors.bg),
            account_avatar("N", colors.success, colors.bg),
         ];
         ui.add(
            AvatarGroup::new(&people)
               .size(34.0)
               .visuals(self.avatar_visuals(colors.bg))
               .label("Thread participants"),
         );

         ui.add(
            Label::new(
               RichText::new("Alice, Mike, Nora +1")
                  .size(self.theme.typography.very_small)
                  .color(colors.text_muted),
               None,
            )
            .interactive(false)
            .wrap(),
         );
      });

      self.card2().show(ui, |ui| {
         ui.set_width(ui.available_width());
         ui.spacing_mut().item_spacing.y = space.sm;
         ui.label(
            RichText::new("Preferences")
               .size(self.theme.typography.small)
               .color(colors.text)
               .strong(),
         );
         ui.add(
            Switch::new(
               &mut self.mute_thread,
               RichText::new("Mute thread")
                  .size(self.theme.typography.very_small)
                  .color(colors.text),
            )
            .size(18.0),
         );
         ui.add(
            Checkbox::new(
               &mut self.notify_on_reply,
               RichText::new("Notify me on reply")
                  .size(self.theme.typography.very_small)
                  .color(colors.text),
            )
            .size(16.0),
         );

         ui.add_space(space.xs);
         ui.label(
            RichText::new("Priority")
               .size(self.theme.typography.very_small)
               .color(colors.text_muted),
         );
         Flex::row("priority_group")
            .gap(space.sm)
            .align_items(CrossAlign::Center)
            .show(ui, |f| {
               for (value, label) in [
                  (Priority::Low, "Low"),
                  (Priority::Normal, "Normal"),
                  (Priority::High, "High"),
               ] {
                  f.item(Item::new(), |ui| {
                     ui.add(
                        Radio::new(
                           &mut self.priority,
                           value,
                           RichText::new(label)
                              .size(self.theme.typography.very_small)
                              .color(colors.text),
                        )
                        .size(16.0),
                     );
                  });
               }
            });
      });

      self.card2().show(ui, |ui| {
         ui.set_width(ui.available_width());
         ui.spacing_mut().item_spacing.y = space.sm;
         Flex::row("attachments_head")
            .justify(Justify::SpaceBetween)
            .align_items(CrossAlign::Center)
            .show(ui, |f| {
               f.item(Item::new(), |ui| {
                  ui.label(
                     RichText::new("Attachments")
                        .size(self.theme.typography.small)
                        .color(colors.text)
                        .strong(),
                  );
               });
               f.item(Item::new(), |ui| {
                  Badge::new(message.attachments.len().to_string())
                     .color(colors.info)
                     .text_color(colors.bg)
                     .min_diameter(18.0)
                     .show(ui);
               });
            });

         if message.attachments.is_empty() {
            self.caption(ui, "No files on this thread.");
         } else {
            Table::new("thread_attachments", message.attachments)
               .columns([Column::Fr(1.0), Column::Fixed(48.0), Column::Fixed(42.0)])
               .gap(space.xs)
               .row_gap(space.xs)
               .header_rule(true)
               .key(message.id)
               .show(ui, |table| {
                  table.header(|header| {
                     header.cell(|ui| {
                        ui.label(
                           RichText::new("File")
                              .size(self.theme.typography.very_small)
                              .color(colors.text_muted),
                        );
                     });
                     header.cell(|ui| {
                        ui.label(
                           RichText::new("Size")
                              .size(self.theme.typography.very_small)
                              .color(colors.text_muted),
                        );
                     });
                     header.cell(|_ui| {});
                  });
                  table.rows(|row, file| {
                     row.cell(|ui| {
                        ui.add(
                           Label::new(
                              RichText::new(file.name)
                                 .size(self.theme.typography.very_small)
                                 .color(colors.text),
                              None,
                           )
                           .interactive(false)
                           .wrap_mode(TextWrapMode::Truncate),
                        );
                     });
                     row.cell(|ui| {
                        ui.label(
                           RichText::new(file.size)
                              .size(self.theme.typography.very_small)
                              .color(colors.text_muted),
                        );
                     });
                     row.cell(|ui| {
                        ui.add(Button::new("Save").small().min_size(vec2(38.0, 24.0)));
                     });
                  });
               });
         }
      });

      self.card2().show(ui, |ui| {
         ui.set_width(ui.available_width());
         ui.spacing_mut().item_spacing.y = space.sm;
         ui.label(
            RichText::new("Activity")
               .size(self.theme.typography.small)
               .color(colors.text)
               .strong(),
         );
         activity_row(
            ui,
            self,
            colors.success,
            "Delivered to 3 recipients",
            "10:24",
         );
         activity_row(ui, self, colors.info, "Opened on mobile", "10:31");
         activity_row(
            ui,
            self,
            colors.warning,
            "Reply requested",
            "Yesterday",
         );
      });

      self.card2().show(ui, |ui| {
         ui.set_width(ui.available_width());
         Flex::row("details_footer")
            .justify(Justify::SpaceBetween)
            .align_items(CrossAlign::Center)
            .show(ui, |f| {
               f.item(Item::new(), |ui| {
                  self.caption(ui, "Thread ID 0x4A92");
               });
               f.item(Item::new(), |ui| {
                  ui.add(Button::new("Export").small().min_size(vec2(58.0, 24.0)));
               });
            });
      });
   }
}

fn tag_chip(ui: &mut Ui, app: &MailApp, text: &str, color: Color32) {
   let colors = app.theme.colors;
   let ink = if app.theme.dark {
      color
   } else {
      mix(color, colors.text, 0.28)
   };
   Frame::new()
      .fill(mix(color, colors.bg, 0.82))
      .corner_radius(4)
      .inner_margin(Margin::symmetric(6, 2))
      .show(ui, |ui| {
         ui.label(RichText::new(text).size(app.theme.typography.very_small).color(ink));
      });
}

fn activity_row(ui: &mut Ui, app: &MailApp, color: Color32, text: &str, time: &str) {
   let colors = app.theme.colors;
   let space = app.theme.spacing;
   Flex::row(("activity", text))
      .gap(space.sm)
      .align_items(CrossAlign::Center)
      .show(ui, |f| {
         f.item(Item::fixed(12.0), |ui| {
            Badge::dot().color(color).min_diameter(7.0).show(ui);
         });
         f.item(Item::grow(1.0), |ui| {
            ui.label(
               RichText::new(text)
                  .size(app.theme.typography.very_small)
                  .color(colors.text_muted),
            );
         });
         f.item(Item::new(), |ui| {
            ui.label(
               RichText::new(time)
                  .size(app.theme.typography.very_small)
                  .color(colors.text_muted),
            );
         });
      });
}

// ---------------------------------------------------------------------------
// Overlays
// ---------------------------------------------------------------------------

impl MailApp {
   fn compose_modal(&mut self, ui: &mut Ui) {
      let mut open = self.compose_open;
      let mut discard = false;
      let mut send = false;
      let colors = self.theme.colors;
      let space = self.theme.spacing;
      let small = self.theme.typography.small;
      let very_small = self.theme.typography.very_small;
      let send_visuals = self.filled_button_visuals(colors.accent);

      Modal::new("compose_modal", &mut open)
         .heading("New message")
         .subtitle("Reference compose flow - nothing leaves the demo.")
         .max_width(580.0)
         .footer(|ui| {
            Flex::row("compose_footer")
               .justify(Justify::SpaceBetween)
               .align_items(CrossAlign::Center)
               .show(ui, |f| {
                  f.item(Item::new(), |ui| {
                     ui.label(
                        RichText::new("Draft saved locally")
                           .size(very_small)
                           .color(colors.text_muted),
                     );
                  });
                  f.item(Item::new(), |ui| {
                     Flex::row("compose_actions")
                        .fill_main(false)
                        .gap(space.sm)
                        .align_items(CrossAlign::Center)
                        .show(ui, |f| {
                           f.item(Item::new(), |ui| {
                              let button =
                                 Button::new("Discard").small().min_size(vec2(84.0, 32.0));
                              if ui.add(button).clicked() {
                                 discard = true;
                              }
                           });
                           f.item(Item::new(), |ui| {
                              let button =
                                 Button::new(RichText::new("Send").size(small).color(colors.bg))
                                    .visuals(send_visuals)
                                    .min_size(vec2(88.0, 32.0));
                              if ui.add(button).clicked() {
                                 send = true;
                              }
                           });
                        });
                  });
               });
         })
         .show(ui.ctx(), |ui| {
            ui.spacing_mut().item_spacing.y = space.md;
            Flex::row("compose_identity")
               .justify(Justify::SpaceBetween)
               .align_items(CrossAlign::Center)
               .show(ui, |f| {
                  f.item(Item::new(), |ui| {
                     ui.label(RichText::new("From").size(small).color(colors.text_muted));
                  });
                  f.item(Item::new(), |ui| {
                     ui.label(
                        RichText::new(format!(
                           "{}  <{}>",
                           self.account_name(),
                           self.account_email()
                        ))
                        .size(very_small)
                        .color(colors.text),
                     );
                     ui.add_space(space.xs);
                  });
               });

            field_label(ui, self, "To");
            ui.add(
               SecureTextEdit::singleline(&mut self.compose_to)
                  .id_salt("compose_to")
                  .hint_text(RichText::new("name@example.com").color(colors.text_muted))
                  .margin(space.margin(space.sm))
                  .desired_width(f32::INFINITY)
                  .font(FontId::proportional(self.theme.typography.normal)),
            );

            Flex::row("compose_cc_head")
               .justify(Justify::SpaceBetween)
               .align_items(CrossAlign::Center)
               .show(ui, |f| {
                  f.item(Item::new(), |ui| field_label(ui, self, "Cc"));
                  f.item(Item::new(), |ui| {
                     ui.label(RichText::new("Optional").size(very_small).color(colors.text_muted));
                  });
               });
            ui.add(
               SecureTextEdit::singleline(&mut self.compose_cc)
                  .id_salt("compose_cc")
                  .hint_text(RichText::new("Add people to copy").color(colors.text_muted))
                  .margin(space.margin(space.sm))
                  .desired_width(f32::INFINITY)
                  .font(FontId::proportional(self.theme.typography.normal)),
            );

            field_label(ui, self, "Subject");
            ui.add(
               SecureTextEdit::singleline(&mut self.compose_subject)
                  .id_salt("compose_subject")
                  .hint_text(RichText::new("Subject").color(colors.text_muted))
                  .margin(space.margin(space.sm))
                  .desired_width(f32::INFINITY)
                  .font(FontId::proportional(self.theme.typography.normal)),
            );

            field_label(ui, self, "Message");
            ui.add(
               SecureTextEdit::multiline(&mut self.compose_body)
                  .id_salt("compose_body")
                  .hint_text(
                     RichText::new("Write something worth reading...").color(colors.text_muted),
                  )
                  .margin(space.margin(space.sm))
                  .desired_width(f32::INFINITY)
                  .desired_rows(7)
                  .font(FontId::proportional(self.theme.typography.normal)),
            );

            ui.add_space(space.sm);
            Flex::row("compose_options")
               .justify(Justify::SpaceBetween)
               .align_items(CrossAlign::Center)
               .show(ui, |f| {
                  f.item(Item::new(), |ui| {
                     ui.add(
                        Checkbox::new(
                           &mut self.compose_receipt,
                           RichText::new("Request read receipt").size(small).color(colors.text),
                        )
                        .size(16.0),
                     );
                  });
                  f.item(Item::new(), |ui| {
                     ui.label(
                        RichText::new("Signed with your default identity")
                           .size(very_small)
                           .color(colors.text_muted),
                     );
                  });
               });
         });

      if discard || send {
         open = false;
         if discard {
            self.compose_to.clear();
            self.compose_cc.clear();
            self.compose_subject.clear();
            self.compose_body.clear();
         }
      }
      self.compose_open = open;
   }

   fn delete_modal(&mut self, ui: &mut Ui) {
      let mut open = self.delete_open;
      let mut close = false;
      let colors = self.theme.colors;
      let space = self.theme.spacing;
      let small = self.theme.typography.small;
      let delete_visuals = self.filled_button_visuals(colors.error);

      Modal::new("delete_modal", &mut open)
         .heading("Delete message")
         .subtitle("Reference confirmation - the thread stays put.")
         .alert(true)
         .max_width(430.0)
         .footer(|ui| {
            Flex::row("delete_footer")
               .justify(Justify::End)
               .gap(space.sm)
               .align_items(CrossAlign::Center)
               .show(ui, |f| {
                  f.item(Item::new(), |ui| {
                     let cancel = Button::new("Cancel").small().min_size(vec2(84.0, 32.0));
                     if ui.add(cancel).clicked() {
                        close = true;
                     }
                  });
                  f.item(Item::new(), |ui| {
                     let confirm = Button::new(RichText::new("Hold to delete").size(small).color(colors.bg))
                        .visuals(delete_visuals)
                        .hold(0.9)
                        .hold_fill(colors.error)
                        .hold_burst(HoldBurst::Glow)
                        .min_size(vec2(126.0, 32.0));
                     if ui.add(confirm).clicked() {
                        close = true;
                     }
                  });
               });
         })
         .show(ui.ctx(), |ui| {
            ui.label(
               RichText::new("This would move the thread to Trash in a real client.")
                  .size(self.theme.typography.normal)
                  .color(colors.text),
            );
            ui.add_space(space.sm);
            ui.label(
               RichText::new(
                  "Use the hold gesture to check the destructive fill, the error slot, and the footer actions.",
               )
               .size(small)
               .color(colors.text_muted),
            );
         });

      if close {
         open = false;
      }
      self.delete_open = open;
   }

   fn shortcuts_window(&mut self, ui: &mut Ui) {
      let mut open = self.shortcuts_open;
      if !open {
         return;
      }
      let colors = self.theme.colors;
      let space = self.theme.spacing;
      let mut close = false;

      Window::new("Keyboard shortcuts")
         .open(&mut open)
         .frame(self.theme.window_frame)
         .title_frame(self.theme.window_frame.stroke(Stroke::NONE))
         .default_width(380.0)
         .resizable(false)
         .collapsible(false)
         .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
         .show(ui.ctx(), |ui| {
            ui.spacing_mut().item_spacing.y = space.xs;
            ui.label(
               RichText::new("Work without leaving the keyboard")
                  .size(self.theme.typography.small)
                  .color(colors.text_muted),
            );
            ui.add_space(space.sm);
            for (key, description) in [
               ("C", "Compose a new message"),
               ("R", "Reply to the selected thread"),
               ("A", "Reply to all"),
               ("F", "Forward"),
               ("/", "Focus search"),
               ("J / K", "Move down / up the inbox"),
               ("E", "Archive the thread"),
               ("#", "Delete with confirmation"),
               ("?", "Show this window"),
            ] {
               shortcut_row(ui, self, key, description);
            }
            ui.add_space(space.sm);
            Flex::row("shortcut_footer")
               .justify(Justify::SpaceBetween)
               .align_items(CrossAlign::Center)
               .show(ui, |f| {
                  f.item(Item::new(), |ui| {
                     ui.label(
                        RichText::new("Reference bindings - not wired")
                           .size(self.theme.typography.very_small)
                           .color(colors.text_muted),
                     );
                  });
                  f.item(Item::new(), |ui| {
                     if ui.add(Button::new("Close").small().min_size(vec2(64.0, 26.0))).clicked() {
                        close = true;
                     }
                  });
               });
         });

      if close {
         open = false;
      }
      self.shortcuts_open = open;
   }
}

// ---------------------------------------------------------------------------
// Small helpers
// ---------------------------------------------------------------------------

fn field_label(ui: &mut Ui, app: &MailApp, text: &str) {
   ui.label(
      RichText::new(text)
         .size(app.theme.typography.small)
         .color(app.theme.colors.text),
   );
}

fn shortcut_row(ui: &mut Ui, app: &MailApp, key: &str, description: &str) {
   let colors = app.theme.colors;
   let space = app.theme.spacing;

   Flex::row(("shortcut", key))
      .gap(space.md)
      .align_items(CrossAlign::Center)
      .show(ui, |f| {
         f.item(Item::fixed(44.0), |ui| {
            app.theme
               .frame2
               .inner_margin(space.margin_xy(space.sm, space.xs))
               .show(ui, |ui| {
                  ui.set_width(28.0);
                  ui.vertical_centered(|ui| {
                     ui.label(
                        RichText::new(key)
                           .size(app.theme.typography.very_small)
                           .color(colors.text)
                           .strong(),
                     );
                  });
               });
         });
         f.item(Item::grow(1.0), |ui| {
            ui.label(
               RichText::new(description)
                  .size(app.theme.typography.small)
                  .color(colors.text_muted),
            );
         });
      });
}

fn account_avatar(initial: &str, plate: Color32, ink: Color32) -> Avatar {
   Avatar::initials(initial).plate(plate).ink(ink)
}

fn initial(name: &str) -> String {
   name
      .chars()
      .next()
      .map(|character| character.to_uppercase().collect())
      .unwrap_or_else(|| "?".to_owned())
}

fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
   let t = t.clamp(0.0, 1.0);
   let lerp = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
   Color32::from_rgba_unmultiplied(
      lerp(a.r(), b.r()),
      lerp(a.g(), b.g()),
      lerp(a.b(), b.b()),
      a.a(),
   )
}

#[derive(Clone, Copy)]
enum Icon {
   Inbox,
   Star,
   Sent,
   Draft,
   Archive,
   Trash,
}

fn icon(ui: &mut Ui, kind: Icon, color: Color32, size: f32) {
   let (rect, _) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
   paint_icon(ui, rect, kind, color);
}

fn paint_icon(ui: &mut Ui, rect: Rect, kind: Icon, color: Color32) {
   let painter = ui.painter();
   let c = rect.center();
   let s = rect.width().min(rect.height());
   let stroke = Stroke::new((s * 0.09).clamp(1.0, 1.8), color);

   match kind {
      Icon::Inbox => {
         let body = Rect::from_center_size(c, vec2(s * 0.66, s * 0.54));
         painter.rect_stroke(body, 2.0, stroke, StrokeKind::Inside);
         let y = body.center().y + s * 0.02;
         painter.line_segment(
            [
               pos2(body.left() + s * 0.04, y),
               pos2(body.center().x - s * 0.05, y),
            ],
            stroke,
         );
         painter.line_segment(
            [
               pos2(body.center().x + s * 0.05, y),
               pos2(body.right() - s * 0.04, y),
            ],
            stroke,
         );
      }
      Icon::Star => {
         let points = [
            (0.00, -0.44),
            (0.12, -0.14),
            (0.44, -0.12),
            (0.19, 0.10),
            (0.27, 0.42),
            (0.00, 0.25),
            (-0.27, 0.42),
            (-0.19, 0.10),
            (-0.44, -0.12),
            (-0.12, -0.14),
         ];
         for pair in points.windows(2) {
            painter.line_segment(
               [
                  pos2(c.x + pair[0].0 * s, c.y + pair[0].1 * s),
                  pos2(c.x + pair[1].0 * s, c.y + pair[1].1 * s),
               ],
               stroke,
            );
         }
         painter.line_segment(
            [
               pos2(c.x + points[9].0 * s, c.y + points[9].1 * s),
               pos2(c.x + points[0].0 * s, c.y + points[0].1 * s),
            ],
            stroke,
         );
      }
      Icon::Sent => {
         painter.line_segment(
            [
               pos2(c.x - s * 0.28, c.y + s * 0.30),
               pos2(c.x + s * 0.30, c.y - s * 0.30),
            ],
            stroke,
         );
         painter.line_segment(
            [
               pos2(c.x + s * 0.04, c.y - s * 0.30),
               pos2(c.x + s * 0.30, c.y - s * 0.30),
            ],
            stroke,
         );
         painter.line_segment(
            [
               pos2(c.x + s * 0.30, c.y - s * 0.30),
               pos2(c.x + s * 0.30, c.y - s * 0.04),
            ],
            stroke,
         );
      }
      Icon::Draft => {
         painter.line_segment(
            [
               pos2(c.x - s * 0.24, c.y + s * 0.30),
               pos2(c.x + s * 0.30, c.y - s * 0.26),
            ],
            stroke,
         );
         painter.line_segment(
            [
               pos2(c.x + s * 0.30, c.y - s * 0.26),
               pos2(c.x + s * 0.10, c.y - s * 0.28),
            ],
            stroke,
         );
         painter.line_segment(
            [
               pos2(c.x + s * 0.30, c.y - s * 0.26),
               pos2(c.x + s * 0.28, c.y - s * 0.06),
            ],
            stroke,
         );
      }
      Icon::Archive => {
         painter.rect_stroke(
            Rect::from_center_size(c, vec2(s * 0.66, s * 0.52)),
            2.0,
            stroke,
            StrokeKind::Inside,
         );
         painter.line_segment(
            [
               pos2(c.x - s * 0.33, c.y - s * 0.18),
               pos2(c.x + s * 0.33, c.y - s * 0.18),
            ],
            stroke,
         );
         painter.line_segment(
            [
               pos2(c.x - s * 0.10, c.y + s * 0.06),
               pos2(c.x + s * 0.10, c.y + s * 0.06),
            ],
            stroke,
         );
      }
      Icon::Trash => {
         painter.line_segment(
            [
               pos2(c.x - s * 0.28, c.y - s * 0.24),
               pos2(c.x + s * 0.28, c.y - s * 0.24),
            ],
            stroke,
         );
         painter.line_segment(
            [
               pos2(c.x - s * 0.16, c.y - s * 0.32),
               pos2(c.x + s * 0.16, c.y - s * 0.32),
            ],
            stroke,
         );
         painter.rect_stroke(
            Rect::from_center_size(
               pos2(c.x, c.y + s * 0.04),
               vec2(s * 0.44, s * 0.54),
            ),
            2.0,
            stroke,
            StrokeKind::Inside,
         );
         painter.line_segment(
            [
               pos2(c.x - s * 0.08, c.y - s * 0.08),
               pos2(c.x - s * 0.08, c.y + s * 0.22),
            ],
            stroke,
         );
         painter.line_segment(
            [
               pos2(c.x + s * 0.08, c.y - s * 0.08),
               pos2(c.x + s * 0.08, c.y + s * 0.22),
            ],
            stroke,
         );
      }
   }
}

fn brand_mark(ui: &mut Ui, colors: ThemeColors) {
   let (rect, _) = ui.allocate_exact_size(vec2(22.0, 22.0), Sense::hover());
   ui.painter().rect_filled(rect, 6.0, colors.accent);
   ui.painter().text(
      rect.center(),
      Align2::CENTER_CENTER,
      "e",
      FontId::proportional(15.0),
      colors.bg,
   );
}
