use crate::theme::Theme;
use crate::widgets::{
   Button,
   secure_text_edit::{SecureTextEdit, SecureTextEditOutput, TextBuffer},
};
use egui::*;
use egui_lucide::Lucide;

#[cfg(all(
   feature = "qr-scanner",
   any(target_os = "linux", target_os = "windows")
))]
use super::QRScanner;

use secure_types::SecureString;

/// A secure input field that can be used to edit a text containing sensitive information.
///
/// Show/hide uses Lucide `Eye` / `EyeOff`; the QR button (Linux/Windows + `qr-scanner`)
/// uses `ScanQrCode`. Both are colored with [`crate::theme::ThemeColors::text`].
///
/// The host app must register egui's SVG loader once:
/// `egui_extras::install_image_loaders(ctx)` with the `egui_extras` `svg` feature.
#[derive(Clone)]
pub struct SecureInputField {
   open: bool,
   pub(crate) text: SecureString,
   text_hidden: bool,
   id: &'static str,
   icon_size: Vec2,
   min_size: Vec2,
   inner_margin: Option<Margin>,
   #[cfg(all(
      feature = "qr-scanner",
      any(target_os = "linux", target_os = "windows")
   ))]
   qr_scanner: QRScanner,
   qr_enabled: bool,
}

impl SecureInputField {
   /// Create a new secure input field.
   ///
   /// # Arguments
   ///
   /// * `id` - The id of the input field, this will also be used as the name of the input field
   /// * `text_hidden` - Whether the text is masked
   /// * `open` - Whether the input field is open
   ///
   /// # Panics
   ///
   /// If the `SecureString` allocation fails.
   pub fn new(id: &'static str, text_hidden: bool, open: bool) -> Self {
      Self {
         open,
         text: SecureString::new_with_capacity(32).unwrap(),
         text_hidden,
         id,
         icon_size: vec2(20.0, 20.0),
         min_size: vec2(300.0, 20.0),
         inner_margin: None,
         #[cfg(all(
            feature = "qr-scanner",
            any(target_os = "linux", target_os = "windows")
         ))]
         qr_scanner: QRScanner::new(),
         qr_enabled: true,
      }
   }

   /// Whether the field is currently shown.
   pub fn is_open(&self) -> bool {
      self.open
   }

   /// Start showing the field on the next [`Self::show`].
   pub fn open(&mut self) {
      self.open = true;
   }

   /// Hide the field. Does not erase the text.
   pub fn close(&mut self) {
      self.open = false;
   }

   /// Erase the text from memory
   pub fn erase(&mut self) {
      self.text.erase();
   }

   /// Builder: Lucide icon size for the show/hide (and QR) buttons.
   ///
   /// Display size is `size.x` (icons are square).
   pub fn icon_size(mut self, size: Vec2) -> Self {
      self.icon_size = size;
      self
   }

   /// Builder: minimum allocated size of the text field.
   pub fn with_min_size(mut self, size: Vec2) -> Self {
      self.min_size = size;
      self
   }

   /// Return the minimum allocated size of the text field.
   pub fn min_size(&self) -> Vec2 {
      self.min_size
   }

   /// Builder: inner margin around the text.
   pub fn inner_margin(mut self, margin: Margin) -> Self {
      self.inner_margin = Some(margin);
      self
   }

   /// Builder: show the QR-scan button (Linux/Windows + `qr-scanner` only).
   pub fn qr_enabled(mut self, enabled: bool) -> Self {
      self.qr_enabled = enabled;
      self
   }

   /// Whether the text is currently masked.
   pub fn is_text_hidden(&self) -> bool {
      self.text_hidden
   }

   /// Return a clone of the text
   pub fn text(&self) -> SecureString {
      self.text.clone()
   }

   /// Set the id of this input field
   pub fn set_id(&mut self, id: &'static str) {
      self.id = id;
   }

   /// Set whether this input field is hidden
   pub fn set_text_hidden(&mut self, text_hidden: bool) {
      self.text_hidden = text_hidden;
   }

   /// Set the text of this input field
   pub fn set_text(&mut self, text: SecureString) {
      self.text = text;
   }

   /// Set the minimum size of this input field
   pub fn set_min_size(&mut self, size: Vec2) {
      self.min_size = size;
   }

   /// Set the icon size of this input field
   pub fn set_icon_size(&mut self, size: Vec2) {
      self.icon_size = size;
   }

   /// Set the inner margin of this input field
   pub fn set_inner_margin(&mut self, margin: Margin) {
      self.inner_margin = Some(margin);
   }

   /// Enable the QR scanner
   pub fn enable_qr_scanner(&mut self) {
      self.qr_enabled = true;
   }

   /// Disable the QR scanner
   pub fn disable_qr_scanner(&mut self) {
      self.qr_enabled = false;
   }

   /// Show this input field
   ///
   /// # Returns
   /// `SecureTextEditOutput`
   pub fn show(&mut self, ui: &mut Ui) -> Option<SecureTextEditOutput> {
      if !self.open {
         return None;
      }

      let mut hidden = self.is_text_hidden();

      let (output, qr_clicked) = show_field_chrome(
         ui,
         self.id,
         &mut self.text,
         &mut hidden,
         self.min_size,
         self.inner_margin,
         self.icon_size,
         self.qr_enabled,
      );

      #[cfg(all(
         feature = "qr-scanner",
         any(target_os = "linux", target_os = "windows")
      ))]
      {
         if let Some(res) = drive_qr_scanner(
            &mut self.qr_scanner,
            ui.ctx(),
            qr_clicked,
            self.qr_enabled,
         ) {
            self.set_text(res);
         }
      }

      #[cfg(not(all(
         feature = "qr-scanner",
         any(target_os = "linux", target_os = "windows")
      )))]
      let _ = qr_clicked;

      self.set_text_hidden(hidden);

      Some(output)
   }
}

