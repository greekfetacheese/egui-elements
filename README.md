# egui-elements

Themes, widgets, and components for [egui](https://github.com/emilk/egui).

Make a good-looking egui app without rebuilding the same chrome in every project: a themed widget set,
seven palettes with a live editor, a flexbox-style layout engine, eased motion, particle effects, and a
few feature-gated composites.

It is **not** a drop-in replacement for stock egui widgets. This crate's widgets read their chrome from
an installed `Theme`, and they are builder-shaped.

`Button`, `ComboBox`, `Window`, `Spinner` and `Checkbox` share their names with egui's types: import this
crate's widgets *after* `use eframe::egui::*`, or alias one of the two.

## What's in it

| Area | What you get |
|---|---|
| **Widgets** | `Button` (+ hold-to-confirm), `Badge`, `Label` / `MultiLabel`, `Row`, `ComboBox`, `Menu` / `MenuItem` / `SubMenuItem` / `MenuSection`, `Chip`, `Divider`, `Slider`, the toggle family, `Tabs`, `AvatarGroup`, `Indicator`, `FpsMeter`, `Modal`, `Window`, `Popover`, `Toast` / `Toasts`, and the `Spinner` / `Ripple` / `Orb` loaders → [docs/widgets.md](docs/widgets.md) |
| **Layout** | a CSS-flexbox subset (`Flex`, `Item`) and a data-driven `Table` with `fixed` / `auto` / `fr` columns, a pinned header and row dividers → [docs/layout.md](docs/layout.md) |
| **Motion** | eased values (`animate`, `Motion`, `Easing`), interpolation (`Lerp`), and the `Fade` / `Collapse` containers → [docs/motion.md](docs/motion.md) |
| **Particles** | `Snow`, `Light` and `Lightning`, each with a themed settings window → [docs/particles.md](docs/particles.md) |
| **Components** | credentials form, virtual keyboard, secure / plain input fields, QR image, QR scanner — feature-gated → [docs/components.md](docs/components.md) |
| **Theme** | seven palettes, the `ThemeColors` roles, and a live editor → [docs/theme.md](docs/theme.md) · [docs/editor.md](docs/editor.md) |
| **Utils** | theme switcher, colour mixing, WCAG contrast, HSLA → [docs/utils.md](docs/utils.md) |
| **Fills** | a fill slot holds a solid colour *or* a two-stop `Gradient` → [docs/gradient.md](docs/gradient.md) |

The reference pages under [`docs/`](docs/README.md) are the full usage: one page per area, every entry
with its constructor and a runnable example.

## Installation

```toml
[dependencies]
egui-elements = "1.4"
```

With every optional feature (what the showcase demo needs):

```toml
egui-elements = { version = "1.4", features = ["full"] }
```

Built against egui 0.37. Enable only the features you need — nothing below is on by default:

| Feature | What it enables |
|---------|-----------------|
| `serde` | `Serialize` / `Deserialize` on theme types, plus `egui/serde` |
| `lucide` | [`egui-lucide`](https://crates.io/crates/egui-lucide) SVG icons (`Eye` / `EyeOff` / `ScanQrCode`). Also enabled by `secure-types` and `qr-scanner`. Host apps must call `egui_extras::install_image_loaders` with the `svg` feature. |
| `secure-types` | [`secure-types`](https://crates.io/crates/secure-types) (`SecureString`) buffers; `SecureInputField`, `CredentialsForm`, `VirtualKeyboard`. Enables `lucide`. |
| `qr-image` | encode a string as a PNG QR `egui::Image` (`qrcodegen-no-heap` + `image`) |
| `qr-scanner` | Linux/Windows screen-capture QR scanner (`xcap` + `enigo` + `rqrr-zeroize`). Also enables `secure-types` and `lucide`; `QRScanner` is exported on Linux and Windows only. System packages: [docs/components.md](docs/components.md#qr-scanner-system-packages). |
| `elegance` | convert the current theme into [`egui-elegance`](https://crates.io/crates/egui-elegance) so elegance widgets match |
| `full` | all of the above |

Widgets, themes, the editor and the layout / motion engines work with **no features** enabled.

## Quick start

```rust
use egui::{Context, Ui};
use egui_elements::{Button, Theme, ThemeKind};

fn ui(ctx: &Context, ui: &mut Ui) {
    let mut theme = Theme::new(ThemeKind::TokyoNight);
    theme.install(ctx);

    if ui.add(Button::new("Save")).clicked() {
        // …
    }
}
```

`Theme::install` writes the `egui::Style` and the per-widget visuals onto the context. The crate's
widgets read them from there, so a widget follows the installed theme without being told; a single widget
can still override through `.visuals(..)`.

A fill that is not a surface — a primary button in `accent`, a destructive one in `error` — needs its own
ink. `ButtonVisuals::filled(fill, colors)` supplies it: the label becomes `colors.ink_on(fill)`, so the
copy clears AA on every theme without a second palette.

```rust
use egui_elements::Button;

let primary = theme.button_visuals().filled(theme.colors.accent, theme.colors);
let danger = theme.button_visuals().filled(theme.colors.error, theme.colors);

ui.add(Button::new("Send").visuals(primary));
ui.add(Button::new("Delete").visuals(danger));
```

## Themes

Seven palettes ship as `ThemeKind`:

| Kind | Notes |
|------|-------|
| `TokyoNight` | Dark. [tokyo-night-vscode-theme](https://github.com/tokyo-night/tokyo-night-vscode-theme) |
| `TokyoNightLight` | Light variant of the same |
| `McLaren650Gts` | 2015 McLaren 650S GT3 livery |
| `Reverie` | [Reverie Obsidian theme](https://github.com/santiyounger/Reverie-Obsidian-Theme) |
| `ShadeSanctuary` | [Shade Sanctuary](https://github.com/Elevict/Shade-Sanctuary) |
| `Wasp` | Dark [Wasp Obsidian theme](https://github.com/santiyounger/Wasp-Obsidian-Theme) |
| `WaspLight` | Light variant of the same |
| `Custom(String)` | runtime-defined: the persisted `Theme` is the definition, the string is its name |

A theme is a palette plus geometry — spacing, typography, frame chrome — and every widget is painted from
the palette's named slots:

| Slot | Job |
|------|-----|
| `bg` | page background |
| `widget_bg` | raised surface: a card, a field, a button |
| `hover` | pointer over a surface |
| `highlight` | chosen / selected fill |
| `text` / `text_muted` | body ink / secondary ink |
| `border` | idle outline |
| `accent` | focus, the active indicator, links |
| `error` / `warning` / `success` / `info` | semantics |
| `title_bar` | window title strip |

Four rules keep a palette coherent — they are exactly what a hand-made theme gets wrong:

- **`highlight` is not `hover`.** Hover means "the pointer is here"; highlight means "this is the choice".
- **Elevation is the shadow, not a lighter `bg`.** Do not widen the `bg` / `widget_bg` contrast to fake
  depth; nesting is described by surface role, and the shadow carries it.
- **`text_muted` is secondary text**, never a selection fill and never a hover border.
- **A chromatic fill carries its own ink** — `ink_on(fill)`, not `text`.

Live palette switching and editing:

```rust
use egui_elements::editor::ThemeEditor;
use egui_elements::utils::theme_switcher;
use egui_elements::{Theme, ThemeKind};

fn chrome(theme: &mut Theme, editor: &mut ThemeEditor, ui: &mut egui::Ui) {
    if let Some(new_theme) = theme_switcher(theme, ui) {
        *theme = new_theme;
    }
    editor.show(theme, ui);
    theme.install(ui.ctx());
}
```

`theme_switcher` installs the palette you pick and returns it. `ThemeEditor::show` mutates the theme in
place — palette, frames, widget states, typography, spacing, a contrast audit and a live preview — so
install **after** it, every frame. Save / Load buttons appear once the host wires `ThemeEditor::set_io`:
the crate ships no dialog and no serializer dependency, and the demo wires `rfd` + `serde_json`.

A customized theme keeps its own name instead of pretending to be a built-in, and with the `serde` feature
the whole `Theme` round-trips:

```rust
let mut theme = Theme::new(ThemeKind::TokyoNight);
theme.colors.accent = egui::Color32::from_rgb(255, 91, 103);
theme.rename("Sunset");

let json = serde_json::to_string(&theme)?; // "kind":{"Custom":"Sunset"}
let mut restored: Theme = serde_json::from_str(&json)?;
restored.install(&ctx);
```

`ThemeKind::Custom` is an identity only: `Theme::new` cannot rebuild a palette from a name (it falls back
to Tokyo Night; `Theme::try_new` returns `None`). The contract behind every slot — surfaces, widget
states, contrast policy, how to add a palette — is [docs/theme.md](docs/theme.md); the editor's panels are
[docs/editor.md](docs/editor.md).

## Demos

| Example | What it proves | Command |
|---|---|---|
| `demo` | the capability gallery: palette swatches, every widget on `bg` / `frame1` / a `Window`, the editor | `cargo run --example demo --features full` |
| `email` | product chrome: a mail UI built on `Flex`, spacing tokens, no magic gaps | `cargo run --example email` |
| `telegram` | a two-pane `Flex` shell with overlays and a live theme-audit panel | `cargo run --features secure-types --example telegram` |
| `layout` | six flex / table shapes that used to be hand-rolled arithmetic | `cargo run --example layout` |
| `motion` | `animate`, `Fade`, `Collapse` on one screen | `cargo run --example motion` |
| `particles` / `light` / `lightning` | each particle effect with its settings window | `cargo run --example particles` |
| `popover` | anchored picker, action menu and above-side inspector | `cargo run --example popover` |
| `gradient` | `Fill` / `Gradient` on frames and buttons | `cargo run --example gradient` |

Every widget also has its own proof example — `checkbox`, `radio`, `switch`, `slider`, `tabs`,
`toast`, `badge`, `avatar`, `indicator`, `spinner`, `ripples`, `orb`, `popover`, `menu`, `combo_box`,
`text_edit`, `hold_to_confirm`, `label_sections`, `fps_meter`. The table in
[docs/widgets.md](docs/widgets.md) maps each entry to its example.

## License

MIT. Icons from the `lucide` feature are [Lucide](https://lucide.dev) (ISC).

The `Menu` family is adapted from [egui-elegance](https://github.com/matrix-research-inc/egui-elegance)
(MIT OR Apache-2.0, © Stephen Berry).

The Snow particle effect is ported from [egui-snow](https://github.com/LaVashikk/egui-snow) (MIT, © 2025 LaVashikk).

The `Ripple` loader is ported from [alexruix / fluffy-starfish-52](https://uiverse.io/alexruix/fluffy-starfish-52) (MIT).

The `Orb` loader is ported from [dexter-st / bright-lizard-8](https://uiverse.io/dexter-st/bright-lizard-8) (MIT).

The `Toast` and `Indicator` widgets are forked from [egui-elegance](https://github.com/stephenberry/egui-elegance) (MIT OR Apache-2.0, © 2026 Stephen Berry).
