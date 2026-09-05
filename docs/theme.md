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
| Active | `widget_bg` | `widget_bg` | `border` |
| Open | `widget_bg` | `bg` | `border` |

`Visuals.selection`: `bg_fill = highlight`, `stroke = accent`. Never `text_muted`.

egui `egui::Window` title (top layer) overwrites `title_frame.fill` with `widgets.open.weak_bg_fill`. App windows should use `egui_elements::widgets::Window`, which keeps the requested title fill regardless of how many windows are open. The `open.weak_bg_fill = bg` slot remains as a fallback for any leftover stock `egui::Window`.

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

Inner margin `10`, outer margin `5`, button padding `(10, 8)` are widget chrome. Layout rhythm is [`ThemeSpacing`](../src/theme.rs) (`xs` 4 / `sm` 8 / `md` 12 / `lg` 16 / `xl` 24). A new theme may change the numbers but should keep **one scale** — do not invent `7` / `10` / `18` in app layout. `Theme::install` writes `md` into `Style.spacing.item_spacing` and `button_padding` / `window_margin` from the chrome fields.

| Token | Default | Job |
|-------|---------|-----|
| `xs` | 4 | Tight meta, icon+label, stacked lines in a row |
| `sm` | 8 | Dense lists, compact control padding |
| `md` | 12 | Default item spacing, related group |
| `lg` | 16 | Section padding (header, pane inner) |
| `xl` | 24 | Between groups |

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
- Spacing scale: xs 4, sm 8, md 12, lg 16, xl 24
- Animation 0.3s
- `image_tint_recommended`: true (very dark canvas)

---

## 12a. Reference: Tokyo Night Light

Based on [tokyo-night-light-color-theme.json](https://github.com/tokyo-night/tokyo-night-vscode-theme/blob/master/themes/tokyo-night-light-color-theme.json), retuned for product UI.

Cool paper canvas. Elevation by shadow. Teal focus. Iris links. Wine / amber / pine semantics. Same jobs as Tokyo Night.

Official terminal ansi / `descriptionForeground` are too gray on storm (status glyphs look washed). Ink is darkened vs `#343b58`; chromatics keep AA-safe luminance but pick chroma so they read as status, not comments.

### Named paints

| Const | RGB | Hex | Maps to |
|-------|-----|-----|---------|
| `STORM` | 214, 216, 223 | `#d6d8df` | `bg`, `title_bar`, text on chromatic fills |
| `PAPER` | 230, 231, 237 | `#e6e7ed` | `widget_bg` |
| `HAZE` | 200, 203, 214 | `#c8cbd6` | `hover` |
| `HIGHLIGHT` | 172, 176, 191 | `#acb0bf` | `highlight` (selection, no alpha) |
| `MIST` | 193, 194, 199 | `#c1c2c7` | `border`, widget shadow |
| `FADED_MIST` | 193, 194, 199, 118 | | Native window shadow only |
| `DUSK` | 122, 133, 168 | `#7a85a8` | Hover **border** only (not a `ThemeColors` slot) |
| `MUTED` | 61, 69, 102 | `#3d4566` | `text_muted` (official placeholder `#707280` fails AA) |
| `INK` | 42, 47, 74 | `#2a2f4a` | `text` (darker than official `#343b58`) |
| `TEAL` | 10, 90, 110 | `#0a5a6e` | `accent` (official `#166775` / `#006c86` read as gray-teal) |
| `WINE` | 155, 32, 54 | `#9b2036` | `error` (crimson; official `#8c4351` is dusty rose) |
| `AMBER` | 138, 61, 0 | `#8a3d00` | `warning` (burnt orange; official `#8f5e15` fails AA and looks muddy) |
| `PINE` | 13, 92, 64 | `#0d5c40` | `success` (forest; official `#33635c` is gray-green) |
| `IRIS` | 85, 32, 168 | `#5520a8` | `info` (violet; official `#5a3e8e` is gray-purple) |

### Contrast (approx.)

| Pair | Ratio |
|------|-------|
| `text` on `bg` / `widget_bg` / `hover` | 9.2 / 10.6 / 8.1 |
| `text` on `highlight` | 6.1 |
| `text_muted` on `bg` / `widget_bg` | 6.6 / 7.6 |
| `bg` on accent / error / warning / success / info | 5.5 / 5.6 / 5.4 / 5.6 / 6.8 |
| Chromatics as glyphs on `bg` | all ≥ 4.5 (`warning` 5.37 lowest) |
| Chromatics as glyphs on `hover` | all ≥ 4.5 (`warning` 4.72 lowest) |
| `bg` vs `widget_bg` | 1.15 |
| `highlight` vs `hover` | 1.33 |

### Geometry

- Corner radius: 6 (frames, combo, text edit), 3 (buttons)
- Inner margin 10, outer 5, button padding (10, 8)
- Spacing scale: xs 4, sm 8, md 12, lg 16, xl 24
- Animation 0.3s
- `image_tint_recommended`: false

---

## 13. Known gaps

Not palette problems. Track them when they block a theme or the demo.

1. **`on_*` slots** are a rule (`bg`) but not fields on `ThemeColors` yet.
2. **`Button::bg_color`** still snaps hover/click to gray `hover` / `widget_bg`. Semantic fills must stay in-family.
3. **Hover-border chrome** is theme-local (Tokyo Night `DUSK`, Tokyo Night Light `DUSK`, McLaren `HOVER_CHROME`). Do not overload `text_muted` or `border` for it.
4. **`error` / `info` on `hover`** may sit just under 4.5:1. Acceptable; don’t lighten the semantic set unless glyphs-on-hovered-rows become common.

When a theme’s hex change, update its reference section. When a **role** changes, update §2–§9 first, then the implementations.