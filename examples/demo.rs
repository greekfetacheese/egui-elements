//! Showcase for `egui-elements`: palette combinations, custom widgets,
//! components, a theme switcher, and the live theme editor.
//!
//! Run with: `cargo run --example demo --features full`

use std::time::Instant;

use eframe::egui::*;
use egui_elements::components::{CredentialsForm, QrEncoding, QrImage, SecureInputField};
use egui_elements::editor::ThemeEditor;
use egui_elements::egui_lucide::Lucide;
use egui_elements::gradient::{Fill, GradientDir};
use egui_elements::theme::{Theme, ThemeKind, ThemeVariant};
use egui_elements::utils;
use egui_elements::widgets::{
   Avatar, AvatarGroup, AvatarVisuals, Badge, BadgeCorner, BadgePulse, Button, Checkbox, ChipRole,
   ComboBox, FpsMeter, Indicator, IndicatorState, Label, Modal, Orb, Radio, Ripple, SecureTextEdit,
   Slider, Spinner, Switch, Tabs, TabsStyle, Toast, Toasts, Window,
};

/// Display budget for the demo's QR symbol. The raster is built for this size and
/// drawn at its natural size, so a scanner sees crisp module edges.
const QR_DISPLAY_PX: u32 = 140;

fn main() -> eframe::Result {
   let options = eframe::NativeOptions {
      viewport: ViewportBuilder::default()
         .with_title("egui-elements demo")
         .with_inner_size([1180.0, 820.0])
         .with_resizable(true)
         // The Linux QR scanner is a fullscreen transparent overlay that shows the
         // desktop through itself. eframe's glow backend builds one GL config for
         // the whole process from this flag, so without it the overlay is an opaque
         // 24-bit window and the scanner only ever sees itself. Panels paint every
         // pixel, so an ARGB root window looks unchanged.
         .with_transparent(true),
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
   vk_key_width: f32,
   vk_key_height: f32,
   vk_text_size: f32,
   vk_button_padding_x: f32,
   vk_button_padding_y: f32,
   check: bool,
   check_on: bool,
   check_off: bool,
   switch_on: bool,
   switch_off: bool,
   slider_value: f32,
   radio: u8,
   tabs: usize,
   slider: f32,
   items: Items,
   text_single: String,
   text_multi: String,
   text_password: String,
   credentials: CredentialsForm,
   secure_field: SecureInputField,
   qr: QrImage,
   fps: FpsMeter,
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

      let mut editor = ThemeEditor::new();
      editor.set_io(
         |theme| {
            let Some(path) = rfd::FileDialog::new()
               .add_filter("egui-elements theme", &["json"])
               .set_file_name(format!(
                  "{}.json",
                  theme.kind.to_str().replace(' ', "-").to_lowercase()
               ))
               .save_file()
            else {
               return Ok(None);
            };

            let json = serde_json::to_string_pretty(theme).map_err(|error| error.to_string())?;
            std::fs::write(&path, json).map_err(|error| error.to_string())?;

            Ok(Some(format!("Saved {}", path.display())))
         },
         || {
            let Some(path) =
               rfd::FileDialog::new().add_filter("egui-elements theme", &["json"]).pick_file()
            else {
               return Ok(None);
            };

            let json = std::fs::read_to_string(&path).map_err(|error| error.to_string())?;
            let theme: Theme = serde_json::from_str(&json).map_err(|error| error.to_string())?;

            Ok(Some(theme))
         },
      );

      Self {
         theme,
         editor,
         window_open: false,
         modal_open: false,
         confirm_password: true,
         virtual_keyboard: true,
         vk_key_width: 30.0,
         vk_key_height: 30.0,
         vk_text_size: 16.0,
         vk_button_padding_x: 4.0,
         vk_button_padding_y: 4.0,
         check: true,
         check_on: true,
         check_off: false,
         switch_on: true,
         switch_off: false,
         slider_value: 40.0,
         radio: 0,
         tabs: 0,
         slider: 42.0,
         items: Items::default(),
         text_single: String::from("Single-line text"),
         text_multi: String::from("Multiline\nSecureTextEdit"),
         text_password: String::from("hunter2"),
         credentials,
         secure_field,
         qr: QrImage::with_encoding(
            "egui-elements",
            "bytes://egui-elements-demo".to_string(),
            QrEncoding {
               target_px: QR_DISPLAY_PX,
               ..Default::default()
            },
         ),
         fps: FpsMeter::new().open(false),
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
   /// eframe clears every viewport with a 71 %-alpha dark grey by default, and
   /// the Linux QR overlay is part of the screenshot: that film is baked into
   /// what the scanner decodes. The panels paint their own opaque background, so
   /// a transparent clear only reaches the overlay.
   fn clear_color(&self, _visuals: &Visuals) -> [f32; 4] {
      [0.0, 0.0, 0.0, 0.0]
   }

   fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
      let start = Instant::now();

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
         .frame(Frame::new().fill(bg).inner_margin(Margin::same(10)))
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
                  self.gradients_section(ui);
                  ui.add_space(28.0);
                  self.widgets_on_bg(ui);
                  ui.add_space(28.0);
                  self.widgets_on_frame1(ui);
                  ui.add_space(28.0);
                  self.components_section(ui);
                  ui.add_space(48.0);
               });
         });

      // The meter measures this demo's own frame: tick once with the host
      // clock, then paint the floating window (a no-op while it is closed).
      self.fps.frame(start.elapsed());
      if self.fps.show_window(ui.ctx()).is_some() {
         // The meter never schedules frames; keep the demo live while its
         // window is open.
         ui.ctx().request_repaint();
      }

      // The toast stack is an overlay (Order::Tooltip): render it after every panel
      // so it floats above them, whichever surface fired it.
      Toasts::new().render(ui.ctx());
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
            let fps_open = self.fps.is_open();
            let fps_text = if fps_open { "Hide FPS" } else { "Show FPS" };
            if ui.add(Button::new(fps_text).min_size(vec2(96.0, 28.0))).clicked() {
               self.fps.set_open(!fps_open);
            }

            ui.add_space(8.0);

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

   fn gradients_section(&mut self, ui: &mut Ui) {
      self.heading(ui, "Gradient fills");
      self.muted(
         ui,
         "Fill::Gradient paints a two-stop mesh. Theme::variant = Gradient turns every themed \
          fill into a soft bevel of its own colour, clamped so the ink keeps AA contrast.",
      );
      ui.add_space(8.0);

      ui.horizontal(|ui| {
         ui.label(RichText::new("Variant").color(self.theme.colors.text_muted));

         for variant in ThemeVariant::to_vec() {
            let picked =
               ui.selectable_label(self.theme.variant == variant, variant.to_str()).clicked();

            if picked {
               self.theme.set_variant(variant);
            }
         }

         if self.theme.variant == ThemeVariant::Gradient {
            ui.add(Slider::new(&mut self.theme.gradient_strength, 0.0..=1.0).text("Bevel depth"));

            ui.label(RichText::new("Direction").color(self.theme.colors.text_muted));

            for dir in GradientDir::ALL {
               let picked =
                  ui.selectable_label(self.theme.gradient_dir == dir, dir.to_str()).clicked();

               if picked {
                  self.theme.gradient_dir = dir;
               }
            }
         }
      });

      ui.add_space(8.0);

      // Two uigradients pairs and a near-black one: the ramps the primitive is for.
      let swatches = [
         (
            "Visual Blue",
            Fill::horizontal(
               Color32::from_rgb(0x00, 0xc9, 0x96),
               Color32::from_rgb(0x00, 0x3d, 0x4d),
            ),
         ),
         (
            "Coffee Gold",
            Fill::horizontal(
               Color32::from_rgb(0xc9, 0x98, 0x46),
               Color32::from_rgb(0x55, 0x40, 0x23),
            ),
         ),
         (
            "Moonwalker",
            Fill::horizontal(
               Color32::from_rgb(0x00, 0x00, 0x00),
               Color32::from_rgb(0x15, 0x23, 0x31),
            ),
         ),
      ];

      ui.horizontal(|ui| {
         ui.spacing_mut().item_spacing.x = 16.0;

         for (name, fill) in swatches {
            ui.vertical(|ui| {
               fill.show_rounded(ui, vec2(180.0, 64.0), 8);
               ui.label(RichText::new(name).color(self.theme.colors.text_muted));
            });
         }
      });

      ui.add_space(12.0);

      // Real chrome under the same variant, so the clamp is visible on text-bearing fills.
      let button_visuals = self.theme.button_visuals();
      let label_visuals = self.theme.label_visuals();

      ui.horizontal(|ui| {
         ui.add(Button::new("Idle").visuals(button_visuals));
         ui.add(Button::new("Selected").selected(true).visuals(button_visuals));
         ui.add(Label::new("Interactive label", None).interactive(true).visuals(label_visuals));
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
         ui.spacing_mut().item_spacing.x = 12.0;
         ui.spacing_mut().item_spacing.y = 12.0;

         for corner in BadgeCorner::to_vec() {
            let badge = Badge::new("3").color(self.theme.colors.error).corner(*corner);
            let btn = Button::new(RichText::new(corner.to_str()).size(size).color(text_color))
               .badge(badge)
               .min_size(button_size);

            ui.scope(|ui| {
               // Give more space for the badge
               ui.spacing_mut().button_padding.x += 12.0;
               ui.add(btn);
            });
         }
      });

      ui.add_space(8.0);

      ui.horizontal_wrapped(|ui| {
         ui.spacing_mut().item_spacing.x = 12.0;
         ui.spacing_mut().item_spacing.y = 12.0;

         let colors = self.theme.colors;
         let chips = [
            ("Jedi", "0", colors.success),
            ("Sith", "2", colors.error),
            ("Chosen", "1", colors.info),
            ("Episodes", "6", colors.warning),
         ];

         for (label, count, fill) in chips {
            let badge = Badge::new(count).color(fill).straddle();
            ui.add(
               Button::new(RichText::new(label).size(size).color(text_color))
                  .badge(badge)
                  .min_size(button_size),
            );
         }

         // One pulsing badge, so the gallery shows the halo on every surface.
         ui.add(
            Button::new(RichText::new("Pulse").size(size).color(text_color))
               .badge(Badge::new("3").color(colors.error).straddle().pulse(BadgePulse::new()))
               .min_size(button_size),
         );
      });

      ui.add_space(8.0);

      ui.horizontal_wrapped(|ui| {
         ui.spacing_mut().item_spacing.x = 12.0;
         ui.spacing_mut().item_spacing.y = 12.0;

         // Every surface can fire the stack; the description names the surface, so a
         // toast fired from the window gallery is identifiable. The renderer itself is
         // global and runs once per frame at the end of `ui()`.
         let tones = [
            ("Success", ChipRole::Success),
            ("Error", ChipRole::Error),
            ("Warning", ChipRole::Warning),
            ("Info", ChipRole::Info),
         ];

         for (label, tone) in tones {
            let button =
               Button::new(RichText::new(label).size(size).color(text_color)).min_size(button_size);

            if ui.add(button).clicked() {
               Toast::new(format!("{label} toast"))
                  .tone(tone)
                  .description(format!("Fired from the {id_salt} gallery"))
                  .show(ui.ctx());
            }
         }
      });

      ui.add_space(8.0);

      ui.horizontal_wrapped(|ui| {
         ui.spacing_mut().item_spacing.x = 18.0;
         ui.spacing_mut().item_spacing.y = 12.0;

         // The ring has to match the surface the stack sits on, or the overlap reads as a gap.
         let ring = if id_salt == "bg" {
            self.theme.colors.bg
         } else {
            self.theme.colors.widget_bg
         };
         let visuals = AvatarVisuals {
            ring,
            ..AvatarVisuals::from_theme(self.theme.colors)
         };
         let stack = [
            Avatar::initials("A"),
            Avatar::initials("S"),
            Avatar::initials("B"),
            Avatar::initials("M"),
         ];
         ui.add(AvatarGroup::new(&stack).size(32.0).visuals(visuals));
      });

      ui.add_space(8.0);

      ui.horizontal_wrapped(|ui| {
         ui.spacing_mut().item_spacing.x = 12.0;
         ui.spacing_mut().item_spacing.y = 12.0;

         for size in [20.0, 32.0, 44.0] {
            ui.add(Spinner::new().size(size));
         }
         for size in [26.0, 40.0] {
            ui.add(Ripple::new().size(size));
         }
         ui.add(Orb::new().size(40.0));
         ui.add(Orb::new().size(64.0).text("Loading"));
      });

      ui.add_space(8.0);

      ui.horizontal_wrapped(|ui| {
         ui.spacing_mut().item_spacing.x = 18.0;
         ui.spacing_mut().item_spacing.y = 12.0;
         ui.add(Checkbox::new(
            &mut self.check_on,
            RichText::new("Checkbox (themed)").size(size).color(text_color),
         ));
         ui.add(Checkbox::new(
            &mut self.check_off,
            RichText::new("Unchecked").size(size).color(text_color),
         ));
         ui.add(Checkbox::without_text(&mut self.check_off));
      });

      ui.add_space(8.0);

      ui.horizontal_wrapped(|ui| {
         ui.spacing_mut().item_spacing.x = 18.0;
         ui.spacing_mut().item_spacing.y = 12.0;
         ui.add(Switch::new(
            &mut self.switch_on,
            RichText::new("Switch (themed)").size(size).color(text_color),
         ));
         ui.add(Switch::new(
            &mut self.switch_off,
            RichText::new("Off").size(size).color(text_color),
         ));
         ui.add(Switch::without_text(&mut self.switch_off));
      });

      ui.add_space(8.0);

      ui.horizontal_wrapped(|ui| {
         ui.spacing_mut().item_spacing.x = 18.0;
         ui.spacing_mut().item_spacing.y = 12.0;
         ui.add(
            Slider::new(&mut self.slider_value, 0.0..=100.0)
               .text("Slider (themed)")
               .width(240.0)
               .step_by(10.0)
               .markers(true)
               .tooltip(true),
         );
      });

      ui.add_space(8.0);

      ui.horizontal_wrapped(|ui| {
         ui.spacing_mut().item_spacing.x = 18.0;
         ui.spacing_mut().item_spacing.y = 12.0;

         // The pulse reuses the badge's loop, phased from the light's own id.
         ui.add(Indicator::new(IndicatorState::On).size(12.0));
         ui.add(Indicator::new(IndicatorState::On).size(12.0).pulse(BadgePulse::new()));
         ui.add(Indicator::new(IndicatorState::Connecting).size(12.0));
         ui.add(Indicator::new(IndicatorState::Off).size(12.0));
      });

      ui.add_space(8.0);

      ui.horizontal_wrapped(|ui| {
         ui.spacing_mut().item_spacing.x = 18.0;
         ui.spacing_mut().item_spacing.y = 12.0;
         // Every surface carries the strip, so the track and the lift are checked on bg, frame1 and
         // the window: the id salt keeps their indicators from colliding.
         Tabs::new(
            format!("{id_salt}_tabs"),
            &mut self.tabs,
            ["All", "Media", "Files"],
         )
         .fill_width(false)
         .show(ui);
         Tabs::new(
            format!("{id_salt}_tabs_underlined"),
            &mut self.tabs,
            ["All", "Media", "Files"],
         )
         .style(TabsStyle::Underline)
         .fill_width(false)
         .show(ui);
      });

      ui.add_space(8.0);

      ui.horizontal_wrapped(|ui| {
         ui.spacing_mut().item_spacing.x = 18.0;
         ui.spacing_mut().item_spacing.y = 12.0;
         ui.add(Radio::new(
            &mut self.radio,
            0u8,
            RichText::new("Radio (themed)").size(size).color(text_color),
         ));
         ui.add(Radio::new(
            &mut self.radio,
            1u8,
            RichText::new("Second option").size(size).color(text_color),
         ));
         ui.add(Radio::without_text(&mut self.radio, 2u8));
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
            .interactive(true)
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
         "CredentialsForm, SecureInputField, VirtualKeyboard, QrImage, and QR scanner (Linux/Windows).",
      );
      ui.add_space(8.0);

      let frame1 = self.theme.frame1;
      let form_width = 640.0;

      ui.vertical_centered(|ui| {
         frame1.show(ui, |ui| {
            ui.set_width(form_width);

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

               if self.virtual_keyboard {
                  ui.horizontal(|ui| {
                     ui.spacing_mut().item_spacing.x = 16.0;
                     ui.add(Slider::new(&mut self.vk_key_width, 16.0..=72.0).text("Key width"));
                     ui.add(Slider::new(&mut self.vk_key_height, 16.0..=72.0).text("Key height"));
                     ui.add(Slider::new(&mut self.vk_text_size, 8.0..=32.0).text("Text size"));
                  });
                  ui.horizontal(|ui| {
                     ui.spacing_mut().item_spacing.x = 16.0;
                     ui.add(
                        Slider::new(&mut self.vk_button_padding_x, 0.0..=16.0)
                           .text("Button padding X"),
                     );
                     ui.add(
                        Slider::new(&mut self.vk_button_padding_y, 0.0..=16.0)
                           .text("Button padding Y"),
                     );
                  });
                  self
                     .credentials
                     .set_virtual_keyboard_key_size(vec2(self.vk_key_width, self.vk_key_height));
                  self.credentials.set_virtual_keyboard_button_padding(vec2(
                     self.vk_button_padding_x,
                     self.vk_button_padding_y,
                  ));
                  self.credentials.set_virtual_keyboard_text_size(self.vk_text_size);
               }

               ui.add_space(8.0);

               self.credentials.show(ui);
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
                     "Toggle visibility, QR scan button available on Linux and Windows.",
                  );

                  let output = self.secure_field.show(ui);
                  self.subheading(ui, "QrImage");

                  if let Some(output) = output {
                     if output.response.changed() {
                        self.qr.clear(ui.ctx());
                        let uri = "bytes://egui-elements-demo".to_string();
                        let text = self.secure_field.text();

                        let data = text.unlock_str(|s| s.to_string());
                        let new_qr = QrImage::with_encoding(
                           &data,
                           uri,
                           QrEncoding {
                              target_px: QR_DISPLAY_PX,
                              ..Default::default()
                           },
                        );
                        self.qr = new_qr;
                     }
                  }

                  if self.qr.has_error() {
                     ui.label(
                        RichText::new(self.qr.error().map(|e| e.to_string()).unwrap_or_default())
                           .color(self.theme.colors.error),
                     );
                  } else {
                     // Draw at the raster's own size: the symbol is built from
                     // whole modules, so this needs no resampling and the module
                     // edges stay sharp for a scanner.
                     let size = self.qr.image_size_pt(ui.ctx().pixels_per_point());
                     ui.add(self.qr.image().fit_to_exact_size(size));
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
         self.window_open = open;
      }
   }

   fn show_modal(&mut self, ui: &mut Ui) {
      let mut open = self.modal_open;

      let theme = Theme::current(ui.ctx());
      let frame = theme.window_frame.fill(theme.frame1.fill);
      let title = RichText::new("Verify Credentials").size(theme.typography.heading);
      let icon = Lucide::ShieldCheck.size(28.0).color(theme.colors.accent).image();

      Modal::new("demo_modal", &mut open)
         .heading(title)
         .center_header(true)
         .center_header_icon(false)
         .header_separator(false)
         .header_icon(icon)
         .frame(frame)
         .show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
               ui.set_min_size(vec2(550.0, 350.0));

               ui.scope(|ui| {
                  ui.spacing_mut().button_padding = vec2(4.0, 4.0);
                  self.credentials.set_confirm_password(false);
                  self.credentials.show(ui);
               });

               let text = RichText::new("Confirm").size(self.theme.typography.large);
               let button = Button::new(text).min_size(vec2(200.0, 30.0));

               if ui.add(button).clicked() {}
            });
         });
      self.modal_open = open;
   }
}
