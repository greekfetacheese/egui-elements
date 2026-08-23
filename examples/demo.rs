//! Showcase for `egui-elements`: palette combinations, custom widgets,
//! components, a theme switcher, and the live theme editor.
//!
//! Run with: `cargo run --example demo --features full`

use eframe::egui::*;
use egui_elements::components::{CredentialsForm, QrImage, SecureInputField};
use egui_elements::editor::ThemeEditor;
use egui_elements::theme::{Theme, ThemeKind};
use egui_elements::utils;
use egui_elements::widgets::{Button, ComboBox, Window, Label, Modal, SecureTextEdit};

fn main() -> eframe::Result {
   let options = eframe::NativeOptions {
      viewport: ViewportBuilder::default()
         .with_title("egui-elements demo")
         .with_inner_size([1180.0, 820.0])
         .with_resizable(true),
      ..Default::default()
   };

   eframe::run_native(
      "egui-elements demo",
      options,
      Box::new(|cc| {
         egui_extras::install_image_loaders(&cc.egui_ctx);
         Ok(Box::new(DemoApp::new(cc)))
      }),
   )
}

#[derive(Clone, PartialEq)]
pub struct Items {
   pub items: Vec<String>,
   pub selected: Option<String>,
}

impl Default for Items {
   fn default() -> Self {
      Self {
         items: vec![
            "Item 1".to_owned(),
            "Item 2".to_owned(),
            "Item 3".to_owned(),
         ],
         selected: None,
      }
   }
}

impl Items {
   fn selected_str(&self) -> String {
      match self.selected {
         Some(ref item) => item.clone(),
         None => "Select an Item".to_owned(),
      }
   }

   fn all(&self) -> Vec<String> {
      self.items.clone()
   }

   fn add(&mut self) {
      let mut num = self.items.len();
      loop {
         let item = format!("Item {}", num);
         if !self.items.contains(&item) {
            self.items.push(item);
            break;
         }
         num += 1;
      }
   }

   fn remove(&mut self) {
      if let Some(selected) = self.selected.clone() {
         self.items.retain(|item| item != &selected);
         if self.items.len() > 0 {
            self.selected = Some(self.items[0].clone());
         } else {
            self.selected = None;
         }
      } else {
         // remove last item
         self.items.pop();
         if self.items.len() > 0 {
            self.selected = Some(self.items[0].clone());
         } else {
            self.selected = None;
         }
      }
   }
}

struct DemoApp {
   theme: Theme,
   editor: ThemeEditor,
   window_open: bool,
   modal_open: bool,
   confirm_password: bool,
   virtual_keyboard: bool,
   check: bool,
   slider: f32,
   items: Items,
   text_single: String,
   text_multi: String,
   text_password: String,
   credentials: CredentialsForm,
   secure_field: SecureInputField,
   qr: QrImage,
}

impl DemoApp {
   fn new(cc: &eframe::CreationContext<'_>) -> Self {
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&cc.egui_ctx);

      let mut credentials = CredentialsForm::new()
         .with_open(true)
         .with_confirm_password(true)
         .with_min_size(vec2(280.0, 20.0))
         .with_y_spacing(18.0);
      credentials.enable_virtual_keyboard();
      credentials.set_icon_size(vec2(20.0, 20.0));

      let mut secure_field = SecureInputField::new("Secret", true, true);
      secure_field.set_min_size(vec2(280.0, 20.0));

      Self {
         theme,
         editor: ThemeEditor::new(),
         window_open: false,
         modal_open: false,
         confirm_password: true,
         virtual_keyboard: true,
         check: true,
         slider: 42.0,
         items: Items::default(),
         text_single: String::from("Single-line text"),
         text_multi: String::from("Multiline\nSecureTextEdit"),
         text_password: String::from("hunter2"),
         credentials,
         secure_field,
         qr: QrImage::new(
            "egui-elements",
            "bytes://egui-elements-demo".to_string(),
         ),
      }
   }

   fn heading(&self, ui: &mut Ui, text: &str) {
      ui.label(
         RichText::new(text)
            .size(self.theme.typography.heading)
            .color(self.theme.colors.text)
            .strong(),
      );
   }

   fn subheading(&self, ui: &mut Ui, text: &str) {
      ui.label(
         RichText::new(text)
            .size(self.theme.typography.large)
            .color(self.theme.colors.text),
      );
   }

   fn text(&self, ui: &mut Ui, text: &str) {
      ui.label(
         RichText::new(text)
            .size(self.theme.typography.normal)
            .color(self.theme.colors.text),
      );
   }

   fn muted(&self, ui: &mut Ui, text: &str) {
      ui.label(
         RichText::new(text)
            .size(self.theme.typography.small)
            .color(self.theme.colors.text_muted),
      );
   }
}

