# Theme specification

This document is the contract for **adding or editing a theme** in `egui-elements`.

Tokyo Night (`src/themes/tokyo_night.rs`) is the first implementation. It is not frozen as a look, but its **roles, surfaces, and state mapping** are the source of truth. A new theme may change hex values; it should not change what a slot *means*.

Verify every theme in the demo:

```bash
cargo run --example demo --features full
```

Required surfaces: widget gallery on **main bg**, inside **`frame1`**, and on a **`Window`** using `theme.window_frame`. If a control only works on one of those, the theme is incomplete.

---

## 1. Intent

Themes are product UI, not syntax-highlighter dumps.

- Every color must have a job.
- Widgets must **match each other** and **match both surfaces** (canvas and elevated / window).
- Idle, hover, click, selected, and open must be distinguishable.
- Elevation is a **soft shadow** (and optional stroke), not a large luminance jump between `bg` and `widget_bg`.

Do not invent one-off hex in widgets. Paint from `ThemeColors` and `ThemeVisuals`. Theme-local constants (Tokyo Night `DUSK`, `FADED_BLUE`) are allowed only inside that theme’s visuals, never as a substitute for a `ThemeColors` slot.

---

## 2. Palette slots (`ThemeColors`)

These slots are the public palette. Names are roles.

| Slot | Job | May alias | Must not be used as |
|------|-----|-----------|---------------------|
| `bg` | App canvas. `frame2` fill. Panel / code / faint bg. | `title_bar` | Default fill of buttons, windows, `frame1` |
| `widget_bg` | Elevated surface. Default widget fill. `frame1` and `window_frame` fill. | — | Canvas |
| `hover` | Pointer-over **fill lift** (gray step, no chroma) | — | Selected, muted text, selection |
| `highlight` | Selected / emphasis **fill**. Combo selected row. `egui` selection bg. Idle stock slider track. | — | Hover |
| `title_bar` | Custom window chrome | `bg` | Widget fill |
| `text` | Body, labels, button labels on surfaces | — | Text on chromatic **fills** |
| `text_muted` | Hints, secondary copy, captions. Must stay readable. | — | Selection fill, hover border |
| `border` | Structural stroke. Frame / widget **shadow color** (solid or alpha) | — | Hover border (that is chrome, often a mid tone) |
| `accent` | Focus, primary action, text-edit open/hover border | — | Error / success / info |
| `error` | Danger, destructive, invalid | — | Text color on its own fill |
| `warning` | Caution | — | Text color on its own fill |
| `success` | Positive, complete | — | Text color on its own fill |
| `info` | Hyperlinks, secondary emphasis | `Visuals.hyperlink_color` | Accent / focus ring |

**Aliasing is allowed. Collapsing jobs is not.** Tokyo Night may set `title_bar = bg`. It must not set `highlight = hover` or `text_muted` as selection fill.

Until dedicated `on_accent` / `on_error` / `on_warning` / `on_success` / `on_info` slots exist on `ThemeColors`, text sitting **on** a chromatic fill uses **`bg`**.

---

## 3. Surfaces

Two surfaces. Everything else is a frame sitting on one of them.

| Surface | Fill | Typical content |
|---------|------|-----------------|
| Canvas | `bg` | App background, `frame2` (nested / list rows) |
| Elevated | `widget_bg` | `frame1` (major sections), `window_frame`, default widget fill |

`bg` vs `widget_bg` may be a **tiny** luminance step (Tokyo Night is ~1.05:1). That is intentional. Do not “fix” it by making `widget_bg` a third plane — that fights the rule that the **same** widget visuals work on both surfaces.

`egui::Visuals.window_fill` / `panel_fill` stay `bg`. App windows must use `theme.window_frame` (elevated), not raw egui window fill.

---

## 4. Identity rule (the spine)

A control must be findable **idle** on **both** `bg` and `widget_bg` without a per-surface override.

**Custom widgets** (`Button`, `ComboBox`, …): identity is **shadow** (and hover border), not fill contrast.

- Idle fill = `widget_bg`
- Idle border = transparent (or a hairline if shadow is not enough)
- Shadow color = `border` (optionally with alpha)

On canvas, fill steps up from `bg`. On a window, fill matches the window and the shadow still separates the control.

**Stock egui widgets** (slider, checkbox, radio, …): identity is **fill**. Idle `bg_fill` = `highlight` because many of these have no border and would vanish on `widget_bg`.

Do not unify those two recipes. The next theme must implement **both**.

---

## 5. Widget states

Map every interactive control through these states. **Selected ≠ hover.**

