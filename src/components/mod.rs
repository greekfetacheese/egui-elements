//! Higher-level composites built on the themed widgets.
//!
//! Every item here is feature-gated:
//!
//! | Type | Feature | Notes |
//! |------|---------|-------|
//! | [`SecureInputField`] | `secure-types` | Masked field with Lucide show/hide; optional QR fill on Linux/Windows |
//! | [`CredentialsForm`] | `secure-types` | Username / password / confirm + optional virtual keyboard |
//! | [`VirtualKeyboard`] | `secure-types` | On-screen keyboard writing into a `SecureString` |
//! | [`QrImage`] | `qr-image` | Encode text as a PNG QR `egui::Image` |
//! | [`QRScanner`] | `qr-scanner` | Overlay that captures the monitor and decodes a QR (Linux and Windows) |
//!
//! `secure-types` and `qr-scanner` also enable `lucide`. `qr-scanner` also
//! pulls in `secure-types` (decoded payload is a `SecureString`). Enabling
//! `qr-scanner` on an unsupported target compiles the crate but does not
//! export [`QRScanner`].

#[cfg(feature = "secure-types")]
mod input_field;

#[cfg(all(feature = "qr-scanner", any(target_os = "linux", target_os = "windows")))]
mod qr_scanner;

#[cfg(feature = "secure-types")]
mod virtual_keyboard;

#[cfg(feature = "qr-image")]
mod qr_image;

#[cfg(feature = "secure-types")]
mod credentials_form;

#[cfg(feature = "secure-types")]
pub use secure_types;

#[cfg(feature = "secure-types")]
pub use input_field::SecureInputField;

#[cfg(all(feature = "qr-scanner", any(target_os = "linux", target_os = "windows")))]
pub use qr_scanner::QRScanner;

#[cfg(feature = "secure-types")]
pub use virtual_keyboard::VirtualKeyboard;

#[cfg(feature = "qr-image")]
pub use qr_image::QrImage;

#[cfg(feature = "secure-types")]
pub use credentials_form::CredentialsForm;