impl eframe::App for DemoApp {
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

      self.show_window(ui);
      self.show_modal(ui);

      let bg = self.theme.colors.bg;
      CentralPanel::default()
         .frame(Frame::new().fill(bg).inner_margin(Margin::same(16)))
         .show(ui, |ui| {
            ScrollArea::vertical()
               .id_salt("main_scroll_area")
               .auto_shrink([false, false])
               .show(ui, |ui| {
                  ui.spacing_mut().item_spacing = vec2(16.0, 20.0);
                  ui.spacing_mut().button_padding = self.theme.button_padding;

                  self.palette_section(ui);
                  ui.add_space(28.0);
                  self.typography_section(ui);
                  ui.add_space(28.0);
                  self.frames_section(ui);
                  ui.add_space(28.0);
                  self.widgets_on_bg(ui);
                  ui.add_space(28.0);
                  self.widgets_on_frame1(ui);
                  ui.add_space(28.0);
                  self.components_section(ui);
                  ui.add_space(48.0);
               });
         });
   }
}

impl DemoApp {
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
         self.muted(ui, "Theme palette & widget showcase");

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

            ui.add_space(8.0);

            let window_btn = Button::new("Open Window").min_size(vec2(120.0, 28.0));
            if ui.add(window_btn).clicked() {
               if !self.window_open {
                  self.theme.overlay_manager.window_opened();
               }
               self.window_open = true;
            }

            ui.add_space(8.0);

            let modal_btn = Button::new("Open Modal").min_size(vec2(120.0, 28.0));
            if ui.add(modal_btn).clicked() {
               self.modal_open = true;
            }

            ui.add_space(12.0);

