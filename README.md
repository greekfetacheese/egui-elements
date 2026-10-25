# egui-elements

Themes, widgets, and components for [egui](https://github.com/emilk/egui).

The goal of this crate is to make it easy to create nice
looking apps with egui.

egui-elements is far from perfect and there may be breaking changes to the color palettes as egui evolves.

This crate is **not** a drop-in replacement for stock egui widgets. It ships:

- A widget set (`Button`, `Label`, `ComboBox`, `SecureTextEdit`, `Modal`, `MultiLabel`, `Window`) that reads visuals from an installed theme, corner `Badges` for buttons, and hold-to-confirm buttons with a configurable progress sweep and completion burst
- `Snow` and `Light` particle effects with drop-in, themed settings windows (`Snow` ported from [egui-snow](https://github.com/LaVashikk/egui-snow), MIT)
- Seven built-in palettes (`ThemeKind`) plus a live `ThemeEditor` (WIP)
- Feature-gated composites: credentials form, QR image, Linux QR scanner

`Button`, `ComboBox`, and `Window` have the same names as the egui types. Import this crate's widgets *after* `use eframe::egui::*`, or alias one of them.

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
use egui_elements::{Theme, ThemeKind};
use egui_elements::Button;

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
use egui_elements::{Theme, ThemeKind};
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
use egui_elements::{Button, ComboBox, Label, Modal, SecureTextEdit, Window};

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

### Badges

Attach a notification `Badge` to a corner of a `Button` to signal unseen content
behind it - e.g. the number changes:

```rust
use egui_elements::{Badge, BadgeCorner, Button};

let changes = 3;
ui.add(
    Button::new("Notifications")
        .badge(Badge::new(changes.to_string()).color(theme.colors.error)),
);
```

`Badge::new(text)` shows that text in the top-right corner, filled with the theme
error color; `Badge::dot()` is a plain coloured dot. `.corner(BadgeCorner::TopLeft)` /
`TopRight` / `BottomLeft` / `BottomRight` picks the corner, `.color(...)` and
`.text_color(...)` override the fill and glyph colors (text defaults to
`theme.colors.bg` so it stays readable on chromatic fills).

The badge is an overlay: it is painted on top of the button and never changes
its size or layout.

### Hold-to-confirm buttons

`Button::hold(seconds)` turns a button into a hold-to-confirm action: it fills a
left-to-right sweep for `seconds` and only then reports
[`clicked()`](https://docs.rs/egui/latest/egui/struct.Response.html#method.clicked).

> **`clicked()` means the hold completed.** A plain click — press and release
> under `seconds` — never reports `clicked()`. Use it for destructive or
> irreversible actions where a misclick must not fire.

```rust
use egui_elements::{Button, HoldBurst, HoldSettings};

if ui.add(Button::new("Delete vault").hold(1.0)).clicked() {
    // only reached once the sweep has filled
}

// One struct carries the duration, the progressive fill and the burst.
let mut settings = HoldSettings {
    seconds: 0.8,
    fill_from: Some(theme.colors.error),   // None = the installed theme's accent
    fill_to: Some(theme.colors.warning),   // shift as the hold fills; None = solid
    burst: HoldBurst::Lightning,           // Off / Glow (default) / Lightning
    ..Default::default()
};
settings.open = true;                      // draw the themed settings window
settings.show_window(ui.ctx());            // …or just `body_ui(ui)` in your own panel
ui.add(Button::new("Strike").hold_settings(&settings));

// The per-field shortcuts compose in call order instead:
ui.add(
    Button::new("Strike")
        .hold(0.8)
        .hold_fill(theme.colors.error)
        .hold_burst(HoldBurst::Lightning),
);
```

`HoldSettings` also carries `burst_color` (`None` = the resolved fill colour),
`burst_intensity` (`0.0..=1.0`, a multiplier over the effect's own profile — `0`
disables the burst) and `burst_ms`. For `HoldBurst::Lightning` it carries
`burst_bolts`, `burst_branch_depth`, and the effect's `burst_thickness` /
`burst_jitter` (defaults `2.0` and `0.18`, the effect's own). For
`HoldBurst::Glow` it shapes the fan with `burst_glow_from` / `burst_glow_to`
(the start/end radius as a multiple of the button's shorter side, default
`1.0` → `2.2`, with `to` clamped to at least `from`) and `burst_glow_softness`
(`1.0` = the shared profile exactly; above `1.0` spreads the falloff, below
`1.0` tightens it). Its defaults (`1 s`, theme accent, a solid fill and a
300 ms `Glow`) reproduce a button that configures nothing.

Press and release under the duration resets the sweep and never fires; dragging
the pointer off the button resets it too. The widget tracks the press itself, so
holds longer than egui's `max_click_duration` (0.8 s by default) still complete;
on a touch screen egui may also report a long touch as a secondary-click, which
this widget leaves alone. A focused button confirms immediately
on Enter or Space, matching egui's keyboard activation. `seconds <= 0.0` behaves
as if `hold` was never called. When the hold fires, the fill clears and the burst
takes over; `HoldBurst::Glow` grows a soft glow at the button's centre (the
`burst_glow_from` / `burst_glow_to` / `burst_glow_softness` knobs set its shape
and falloff), `HoldBurst::Lightning` strikes a bolt across it (configurable
thickness and jitter). The burst fizzles out on its own and requests no repaints
once idle.

### Particles: Snow

`Snow` draws a falling-snow effect on an egui layer, so it never disturbs your
layout. `SnowSettings` collects the public knobs and opens a themed settings
window with `show_window`; `Snow::from_settings` turns them into a configured
effect. The effect is ported from [egui-snow](https://github.com/LaVashikk/egui-snow)
(MIT, © 2025 LaVashikk) and keeps the upstream algorithm and defaults.

```rust
use egui_elements::particles::{Snow, SnowSettings};

let mut settings = SnowSettings::default();
settings.open = true;

// every frame:
settings.show_window(ctx);                              // themed floating settings window + scroll
if settings.active {
    Snow::from_settings("snow", &settings).show(ui);    // layout-neutral (rendered on a layer)
}
```

Prefer your own container? `SnowSettings::body_ui(&mut settings, ui)` draws just the
controls (no window chrome) inside a `SidePanel`, `Window`, or `ScrollArea` — wrap it in
an `egui::ScrollArea` when that container can be shorter than the controls.

```rust
egui::SidePanel::right("snow").show(ctx, |ui| {
    settings.body_ui(ui);
});
```

### Particles: Light

`Light` paints soft radial glow motes and rising embers on an egui layer, so it
never disturbs your layout. `LightSettings` collects the public knobs and opens a
themed settings window with `show_window`; `Light::from_settings` turns them into
a configured effect. `LightMotion` picks the behaviour: `Drift` wanders and
bounces off the area edges, `Rise` floats embers up from the bottom edge.

```rust
use egui_elements::particles::{Light, LightSettings};

let mut settings = LightSettings::default();
settings.open = true;

// every frame:
settings.show_window(ctx);                              // themed floating settings window + scroll
if settings.active {
    Light::from_settings("light", &settings).show(ui);  // layout-neutral (rendered on a layer)
}
```

Prefer your own container? `LightSettings::body_ui(&mut settings, ui)` draws just
the controls (no window chrome), exactly like `SnowSettings::body_ui`.

### Particles: Lightning

`Lightning` paints a jagged bolt plus a full-area flash — the "room lighting up"
— on an egui layer, so it never disturbs your layout. A strike is episodic: it
lives for a configurable duration and flickers through several sub-flashes.
`LightningSettings` collects the public knobs and opens a themed settings window
with `show_window`; `Lightning::from_settings` turns them into a configured
effect.

Trigger a strike yourself with `Lightning::strike`, or set
`ambient_interval_secs` and let the scheduler fire on its own. Use one
`id_source` for `show` and `strike` so both address the same state; the settings
window's "Strike now" button uses `LightningSettings::STRIKE_ID`.

```rust
use egui_elements::particles::{Lightning, LightningSettings};

let mut settings = LightningSettings::default();
settings.open = true;

// every frame:
settings.show_window(ctx);                              // themed floating settings window + scroll
if settings.active {
    Lightning::from_settings(LightningSettings::STRIKE_ID, &settings).show(ui);
}

// on a host event (button click, timer, …):
Lightning::from_settings(LightningSettings::STRIKE_ID, &settings).strike(ctx);
```

Prefer your own container? `LightningSettings::body_ui(&mut settings, ui)` draws
just the controls (no window chrome), exactly like the other effects.

While a strike animates the effect requests a repaint; when idle it requests
nothing (ambient off) or wakes exactly for the next ambient strike, so it is safe
to leave running in a battery-sensitive host.

### Credentials form (`secure-types`)

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

### QR scanner (`qr-scanner`, Linux and Windows)

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

## Features

| Feature | Default | What it enables |
|---------|:-------:|-----------------|
| `serde` | | `Serialize` / `Deserialize` on theme types, plus `egui/serde` |
| `lucide` | | [`egui-lucide`](https://crates.io/crates/egui-lucide) SVG icons (`Eye` / `EyeOff` / `ScanQrCode`). Also enabled by `secure-types` and `qr-scanner`. Host apps must call `egui_extras::install_image_loaders` with the `svg` feature. |
| `secure-types` | | [`secure-types`](https://crates.io/crates/secure-types) (`SecureString`) buffers; `SecureInputField`, `CredentialsForm`, `VirtualKeyboard`. Enables `lucide`. |
| `qr-image` | | Encode a string as a PNG QR `egui::Image` (`qrcodegen-no-heap` + `image`) |
| `qr-scanner` | | Linux/Windows screen-capture QR scanner (`xcap` + `enigo` + `rqrr-zeroize`). Also enables `secure-types` (the dependency) and `lucide`. The `QRScanner` type is exported on Linux and Windows. |
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

Capability gallery (palette, widgets, components):

```sh
cargo run --example demo --features full
```

Product-framed placeholder (mail client UI):

```sh
cargo run --example email
```

Interactive snow with the public settings window:

```sh
cargo run --example particles
```

Interactive light with the public settings window:

```sh
cargo run --example light
```

Interactive lightning with the public settings window and an ambient scheduler:

```sh
cargo run --example lightning
```

Hold-to-confirm buttons sharing one `HoldSettings` window (fill shift, bursts):

```sh
cargo run --example hold_to_confirm
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

MIT. Icons from the `lucide` feature are [Lucide](https://lucide.dev) (ISC).

The Snow particle effect is ported from [egui-snow](https://github.com/LaVashikk/egui-snow) (MIT, © 2025 LaVashikk).
