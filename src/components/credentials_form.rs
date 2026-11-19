use std::fmt;
use std::sync::Arc;

use super::input_field::SecureInputField;
use super::virtual_keyboard::VirtualKeyboard;
use crate::announce::Announcer;
use crate::widgets::secure_text_edit::SecureSpeech;
use egui::*;

use secure_types::SecureString;

/// The row an announcement is about.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnnouncedField {
   /// The username row.
   Username,
   /// The password row.
   Password,
   /// The confirm-password row.
   ConfirmPassword,
}

impl AnnouncedField {
   /// The field's name as it is spoken.
   ///
   /// Matches the visible row label, so focus and an announcement name the field the
   /// same way.
   pub fn label(self) -> &'static str {
      match self {
         Self::Username => "Username",
         Self::Password => "Password",
         Self::ConfirmPassword => "Confirm Password",
      }
   }
}

/// What the live status node is about, handed to a
/// [custom announcement](CredentialsForm::with_announce_text).
///
/// It carries the field and how many characters it holds -- never the text.
#[derive(Clone, Copy, Debug)]
pub struct Announcement {
   field: AnnouncedField,
   characters: usize,
   speech: SecureSpeech,
}

impl Announcement {
   /// The field the announcement is about.
   pub fn field(&self) -> AnnouncedField {
      self.field
   }

   /// How many characters the field holds.
   pub fn characters(&self) -> usize {
      self.characters
   }

   /// What the focused field is doing with its mask.
   pub fn speech(&self) -> SecureSpeech {
      self.speech
   }

   /// The built-in wording.
   ///
   /// A masked field says how much it holds; a plain one says nothing while it has text
   /// (the reader key-echoes those characters already) and speaks only its empty state,
   /// which nothing else reports.
   pub fn default_text(&self) -> String {
      match self.field {
         AnnouncedField::Username if self.characters == 0 => {
            format!("{}, empty", self.field.label())
         }
         AnnouncedField::Username => String::new(),
         field => count_text(field.label(), self.characters),
      }
   }
}

impl fmt::Display for Announcement {
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      f.write_str(&self.default_text())
   }
}

/// The wording of every announcement the form makes.
type AnnounceFormatter = Arc<dyn Fn(&Announcement) -> String + Send + Sync>;

/// A credentials form that can be used to edit username, password, and optionally confirm password.
///
/// The form owns the live announcement a screen reader hears -- the focused masked field's
/// length, a plain field's empty state. A [`SecureInputField`] shown on its own has none.
pub struct CredentialsForm {
   open: bool,
   confrim_password: bool,
   username: SecureInputField,
   password: SecureInputField,
   confirm_password: SecureInputField,
   y_spacing: f32,
   x_spacing: f32,
   last_focused: Option<&'static str>,
   virtual_keyboard: VirtualKeyboard,
   speech: SecureSpeech,
   announcer: Announcer,
   announcement_text: Option<AnnounceFormatter>,
}

impl CredentialsForm {
   /// Create a new credentials form.
   ///
   /// # Panics
   ///
   /// If the `SecureString` allocation fails.
   pub fn new() -> Self {
      let username = SecureInputField::new("Username", false, true);
      let password = SecureInputField::new("Password", true, true);
      let confirm_password = SecureInputField::new("Confirm Password", true, true);
      Self {
         open: false,
         confrim_password: false,
         username,
         password,
         confirm_password,
         y_spacing: 15.0,
         x_spacing: 10.0,
         last_focused: None,
         virtual_keyboard: VirtualKeyboard::new(false),
         speech: SecureSpeech::Masked,
         announcer: Announcer::new("credentials_form"),
         announcement_text: None,
      }
   }

   /// Whether the form is currently shown.
   pub fn is_open(&self) -> bool {
      self.open
   }

   /// Start showing the form on the next [`Self::show`].
   pub fn open(&mut self) {
      self.open = true;
   }

   /// Hide the form. Does not erase the fields.
   pub fn close(&mut self) {
      self.open = false;
   }

   /// Erase the credentials from memory by zeroizing the username and password fields.
   pub fn erase(&mut self) {
      self.username.erase();
      self.password.erase();
      self.confirm_password.erase();
   }

   /// Reset the state of the credentials form.
   pub fn reset(&mut self) {
      *self = Self::new();
   }

   /// Builder: what assistive technology receives while the password is masked.
   pub fn with_speech(mut self, speech: SecureSpeech) -> Self {
      self.set_speech(speech);
      self
   }

