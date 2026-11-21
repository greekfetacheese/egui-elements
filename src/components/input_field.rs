use crate::theme::Theme;
use crate::widgets::{
   Button,
   secure_text_edit::{SecureSpeech, SecureTextEdit, SecureTextEditOutput, TextBuffer},
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
   /// Draw the show/hide (`Eye` / `EyeOff`) button. A control flag only: it never touches
   /// `text_hidden`, so a masked field with the toggle off stays masked.
   mask_toggle: bool,
   speech: SecureSpeech,
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
         mask_toggle: true,
         speech: SecureSpeech::Masked,
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

   /// Stop an open QR overlay, if any.
   ///
   /// A disabled or replaced field stops advancing its scanner (the shared driver no-ops once
   /// `qr_enabled` is off), and a scanner that is still open keeps a capture worker running, so
   /// a host that stops drawing this field must call this. Same contract as
   /// [`InputField::close_qr_scanner`].
   pub fn close_qr_scanner(&mut self) {
      #[cfg(all(
         feature = "qr-scanner",
         any(target_os = "linux", target_os = "windows")
      ))]
      self.qr_scanner.close();
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

   /// Builder: draw the show/hide (`Eye` / `EyeOff`) button. Default `true`.
   ///
   /// `false` removes the control only: the field keeps the `text_hidden` state it was built
   /// with, so a masked field stays masked and only
   /// [`set_text_hidden`](Self::set_text_hidden) changes the state. Turn it off for a host whose
   /// users cannot use a visual-only control -- the button is a focus stop that can never help
   /// them.
   pub fn mask_toggle(mut self, enabled: bool) -> Self {
      self.mask_toggle = enabled;
      self
   }

   /// Draw or drop the show/hide button; see [`mask_toggle`](Self::mask_toggle).
   pub fn set_mask_toggle(&mut self, enabled: bool) {
      self.mask_toggle = enabled;
   }

   /// Whether the text is currently masked.
   pub fn is_text_hidden(&self) -> bool {
      self.text_hidden
   }

   /// What assistive technology receives while the text is masked.
   ///
   /// Defaults to [`SecureSpeech::Masked`].
   pub fn with_speech(mut self, speech: SecureSpeech) -> Self {
      self.speech = speech;
      self
   }

   /// What assistive technology receives while the text is masked.
   pub fn set_speech(&mut self, speech: SecureSpeech) {
      self.speech = speech;
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

   /// Disable the QR scanner, stopping an overlay that is already open.
   pub fn disable_qr_scanner(&mut self) {
      self.qr_enabled = false;
      self.close_qr_scanner();
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
         Some(&mut hidden),
         self.mask_toggle,
         self.min_size,
         self.inner_margin,
         WidgetText::default(),
         self.icon_size,
         self.qr_enabled,
         self.speech,
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

/// A themed single-line input field backed by an owned [`String`].
///
/// Same chrome as [`SecureInputField`] minus the show/hide toggle — the field
/// name above the row and the Lucide `ScanQrCode` button — but the buffer is a
/// plain [`String`] instead of a [`SecureString`]. The text is never masked:
/// this field is for non-sensitive input.
///
/// Owning its buffer is what lets a host **store** the field in its UI state,
/// and that is what makes the QR button work: the scanner's live state lives on
/// the field, so the field has to survive across frames for a scan to land. It
/// does not zeroize on drop or [`Self::erase`]. Use [`SecureInputField`] for
/// secrets.
///
/// The host app must register egui's SVG loader once:
/// `egui_extras::install_image_loaders(ctx)` with the `egui_extras` `svg` feature.
///
/// Keep it on the app: it is `'static`-storable, and [`set_text`](Self::set_text),
/// [`set_min_size`](Self::set_min_size), [`set_inner_margin`](Self::set_inner_margin) and
/// [`set_id`](Self::set_id) work between frames before `show`. Rebuilding it every frame resets the
/// scanner it owns.
pub struct InputField {
   open: bool,
   text: String,
   id: &'static str,
   icon_size: Vec2,
   min_size: Vec2,
   inner_margin: Option<Margin>,
   /// Placeholder shown while the buffer is empty.
   hint_text: WidgetText,
   #[cfg(all(
      feature = "qr-scanner",
      any(target_os = "linux", target_os = "windows")
   ))]
   qr_scanner: QRScanner,
   qr_enabled: bool,
}

