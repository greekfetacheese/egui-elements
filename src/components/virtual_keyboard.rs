use crate::theme::Theme;
use crate::widgets::Button;
use egui::*;

use secure_types::SecureString;

/// Which credentials row a [`VirtualKeyboard`] is typing into.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InputField {
   Username,
   Password,
   ConfirmPassword,
}

const DEFAULT_KEY_SIZE: Vec2 = vec2(30.0, 30.0);
const DEFAULT_BUTTON_PADDING: Vec2 = vec2(4.0, 4.0);
const DEFAULT_SPACE_WIDTH: f32 = 280.0;
const DEFAULT_TEXT_SIZE: f32 = 16.0;
const DEFAULT_SPACING: Vec2 = vec2(10.0, 10.0);
const DEFAULT_MAX_UI_SIZE: Vec2 = vec2(600.0, 200.0);

/// A virtual keyboard that can be used to edit an input field.
pub struct VirtualKeyboard {
   open: bool,
   active_target: InputField,
   shift_active: bool,
   caps_lock_active: bool,
   key_size: Vec2,
   max_ui_size: Vec2,
   button_padding: Vec2,
   spacing: Vec2,
   text_size: f32,
}

impl VirtualKeyboard {
   /// Create a keyboard. Pass `true` to show it immediately.
   pub fn new(open: bool) -> Self {
      Self {
         open,
         active_target: InputField::Username,
         shift_active: false,
         caps_lock_active: false,
         key_size: DEFAULT_KEY_SIZE,
         max_ui_size: DEFAULT_MAX_UI_SIZE,
         button_padding: DEFAULT_BUTTON_PADDING,
         text_size: DEFAULT_TEXT_SIZE,
         spacing: DEFAULT_SPACING,
      }
   }

   /// Show the keyboard on the next [`Self::show`].
   pub fn open(&mut self) {
      self.open = true;
   }

   /// Whether the keyboard is currently shown.
   pub fn is_open(&self) -> bool {
      self.open
   }

   /// Hide the keyboard.
   pub fn close(&mut self) {
      self.open = false;
   }

   /// Direct keystrokes at a different credentials row.
   pub fn set_active_target(&mut self, target: InputField) {
      self.active_target = target;
   }

   /// Minimum size of each key button. Default is `[DEFAULT_KEY_SIZE].
   pub fn with_key_size(mut self, size: Vec2) -> Self {
      self.key_size = size;
      self
   }

   /// Minimum size of each key button. Default is `[DEFAULT_KEY_SIZE].
   pub fn set_key_size(&mut self, size: Vec2) {
      self.key_size = size;
   }

   /// Current minimum size of each key button.
   pub fn key_size(&self) -> Vec2 {
      self.key_size
   }

   /// Text size for each key button. Default is `[DEFAULT_TEXT_SIZE].
   pub fn with_text_size(mut self, size: f32) -> Self {
      self.text_size = size;
      self
   }

   /// Text size for each key button. Default is `[DEFAULT_TEXT_SIZE].
   pub fn set_text_size(&mut self, size: f32) {
      self.text_size = size;
   }

   /// Spacing between each key button. Default is `[DEFAULT_SPACING].
   pub fn with_spacing(mut self, spacing: Vec2) -> Self {
      self.spacing = spacing;
      self
   }

   /// Spacing between each key button. Default is `[DEFAULT_SPACING].
   pub fn set_spacing(&mut self, spacing: Vec2) {
      self.spacing = spacing;
   }

   /// Maximum size of the virtual keyboard. Default is `[DEFAULT_MAX_UI_SIZE].
   pub fn with_max_ui_size(mut self, size: Vec2) -> Self {
      self.max_ui_size = size;
      self
   }

   /// Maximum size of the virtual keyboard. Default is `[DEFAULT_MAX_UI_SIZE].
   pub fn set_max_ui_size(&mut self, size: Vec2) {
      self.max_ui_size = size;
   }

   /// Current maximum size of the virtual keyboard.
   pub fn max_ui_size(&self) -> Vec2 {
      self.max_ui_size
   }