/// A themed single-line input field backed by any [`TextBuffer`].
///
/// Same chrome as [`SecureInputField`] — the field name above the row and a
/// Lucide `Eye` / `EyeOff` show/hide toggle — but it **borrows** its buffer
/// instead of owning a [`SecureString`], so it can drive a plain [`String`]
/// (or any other [`TextBuffer`]) for text that is not a secret.
///
/// Because it only borrows the buffer it allocates nothing and does not zeroize
/// on [`Self::erase`]. Use [`SecureInputField`] for secrets.
///
/// The QR button (Linux/Windows + `qr-scanner`) is the same as
/// [`SecureInputField`]'s, but a decode is copied into the plain buffer and so is
/// no longer wiped — fine for public data such as a recipient address, not for
/// secrets.
///
/// The host app must register egui's SVG loader once:
/// `egui_extras::install_image_loaders(ctx)` with the `egui_extras` `svg` feature.
pub struct InputField<'a> {
   open: bool,
   text: &'a mut dyn TextBuffer,
   text_hidden: bool,
   id: &'static str,
   icon_size: Vec2,
   min_size: Vec2,
   inner_margin: Option<Margin>,
   #[cfg(all(
      feature = "qr-scanner",
      any(target_os = "linux", target_os = "windows")
   ))]
   qr_scanner: QRScanner,
   qr_enabled: bool,
}