   /// What assistive technology receives while the password is masked.
   pub fn set_speech(&mut self, speech: SecureSpeech) {
      self.speech = speech;
      self.username.set_speech(speech);
      self.password.set_speech(speech);
      self.confirm_password.set_speech(speech);
   }

   /// Builder: build the announcement yourself.
   ///
   /// The closure runs every frame with the focused field and its character count, and its
   /// return value replaces the built-in wording. Use it to add context the form does not
   /// know (remaining attempts, a different language) -- never to speak the secret itself:
   /// the text enters the accessibility tree and the screen reader's speech history.
   pub fn with_announce_text(
      mut self,
      text: impl Fn(&Announcement) -> String + Send + Sync + 'static,
   ) -> Self {
      self.set_announce_text(text);
      self
   }

   /// Build the announcement yourself; see [`Self::with_announce_text`].
   pub fn set_announce_text(
      &mut self,
      text: impl Fn(&Announcement) -> String + Send + Sync + 'static,
   ) {
      self.announcement_text = Some(Arc::new(text));
   }

   /// Go back to the built-in wording.
   pub fn clear_announce_text(&mut self) {
      self.announcement_text = None;
   }

   /// Whether the confirm-password row is enabled.
   pub fn is_confirm_password(&self) -> bool {
      self.confrim_password
   }

   /// Copy password to confirm password
   fn copy_passwd_to_confirm(&mut self) {
      let new_text = self.password.text.clone();
      self.confirm_password.set_text(new_text);
   }

   /// Return a clone of the username
   pub fn username(&self) -> SecureString {
      self.username.text.clone()
   }

   /// Return a clone of the password
   pub fn password(&self) -> SecureString {
      self.password.text.clone()
   }

   /// Return a clone of the confirm password
   pub fn confirm_password(&self) -> SecureString {
      self.confirm_password.text.clone()
   }

   /// Builder: start open or closed.
   pub fn with_open(mut self, open: bool) -> Self {
      self.open = open;
      self
   }

   /// Initialize with enabled virtual keyboard.
   pub fn with_enabled_virtual_keyboard(mut self) -> Self {
      self.virtual_keyboard.open();
      self
   }

   /// Initialize with a custom icon size.
   pub fn with_icon_size(mut self, size: Vec2) -> Self {
      self.username.set_icon_size(size);
      self.password.set_icon_size(size);
      self.confirm_password.set_icon_size(size);
      self
   }

   /// Enable the virtual keyboard.
   pub fn enable_virtual_keyboard(&mut self) {
      self.virtual_keyboard.open();
   }

   /// Disable the virtual keyboard.
   pub fn disable_virtual_keyboard(&mut self) {
      self.virtual_keyboard.close();
   }

   /// Builder: minimum size of each virtual-keyboard key.
   pub fn with_virtual_keyboard_key_size(mut self, size: Vec2) -> Self {
      self.virtual_keyboard.set_key_size(size);
      self
   }

   /// Minimum size of each virtual-keyboard key.
   pub fn set_virtual_keyboard_key_size(&mut self, size: Vec2) {
      self.virtual_keyboard.set_key_size(size);
   }

   /// Builder: inner padding of each virtual-keyboard key.
   pub fn with_virtual_keyboard_button_padding(mut self, padding: Vec2) -> Self {
      self.virtual_keyboard.set_button_padding(padding);
      self
   }

   /// Inner padding of each virtual-keyboard key.
   pub fn set_virtual_keyboard_button_padding(&mut self, padding: Vec2) {
      self.virtual_keyboard.set_button_padding(padding);
   }

   /// Text size for each virtual-keyboard key.
   pub fn with_virtual_keyboard_text_size(mut self, size: f32) -> Self {
      self.virtual_keyboard.set_text_size(size);
      self
   }

   /// Text size for each virtual-keyboard key.
   pub fn set_virtual_keyboard_text_size(&mut self, size: f32) {
      self.virtual_keyboard.set_text_size(size);
   }

   /// Spacing between each virtual-keyboard key.
   pub fn with_virtual_keyboard_spacing(mut self, spacing: Vec2) -> Self {
      self.virtual_keyboard.set_spacing(spacing);
      self
   }

   /// Spacing between each virtual-keyboard key.
   pub fn set_virtual_keyboard_spacing(&mut self, spacing: Vec2) {
      self.virtual_keyboard.set_spacing(spacing);
   }

   /// Maximum size of the virtual keyboard.
   pub fn with_virtual_keyboard_max_ui_size(mut self, size: Vec2) -> Self {
      self.virtual_keyboard.set_max_ui_size(size);
      self
   }

