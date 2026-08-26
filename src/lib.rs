//! Widgets, themes, and optional components for [egui](https://github.com/emilk/egui).
//!
//! This crate is **not** a drop-in replacement for stock egui widgets. It
//! ships a themed widget set, a palette of built-in looks, a live theme
//! editor, and a few higher-level components (credentials form, QR image,
//! Linux QR scanner).
//!
//! There are no crate-root re-exports. Import from the modules:
//!
//! ```
//! use egui_elements::theme::{Theme, ThemeKind};
//! use egui_elements::widgets::{Button, ComboBox, Label, Modal, SecureTextEdit, Window};
//! use egui_elements::utils::theme_switcher;
//! ```
//!
//! # Quick start
//!
//! ```no_run
//! use egui::{Context, Ui};
//! use egui_elements::theme::{Theme, ThemeKind};
//! use egui_elements::widgets::Button;
//!
//! fn ui(ctx: &Context, ui: &mut Ui) {
//!     let mut theme = Theme::new(ThemeKind::TokyoNight);
//!     theme.install(ctx);
//!
//!     if ui.add(Button::new("Save")).clicked() {
//!         // …
//!     }
//! }
//! ```
//!
//! Custom widgets pick up [`visuals`] from the installed
//! [`Theme`](crate::theme::Theme). You can still override a single widget
//! with `.visuals(...)`.
//!
//! # Modules
//!
//! - [`theme`] / [`themes`] — palettes, [`ThemeKind`](crate::theme::ThemeKind), install
//! - [`widgets`] — [`Button`](crate::widgets::Button), [`Label`](crate::widgets::Label),
//!   [`ComboBox`](crate::widgets::ComboBox), [`Modal`](crate::widgets::Modal),
//!   [`SecureTextEdit`](crate::widgets::SecureTextEdit), [`MultiLabel`](crate::widgets::MultiLabel),
//!   [`Window`](crate::widgets::Window)
//! - [`components`] — feature-gated composites (`secure-types`, `qr-image`, `qr-scanner`)
//! - [`editor`] — live [`ThemeEditor`](crate::editor::ThemeEditor)
//! - [`utils`] — [`theme_switcher`](crate::utils::theme_switcher), HSLA helpers
//! - [`overlay`] — dimming overlay for stacked windows
//! - [`visuals`] — per-widget chrome (`ButtonVisuals`, `TextEditVisuals`, …)
//!
//! # Features
//!
//! | Feature | What it enables |
//! |---------|-----------------|
//! | `serde` | `Serialize` / `Deserialize` on theme types; `egui/serde` |
//! | `secure-types` | [`SecureString`](https://docs.rs/secure-types) buffers, credentials form, virtual keyboard |
//! | `qr-image` | Encode a string as a QR [`Image`](egui::Image) |
//! | `qr-scanner` | Linux screen-capture QR scanner (`xcap` + `rqrr-zeroize`) |
//! | `elegance` | Push the current palette into [`egui-elegance`](https://docs.rs/egui-elegance) |
//! | `full` | All of the above |
//!
//! Run the showcase with:
//!
//! ```text
//! cargo run --example demo --features full
//! ```
//!
//! See the crate README for system packages required by `qr-scanner`.
//!
//! [`ComboBox`](crate::widgets::ComboBox) and [`Window`](crate::widgets::Window) have
//! the same names as [`egui::ComboBox`] and [`egui::Window`]. Import this crate's
//! widgets after `use egui::*`, or alias one of them.

pub mod components;
pub mod editor;
pub mod overlay;
pub mod theme;
pub mod themes;
pub mod utils;
pub mod visuals;
pub mod widgets;

pub use overlay::OverlayManager;
pub use theme::{Theme, ThemeKind};
pub use widgets::{Button, ComboBox, Frame, Label, Modal, MultiLabel, Window};

#[cfg(feature = "secure-types")]
pub use widgets::SecureTextEdit;

#[cfg(all(feature = "qr-scanner", target_os = "linux"))]
pub use components::QRScanner;

#[cfg(feature = "secure-types")]
pub use components::{CredentialsForm, SecureInputField, VirtualKeyboard};

#[cfg(feature = "qr-image")]
pub use components::QrImage;