impl InputField {
   /// Create a new input field with an empty buffer.
   ///
   /// # Arguments
   ///
   /// * `id` - The id of the input field; also the field-name label above the row
   ///   (an empty `id` draws no label)
   /// * `open` - Whether the input field is open
   pub fn new(id: &'static str, open: bool) -> Self {
      Self {
         open,
         text: String::new(),
         id,
         icon_size: vec2(20.0, 20.0),
         min_size: vec2(300.0, 20.0),
         inner_margin: None,
         hint_text: WidgetText::default(),
         #[cfg(all(
            feature = "qr-scanner",
            any(target_os = "linux", target_os = "windows")
         ))]
         qr_scanner: QRScanner::new(),
         qr_enabled: true,
      }
   }

   /// Builder: placeholder shown while the buffer is empty.
   pub fn hint_text(mut self, hint_text: impl Into<WidgetText>) -> Self {
      self.hint_text = hint_text.into();
      self
   }

   /// Set the placeholder shown while the buffer is empty.
   pub fn set_hint_text(&mut self, hint_text: impl Into<WidgetText>) {
      self.hint_text = hint_text.into();
   }

   /// Whether the field is currently shown.
   pub fn is_open(&self) -> bool {
      self.open
   }

   /// Start showing the field on the next [`Self::show`].
   pub fn open(&mut self) {
      self.open = true;
   }

   /// Hide the field. Does not erase the text. Stops any open QR overlay.
   pub fn close(&mut self) {
      self.open = false;
      self.close_qr_scanner();
   }

   /// Stop an open QR overlay, if any.
   ///
   /// The scanner keeps a capture worker running while an overlay is open, and
   /// the field keeps the scanner alive — so a host that stops drawing the field
   /// (hides it, replaces it in the layout, …) must call this, or that worker
   /// would outlive the field that opened it.
   pub fn close_qr_scanner(&mut self) {
      #[cfg(all(
         feature = "qr-scanner",
         any(target_os = "linux", target_os = "windows")
      ))]
      self.qr_scanner.close();
   }

   /// Clear the buffer. Does not zeroize; use [`SecureInputField::erase`] for
   /// secrets.
   pub fn erase(&mut self) {
      self.text.clear();
   }

   /// The buffer contents.
   pub fn text(&self) -> &str {
      &self.text
   }

   /// Replace the buffer contents with `text`.
   pub fn set_text(&mut self, text: &str) {
      self.text.clear();
      self.text.push_str(text);
   }

   /// Builder: Lucide icon size for the QR button.
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

   /// Set the id of this input field
   pub fn set_id(&mut self, id: &'static str) {
      self.id = id;
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

      let (output, qr_clicked) = show_field_chrome(
         ui,
         self.id,
         &mut self.text,
         None,
         // `InputField` never masks: `hidden: None` is already "no toggle".
         true,
         self.min_size,
         self.inner_margin,
         self.hint_text.clone(),
         self.icon_size,
         self.qr_enabled,
         SecureSpeech::Masked,
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
/// above the row, a single-line [`SecureTextEdit`], the Lucide `Eye` / `EyeOff`
/// show/hide toggle when `hidden` is `Some` and `mask_toggle` is set — plus the
/// QR button when `qr_enabled` and the `qr-scanner` feature is active on Linux/Windows.
///
/// An empty `id` skips the field-name label: the host already labels the field.
///
/// `hidden` is the masking state and the flag behind the toggle; `None` means
/// the field is never masked and no toggle is drawn. It is toggled in place, and
/// the caller stores it back on its field.
///
/// `mask_toggle` draws that toggle or leaves it out. It is a control flag, not a
/// state: the masking itself still follows `hidden`, so a host can keep the mask
/// and drop the button.
///
/// Returns the edit output and whether the QR button was clicked. The caller
/// owns the scanner lifecycle, so the scanner stays out of this helper.
#[allow(clippy::too_many_arguments)]
fn show_field_chrome(
   ui: &mut Ui,
   id: &'static str,
   text: &mut dyn TextBuffer,
   mut hidden: Option<&mut bool>,
   mask_toggle: bool,
   min_size: Vec2,
   inner_margin: Option<Margin>,
   hint_text: WidgetText,
   icon_size: Vec2,
   qr_enabled: bool,
   speech: SecureSpeech,
) -> (SecureTextEditOutput, bool) {
   let theme = Theme::current(ui.ctx());

   // The visible field name above the row is also the field's accessible name.
   let label = if id.is_empty() {
      None
   } else {
      Some(ui.label(RichText::new(id).size(theme.typography.large)))
   };

   let margin = inner_margin.unwrap_or_else(|| Margin::same(theme.inner_margin));
   let row_height = (theme.typography.normal + margin.sum().y).max(min_size.y);
   let row_size = vec2(min_size.x, row_height);

   let password = hidden.as_deref().copied().unwrap_or(false);
   let text_edit = SecureTextEdit::singleline(text)
      .min_size(min_size)
      .margin(margin)
      .hint_text(hint_text)
      .password(password)
      .speech(speech)
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
         let mut output = text_edit.show(ui);

         if let Some(label) = &label {
            output.response = output.response.clone().labelled_by(label.id);
         }

         let text_color = theme.colors.text;
         if mask_toggle {
            if let Some(state) = hidden.as_deref_mut() {
               let icon = if *state { Lucide::EyeOff } else { Lucide::Eye };
               let img = icon.size(icon_size.x).color(text_color).image();
               // An icon-only button has no text, so it needs the text alternative: name
               // the action the click performs, not the icon that draws it.
               let alt = if *state {
                  "Show credentials"
               } else {
                  "Hide credentials"
               };
               if ui.add(Button::image(img).alt(alt)).clicked() {
                  *state = !*state;
               }
            }
         }

         #[cfg(all(
            feature = "qr-scanner",
            any(target_os = "linux", target_os = "windows")
         ))]
         {
            if qr_enabled {
               let icon = Lucide::ScanQrCode;
               let img = icon.size(icon_size.x).color(text_color).image();
               if ui.add(Button::image(img).alt("Scan a QR code")).clicked() {
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
   use crate::theme::ThemeKind;
   use egui::accesskit::{Node, Role};
   use egui::{Context, RawInput, Ui};

   /// The two fields differ only in their buffer type; the chrome under test is shared.
   trait FieldUnderTest {
      fn show_here(&mut self, ui: &mut Ui);
   }

   impl FieldUnderTest for SecureInputField {
      fn show_here(&mut self, ui: &mut Ui) {
         self.show(ui);
      }
   }

   impl FieldUnderTest for InputField {
      fn show_here(&mut self, ui: &mut Ui) {
         self.show(ui);
      }
   }

   /// The nodes of one AccessKit-enabled frame of `field`.
   fn field_nodes(field: &mut impl FieldUnderTest) -> Vec<Node> {
      let ctx = Context::default();
      ctx.enable_accesskit();
      Theme::new(ThemeKind::TokyoNight).install(&ctx);

      let mut output = ctx.run_ui(RawInput::default(), |ui| field.show_here(ui));
      let update = output
         .platform_output
         .accesskit_update
         .take()
         .expect("an AccessKit-enabled frame must produce a tree update");
      let nodes = update.nodes.into_iter().map(|(_, node)| node).collect();
      output.drop_without_applying_deltas();

      nodes
   }

   /// The accessible names of every button in the frame.
   fn button_names(nodes: &[Node]) -> Vec<String> {
      nodes
         .iter()
         .filter(|node| node.role() == Role::Button)
         .map(|node| node.label().unwrap_or("<unnamed>").to_owned())
         .collect()
   }

   /// The eye toggle is named for the action the click performs, so its name flips
   /// with the masking state.
   #[test]
   fn the_eye_button_names_the_visibility_action() {
      let mut masked = SecureInputField::new("Password", true, true);
      let masked = button_names(&field_nodes(&mut masked));
      assert!(
         masked.iter().any(|name| name == "Show credentials"),
         "a masked field must offer to show: {masked:?}"
      );

      let mut visible = SecureInputField::new("Password", false, true);
      let visible = button_names(&field_nodes(&mut visible));
      assert!(
         visible.iter().any(|name| name == "Hide credentials"),
         "a visible field must offer to hide: {visible:?}"
      );
   }

   /// The QR button is an icon too, so it needs the same text alternative. It is drawn
   /// only on the platforms the scanner supports.
   #[cfg(all(
      feature = "qr-scanner",
      any(target_os = "linux", target_os = "windows")
   ))]
   #[test]
   fn the_qr_button_is_named_and_every_chrome_button_has_a_name() {
      let mut field = InputField::new("Address", true);
      let names = button_names(&field_nodes(&mut field));

      assert!(
         names.iter().any(|name| name == "Scan a QR code"),
         "the QR button is unnamed: {names:?}"
      );
      assert!(
         !names.iter().any(|name| name == "<unnamed>"),
         "every chrome button must be named: {names:?}"
      );
   }

   /// The toggle is a control the host can leave out, and the setter drives the same flag.
   #[test]
   fn the_mask_toggle_can_be_left_out() {
      let mut default = SecureInputField::new("Password", true, true);
      let default = button_names(&field_nodes(&mut default));
      assert!(
         default.iter().any(|name| name == "Show credentials"),
         "a masked field draws the toggle by default: {default:?}"
      );

      let mut bare = SecureInputField::new("Password", true, true).mask_toggle(false);
      let nodes = field_nodes(&mut bare);
      let names = button_names(&nodes);
      assert!(
         !names
            .iter()
            .any(|name| name == "Show credentials" || name == "Hide credentials"),
         "the toggle must not be drawn: {names:?}"
      );
      assert!(
         nodes.iter().any(|node| node.role() == Role::PasswordInput),
         "the field itself must still be drawn: {names:?}"
      );

      bare.set_mask_toggle(true);
      let back = button_names(&field_nodes(&mut bare));
      assert!(
         back.iter().any(|name| name == "Show credentials"),
         "the setter must draw the toggle again: {back:?}"
      );
   }

   /// Dropping the control keeps the mask: a reader still receives one mask character per
   /// character of the secret.
   #[test]
   fn a_field_without_its_toggle_still_masks_its_value_for_accesskit() {
      let mut field = SecureInputField::new("Password", true, true).mask_toggle(false);
      field.set_text(SecureString::from("hunter2"));

      let nodes = field_nodes(&mut field);
      let node = nodes
         .iter()
         .find(|node| node.role() == Role::PasswordInput)
         .expect("a masked field must stay a password input");
      let mask = std::iter::repeat(egui::epaint::text::PASSWORD_REPLACEMENT_CHAR)
         .take(7)
         .collect::<String>();

      assert_eq!(node.value(), Some(mask.as_str()));
      assert!(!node.value().unwrap_or_default().contains("hunter2"));
   }

   /// `mask_toggle` is a control flag, not a mask: a field built visible with the toggle off
   /// stays visible, which its plaintext value proves.
   #[test]
   fn the_mask_toggle_never_changes_the_masking_state() {
      let mut field = SecureInputField::new("Password", false, true).mask_toggle(false);
      field.set_text(SecureString::from("hunter2"));

      let nodes = field_nodes(&mut field);
      let names = button_names(&nodes);
      assert!(
         !names
            .iter()
            .any(|name| name == "Show credentials" || name == "Hide credentials"),
         "no toggle is drawn: {names:?}"
      );

      let node = nodes
         .iter()
         .find(|node| node.role() == Role::TextInput)
         .expect("a visible field must stay a plain text input");
      assert_eq!(node.value(), Some("hunter2"));
   }

   /// The QR button follows `qr_enabled`, on the platform that draws it.
   #[cfg(all(
      feature = "qr-scanner",
      any(target_os = "linux", target_os = "windows")
   ))]
   #[test]
   fn the_qr_button_is_not_drawn_when_disabled() {
      let mut enabled = InputField::new("Address", true);
      let enabled = button_names(&field_nodes(&mut enabled));
      assert!(
         enabled.iter().any(|name| name == "Scan a QR code"),
         "the QR button is drawn by default: {enabled:?}"
      );

      let mut disabled = InputField::new("Address", true).qr_enabled(false);
      let disabled = button_names(&field_nodes(&mut disabled));
      assert!(
         !disabled.iter().any(|name| name == "Scan a QR code"),
         "qr_enabled(false) must leave the button out: {disabled:?}"
      );

      // The mutable setter a host flips at runtime: the QR button goes, the rest of the
      // chrome stays. (It also stops an open overlay; that has no test without a display.)
      let mut secure = SecureInputField::new("Password", true, true);
      secure.disable_qr_scanner();
      let names = button_names(&field_nodes(&mut secure));
      assert!(
         !names.iter().any(|name| name == "Scan a QR code"),
         "disable_qr_scanner must leave the button out: {names:?}"
      );
      assert!(
         names.iter().any(|name| name == "Show credentials"),
         "the rest of the chrome must still draw: {names:?}"
      );
   }
}