   /// Maximum size of the virtual keyboard.
   pub fn set_virtual_keyboard_max_ui_size(&mut self, size: Vec2) {
      self.virtual_keyboard.set_max_ui_size(size);
   }

   /// Set whether to enable the confirm password field.
   pub fn set_confirm_password(&mut self, confirm_password: bool) {
      self.confrim_password = confirm_password;
   }

   /// Set the minimum size of this input field
   pub fn set_min_size(&mut self, size: Vec2) {
      self.username.set_min_size(size);
      self.password.set_min_size(size);
      self.confirm_password.set_min_size(size);
   }

   /// Adjust the icon size.
   pub fn set_icon_size(&mut self, size: Vec2) {
      self.username.set_icon_size(size);
      self.password.set_icon_size(size);
      self.confirm_password.set_icon_size(size);
   }

   /// Set the username text
   pub fn set_username_text(&mut self, text: SecureString) {
      self.username.set_text(text);
   }

   /// Set the password text
   pub fn set_password_text(&mut self, text: SecureString) {
      self.password.set_text(text);
   }

   /// Set the confirm password text
   pub fn set_confirm_password_text(&mut self, text: SecureString) {
      self.confirm_password.set_text(text);
   }

   /// Adjust the y spacing between the input fields.
   pub fn with_y_spacing(mut self, y_spacing: f32) -> Self {
      self.y_spacing = y_spacing;
      self
   }

   /// Adjust the x spacing between the input fields.
   pub fn with_x_spacing(mut self, x_spacing: f32) -> Self {
      self.x_spacing = x_spacing;
      self
   }

   /// Whether to enable the confirm password field.
   pub fn with_confirm_password(mut self, confirm_password: bool) -> Self {
      self.confrim_password = confirm_password;
      self
   }

   /// Builder: set the minimum size of every input row.
   pub fn with_min_size(mut self, size: Vec2) -> Self {
      let username = self.username.clone();
      let password = self.password.clone();
      let confirm_password = self.confirm_password.clone();

      let username = username.with_min_size(size);
      let password = password.with_min_size(size);
      let confirm_password = confirm_password.with_min_size(size);

      self.username = username;
      self.password = password;
      self.confirm_password = confirm_password;

      self
   }

   /// Return the minimum allocated size of the form.
   pub fn min_size(&self) -> Vec2 {
      self.username.min_size().max(self.password.min_size())
   }

   /// What the live status node says about the field in focus.
   ///
   /// Only the masked fields are counted: the username is not hidden, so a screen reader
   /// already echoes its keystrokes and a count would only double the speech. The count is
   /// the whole point -- the text never leaves the process.
   fn announce_text(&self) -> String {
      match self.announcement() {
         Some(announcement) => match &self.announcement_text {
            Some(text) => text(&announcement),
            None => announcement.default_text(),
         },
         None => String::new(),
      }
   }

   /// The field in focus, as an announcement sees it; none until one has had focus.
   fn announcement(&self) -> Option<Announcement> {
      let (field, characters) = match self.last_focused? {
         "Password" => (
            AnnouncedField::Password,
            self.password.text.char_len(),
         ),
         "Confirm Password" => (
            AnnouncedField::ConfirmPassword,
            self.confirm_password.text.char_len(),
         ),
         _ => (
            AnnouncedField::Username,
            self.username.text.char_len(),
         ),
      };

      Some(Announcement {
         field,
         characters,
         speech: self.speech,
      })
   }

   /// Paint the form (and the virtual keyboard, if enabled).
   pub fn show(&mut self, ui: &mut Ui) {
      if !self.open {
         return;
      }

      ui.vertical_centered(|ui| {
         ui.spacing_mut().item_spacing.y = self.y_spacing;
         ui.spacing_mut().item_spacing.x = self.x_spacing;

         let username_shown = self.username.show(ui);
         let password_shown = self.password.show(ui);

         let confirm_shown = if self.confrim_password {
            self.confirm_password.show(ui)
         } else {
            self.copy_passwd_to_confirm();
            None
         };

         let username_focused =
            username_shown.as_ref().is_some_and(|output| output.response.gained_focus());
         let password_focused =
            password_shown.as_ref().is_some_and(|output| output.response.gained_focus());
         let confirm_password_focused =
            confirm_shown.as_ref().is_some_and(|output| output.response.gained_focus());

         if username_focused {
            self.last_focused = Some("Username");
         }

         if password_focused {
            self.last_focused = Some("Password");
         }

         if confirm_password_focused {
            self.last_focused = Some("Confirm Password");
         }

         let target_text = match self.last_focused {
            Some("Password") => &mut self.password.text,
            Some("Confirm Password") => &mut self.confirm_password.text,
            _ => &mut self.username.text,
         };

         ui.add_space(10.0);

         self.virtual_keyboard.show(target_text, ui);

         let text = self.announce_text();
         self.announcer.announce(ui, text);
      });
   }
}

