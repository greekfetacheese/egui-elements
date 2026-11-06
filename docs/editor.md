# Theme editor

A live editor for a `Theme`: palette slots, frame chrome, widget-state colours, typography, spacing, a
contrast audit and a preview, in one `egui::Window`. It mutates the theme you hand it, so the host keeps
ownership and installs the result.

```rust
use egui_elements::editor::ThemeEditor;
```

## `ThemeEditor`

`ThemeEditor::new()` builds a closed editor with a theme-following background. `show(&mut theme, &mut ui)`
draws it and returns the new theme when its inner switcher picks a built-in kind; every other edit —
colours, renames, resets, reverts, loads — mutates `theme` in place.

**Signature** `ThemeEditor::new() -> ThemeEditor` · `.show(&mut Theme, &mut Ui) -> Option<Theme>` ·
`.set_io(save, load)`

```rust
use egui_elements::editor::ThemeEditor;

// Once, in the app state.
let mut editor = ThemeEditor::new();

// Every frame, after the switcher.
editor.open = show_editor;
if let Some(picked) = editor.show(&mut self.theme, ui) {
    self.theme = picked;
}
self.theme.clone().install(ui.ctx());
```

Installing **every frame** is the contract, not only on `Some`: `show` edits `theme` in place, so the
installed visuals have to be rewritten or the widgets keep the old palette. `install` takes `&mut self`,
which is why the snippet clones.

### Panels

| Panel | What it edits |
|---|---|
| Theme Colors | every `ThemeColors` slot |
| Theme Frames | `frame1` and `frame2` chrome — fill, stroke, radius, shadow, margins |
| Widget states | idle / hover / press / selected chrome per widget family |
| Typography | the named text sizes (`normal`, `small`, …) |
| Spacing | `ThemeSpacing` (`xs` … `xl`) and the chrome fields it feeds |
| Contrast Audit | each slot pair checked against WCAG AA (text 4.5, UI 3.0) |
| Preview | a live widget sample in the current palette |
| Name / Reset / Revert | rename the theme, reset to the built-in palette, revert to the snapshot taken on open |
| Save / Load | host-wired file dialogs (`set_io`) |

`color` (the selected slot) and `widget_state` (the widget family being edited) are public fields, so a
host can drive or restore the editor programmatically.

### Save / Load

The crate has no dialog and no serializer dependency. The host wires the buttons with `set_io`; the demo
uses `rfd` + `serde_json` (the `serde` feature is what makes a `Theme` serializable):

```rust
use egui_elements::{Theme, editor::ThemeEditor};

let mut editor = ThemeEditor::new();
editor.set_io(
    |theme| {
        let json = serde_json::to_string_pretty(theme).map_err(|e| e.to_string())?;
        std::fs::write("theme.json", json).map_err(|e| e.to_string())?;
        Ok(Some(String::from("Saved theme.json")))
    },
    || {
        let json = std::fs::read_to_string("theme.json").map_err(|e| e.to_string())?;
        let theme: Theme = serde_json::from_str(&json).map_err(|e| e.to_string())?;
        Ok(Some(theme))
    },
);
```

`save` returns the status line to show (`None` when the user cancelled) and `load` returns the theme the
user picked. Both closures are `'static`, so they may own whatever they need.

### Fields

`open` (a closed editor is a no-op), `size` (`(width, height)`, default `(300, 300)`), `bg_color`
(`Color32::TRANSPARENT` follows the theme's window frame; set a colour to pin an override), `color`,
`widget_state` and `hsla_edit_button`.

Specification behind the panels: [theme.md](theme.md). The demo wires the whole editor:

```sh
cargo run --example demo --features full
```