            if let Some(new_theme) = utils::theme_switcher(&self.theme, ui) {
               self.theme = new_theme;
            }
         });
      });
   }

   fn palette_section(&mut self, ui: &mut Ui) {
      self.heading(ui, "Color palette");
      self.muted(
         ui,
         "Every slot in ThemeColors. Edit live with Theme Editor.",
      );
      ui.add_space(8.0);

      let colors = [
         ("bg", self.theme.colors.bg),
         ("widget_bg", self.theme.colors.widget_bg),
         ("hover", self.theme.colors.hover),
         ("title_bar", self.theme.colors.title_bar),
         ("text", self.theme.colors.text),
         ("text_muted", self.theme.colors.text_muted),
         ("highlight", self.theme.colors.highlight),
         ("border", self.theme.colors.border),
         ("accent", self.theme.colors.accent),
         ("error", self.theme.colors.error),
         ("warning", self.theme.colors.warning),
         ("success", self.theme.colors.success),
         ("info", self.theme.colors.info),
      ];

      ScrollArea::horizontal()
         .id_salt("colors")
         .auto_shrink([false, true])
         .show(ui, |ui| {
            ui.set_height(100.0);
            ui.horizontal(|ui| {
               ui.spacing_mut().item_spacing.x = 16.0;
               for (name, color) in colors {
                  ui.vertical(|ui| {
                     ui.spacing_mut().item_spacing.y = 8.0;
                     ui.set_min_width(96.0);
                     let (rect, _) = ui.allocate_exact_size(vec2(96.0, 44.0), Sense::hover());
                     ui.painter().rect(
                        rect,
                        CornerRadius::same(self.theme.corner_radius),
                        color,
                        Stroke::new(1.0, self.theme.colors.border),
                        StrokeKind::Inside,
                     );
                     ui.label(
                        RichText::new(name)
                           .size(self.theme.typography.very_small)
                           .color(self.theme.colors.text),
                     );
                  });
               }
            });
         });
   }

   fn typography_section(&mut self, ui: &mut Ui) {
      self.heading(ui, "Typography");
      ui.add_space(8.0);
      let t = &self.theme.typography;
      let color = self.theme.colors.text;
      ui.vertical(|ui| {
         ui.spacing_mut().item_spacing.y = 10.0;
         ui.label(RichText::new("Heading").size(t.heading).color(color));
         ui.label(RichText::new("Very large").size(t.very_large).color(color));
         ui.label(RichText::new("Large").size(t.large).color(color));
         ui.label(RichText::new("Normal").size(t.normal).color(color));
         ui.label(RichText::new("Small").size(t.small).color(color));
         ui.label(RichText::new("Very small").size(t.very_small).color(color));
         ui.label(
            RichText::new("Muted / hint text")
               .size(t.normal)
               .color(self.theme.colors.text_muted),
         );
      });
   }

   fn frames_section(&mut self, ui: &mut Ui) {
      self.heading(ui, "Frames on main background");
      self.muted(
         ui,
         "frame1 is the major section container (widget_bg). frame2 is nested (bg fill).",
      );
      ui.add_space(8.0);

      let frame1 = self.theme.frame1;
      let mut frame2 = self.theme.frame2.outer_margin(Margin::ZERO);
      let frame2_visuals = self.theme.visuals.frame2_visuals;

      ScrollArea::horizontal()
         .id_salt("frames_section")
         .auto_shrink([false, true])
         .show(ui, |ui| {
            ui.horizontal(|ui| {
               ui.spacing_mut().item_spacing.x = 20.0;

               let large_t = self.theme.typography.large;
               let normal_t = self.theme.typography.normal;
               let muted_color = self.theme.colors.text_muted;

               ui.vertical(|ui| {
                  ui.set_width(360.0);

                  frame1.show(ui, |ui| {
                     ui.vertical(|ui| {
                        ui.set_width(ui.available_width());
                        ui.spacing_mut().item_spacing.y = 12.0;
                        self.subheading(ui, "Frame 1");
                        self.muted(ui, "Base container on bg");
                        ui.add_space(4.0);

                        self.text(ui, "Interactive nested list-item frames");

                        for i in 0..4 {
                           utils::frame(&mut frame2, frame2_visuals, ui, |ui| {
                              ui.vertical(|ui| {
                                 ui.set_width(ui.available_width());
                                 ui.spacing_mut().item_spacing.y = 6.0;

                                 let text = RichText::new(format!("Frame {}", i)).size(large_t);
                                 let label = Label::new(text, None).interactive(false);
                                 ui.add(label);

                                 let text = RichText::new("Nested list-item frame")
                                    .size(normal_t)
                                    .color(muted_color);
                                 let label = Label::new(text, None).interactive(false);
                                 ui.add(label);
                              });
                           });
                        }
                     });
                  });
               });

               ui.vertical(|ui| {
                  ui.set_width(520.0);
                  frame1.show(ui, |ui| {
                     ui.vertical(|ui| {
                        ui.set_width(ui.available_width());
                        ui.spacing_mut().item_spacing.y = 10.0;
                        self.subheading(ui, "Nested list");
                        self.muted(ui, "frame1 section with frame2 rows");
                        ui.add_space(4.0);
                        self.nested_list(ui);
                     });
                  });
               });
            });
         });
   }

   fn nested_list(&mut self, ui: &mut Ui) {
      let frame2 = self.theme.frame2.outer_margin(Margin::ZERO);
      let inner = self.theme.frame2.outer_margin(Margin::ZERO);
      let text_color = self.theme.colors.text;
      let size = self.theme.typography.normal;
      let small = self.theme.typography.small;

      let rows: [(&str, &str, Color32); 4] = [
         ("Inbox", "12 unread", self.theme.colors.info),
         ("Drafts", "2", self.theme.colors.text_muted),
         ("Sent", "ok", self.theme.colors.success),
         ("Spam", "blocked", self.theme.colors.warning),
      ];

      for (name, status, status_color) in rows {
         frame2.show(ui, |ui| {
            ui.vertical(|ui| {
               ui.set_width(ui.available_width());
               ui.horizontal(|ui| {
                  ui.add(
                     Label::new(
                        RichText::new(name).size(size).color(text_color),
                        None,
                     )
                     .expand(Some(4.0))
                     .interactive(false),
                  );
                  ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                     ui.label(RichText::new(status).size(small).color(status_color));
                  });
               });
            });
         });
      }

      ui.add_space(6.0);
      self.muted(ui, "Expanded row with nested frame2 items");
      frame2.show(ui, |ui| {
         ui.vertical(|ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 8.0;
            ui.horizontal(|ui| {
               ui.add(
                  Label::new(
                     RichText::new("Wallets").size(size).color(text_color),
                     None,
                  )
                  .interactive(false),
               );
               ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                  ui.label(RichText::new("2 accounts").size(small).color(self.theme.colors.info));
               });
            });

            let nested: [(&str, &str, Color32); 2] = [
               (
                  "GreekFetaCheese",
                  "$1,600.00",
                  self.theme.colors.success,
               ),
               (
                  "Cold storage",
                  "$12,400.00",
                  self.theme.colors.text,
               ),
            ];
            for (name, amount, amount_color) in nested {
               inner.show(ui, |ui| {
                  ui.horizontal(|ui| {
                     ui.set_width(ui.available_width());
                     ui.label(RichText::new(name).size(small).color(text_color));
                     ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(RichText::new(amount).size(small).color(amount_color));
                     });
                  });
               });
            }
         });
      });
   }

   fn widgets_on_bg(&mut self, ui: &mut Ui) {
      self.heading(ui, "Widgets on main background");
      self.muted(
         ui,
         "Custom widgets sitting directly on colors.bg — contrast against the canvas.",
      );
      ui.add_space(12.0);
      self.widget_gallery("bg", ui);
   }

   fn widgets_on_frame1(&mut self, ui: &mut Ui) {
      self.heading(ui, "Widgets on Frame 1");
      self.muted(
         ui,
         "Same widget set inside frame1 (widget_bg). Compare fills, borders, and hover.",
      );
      ui.add_space(12.0);
      let frame1 = self.theme.frame1;
      frame1.show(ui, |ui| {
         ui.vertical(|ui| {
            ui.set_width(ui.available_width());
            self.widget_gallery("frame1", ui);
         });
      });
   }

   fn widget_gallery(&mut self, id_salt: &str, ui: &mut Ui) {
      let text_color = self.theme.colors.text;
      let muted = self.theme.colors.text_muted;
      let size = self.theme.typography.normal;
      let button_size = vec2(120.0, 36.0);

      ui.spacing_mut().item_spacing = vec2(16.0, 16.0);

      ui.horizontal_wrapped(|ui| {
         ui.spacing_mut().item_spacing.x = 12.0;
         ui.spacing_mut().item_spacing.y = 12.0;

         let btn = Button::new(RichText::new("Small").size(size).color(text_color)).small();
         ui.add(btn);

         let btn =
            Button::new(RichText::new("Button").size(size).color(text_color)).min_size(button_size);
         ui.add(btn);

         let btn = Button::new(RichText::new("Selected").size(size).color(text_color))
            .selected(true)
            .min_size(button_size);
         ui.add(btn);
      });

      ui.add_space(8.0);

      ui.horizontal_wrapped(|ui| {
         ui.spacing_mut().item_spacing.x = 18.0;
         ui.spacing_mut().item_spacing.y = 12.0;
         ui.add(
            Label::new(
               RichText::new("Label (interactive)").size(size).color(text_color),
               None,
            )
            .expand(Some(6.0)),
         );
         ui.add(
            Label::new(
               RichText::new("Label (selected)").size(size).color(text_color),
               None,
            )
            .selected(true)
            .expand(Some(6.0)),
         );
         ui.add(
            Label::new(
               RichText::new("Label (static)").size(size).color(text_color),
               None,
            )
            .interactive(false),
         );
      });

      ui.add_space(8.0);

      ui.horizontal(|ui| {
         ui.spacing_mut().item_spacing.x = 12.0;

         let selected = Label::new(
            RichText::new(self.items.selected_str()).size(size).color(text_color),
            None,
         );

         ComboBox::new(format!("{id_salt}_items"), selected)
            .width(180.0)
            .label("Items")
            .show_ui(ui, |ui| {
               ui.spacing_mut().item_spacing.y = 12.0;

               for item in self.items.all() {
                  let label = Label::new(
                     RichText::new(&item).size(size).color(text_color),
                     None,
                  )
                  .expand(Some(4.0))
                  .selected(self.items.selected == Some(item.clone()))
                  .sense(Sense::click())
                  .fill_width(true);

                  if ui.add(label).clicked() {
                     self.items.selected = Some(item);
                  }
               }
            });

         let add_btn = Button::new(RichText::new("Add").size(size).color(text_color))
            .min_size(vec2(20.0, 20.0));

         if ui.add(add_btn).clicked() {
            self.items.add();
         }

         let remove_btn = Button::new(RichText::new("Remove").size(size).color(text_color))
            .min_size(vec2(20.0, 20.0));

         if ui.add(remove_btn).clicked() {
            self.items.remove();
         }
      });

      ui.add_space(8.0);

      ui.horizontal(|ui| {
         ui.spacing_mut().item_spacing.x = 16.0;
         ui.checkbox(
            &mut self.check,
            RichText::new("Checkbox").size(size).color(text_color),
         );
         ui.radio_value(
            &mut self.check,
            true,
            RichText::new("Radio").size(size).color(text_color),
         );

         ui.spacing_mut().item_spacing.x = 3.0;
         ui.spacing_mut().button_padding = vec2(0.0, 0.0);

         ui.add(Slider::new(&mut self.slider, 0.0..=100.0).text("Slider"));
      });

      ui.add_space(8.0);

      ui.horizontal_wrapped(|ui| {
         ui.spacing_mut().item_spacing.x = 20.0;
         ui.spacing_mut().item_spacing.y = 16.0;
         ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 8.0;
            ui.label(RichText::new("SecureTextEdit").size(size).color(text_color));
            ui.add(
               SecureTextEdit::singleline(&mut self.text_single)
                  .id_salt(format!("{id_salt}_single"))
                  .hint_text(RichText::new("Hint / muted").color(muted))
                  .margin(Margin::same(8))
                  .desired_width(220.0)
                  .font(FontId::proportional(size)),
            );
         });
         ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 8.0;
            ui.label(RichText::new("Password").size(size).color(text_color));
            ui.add(
               SecureTextEdit::singleline(&mut self.text_password)
                  .id_salt(format!("{id_salt}_password"))
                  .password(true)
                  .margin(Margin::same(8))
                  .desired_width(180.0)
                  .font(FontId::proportional(size)),
            );
         });
         ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 8.0;
            ui.label(RichText::new("Multiline").size(size).color(text_color));
            ui.add(
               SecureTextEdit::multiline(&mut self.text_multi)
                  .id_salt(format!("{id_salt}_multi"))
                  .margin(Margin::same(8))
                  .desired_width(220.0)
                  .desired_rows(3)
                  .font(FontId::proportional(size)),
            );
         });
      });
   }

   fn components_section(&mut self, ui: &mut Ui) {
      self.heading(ui, "Components");
      self.muted(
         ui,
         "CredentialsForm, SecureInputField, VirtualKeyboard, QrImage, and QR scanner (Linux).",
      );
      ui.add_space(8.0);

      let frame1 = self.theme.frame1;
      let form_width = 640.0;

      ui.horizontal(|ui| {
         ui.add_space(((ui.available_width() - form_width) * 0.5).max(0.0));
         ui.vertical(|ui| {
            ui.set_width(form_width);
            frame1.show(ui, |ui| {
               ui.set_width(ui.available_width());
               ui.vertical_centered(|ui| {
                  ui.spacing_mut().item_spacing.y = 14.0;
                  ui.spacing_mut().button_padding = vec2(4.0, 4.0);

                  self.subheading(ui, "Credentials form");
                  ui.horizontal(|ui| {
                     ui.spacing_mut().item_spacing.x = 16.0;
                     if ui.checkbox(&mut self.confirm_password, "Confirm password").changed() {
                        self.credentials.set_confirm_password(self.confirm_password);
                     }
                     if ui.checkbox(&mut self.virtual_keyboard, "Virtual keyboard").changed() {
                        if self.virtual_keyboard {
                           self.credentials.enable_virtual_keyboard();
                        } else {
                           self.credentials.disable_virtual_keyboard();
                        }
                     }
                  });
                  ui.add_space(8.0);
                  self.credentials.show(ui);
               });
            });
         });
      });

      ui.add_space(20.0);

      ui.horizontal(|ui| {
         ui.add_space(((ui.available_width() - form_width) * 0.5).max(0.0));
         ui.vertical(|ui| {
            ui.set_width(form_width);
            frame1.show(ui, |ui| {
               ui.set_width(ui.available_width());
               ui.vertical_centered(|ui| {
                  ui.spacing_mut().item_spacing.y = 12.0;
                  ui.spacing_mut().button_padding = vec2(4.0, 4.0);

                  self.subheading(ui, "Secure input + QR");
                  self.muted(
                     ui,
                     "Toggle visibility, QR scan button available only on Linux.",
                  );

                  let output = self.secure_field.show(ui);
                  self.subheading(ui, "QrImage");

                  if let Some(output) = output {
                     if output.response.changed() {
                        self.qr.clear(ui.ctx());
                        let uri = "bytes://egui-elements-demo".to_string();
                        let text = self.secure_field.text();

                        let data = text.unlock_str(|s| s.to_string());
                        let new_qr = QrImage::new(&data, uri);
                        self.qr = new_qr;
                     }
                  }

                  if self.qr.has_error() {
                     ui.label(
                        RichText::new(self.qr.error().map(|e| e.to_string()).unwrap_or_default())
                           .color(self.theme.colors.error),
                     );
                  } else {
                     ui.add(self.qr.image().fit_to_exact_size(vec2(140.0, 140.0)));
                  }
               });
            });
         });
      });
   }

   fn show_window(&mut self, ui: &mut Ui) {
      if !self.window_open {
         return;
      }

      let frame = self.theme.window_frame;
      let title_frame = frame.stroke(Stroke::NONE);
      let mut open = self.window_open;
      Window::new("Widgets on a Window")
         .open(&mut open)
         .resizable(true)
         .collapsible(false)
         .title_frame(title_frame)
         .frame(frame)
         .default_size(vec2(640.0, 520.0))
         .show(ui.ctx(), |ui| {
            ui.set_min_width(560.0);
            ui.spacing_mut().button_padding = self.theme.button_padding;

            self.muted(
               ui,
               "window_frame chrome. Same widget gallery as the canvas / frame1 sections.",
            );
            ui.add_space(8.0);
            self.widget_gallery("window", ui);
         });

      if open != self.window_open {
         if !open {
            self.theme.overlay_manager.window_closed();
         }
         self.window_open = open;
      }
   }

   fn show_modal(&mut self, ui: &mut Ui) {
      let mut open = self.modal_open;
      let mut close = false;

      Modal::new("demo_modal", &mut open)
         .heading("Modal")
         .subtitle("Themed card over a dimmed backdrop. Esc / backdrop / × to dismiss.")
         .header_icon("✦")
         .max_width(460.0)
         .footer(|ui| {
            let done = Button::new("Done").min_size(vec2(88.0, 32.0));
            if ui.add(done).clicked() {
               close = true;
            }
         })
         .show(ui.ctx(), |ui| {
            ui.label(
               RichText::new(
                  "Use Modal for confirmations and blocking flows. Footer is right-to-left.",
               )
               .size(self.theme.typography.normal)
               .color(self.theme.colors.text),
            );
            ui.add_space(8.0);
            ui.label(
               RichText::new("Accent / success / warning / error all come from the theme.")
                  .size(self.theme.typography.small)
                  .color(self.theme.colors.text_muted),
            );
         });

      if close {
         open = false;
      }
      self.modal_open = open;
   }
}
