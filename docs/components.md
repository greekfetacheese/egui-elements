
# Components

Higher-level composites that sit on top of the widgets and are **feature-gated**: nothing here costs a
build unless you ask for it.

| Type | Feature | What it is |
|---|---|---|
| `CredentialsForm` | `secure-types` | username / password / confirm, with an optional virtual keyboard |
| `InputField` / `SecureInputField` | `secure-types` | one-line field with trailing icon buttons; the secure one masks |
| `VirtualKeyboard` | `secure-types` | on-screen keyboard writing into a `SecureString` |
| `QrImage` | `qr-image` | encode text as a QR `egui::Image` |
| `QRScanner` | `qr-scanner` | screen-capture QR scanner (Linux and Windows) |

`secure-types` and `qr-scanner` also enable `lucide` (SVG icons). `qr-scanner` pulls in `secure-types`,
because the decoded payload is a `SecureString`.

## CredentialsForm

Show/hide (and the QR-scan button) use [Lucide](https://lucide.dev) SVG icons
via [`egui-lucide`](https://crates.io/crates/egui-lucide), colored with
`theme.colors.text`. Register egui's SVG loader once at startup:

```rust
egui_extras::install_image_loaders(&ctx);
```

`egui_extras` needs the `svg` feature. The `lucide` feature is enabled
automatically by `secure-types` and `qr-scanner`.

```rust
use egui_elements::components::CredentialsForm;

let mut form = CredentialsForm::new()
    .with_open(true)
    .with_confirm_password(true);

form.show(ui);

let username = form.username();
let password = form.password();
form.erase(); // wipe the SecureString fields
```
## InputField / SecureInputField

A single-line field with optional trailing icon buttons (show/hide for secrets, a QR scanner). The
secure variant owns a `SecureString` and can mask; `InputField` owns a plain `String` and never masks.

**Signature** `InputField::new(id, open)` · `SecureInputField::new(id, masked, open)`

```rust
use egui_elements::{InputField, SecureInputField};

struct App {
    search: InputField,
    password: SecureInputField,
}

// Store the fields on the app, not in the frame body:
let mut app = App {
    search: InputField::new("search", true).hint_text("Search"),
    // id, masked, open
    password: SecureInputField::new("password", true, true),
};

app.search.show(ui);
app.password.show(ui);
```

- Both constructors take the open state: `false` starts the field hidden and `show` returns
  `None`. `.open()`, `.is_open()` and `.close()` drive it, and `show` returns
  `Option<SecureTextEditOutput>`.
- `id` is the field-name label; an **empty `id` skips the label**, so pass `""` when the host already
  labels the field (for example under a `Modal` heading).
- The field **owns its buffer** — no lifetime parameter, so it can live in a `'static` host and
  must be stored there (a field rebuilt every frame resets the QR scanner it owns). Buffer access:
  `text() -> &str`, `set_text(..)`, `erase()`; `set_min_size(..)`, `set_inner_margin(..)` and
  `set_id(..)` are the between-frame builders.
- Row width is `min_size.x` **plus one icon button and one gap per trailing icon**; the component
  reserves the icons itself.
- Masking, the `Eye` / `EyeOff` toggle and `is_text_hidden` / `set_text_hidden` exist only on
  `SecureInputField`. Needs `secure-types`; the QR button also needs `qr-scanner`.
## VirtualKeyboard

An on-screen keyboard that writes into a `SecureString` — the "type a password on a touch kiosk" case.
`CredentialsForm` mounts one for you; use it directly when the field is your own.

**Signature** `VirtualKeyboard::new(open).with_key_size(Vec2).with_button_padding(Vec2).with_text_size(f32).with_spacing(Vec2).with_max_ui_size(Vec2)` then `.show(&mut SecureString, ui)`

```rust
use egui::vec2;
use egui_elements::components::secure_types::SecureString;
use egui_elements::VirtualKeyboard;

let mut password = SecureString::new_with_capacity(32).unwrap();

VirtualKeyboard::new(true)
    .with_key_size(vec2(34.0, 34.0))
    .with_text_size(18.0)
    .show(&mut password, ui);
```

Defaults: `30×30` keys, `4×4` padding, text size `16`, spacing `10×10`. Needs `secure-types`. Key rows
shrink-wrap their content — do not build them with `ui.horizontal`.
## QrImage

Use a unique `bytes://…` URI per image so egui's loader cache does not reuse the wrong PNG.

```rust
use egui_elements::components::QrImage;

let mut qr = QrImage::new("ethereum:0x…", "bytes://recv-address".to_owned());
if !qr.has_error() {
    ui.add(qr.image());
}
// When the image leaves the screen, forget it so the bytes can be wiped:
qr.clear(ui.ctx());
```
## QRScanner

```rust
use egui_elements::components::QRScanner;

let mut scanner = QRScanner::new();
scanner.open(ctx.clone());
scanner.show(ctx);

if let Some(payload) = scanner.get_result() {
    scanner.reset();
    // `payload` is a SecureString
}
```

The scanner captures the monitor under the cursor via [`xcap`](https://crates.io/crates/xcap) and decodes with [`rqrr-zeroize`](https://crates.io/crates/rqrr-zeroize) (a fork of `rqrr` that wipes the prepared image buffer on drop). Exported on Linux and Windows, not macOS.

**Where the pixels go:**

- **Linux X11** — in-memory (XCB). Prefer this for sensitive codes.
- **Linux Wayland** — `xcap` asks GNOME Shell or the XDG desktop portal, which writes a PNG under the temp dir and then deletes it. That file exists for a brief moment.
- **Windows** — in-memory GDI (`BitBlt` / `GetDIBits`). No screenshot temp file. (This crate does not enable xcap’s optional `wgc` feature; that path is also in-memory.)
## QR scanner system packages

QR scanner uses the `xcap` crate. You may have to install the following dependencies:

**Debian/Ubuntu:**

```sh
apt-get install pkg-config libclang-dev libxcb1-dev libxrandr-dev libdbus-1-dev libpipewire-0.3-dev libwayland-dev libegl-dev
```

**Alpine:**

```sh
apk add pkgconf llvm19-dev clang19-dev libxcb-dev libxrandr-dev dbus-dev pipewire-dev wayland-dev mesa-dev
```

**Arch Linux:**

```sh
pacman -S base-devel clang libxcb libxrandr dbus libpipewire
```

If it still doesn't compile, try installing the following packages:

```sh
apt install libgbm-dev libdrm-dev libgl1-mesa-dev
```
