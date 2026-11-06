# Utils

Small helpers the crate itself uses: the theme switcher, colour mixing, WCAG contrast, HSLA conversion,
and the palette-remap helpers for host structs that cache theme colours.

```rust
use egui_elements::utils::{
    Hsla, ShadeDirection, TINT_1, contrast_ratio, frame, frame_palette_changed, mix, remap_if_eq,
    remap_frame, remap_frame_visuals, theme_switcher,
};
```

## `theme_switcher`

A `ComboBox` listing every built-in `ThemeKind`, sized to the longest palette name so `McLaren 650GTS`
and `Shade Sanctuary` are not truncated. Picking one installs it and returns it; `None` means nothing
changed this frame.

**Signature** `theme_switcher(&theme, ui) -> Option<Theme>`

```rust
use egui_elements::utils::theme_switcher;

if let Some(theme) = theme_switcher(&self.theme, ui) {
    self.theme = theme;
}
```

Only built-in kinds are offered: a `ThemeKind::Custom` shows in the button but cannot be rebuilt from a
name, so the app reinstalls its own serialized theme instead.

## `mix`

Mix two colours in **unpremultiplied** sRGB: `t = 0` returns `a`, `t = 1` returns `b`, and alpha is
interpolated too. Channels mix gamma-encoded, which is what palette tinting wants — halfway between a
role colour and the canvas reads as halfway.

**Signature** `mix(a: Color32, b: Color32, t: f32) -> Color32` (clamps `t`)

```rust
let tinted = egui_elements::mix(theme.colors.widget_bg, theme.colors.accent, 0.15);
```

## `contrast_ratio`

WCAG 2.2 contrast ratio between two opaque colours, from 1.0 to 21.0. Alpha is ignored: composite a
translucent colour over its surface first. The two thresholds the crate audits against are exported as
`CONTRAST_AA_TEXT` (4.5, body text) and `CONTRAST_AA_UI` (3.0, large text and component boundaries).

**Signature** `contrast_ratio(a: Color32, b: Color32) -> f32`

```rust
use egui_elements::utils::{CONTRAST_AA_TEXT, contrast_ratio};

let ok = contrast_ratio(theme.colors.text, theme.colors.bg) >= CONTRAST_AA_TEXT;
```

## `Hsla` / `ShadeDirection` / `TINT_1`

HSLA in the shape the theme editor edits it: `h` 0..=360, `s` / `l` 0..=100, `a` 0..=1. Conversion
premultiplies on the way back to `Color32`, so a colour round-trips without drifting.

**Signature** `Hsla::from_color32(c)` · `Hsla::from_hex("#RRGGBB")` · `.to_color32()` ·
`.to_rgba_components()` · `.shades(n, ShadeDirection::{Lighter, Darker})`

```rust
use egui_elements::utils::{Hsla, ShadeDirection};

let base = Hsla::from_hex("#7aa2f7").unwrap();
let ramps = base.shades(5, ShadeDirection::Darker); // 5%, 10%, … below the base
assert_eq!(ramps[0], base.to_color32());
```

`TINT_1` is a neutral overlay colour (216, 216, 216) that stays legible on very dark imagery.

## `frame_palette_changed` / `remap_*`

A host that caches theme colours in its own structs (an `egui::Frame` built once, a `FrameVisuals`
stored beside it) has to follow a live palette change without losing the user's own overrides. These
helpers rewrite a slot **only if it still holds the old palette value**, so a hand-picked colour
survives.

```rust
use egui_elements::utils::{frame_palette_changed, remap_frame, remap_frame_visuals};

if frame_palette_changed(&old_colors, &theme.colors) {
    remap_frame(
        &mut self.card,
        old_colors.widget_bg,
        theme.colors.widget_bg,
        old_colors.border,
        theme.colors.border,
    );
    remap_frame_visuals(
        &mut self.card_visuals,
        old_colors.hover,
        theme.colors.hover,
        old_colors.hover,
        theme.colors.hover,
        old_colors.highlight,
        theme.colors.highlight,
    );
}
```

- `frame_palette_changed(old, new)` — `true` if any frame-related slot (`title_bar`, `bg`, `widget_bg`,
  `hover`, `highlight`, `border`) changed.
- `remap_if_eq(slot, old, new)` — the one-slot primitive behind the rest.
- `remap_frame(frame, old_fill, new_fill, old_border, new_border)` — fill, stroke and shadow on an
  `egui::Frame`.
- `remap_frame_visuals(visuals, old_hover, new_hover, old_click, new_click, old_highlight,
  new_highlight)` — the hover / click / selected fills and borders on a `FrameVisuals`.

## `frame`

Paint a frame with hover and click fills taken from `FrameVisuals`, and run a closure inside it. Returns
the inner content's `Response`, so `frame(..).clicked()` works.

**Signature** `frame(&mut frame, visuals: FrameVisuals, ui, add_contents) -> Response`

```rust
use egui_elements::utils::frame;

let response = frame(&mut self.row, theme.frame1_visuals(), ui, |ui| {
    ui.add(egui_elements::Label::new("Row", None));
});
```

For the stateful card that `Frame` already covers, prefer `Frame::surface` / `.interactive(true)` — see
[widgets](widgets.md#frame).
