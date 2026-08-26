//! Product-framed placeholder for `egui-elements`: a mail client layout
//! that exercises widgets the way a real app would. Nothing here talks
//! to a mail backend — folders, messages, and compose are static.
//!
//! Run with: `cargo run --example email`

use eframe::egui::*;
use egui_elements::editor::ThemeEditor;
use egui_elements::theme::{Theme, ThemeKind};
use egui_elements::utils;
use egui_elements::widgets::{
   Button, ComboBox, Frame as Frame2, Label, Modal, MultiLabel, SecureTextEdit,
};

fn main() -> eframe::Result {
   let options = eframe::NativeOptions {
      viewport: ViewportBuilder::default()
         .with_title("egui-elements mail")
         .with_inner_size([1280.0, 820.0])
         .with_resizable(true),
      ..Default::default()
   };

   eframe::run_native(
      "egui-elements mail",
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
}

struct Message {
   id: u32,
   folder: Folder,
   starred: bool,
   unread: bool,
   from: &'static str,
   to: &'static str,
   subject: &'static str,
   preview: &'static str,
   time: &'static str,
   body: &'static str,
}

const MESSAGES: &[Message] = &[
   Message {
      id: 1,
      folder: Folder::Inbox,
      starred: true,
      unread: true,
      from: "Alice Chen",
      to: "you@example.com",
      subject: "Q3 design review",
      preview: "Looking over the latest mockups — the sidebar density feels right.",
      time: "10:24 AM",
      body: "Looking over the latest mockups — the sidebar density feels right.\n\nA few notes from this morning:\n\n• Keep folder labels quiet; unread counts can carry the emphasis.\n• The reading pane should sit on the same canvas as the list, not a second window.\n• Compose stays a modal so the inbox does not lose context.\n\nCan you take a pass on hover vs selected in the thread list?",
   },
   Message {
      id: 2,
      folder: Folder::Inbox,
      starred: false,
      unread: true,
      from: "GitHub",
      to: "you@example.com",
      subject: "[egui-elements] Review requested: #42",
      preview: "mike requested your review on pull request #42: theme tokens.",
      time: "9:02 AM",
      body: "mike requested your review on pull request #42: theme tokens.\n\nhighlight must stay distinct from hover. text_muted is not a selection fill.\n\nhttps://github.com/greekfetacheese/egui-elements/pull/42",
   },
   Message {
      id: 3,
      folder: Folder::Inbox,
      starred: false,
      unread: false,
      from: "Nora Patel",
      to: "you@example.com",
      subject: "Invoice for March",
      preview: "Attached the PDF. Let me know if the line items look off.",
      time: "Yesterday",
      body: "Attached the PDF. Let me know if the line items look off.\n\nTotal: $1,240.00\nDue: April 12\n\nThanks,\nNora",
   },
   Message {
      id: 4,
      folder: Folder::Inbox,
      starred: false,
      unread: false,
      from: "This Week in Rust",
      to: "you@example.com",
      subject: "This Week in Rust #592",
      preview: "Cranelift news, a new egui release, and interesting blog posts.",
      time: "Mon",
      body: "Cranelift news, a new egui release, and interesting blog posts.\n\n— Quote of the week —\n\"Make illegal states unrepresentable.\"\n\nThat's all for this week.",
   },
   Message {
      id: 5,
      folder: Folder::Inbox,
      starred: true,
      unread: false,
      from: "Mike",
      to: "you@example.com",
      subject: "Theme tokens",
      preview: "Can we try the mail layout against frame1 / frame2 before the next cut?",
      time: "Sun",
      body: "Can we try the mail layout against frame1 / frame2 before the next cut?\n\nI want to see buttons, labels, combos, and text edits in a product chrome — not just the gallery.\n\nHeader stays the demo chrome: theme switcher + editor only.",
   },
   Message {
      id: 6,
      folder: Folder::Sent,
      starred: false,
      unread: false,
      from: "you@example.com",
      to: "Alice Chen",
      subject: "Re: Q3 design review",
      preview: "Agreed on the sidebar. I'll mock the reading pane tonight.",
      time: "11:03 AM",
      body: "Agreed on the sidebar. I'll mock the reading pane tonight.\n\nAlso dropping a compose modal so we can see SecureTextEdit stacked in a real flow.",
   },
   Message {
      id: 7,
      folder: Folder::Sent,
      starred: false,
      unread: false,
      from: "you@example.com",
      to: "Nora Patel",
      subject: "Re: Invoice for March",
      preview: "Looks good — paid just now.",
      time: "Yesterday",
      body: "Looks good — paid just now.\n\nThanks,\nYou",
   },
   Message {
      id: 8,
      folder: Folder::Drafts,
      starred: false,
      unread: false,
      from: "you@example.com",
      to: "",
      subject: "Weekend plans",
      preview: "Draft — still missing a recipient.",
      time: "Sat",
      body: "Draft — still missing a recipient.\n\nThinking of heading north if the weather holds.",
   },
   Message {
      id: 9,
      folder: Folder::Drafts,
      starred: false,
      unread: false,
      from: "you@example.com",
      to: "team@example.com",
      subject: "(no subject)",
      preview: "Empty draft.",
      time: "Fri",
      body: "",
   },
   Message {
      id: 10,
      folder: Folder::Archive,
      starred: false,
      unread: false,
      from: "HR",
      to: "you@example.com",
      subject: "Benefits enrollment",
      preview: "Window closes Friday. No action needed if you are staying put.",
      time: "Mar 2",
      body: "Window closes Friday. No action needed if you are staying put.\n\nHR",
   },
   Message {
      id: 11,
      folder: Folder::Trash,
      starred: false,
      unread: false,
      from: "Prize Desk",
      to: "you@example.com",
      subject: "You have won",
      preview: "Definitely not a prize.",
      time: "Mar 1",
      body: "Definitely not a prize.",
   },
];

struct MailApp {
   theme: Theme,
   editor: ThemeEditor,
   folder: Folder,
   selected_id: u32,
   search: String,
   account: String,
   accounts: Vec<String>,
   compose_open: bool,
   delete_open: bool,
   compose_to: String,
   compose_subject: String,
   compose_body: String,
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
         account: "Personal".to_owned(),
         accounts: vec!["Personal".to_owned(), "Work".to_owned()],
         compose_open: false,
         delete_open: false,
         compose_to: String::new(),
         compose_subject: String::new(),
         compose_body: String::new(),
      }
   }

   fn visible_messages(&self) -> Vec<&'static Message> {
      let q = self.search.trim().to_ascii_lowercase();
      MESSAGES
         .iter()
         .filter(|m| match self.folder {
            Folder::Starred => m.starred,
            other => m.folder == other,
         })
         .filter(|m| {
            if q.is_empty() {
               return true;
            }
            m.from.to_ascii_lowercase().contains(&q)
               || m.subject.to_ascii_lowercase().contains(&q)
               || m.preview.to_ascii_lowercase().contains(&q)
         })
         .collect()
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

   fn selected(&self) -> Option<&'static Message> {
      MESSAGES.iter().find(|m| m.id == self.selected_id)
   }

   fn muted(&self, ui: &mut Ui, text: &str) {
      ui.label(
         RichText::new(text)
            .size(self.theme.typography.small)
            .color(self.theme.colors.text_muted),
      );
   }
}