| State | Fill | Border | Shadow | Text |
|-------|------|--------|--------|------|
| Idle | `widget_bg` | Transparent (custom) or `border` (stock / text edit) | `border` | `text` |
| Hover | `hover` | Mid chrome (Tokyo Night: `DUSK`) or `hover` for stock | keep | `text` |
| Click / active | `widget_bg` (return toward idle) | Transparent or `border` | keep | `text` |
| Selected | `highlight` | optional | keep | `text` |
| Open / focus (text edit, combo) | `widget_bg` | `accent` | none on text edit | `text` |
| Disabled | same fills, `text_muted` | as idle | as idle | `text_muted` |

Labels that are interactive inherit button hover/click **fills** but stay transparent idle (no shadow). Static labels have no frame.

`bg_color(...)` on a button is an idle chromatic fill. Hover / click / selected must **stay in that chromatic family** (mix ~15–20% toward `bg` on hover), not snap back to `hover` / `widget_bg`. Text on that fill is `bg` (see §7). This hover-in-family behavior is required by the spec; the widget may still need a code follow-up to honor it.

### Stock `egui::Widgets`

| State | `bg_fill` | `weak_bg_fill` | `bg_stroke` |
|-------|-----------|----------------|-------------|
| Non-interactive | `widget_bg` | `bg` | `border` |
| Inactive | `highlight` | `bg` | `border` |
| Hovered | `widget_bg` | `hover` | `hover` |
| Active / open | `widget_bg` | `widget_bg` | `border` |

`Visuals.selection`: `bg_fill = highlight`, `stroke = accent`. Never `text_muted`.

---

## 6. Frames

Always use the theme frames. Do not ad-hoc `Frame::new().fill(...)`.

| Frame | Fill | Stroke | Shadow | Use |
|-------|------|--------|--------|-----|
| `window_frame` | `widget_bg` | 1px `border` | blur 3, `border` | egui `Window` chrome |
| `frame1` | `widget_bg` | none | blur 2, `border` | Major sections on canvas |
| `frame2` | `bg` | none | blur 2, `border` | Nested items / rows inside `frame1` |

Interactive frame overlays (`FrameVisuals`):

- Hover fill → `hover`
- Click fill → that frame’s idle fill (`widget_bg` / `bg`)
- Border on hover/click may be unused (width 0) if fill is enough

`Theme::remap_derived_frames` keeps these fills in sync when the editor changes the palette, but only if the frame color still matches the previous slot. Customized frames are left alone.

Native `Visuals.window_shadow` may use `border` at reduced alpha (Tokyo Night `FADED_BLUE`). Same hue, different alpha — not a new color.

---

## 7. Chromatics

`accent`, `error`, `warning`, `success`, `info` are **glyphs and fills**, never body text.

**As foreground** (icon, status text, link) on `bg` / `widget_bg` / `hover`: the chromatic itself. Must meet contrast on `bg` and `widget_bg` (see §9).

**As fill** (semantic button, banner):

| Fill | Text on fill |
|------|----------------|
| `accent` / `error` / `warning` / `success` / `info` | `bg` |

Powder / `text` on a chromatic fill is forbidden.

Hyperlinks use `info`, not `accent`. Focus rings and text-edit open borders use `accent`, not `info`.

---

## 8. Typography

`Typography` is sizes only. Color comes from the palette.

| Token | Tokyo Night (px) | Typical use |
|-------|------------------|-------------|
| `heading` | 26 | Section titles |
| `very_large` | 20 | Page-level emphasis |
| `large` | 18 | Subheads |
| `normal` | 16 | Body, button labels, hints |
| `small` | 14 | Compact UI |
| `very_small` | 12 | Swatch names, dense meta |

`text` for body. `text_muted` for hints and secondary copy at `normal` or smaller. Do not use a mid chrome (hover-border color) as type.

Corner radius: theme default `6` for frames, combo, text edit. Buttons may be tighter (`3`) so they read as controls, not panels. That is a geometry token, not a color issue.

Inner margin `10`, outer margin `5`, button padding `(10, 8)` are the Tokyo Night defaults; a new theme may change them but should keep one inner / one outer / one button padding rather than per-widget magic numbers.

---

## 9. Contrast policy

WCAG 2.x relative luminance, sRGB.

