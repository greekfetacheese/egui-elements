//! A themed text edit for ordinary buffers.
//!
//! [`TextEdit`] is the same implementation as
//! [`SecureTextEdit`](super::secure_text_edit::SecureTextEdit), named for the common case: a
//! search field, a composer, a rename box. It is available with or without the `secure-types`
//! feature; masking and `SecureString` buffers stay on the secure name.
//!
//! ```
//! # use egui::__run_test_ui;
//! # use egui_elements::widgets::TextEdit;
//! # __run_test_ui(|ui| {
//! let mut name = String::from("Rey");
//! ui.add(TextEdit::singleline(&mut name).hint_text("Name"));
//! # });
//! ```

pub use super::secure_text_edit::{
   SecureTextEdit as TextEdit, SecureTextEditOutput as TextEditOutput,
};