impl eframe::App for MailApp {
   fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
      self.theme.overlay_manager.paint_overlay(ui.ctx(), true);

      Panel::top("header")
         .frame(
            Frame::new()
               .fill(self.theme.colors.bg)
               .inner_margin(Margin::symmetric(16, 12))
               .stroke(Stroke::new(1.0, self.theme.colors.border)),
         )
         .show(ui, |ui| self.header(ui));

      if let Some(new_theme) = self.editor.show(&mut self.theme, ui) {
         self.theme = new_theme;
      }

      self.theme.install(ui.ctx());

      self.show_compose_modal(ui);
      self.show_delete_modal(ui);

      Panel::top("toolbar")
         .frame(
            Frame::new()
               .fill(self.theme.colors.bg)
               .inner_margin(Margin::symmetric(16, 8))
               .stroke(Stroke::new(1.0, self.theme.colors.border)),
         )
         .show(ui, |ui| self.toolbar(ui));

      Panel::left("folders")
         .resizable(true)
         .default_size(220.0)
         .size_range(180.0..=280.0)
         .frame(
            Frame::new()
               .fill(self.theme.colors.widget_bg)
               .inner_margin(Margin::symmetric(2, 12))
               .stroke(Stroke::new(1.0, self.theme.colors.border)),
         )
         .show(ui, |ui| self.folders(ui));