impl<'a> InputField<'a> {
   /// Create a new input field that borrows `text`.
   ///
   /// # Arguments
   ///
   /// * `id` - The id of the input field, this will also be used as the name of the input field
   /// * `text` - The buffer to edit, e.g. a `String`
   /// * `text_hidden` - Whether the text is masked
   /// * `open` - Whether the input field is open
   pub fn new(
      id: &'static str,
      text: &'a mut dyn TextBuffer,
      text_hidden: bool,
      open: bool,
   ) -> Self {
      Self {
         open,
         text,
         text_hidden,
         id,
         icon_size: vec2(20.0, 20.0),
         min_size: vec2(300.0, 20.0),
         inner_margin: None,
         #[cfg(all(
            feature = "qr-scanner",
            any(target_os = "linux", target_os = "windows")
         ))]
         qr_scanner: QRScanner::new(),
         qr_enabled: true,
      }
   }

   /// Whether the field is currently shown.
   pub fn is_open(&self) -> bool {
      self.open
   }

   /// Start showing the field on the next [`Self::show`].
   pub fn open(&mut self) {
      self.open = true;
   }

   /// Hide the field. Does not erase the text.
   pub fn close(&mut self) {
      self.open = false;
   }

   /// Delete the whole buffer. Does not zeroize; use [`SecureInputField::erase`]
   /// for secrets.
   pub fn erase(&mut self) {
      let len = self.text.char_len();
      self.text.delete_text_char_range(0..len);
   }

   /// A copy of the buffer contents.
   pub fn text(&self) -> String {
      self.text.to_string()
   }

   /// Replace the buffer contents with `text`.
   pub fn set_text(&mut self, text: &str) {
      self.erase();
      self.text.insert_text_at_char_idx(0, text);
   }

   /// Builder: Lucide icon size for the show/hide button.
   ///
   /// Display size is `size.x` (icons are square).
   pub fn icon_size(mut self, size: Vec2) -> Self {
      self.icon_size = size;
      self
   }

   /// Builder: minimum allocated size of the text field.
   pub fn with_min_size(mut self, size: Vec2) -> Self {
      self.min_size = size;
      self
   }

   /// Return the minimum allocated size of the text field.
   pub fn min_size(&self) -> Vec2 {
      self.min_size
   }

   /// Builder: inner margin around the text.
   pub fn inner_margin(mut self, margin: Margin) -> Self {
      self.inner_margin = Some(margin);
      self
   }

   /// Whether the text is currently masked.
   pub fn is_text_hidden(&self) -> bool {
      self.text_hidden
   }

   /// Set the id of this input field
   pub fn set_id(&mut self, id: &'static str) {
      self.id = id;
   }

   /// Set whether this input field is hidden
   pub fn set_text_hidden(&mut self, text_hidden: bool) {
      self.text_hidden = text_hidden;
   }

   /// Set the minimum size of this input field
   pub fn set_min_size(&mut self, size: Vec2) {
      self.min_size = size;
   }

   /// Set the icon size of this input field
   pub fn set_icon_size(&mut self, size: Vec2) {
      self.icon_size = size;
   }

   /// Set the inner margin of this input field
   pub fn set_inner_margin(&mut self, margin: Margin) {
      self.inner_margin = Some(margin);
   }

   /// Builder: show the QR-scan button (Linux/Windows + `qr-scanner` only).
   pub fn qr_enabled(mut self, enabled: bool) -> Self {
      self.qr_enabled = enabled;
      self
   }

   /// Enable the QR scanner
   pub fn enable_qr_scanner(&mut self) {
      self.qr_enabled = true;
   }

   /// Disable the QR scanner
   pub fn disable_qr_scanner(&mut self) {
      self.qr_enabled = false;
   }

   /// Show this input field
   ///
   /// # Returns
   /// `SecureTextEditOutput`
   pub fn show(&mut self, ui: &mut Ui) -> Option<SecureTextEditOutput> {
      if !self.open {
         return None;
      }

      let mut hidden = self.text_hidden;

      let (output, qr_clicked) = show_field_chrome(
         ui,
         self.id,
         &mut *self.text,
         &mut hidden,
         self.min_size,
         self.inner_margin,
         self.icon_size,
         self.qr_enabled,
      );

      #[cfg(all(
         feature = "qr-scanner",
         any(target_os = "linux", target_os = "windows")
      ))]
      {
         if let Some(res) = drive_qr_scanner(
            &mut self.qr_scanner,
            ui.ctx(),
            qr_clicked,
            self.qr_enabled,
         ) {
            let decoded = res.unlock_str(|s| s.to_owned());
            self.set_text(&decoded);
         }
      }

      #[cfg(not(all(
         feature = "qr-scanner",
         any(target_os = "linux", target_os = "windows")
      )))]
      let _ = qr_clicked;

      self.set_text_hidden(hidden);

      Some(output)
   }
}

