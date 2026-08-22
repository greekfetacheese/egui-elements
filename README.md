# egui-elements

Themes, widgets, and components for [egui](https://github.com/emilk/egui).

The goal of this crate is to make it easy to create nice
looking apps with egui.

egui-elements is far from perfect and there may be breaking changes to the color palettes as egui evolves.

This crate is **not** a drop-in replacement for stock egui widgets. It ships:

- A widget set (`Button`, `Label`, `ComboBox`, `SecureTextEdit`, `Modal`, `MultiLabel`) that reads visuals from an installed theme
- Seven built-in palettes (`ThemeKind`) plus a live `ThemeEditor` (WIP)
- Feature-gated composites: credentials form, QR image, Linux QR scanner

There are **no crate-root re-exports**. Import from the modules:

```rust
use egui_elements::theme::{Theme, ThemeKind};
use egui_elements::widgets::{Button, ComboBox, Label, Modal, SecureTextEdit};
use egui_elements::utils::theme_switcher;
use egui_elements::editor::ThemeEditor;
```

`Button` and `ComboBox` have the same names as the egui types. Import this crate's widgets *after* `use eframe::egui::*`, or alias one of them.

## Installation

```toml
[dependencies]
egui-elements = "0.1"
```

Enable only the features you need (see [Features](#features)). For the showcase demo:

```toml
egui-elements = { version = "0.1", features = ["full"] }
```

## Quick start

```rust
use egui::{Context, Ui};
use egui_elements::theme::{Theme, ThemeKind};
use egui_elements::widgets::Button;

fn ui(ctx: &Context, ui: &mut Ui) {
    let mut theme = Theme::new(ThemeKind::TokyoNight);
    theme.install(ctx);

    if ui.add(Button::new("Save")).clicked() {
        // …
    }
}
```

`Theme::install` writes the `egui::Style` and the per-widget visuals onto `ctx`. Custom widgets then pick them up automatically. You can still override a single widget with `.visuals(...)`.

Always call `Theme::install` if you switch themes so egui sees the new look:

```rust
use egui_elements::editor::ThemeEditor;
use egui_elements::theme::{Theme, ThemeKind};
use egui_elements::utils::theme_switcher;

fn chrome(theme: &mut Theme, editor: &mut ThemeEditor, ui: &mut egui::Ui) {
    if let Some(new_theme) = theme_switcher(theme, ui) {
        *theme = new_theme;
    }
    editor.show(theme, ui);
    theme.install(ui.ctx());
}
```

## Usage

### Widgets

```rust
use egui::{Sense, Ui};
use egui_elements::widgets::{Button, ComboBox, Label, Modal, SecureTextEdit};

fn gallery(ui: &mut Ui, text: &mut String, modal_open: &mut bool) {
    ui.add(Button::new("Primary"));
    ui.add(Label::new("Hint", None).interactive(false));

    let current = Label::new(text.as_str(), None);
    ComboBox::new("item", current).width(200.0).show_ui(ui, |ui| {
        for item in ["Alpha", "Beta", "Gamma"] {
            let row = Label::new(item, None)
                .sense(Sense::click())
                .fill_width(true)
                .expand(Some(4.0));
            if ui.add(row).clicked() {
                *text = item.to_owned();
            }
        }
    });

    ui.add(SecureTextEdit::singleline(text).hint_text("Name").password(false));

    Modal::new("confirm", modal_open)
        .heading("Discard changes?")
        .show(ui.ctx(), |ui| {
            ui.label("This cannot be undone.");
        });
}
```

Give each `ComboBox` / `SecureTextEdit` a unique `id_salt` if you repeat the same gallery on several surfaces (bg, frame, window). Otherwise their persistent state collides.

### Credentials form (`secure-types`)

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

### QR image (`qr-image`)

Use a unique `bytes://…` URI per image so egui's loader cache does not reuse the wrong PNG.

```rust
use egui_elements::components::QrImage;

let qr = QrImage::new("ethereum:0x…", "bytes://recv-address".to_owned());
if !qr.has_error() {
    ui.add(qr.image());
}
// When the image leaves the screen, forget it so the bytes can be wiped:
qr.clear(ui.ctx());
```

### QR scanner (`qr-scanner`, Linux only)

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

The scanner captures the monitor under the cursor via [`xcap`](https://crates.io/crates/xcap) and decodes with [`rqrr-zeroize`](https://crates.io/crates/rqrr-zeroize) (a fork of `rqrr` that wipes the prepared image buffer on drop). Prefer X11; on Wayland a screenshot temp file can exist for a brief moment.

## Features

| Feature | Default | What it enables |
|---------|:-------:|-----------------|
| `serde` | | `Serialize` / `Deserialize` on theme types, plus `egui/serde` |
| `secure-types` | | [`secure-types`](https://crates.io/crates/secure-types) (`SecureString`) buffers; `SecureInputField`, `CredentialsForm`, `VirtualKeyboard` |
| `qr-image` | | Encode a string as a PNG QR `egui::Image` (`qrcodegen-no-heap` + `image`) |
| `qr-scanner` | | Linux screen-capture QR scanner (`xcap` + `enigo` + `rqrr-zeroize`). Also enables `secure-types`. The `QRScanner` type is exported only on Linux. |
| `elegance` | | Convert the current theme into [`egui-elegance`](https://crates.io/crates/egui-elegance) so elegance widgets match the theme |
| `full` | | All of the above |

Widgets, themes, the editor, and the overlay work with **no features** enabled.

## QR scanner system dependencies

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

## Demo

```sh
cargo run --example demo --features full
```

## Built-in themes

| `ThemeKind` | Notes |
|-------------|--------|
| `TokyoNight` | Dark. [tokyo-night-vscode-theme](https://github.com/tokyo-night/tokyo-night-vscode-theme) |
| `TokyoNightLight` | Light variant of the same |
| `McLaren650Gts` | 2015 McLaren 650S GT3 livery |
| `Reverie` | [Reverie Obsidian theme](https://github.com/santiyounger/Reverie-Obsidian-Theme) |
| `ShadeSanctuary` | [Shade Sanctuary](https://github.com/Elevict/Shade-Sanctuary) |
| `Wasp` | Dark [Wasp Obsidian theme](https://github.com/santiyounger/Wasp-Obsidian-Theme) |
| `WaspLight` | Light variant of the same |

Palette slot roles (`bg` vs `widget_bg`, `hover` vs `highlight`, …) are documented in [`docs/theme.md`](docs/theme.md).

## License

MIT