| Pair | Minimum | Notes |
|------|---------|-------|
| `text` on `bg`, `widget_bg`, `hover` | **4.5:1** | Body |
| `text` on `highlight` | **4.5:1** | Selected rows / buttons |
| `text_muted` on `bg` and `widget_bg` | **4.5:1** | Hints are functional in this crate (wallets, forms) |
| Chromatic **as glyph** on `bg` and `widget_bg` | **4.5:1** | `error` / `info` may sit just under 4.5 on `hover`; prefer not to perturb the semantic set for that one surface |
| `bg` on chromatic fills | **4.5:1** | `on_*` |
| `bg` vs `widget_bg` | no floor | Tiny step is OK |
| `hover` vs `bg` / `widget_bg` | visible, not a new plane | ~1.3:1 is enough with shadow |
| `highlight` vs `hover` | distinguishable | Selected must not look hovered |

`text_muted` vs `text` should remain a clear step (Tokyo Night ~2.3:1) so muted does not compete with body.

---

## 10. Adding a theme

1. Add `src/themes/<name>.rs` with `theme()`, `style()`, frames, and visuals. Copy Tokyo Night’s structure; replace hex, keep roles.
2. Add `ThemeKind` variant, `to_str`, `to_vec`.
3. Match it in `Theme::new` and `Theme::style`.
4. Palette constants map **onto slots**. Do not skip `highlight` or reuse `hover` / `text_muted` for selection.
5. Custom button/combo idle = `widget_bg` + shadow. Hover border is chrome (may be a theme-local mid tone). Selected fill = `highlight`.
6. Text edit idle border = `border`; hover/open = `accent`.
7. `Visuals.selection.bg_fill = highlight`, stroke = `accent`.
8. Stock inactive `bg_fill = highlight`.
9. Run the demo. Check:
   - Swatches: `hover` ≠ `highlight` ≠ `text_muted`
   - Same gallery on **bg**, **frame1**, **window**
   - Selected button vs hover
   - Combo selected row
   - Muted hint copy
   - Semantic fills with **`bg` text**, idle
   - Nested `frame1` / `frame2`

Do not run `cargo fmt` on the crate. Format only files you touch; match local indent.

---

## 11. Do / don’t

**Do**

- Paint from slots.
- Keep one widget visual set for canvas and window.
- Keep selected, hover, and muted as three paints.
- Use `window_frame` / `frame1` / `frame2`.
- Put dark (`bg`) text on chromatic fills.

**Don’t**

- Use `text` (powder) on accent / success / warning / error / info fills.
- Use `text_muted` as selection or slider fill.
- Set `highlight = hover`.
- Give custom widgets a second palette for windows.
- Widen `bg` vs `widget_bg` just to “add contrast.”
- Stroke `frame1` the same as `window_frame` unless the theme’s elevation model needs it (windows have stroke; section frames usually don’t).
- Read `Visuals.window_fill` as the app window color.

---

## 12. Reference: Tokyo Night