   /// Current text size for each key button.
   pub fn text_size(&self) -> f32 {
      self.text_size
   }

   /// Inner padding of each key button. Default is `[DEFAULT_BUTTON_PADDING].
   pub fn with_button_padding(mut self, padding: Vec2) -> Self {
      self.button_padding = padding;
      self
   }

   /// Inner padding of each key button. Default is `[DEFAULT_BUTTON_PADDING].
   pub fn set_button_padding(&mut self, padding: Vec2) {
      self.button_padding = padding;
   }

   /// Current inner padding of each key button.
   pub fn button_padding(&self) -> Vec2 {
      self.button_padding
   }

   /// Paint the keyboard and write pressed keys into `target_str`.
   pub fn show(&mut self, target_str: &mut SecureString, ui: &mut Ui) {
      if !self.open {
         return;
      }

      // Define the keyboard layout
      let keys_layout_lower = vec![
         vec![
            "`",
            "1",
            "2",
            "3",
            "4",
            "5",
            "6",
            "7",
            "8",
            "9",
            "0",
            "-",
            "=",
            "Backspace",
         ],
         vec![
            "q", "w", "e", "r", "t", "y", "u", "i", "o", "p", "[", "]", "\\",
         ],
         vec![
            "Caps", "a", "s", "d", "f", "g", "h", "j", "k", "l", ";", "'", "Enter",
         ],
         vec![
            "Shift", "z", "x", "c", "v", "b", "n", "m", ",", ".", "/", "Shift",
         ],
      ];
      let keys_layout_upper = vec![
         vec![
            "~",
            "!",
            "@",
            "#",
            "$",
            "%",
            "^",
            "&",
            "*",
            "(",
            ")",
            "_",
            "+",
            "Backspace",
         ],
         vec![
            "Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P", "{", "}", "|",
         ],
         vec![
            "Caps", "A", "S", "D", "F", "G", "H", "J", "K", "L", ":", "\"", "Enter",
         ],
         vec![
            "Shift", "Z", "X", "C", "V", "B", "N", "M", "<", ">", "?", "Shift",
         ],
      ];

      let theme = Theme::current(ui.ctx());
      let frame = theme.frame2.stroke(Stroke::new(1.0, theme.colors.border));

      let is_uppercase = self.shift_active ^ self.caps_lock_active;
      let layout = if is_uppercase {
         &keys_layout_upper
      } else {
         &keys_layout_lower
      };

      let ui_size = self.max_ui_size();

      ui.vertical_centered(|ui| {
         frame.show(ui, |ui| {
            ui.spacing_mut().button_padding = self.button_padding;
            ui.spacing_mut().item_spacing = self.spacing;
            ui.set_max_size(self.max_ui_size);

            for row in layout {
               ui.horizontal(|ui| {
                  ui.set_max_width(ui_size.x);
                  for &key in row {
                     let text = RichText::new(key).size(self.text_size);
                     let key_button = Button::new(text).min_size(self.key_size);
                     if ui.add(key_button).clicked() {
                        self.handle_key_press(key, target_str);
                     }
                  }
               });
            }

            let space_scale = self.key_size.x / DEFAULT_KEY_SIZE.x;
            let space_width = DEFAULT_SPACE_WIDTH * space_scale;
            let button = Button::new(" ").min_size(vec2(space_width, self.key_size.y));
            if ui.add(button).clicked() {
               target_str.push_str(" ");
            }
         });
      });
   }

   fn handle_key_press(&mut self, key: &str, target: &mut SecureString) {
      match key {
         "Backspace" => {
            target.secure_mut(|s| {
               let len = s.char_len();
               if len > 0 {
                  s.delete_text_char_range(len - 1..len);
               }
            });
         }
         "Shift" => {
            self.shift_active = !self.shift_active;
         }
         "Caps" => {
            self.caps_lock_active = !self.caps_lock_active;
            self.shift_active = false; // Typically, pressing Caps disables Shift
         }
         "Enter" => {
            // For now, we do nothing.
         }
         _ => {
            target.push_str(key);
            // Deactivate shift after a character press
            if self.shift_active {
               self.shift_active = false;
            }
         }
      }
   }
}