/// Drive the QR overlay for one frame: open it when `clicked`, advance it, and
/// take a decode if one arrived.
///
/// Returns the decoded payload (a `SecureString`, zeroized when dropped) or
/// `None`. No-op when `!enabled`, so the caller can gate the button and the
/// scanner with the same flag. Shared by [`SecureInputField`] and [`InputField`].
#[cfg(all(
   feature = "qr-scanner",
   any(target_os = "linux", target_os = "windows")
))]
fn drive_qr_scanner(
   scanner: &mut QRScanner,
   ctx: &Context,
   clicked: bool,
   enabled: bool,
) -> Option<SecureString> {
   if !enabled {
      return None;
   }
   if clicked {
      scanner.open(ctx.clone());
   }
   scanner.show(ctx);
   if let Some(res) = scanner.take_result() {
      scanner.reset();
      Some(res)
   } else {
      None
   }
}

/// Shared chrome for [`SecureInputField`] and [`InputField`]: the field name
/// above the row, a single-line [`SecureTextEdit`], and the Lucide `Eye` /
/// `EyeOff` show/hide toggle — plus the QR button when `qr_enabled` and the
/// `qr-scanner` feature is active on Linux/Windows.
///
/// `hidden` is toggled in place; the caller stores it back on its field.
///
/// Returns the edit output and whether the QR button was clicked. The caller
/// owns the scanner lifecycle, so the scanner stays out of this helper.
#[allow(clippy::too_many_arguments)]
fn show_field_chrome(
   ui: &mut Ui,
   id: &'static str,
   text: &mut dyn TextBuffer,
   hidden: &mut bool,
   min_size: Vec2,
   inner_margin: Option<Margin>,
   icon_size: Vec2,
   qr_enabled: bool,
) -> (SecureTextEditOutput, bool) {
   let theme = Theme::current(ui.ctx());
   let field_name = id.to_string();

   ui.label(RichText::new(field_name).size(theme.typography.large));

   let margin = inner_margin.unwrap_or_else(|| Margin::same(theme.inner_margin));
   let row_height = (theme.typography.normal + margin.sum().y).max(min_size.y);
   let row_size = vec2(min_size.x, row_height);

   let text_edit = SecureTextEdit::singleline(text)
      .min_size(min_size)
      .margin(margin)
      .password(*hidden)
      .font(FontId::proportional(theme.typography.normal));

   #[cfg_attr(
      not(all(
         feature = "qr-scanner",
         any(target_os = "linux", target_os = "windows")
      )),
      allow(unused_mut)
   )]
   let mut qr_clicked = false;

   let output = ui.allocate_ui_with_layout(
      row_size,
      Layout::left_to_right(Align::Center),
      |ui| {
         let output = text_edit.show(ui);

         let text_color = theme.colors.text;
         let icon = if *hidden { Lucide::EyeOff } else { Lucide::Eye };
         let img = icon.size(icon_size.x).color(text_color).image();
         if ui.add(Button::image(img)).clicked() {
            *hidden = !*hidden;
         }

         #[cfg(all(
            feature = "qr-scanner",
            any(target_os = "linux", target_os = "windows")
         ))]
         {
            if qr_enabled {
               let icon = Lucide::ScanQrCode;
               let img = icon.size(icon_size.x).color(text_color).image();
               if ui.add(Button::image(img)).clicked() {
                  qr_clicked = true;
               }
            }
         }

         output
      },
   );

   #[cfg(not(all(
      feature = "qr-scanner",
      any(target_os = "linux", target_os = "windows")
   )))]
   let _ = qr_enabled;

   (output.inner, qr_clicked)
}

#[cfg(test)]
mod tests {
   use super::*;

   #[test]
   fn input_field_drives_a_plain_string() {
      let mut buffer = String::new();
      let mut field = InputField::new("Name", &mut buffer, false, true);

      field.set_text("hello");
      assert_eq!(field.text(), "hello");

      field.erase();
      assert_eq!(field.text(), "");
   }
}
