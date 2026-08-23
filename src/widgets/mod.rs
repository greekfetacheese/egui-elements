//! Themed widgets that read visuals from an installed [`Theme`](crate::theme::Theme).
//!
//! These types shadow a few egui names (`Button`, `ComboBox`, `Window`). Prefer
//! `egui_elements::widgets::{Button, ComboBox, Label, Window, …}` after
//! `use eframe::egui::*`, or alias the egui types.
//!
//! Visuals resolve in this order: the value passed to `.visuals(...)` →
//! the theme stored on [`egui::Context`] by [`Theme::install`](crate::theme::Theme::install)
//! → stock [`egui::Style`].

pub mod button;
pub mod combo_box;
pub mod label;
pub mod modal;
pub mod multi_label;
pub mod secure_text_edit;
pub mod window;

pub use button::Button;
pub use combo_box::ComboBox;
pub use label::Label;
pub use modal::Modal;
pub use multi_label::MultiLabel;
pub use secure_text_edit::SecureTextEdit;
pub use window::Window;