Based on [tokyo-night-vscode-theme](https://github.com/tokyo-night/tokyo-night-vscode-theme), retuned for product UI.

Cool night canvas. Elevation by shadow. Cyan focus. Purple links. Salmon / camel / teal semantics.

### Named paints

| Const | RGB | Hex | Maps to |
|-------|-----|-----|---------|
| `DARK` | 22, 22, 30 | `#16161e` | `bg`, `title_bar`, text on chromatic fills |
| `DARK2` | 26, 27, 38 | `#1a1b26` | `widget_bg` |
| `DARK3` | 40, 47, 65 | `#282f41` | `hover` |
| `HIGHLIGHT` | 56, 80, 103 | `#385067` | `highlight` (`DARK3` + 20% accent) |
| `SOFT_BLUE` | 65, 73, 97 | `#414961` | `border`, widget shadow |
| `FADED_BLUE` | 65, 73, 97, 118 | | Native window shadow only |
| `DUSK` | 83, 97, 136 | `#536188` | Hover **border** only (not a `ThemeColors` slot) |
| `MUTED` | 120, 134, 174 | `#7886ae` | `text_muted` (`DUSK` + 35% powder) |
| `POWDER_BLUE` | 190, 204, 244 | `#beccf4` | `text` |
| `LIGHT_BLUE` | 118, 210, 253 | `#76d2fd` | `accent` |
| `SALMON_PINK` | 255, 91, 103 | `#ff5b67` | `error` |
| `CAMEL` | 218, 147, 61 | `#da933d` | `warning` |
| `GREEN` | 72, 182, 120 | `#48b678` | `success` |
| `PASTEL_PURPLE` | 194, 111, 255 | `#c26fff` | `info` |

### Contrast (approx.)

| Pair | Ratio |
|------|-------|
| `text` on `bg` / `widget_bg` / `hover` | 11.3 / 10.7 / 8.4 |
| `text` on `highlight` | 5.2 |
| `text_muted` on `bg` / `widget_bg` | 5.0 / 4.7 |
| `bg` on accent / error / warning / success / info | 10.6 / 6.0 / 7.0 / 7.1 / 6.0 |
| Chromatics as glyphs on `bg` | all ≥ 4.5 (`error` 5.95, `info` 6.01) |
| `bg` vs `widget_bg` | 1.05 |
| `highlight` vs `hover` | 1.60 |

### Geometry

- Corner radius: 6 (frames, combo, text edit), 3 (buttons)
- Inner margin 10, outer 5, button padding (10, 8)
- Animation 0.3s
- `image_tint_recommended`: true (very dark canvas)

---

## 13. Known gaps

Not palette problems. Track them when they block a theme or the demo.

1. **`on_*` slots** are a rule (`bg`) but not fields on `ThemeColors` yet.
2. **`Button::bg_color`** still snaps hover/click to gray `hover` / `widget_bg`. Semantic fills must stay in-family.
3. **Hover-border chrome** is theme-local (Tokyo Night `DUSK`, McLaren `HOVER_CHROME`). Do not overload `text_muted` or `border` for it.
4. **`error` / `info` on `hover`** may sit just under 4.5:1. Acceptable; don’t lighten the semantic set unless glyphs-on-hovered-rows become common.

When a theme’s hex change, update its reference section. When a **role** changes, update §2–§9 first, then the implementations.

---

## 14. Reference: McLaren 650GTS GT3

2015 McLaren 650S GT3 — carbon weave canvas, papaya body, steering-wheel HUD cyan/green. Cypherpunk night UI: dark carbon, orange punch, cyan chrome.

`ThemeKind::McLaren650GtsGt3` — `src/themes/mclaren_650gts_gt3.rs`.

### Role split

| Slot | Paint | Why |
|------|-------|-----|
| `accent` | papaya `#ff7a08` | Brand / focus / text-edit open |
| `info` | HUD cyan `#38d2e6` | Links, tech glow |
| `highlight` | papaya wash `#613f25` | Selected ≠ hover |
| Hover border | `HOVER_CHROME` `#397683` | Cyan chrome, not a slot |
| `hover` | carbon lift `#242830` | Gray step, no chroma |

### Named paints

| Const | RGB | Hex | Maps to |
|-------|-----|-----|---------|
| `CARBON` | 12, 13, 16 | `#0c0d10` | `bg`, `title_bar`, text on chromatic fills |
| `CARBON2` | 18, 20, 24 | `#121418` | `widget_bg` |
| `CARBON3` | 36, 40, 48 | `#242830` | `hover` |
| `HIGHLIGHT` | 97, 63, 37 | `#613f25` | `highlight` (`CARBON3` + 28% papaya) |
| `STEEL` | 58, 68, 78 | `#3a444e` | `border`, widget shadow |
| `FADED_STEEL` | 58, 68, 78, 118 | | Native window shadow only |
| `HOVER_CHROME` | 57, 118, 131 | `#397683` | Hover **border** only |
| `MUTED` | 131, 144, 151 | `#839097` | `text_muted` |
| `HUD_WHITE` | 220, 236, 240 | `#dcecf0` | `text` |
| `PAPAYA` | 255, 122, 8 | `#ff7a08` | `accent` |
| `SIGNAL_RED` | 235, 80, 85 | `#eb5055` | `error` |
| `AMBER` | 240, 186, 48 | `#f0ba30` | `warning` |
| `HUD_GREEN` | 48, 210, 130 | `#30d282` | `success` |
| `HUD_CYAN` | 56, 210, 230 | `#38d2e6` | `info` |

### Contrast (approx.)

| Pair | Ratio |
|------|-------|
| `text` on `bg` / `widget_bg` / `hover` | 16.0 / 15.2 / 12.2 |
| `text` on `highlight` | 7.7 |
| `text_muted` on `bg` / `widget_bg` | 5.9 / 5.6 |
| `bg` on accent / error / warning / success / info | 7.4 / 5.4 / 10.9 / 9.9 / 10.7 |
| Chromatics as glyphs on `bg` / `widget_bg` | all ≥ 4.5 |
| `bg` vs `widget_bg` | 1.05 |
| `highlight` vs `hover` | 1.58 |

### Geometry

- Corner radius: 4 (frames, combo, text edit), 2 (buttons)
- Inner margin 10, outer 5, button padding (10, 8)
- Animation 0.3s
- `image_tint_recommended`: true