/// `"Password, 3 characters"` -- the shape of the secret, never its content.
fn count_text(field: &str, count: usize) -> String {
   match count {
      0 => format!("{field}, empty"),
      1 => format!("{field}, 1 character"),
      count => format!("{field}, {count} characters"),
   }
}

#[cfg(test)]
mod tests {
   use super::*;
   use crate::theme::{Theme, ThemeKind};
   use egui::accesskit::{Live, Node, Role};
   use egui::{Context, RawInput, Ui};

   fn frame(ctx: &Context, add: impl FnMut(&mut Ui)) -> Vec<Node> {
      ctx.enable_accesskit();

      let mut output = ctx.run_ui(RawInput::default(), add);
      let update = output
         .platform_output
         .accesskit_update
         .take()
         .expect("an AccessKit-enabled frame must produce a tree update");
      let nodes = update.nodes.into_iter().map(|(_, node)| node).collect();
      output.drop_without_applying_deltas();

      nodes
   }

   #[test]
   fn announce_text_counts_the_focused_password() {
      let mut form = CredentialsForm::new();

      assert_eq!(form.announce_text(), "");

      form.last_focused = Some("Password");
      assert_eq!(form.announce_text(), "Password, empty");

      form.set_password_text(SecureString::from("abc"));
      assert_eq!(form.announce_text(), "Password, 3 characters");

      form.last_focused = Some("Confirm Password");
      form.set_confirm_password_text(SecureString::from("x"));
      assert_eq!(
         form.announce_text(),
         "Confirm Password, 1 character"
      );
   }

   #[test]
   fn the_username_speaks_only_its_empty_state() {
      let mut form = CredentialsForm::new();
      form.last_focused = Some("Username");

      assert_eq!(form.announce_text(), "Username, empty");

      form.set_username_text(SecureString::from("alice"));
      assert_eq!(form.announce_text(), "");

      form.set_username_text(SecureString::from(""));
      assert_eq!(form.announce_text(), "Username, empty");
   }

   #[test]
   fn a_custom_announcement_replaces_the_wording() {
      let mut form = CredentialsForm::new().with_announce_text(|announcement| {
         format!(
            "{} holds {}",
            announcement.field().label(),
            announcement.characters()
         )
      });

      form.last_focused = Some("Password");
      form.set_password_text(SecureString::from("abc"));

      assert_eq!(form.announce_text(), "Password holds 3");
   }

   #[test]
   fn show_keeps_a_live_status_node() {
      let ctx = Context::default();
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&ctx);
      let mut form = CredentialsForm::new().with_open(true);

      let nodes = frame(&ctx, |ui| form.show(ui));
      let node = nodes
         .iter()
         .find(|node| node.role() == Role::Status)
         .expect("no live status node in the tree");

      assert_eq!(node.live(), Some(Live::Polite));
      // No field has had focus yet, so the form stays quiet instead of naming one.
      assert_eq!(node.label().unwrap_or_default(), "");
   }

   #[test]
   fn show_speaks_a_custom_announcement() {
      let ctx = Context::default();
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&ctx);
      let mut form = CredentialsForm::new()
         .with_open(true)
         .with_announce_text(|announcement| format!("{}, last try", announcement.field().label()));
      form.last_focused = Some("Username");

      let nodes = frame(&ctx, |ui| form.show(ui));
      let node = nodes
         .iter()
         .find(|node| node.role() == Role::Status)
         .expect("no live status node in the tree");

      assert_eq!(node.label(), Some("Username, last try"));
   }

   #[test]
   fn show_speaks_the_focused_field() {
      let ctx = Context::default();
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&ctx);
      let mut form = CredentialsForm::new().with_open(true);
      form.last_focused = Some("Password");
      form.set_password_text(SecureString::from("abc"));

      let nodes = frame(&ctx, |ui| form.show(ui));
      let node = nodes
         .iter()
         .find(|node| node.role() == Role::Status)
         .expect("no live status node in the tree");

      assert_eq!(node.label(), Some("Password, 3 characters"));
   }
}
