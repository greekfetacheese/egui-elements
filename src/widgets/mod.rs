//! Themed widgets that read visuals from an installed [`Theme`](crate::theme::Theme).
//!
//! These types shadow a few egui names (`Button`, `ComboBox`, `Window`). Prefer
//! `egui_elements::widgets::{Button, ComboBox, Label, Window, …}` after
//! `use eframe::egui::*`, or alias the egui types.
//!
//! Visuals resolve in this order: the value passed to `.visuals(...)` →
//! the theme stored on [`egui::Context`] by [`Theme::install`](crate::theme::Theme::install)
//! → stock [`egui::Style`].

pub mod badge;
pub mod button;
pub mod combo_box;
pub mod frame;
pub mod label;
pub mod modal;
pub mod multi_label;
pub mod secure_text_edit;
pub mod spinner;
pub mod toggle;
pub mod window;

pub use badge::{Badge, BadgeAnchor, BadgeBounce, BadgeBounceTrigger, BadgeCorner, BadgePulse};
pub use button::{Button, HoldBurst, HoldSettings};
pub use combo_box::ComboBox;
pub use frame::Frame;
pub use label::Label;
pub use modal::{IntoModalId, Modal};
pub use multi_label::MultiLabel;
pub use secure_text_edit::SecureTextEdit;
pub use spinner::{Spinner, SpinnerShadow};
pub use toggle::{Checkbox, CheckboxShadow, CheckboxVisuals, Radio, ToggleShadow, ToggleVisuals};
pub use window::Window;