      Panel::left("threads")
         .resizable(true)
         .default_size(360.0)
         .size_range(280.0..=480.0)
         .frame(
            Frame::new()
               .fill(self.theme.colors.bg)
               .inner_margin(Margin::symmetric(12, 12))
               .stroke(Stroke::new(1.0, self.theme.colors.border)),
         )
         .show(ui, |ui| self.thread_list(ui));

      let bg = self.theme.colors.bg;
      CentralPanel::default()
         .frame(Frame::new().fill(bg).inner_margin(Margin::same(16)))
         .show(ui, |ui| self.reading_pane(ui));
   }
}

impl MailApp {
   fn header(&mut self, ui: &mut Ui) {
      ui.horizontal(|ui| {
         ui.spacing_mut().button_padding = self.theme.button_padding;

         ui.label(
            RichText::new("egui-elements")
               .size(self.theme.typography.very_large)
               .color(self.theme.colors.accent)
               .strong(),
         );
         ui.add_space(10.0);
         self.muted(ui, "Mail client placeholder");

         ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let editor_text = if self.editor.open {
               "Close Editor"
            } else {
               "Theme Editor"
            };
            let editor_btn = Button::new(editor_text).min_size(vec2(120.0, 28.0));
            if ui.add(editor_btn).clicked() {
               self.editor.open = !self.editor.open;
            }

            ui.add_space(12.0);

            if let Some(new_theme) = utils::theme_switcher(&self.theme, ui) {
               self.theme = new_theme;
            }
         });
      });
   }

   fn toolbar(&mut self, ui: &mut Ui) {
      let text_color = self.theme.colors.text;
      let muted = self.theme.colors.text_muted;
      let size = self.theme.typography.normal;
      let on_accent = self.theme.colors.bg;
      let accent = self.theme.colors.accent;

      ui.horizontal(|ui| {
         ui.spacing_mut().button_padding = self.theme.button_padding;
         ui.spacing_mut().item_spacing.x = 12.0;

         let compose = Button::new(RichText::new("Compose").size(size).color(on_accent))
            .bg_color(accent)
            .min_size(vec2(108.0, 28.0));
         if ui.add(compose).clicked() {
            self.compose_open = true;
         }

         ui.add(
            SecureTextEdit::singleline(&mut self.search)
               .id_salt("mail_search")
               .hint_text(RichText::new("Search mail").color(muted))
               .margin(Margin::same(8))
               .desired_width(260.0)
               .font(FontId::proportional(size)),
         );

         ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let selected = Label::new(
               RichText::new(&self.account).size(size).color(text_color),
               None,
            );
            ComboBox::new("mail_account", selected).width(160.0).label("Account").show_ui(
               ui,
               |ui| {
                  ui.spacing_mut().item_spacing.y = 12.0;
                  for account in self.accounts.clone() {
                     let label = Label::new(
                        RichText::new(&account).size(size).color(text_color),
                        None,
                     )
                     .expand(Some(4.0))
                     .selected(self.account == account)
                     .sense(Sense::click())
                     .fill_width(true);
                     if ui.add(label).clicked() {
                        self.account = account;
                     }
                  }
               },
            );
         });
      });
   }

   fn folders(&mut self, ui: &mut Ui) {
      let text_color = self.theme.colors.text;
      let muted = self.theme.colors.text_muted;
      let size = self.theme.typography.normal;
      let small = self.theme.typography.small;
      let info = self.theme.colors.info;

      ui.spacing_mut().item_spacing.y = 4.0;
      ui.label(RichText::new("Folders").size(self.theme.typography.small).color(muted));
      ui.add_space(4.0);

      // Idle fill matches the sidebar so rows only light up on hover / selected.
      // Name + unread count share that fill.
      let mut visuals = self.theme.frame2_visuals();
      visuals.bg = self.theme.colors.widget_bg;
      visuals.bg_click = self.theme.colors.widget_bg;
      
      let folder_frame = Frame2::from_egui(self.theme.frame2.outer_margin(Margin::ZERO))
         .interactive(true)
         .fill_width(true)
         .square_corners()
         .visuals(visuals);

      let mut clicked = None;
      for folder in Folder::ALL {
         let selected = self.folder == folder;
         let unread = Self::unread_count(folder);
         let name = folder.label();

         let res = folder_frame.selected(selected).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
               ui.spacing_mut().item_spacing.x = 8.0;
               ui.add(
                  Label::new(RichText::new(name).size(size).color(text_color), None)
                     .interactive(false),
               );
               ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                  if unread > 0 {
                     ui.add(
                        Label::new(
                           RichText::new(format!("{unread}")).size(small).color(info),
                           None,
                        )
                        .interactive(false),
                     );
                  }
               });
            });
         });

         if res.response.clicked() {
            clicked = Some(folder);
         }
      }

      if let Some(folder) = clicked {
         self.folder = folder;
         if let Some(first) = self.visible_messages().first() {
            self.selected_id = first.id;
         }
      }

      ui.add_space(16.0);
      ui.label(RichText::new("Labels").size(small).color(muted));
      ui.add_space(4.0);

      let tags = [
         ("Design", self.theme.colors.accent),
         ("Finance", self.theme.colors.success),
         ("Alerts", self.theme.colors.warning),
      ];
      for (name, color) in tags {
         ui.add(
            Label::new(RichText::new(name).size(size).color(color), None)
               .interactive(false)
               .expand(Some(2.0)),
         );
      }
   }

   fn thread_list(&mut self, ui: &mut Ui) {
      let text_color = self.theme.colors.text;
      let muted = self.theme.colors.text_muted;
      let size = self.theme.typography.normal;
      let small = self.theme.typography.small;
      let heading = self.theme.typography.large;
      let messages = self.visible_messages();
      let selected_id = self.selected_id;

      ui.horizontal(|ui| {
         ui.label(RichText::new(self.folder.label()).size(heading).color(text_color).strong());
         ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(RichText::new(format!("{} threads", messages.len())).size(small).color(muted));
         });
      });
      ui.add_space(8.0);

      let visuals = self.theme.frame2_visuals();
      let frame2 = Frame2::from_egui(self.theme.frame2.outer_margin(Margin::ZERO))
         .interactive(true)
         .fill_width(true)
         .visuals(visuals);

      ScrollArea::vertical()
         .id_salt("thread_scroll")
         .auto_shrink([false, false])
         .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 8.0;
            let mut clicked = None;

            if messages.is_empty() {
               ui.add_space(24.0);
               ui.label(RichText::new("No messages in this folder.").size(size).color(muted));
               return;
            }

            for message in messages {
               let selected = message.id == selected_id;
               let from = message.from;
               let subject = message.subject;
               let preview = message.preview;
               let time = message.time;
               let unread = message.unread;
               let starred = message.starred;
               let id = message.id;

               let res = frame2.selected(selected).show(ui, |ui| {
                  ui.vertical(|ui| {
                     ui.set_width(ui.available_width());
                     ui.spacing_mut().item_spacing.y = 4.0;
                     ui.horizontal(|ui| {
                        let from_text = if unread {
                           RichText::new(from).size(size).color(text_color).strong()
                        } else {
                           RichText::new(from).size(size).color(text_color)
                        };
                        ui.add(Label::new(from_text, None).interactive(false));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                           ui.label(RichText::new(time).size(small).color(muted));
                           if starred {
                              ui.label(
                                 RichText::new("★").size(small).color(self.theme.colors.warning),
                              );
                           }
                        });
                     });
                     ui.add(
                        Label::new(
                           RichText::new(subject).size(size).color(text_color),
                           None,
                        )
                        .interactive(false)
                        .wrap(),
                     );
                     ui.add(
                        Label::new(
                           RichText::new(preview).size(small).color(muted),
                           None,
                        )
                        .interactive(false)
                        .wrap(),
                     );
                  });
               });

               if res.response.clicked() {
                  clicked = Some(id);
               }
            }

            if let Some(id) = clicked {
               self.selected_id = id;
            }
         });
   }

   fn reading_pane(&mut self, ui: &mut Ui) {
      let Some(message) = self.selected() else {
         self.muted(ui, "Select a thread.");
         return;
      };

      let text_color = self.theme.colors.text;
      let muted = self.theme.colors.text_muted;
      let size = self.theme.typography.normal;
      let small = self.theme.typography.small;
      let heading = self.theme.typography.heading;
      let on_fill = self.theme.colors.bg;
      let error = self.theme.colors.error;
      let info = self.theme.colors.info;
      let success = self.theme.colors.success;
      let warning = self.theme.colors.warning;

      ui.spacing_mut().button_padding = self.theme.button_padding;
      ui.spacing_mut().item_spacing.y = 10.0;

      ui.label(RichText::new(message.subject).size(heading).color(text_color).strong());

      ui.add(
         MultiLabel::new(vec![
            Label::new(
               RichText::new(message.from).size(size).color(text_color),
               None,
            )
            .interactive(false),
            Label::new(
               RichText::new(format!("→ {}", message.to)).size(small).color(muted),
               None,
            )
            .interactive(false),
            Label::new(
               RichText::new(message.time).size(small).color(muted),
               None,
            )
            .interactive(false),
         ])
         .inter_spacing(12.0),
      );

      ui.horizontal(|ui| {
         ui.spacing_mut().item_spacing.x = 8.0;
         ui.add(
            Button::new(RichText::new("Reply").size(size).color(text_color))
               .min_size(vec2(88.0, 28.0)),
         );
         ui.add(
            Button::new(RichText::new("Reply all").size(size).color(text_color))
               .min_size(vec2(96.0, 28.0)),
         );
         ui.add(
            Button::new(RichText::new("Forward").size(size).color(text_color))
               .min_size(vec2(96.0, 28.0)),
         );
         ui.add(
            Button::new(RichText::new("Archive").size(size).color(on_fill))
               .bg_color(success)
               .min_size(vec2(88.0, 28.0)),
         );
         ui.add(
            Button::new(RichText::new("Star").size(size).color(on_fill))
               .bg_color(warning)
               .min_size(vec2(72.0, 28.0)),
         );
         if ui
            .add(
               Button::new(RichText::new("Delete").size(size).color(on_fill))
                  .bg_color(error)
                  .min_size(vec2(88.0, 28.0)),
            )
            .clicked()
         {
            self.delete_open = true;
         }
         ui.add(
            Button::new(RichText::new("Info").size(size).color(on_fill))
               .bg_color(info)
               .small(),
         );
      });

      ui.separator();

      let frame1 = self.theme.frame1;
      frame1.show(ui, |ui| {
         ui.set_width(ui.available_width());
         ScrollArea::vertical()
            .id_salt("reading_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
               ui.set_width(ui.available_width());
               ui.spacing_mut().item_spacing.y = 10.0;
               for para in message.body.split("\n\n") {
                  ui.add(
                     Label::new(
                        RichText::new(para).size(size).color(text_color),
                        None,
                     )
                     .interactive(false)
                     .wrap(),
                  );
               }

               ui.add_space(12.0);
               let quote = self.theme.frame2.outer_margin(Margin::ZERO);
               quote.show(ui, |ui| {
                  ui.set_width(ui.available_width());
                  ui.label(
                     RichText::new("Quoted from earlier in the thread")
                        .size(small)
                        .color(muted)
                        .italics(),
                  );
                  ui.add_space(4.0);
                  ui.add(
                     Label::new(
                        RichText::new(message.preview).size(small).color(muted),
                        None,
                     )
                     .interactive(false)
                     .wrap(),
                  );
               });
            });
      });
   }

   fn show_compose_modal(&mut self, ui: &mut Ui) {
      let mut open = self.compose_open;
      let mut close = false;
      let mut send = false;
      let muted = self.theme.colors.text_muted;
      let size = self.theme.typography.normal;
      let text_color = self.theme.colors.text;
      let on_accent = self.theme.colors.bg;
      let accent = self.theme.colors.accent;

      Modal::new("compose_modal", &mut open)
         .heading("New message")
         .subtitle("Placeholder compose — nothing is sent.")
         .header_icon("✉")
         .max_width(520.0)
         .footer(|ui| {
            let send_btn = Button::new(RichText::new("Send").color(on_accent))
               .bg_color(accent)
               .min_size(vec2(88.0, 32.0));
            if ui.add(send_btn).clicked() {
               send = true;
            }
            let discard = Button::new("Discard").min_size(vec2(88.0, 32.0));
            if ui.add(discard).clicked() {
               close = true;
            }
         })
         .show(ui.ctx(), |ui| {
            ui.spacing_mut().item_spacing.y = 10.0;
            ui.label(RichText::new("To").size(size).color(text_color));
            ui.add(
               SecureTextEdit::singleline(&mut self.compose_to)
                  .id_salt("compose_to")
                  .hint_text(RichText::new("name@example.com").color(muted))
                  .margin(Margin::same(8))
                  .desired_width(f32::INFINITY)
                  .font(FontId::proportional(size)),
            );
            ui.label(RichText::new("Subject").size(size).color(text_color));
            ui.add(
               SecureTextEdit::singleline(&mut self.compose_subject)
                  .id_salt("compose_subject")
                  .hint_text(RichText::new("Subject").color(muted))
                  .margin(Margin::same(8))
                  .desired_width(f32::INFINITY)
                  .font(FontId::proportional(size)),
            );
            ui.label(RichText::new("Message").size(size).color(text_color));
            ui.add(
               SecureTextEdit::multiline(&mut self.compose_body)
                  .id_salt("compose_body")
                  .hint_text(RichText::new("Write something…").color(muted))
                  .margin(Margin::same(8))
                  .desired_width(f32::INFINITY)
                  .desired_rows(8)
                  .font(FontId::proportional(size)),
            );
         });

      if close || send {
         open = false;
         if close {
            self.compose_to.clear();
            self.compose_subject.clear();
            self.compose_body.clear();
         }
      }
      self.compose_open = open;
   }

   fn show_delete_modal(&mut self, ui: &mut Ui) {
      let mut open = self.delete_open;
      let mut close = false;
      let on_fill = self.theme.colors.bg;
      let error = self.theme.colors.error;
      let size = self.theme.typography.normal;
      let text_color = self.theme.colors.text;
      let muted = self.theme.colors.text_muted;

      Modal::new("delete_modal", &mut open)
         .heading("Delete message")
         .subtitle("Placeholder — the thread stays put.")
         .header_icon("⚠")
         .alert(true)
         .max_width(420.0)
         .footer(|ui| {
            let confirm = Button::new(RichText::new("Delete").color(on_fill))
               .bg_color(error)
               .min_size(vec2(88.0, 32.0));
            if ui.add(confirm).clicked() {
               close = true;
            }
            let cancel = Button::new("Cancel").min_size(vec2(88.0, 32.0));
            if ui.add(cancel).clicked() {
               close = true;
            }
         })
         .show(ui.ctx(), |ui| {
            ui.label(
               RichText::new("This would move the thread to Trash in a real client.")
                  .size(size)
                  .color(text_color),
            );
            ui.add_space(6.0);
            ui.label(
               RichText::new("Use it to check the alert modal, error fill, and footer actions.")
                  .size(self.theme.typography.small)
                  .color(muted),
            );
         });

      if close {
         open = false;
      }
      self.delete_open = open;
   }
}
