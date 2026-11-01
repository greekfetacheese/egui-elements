# Layout

Rows and columns for `egui-elements`, with the vocabulary of CSS flexbox.

```rust
use egui_elements::layout::{Axis, CrossAlign, Flex, FlexUi, Item, ItemSize, Justify};
```

Run the proof: `cargo run --example layout`.

---

## 1. What this is

A **CSS-flexbox subset** for egui: a container that measures its items first and then places them into exact
cell rects, on one axis, with a gap, an optional free-space distribution and a cross-axis alignment per item.

The subset exists because host code kept re-deriving it. The same three shapes show up in every app — a label
centred against a button, a row of N buttons that share the width equally, a left/right action bar — and every
host solved them with hand-rolled arithmetic over `ui.available_width()`. That arithmetic drifts the moment a
caption or a spacer changes, and the `ui.horizontal` variants it replaces do not centre on the tallest item at
all (see §4).

The mapping is meant to be mechanical. Anyone who has used flexbox should be able to read the table below and
write the row without reading this module's source.

---

## 2. The CSS → API map

| CSS | here |
|-----|------|
| `display: flex` | `Flex::row(id)` / `Flex::column(id)` |
| `flex-direction` | those two constructors (`Axis`) |
| `gap` | `.gap(theme.spacing.sm)` |
| `justify-content` | `.justify(Justify::…)` |
| `align-items` | `.align_items(CrossAlign::…)` |
| `align-self` | `Item::…align(CrossAlign::…)` |
| `flex: <n>` | `Item::grow(n)` (basis 0) |
| `width: Npx` | `Item::fixed(N)` |
| `flex-basis: auto` | `Item::new()` (measured content size) |
| `flex-wrap`, `order`, `flex-grow` with `basis: auto`, percentages, baseline alignment | **not supported** |

Gaps come from the theme, never from a literal: `theme.spacing` is `xs` 4, `sm` 8, `md` 12, `lg` 16, `xl` 24.
With no `.gap(...)`, the container falls back to `ui.spacing().item_spacing` on its own axis, which
`Theme::install` already sets from the same scale.

---

## 3. The measure-pass contract

A container cannot place an item before it knows the item's size, so the **first frame** of a container — and
any frame where the item count, the available space, or an item's spec changes — runs the item closures in an
invisible `sizing_pass` and discards the frame. A closure may therefore run **twice** on those frames.
Interactions are inert during the measuring pass (the ui is `disabled`), so state must be mutated from a
widget response, never from the closure body alone: a closure that increments a counter or pushes to a `Vec`
outside a response would apply twice.

Steady state is one pass.

The first frame is measured invisibly and discarded, so do not chase a value that looks empty on that frame;
from the second frame the rects are exact and stable.

`max_passes` interacts with this: a host that sets `egui::Options::max_passes` to `1` gets one blank frame for
a new container instead of a measure pass.

---

## 4. Container extents

- A **row** spans the available width and is as tall as its tallest item.
- A **column** is as tall as its content and spans the available width; its items align inside that width.
  Call `.fill_main(true)` on a column to eat the parent's height.

That is the difference from egui's `ui.horizontal`, which seeds the row's height from `interact_size.y` before
any item has been measured. A themed `Button` is taller than that seed, so it overflows the row and the label
beside it is never re-centred on the button. A flex row measures both items with no seed at all and centres on
the tallest one.

---

## 5. `id_salt` rules

`id_salt` becomes a **globally unique** id (`Id::unique(..)`), not a child of the surrounding `ui`. Two
containers must never share a salt in one frame: the second would silently reuse the first one's cache. In a
loop, include the index in the salt:

```rust
for (i, row) in rows.iter().enumerate() {
   Flex::row(("row", i)).gap(theme.spacing.sm).show(ui, |f| { /* … */ });
}
```

Because the salt is global, a container nested inside another container needs its own salt just like a
top-level one — nesting does not namespace anything.

---

## 6. Recipes

### Equal-width button row

`Item::grow(1.0)` is `flex: 1` with a basis of zero, so the captions never decide the widths. There is no
`(w - gap * (n - 1)) / n` in the host. A `Button` does not fill its cell on its own, so each one asks for the
cell width it was given:

```rust
use eframe::egui::*;
use egui_elements::layout::{Flex, Item};
use egui_elements::widgets::Button;
use egui_elements::Theme;

fn export_row(ui: &mut Ui, theme: &Theme, captions: &[&str]) {
   Flex::row("export_row")
      .gap(theme.spacing.sm)
      .show(ui, |f| {
         for text in captions {
            f.item(Item::grow(1.0), |ui| {
               let width = ui.available_width();
               ui.add(
                  Button::new(*text)
                     .min_size(vec2(width, 0.0))
                     .label_align(Align::Center),
               );
            });
         }
      });
}
```

Adding or removing a caption reflows the rest: three captions share the width three ways, two share it two
ways, with no host-side count to keep in sync.

### Label next to a button

The `demo2` case: the row is as tall as the button, and the label is centred on it.

```rust
use egui_elements::layout::{CrossAlign, Flex, Item};
use egui_elements::widgets::{Button, Label};
use egui_elements::Theme;

Flex::row("label_button")
   .gap(theme.spacing.sm)
   .align_items(CrossAlign::Center)
   .show(ui, |f| {
      f.item(Item::new(), |ui| {
         ui.add(Label::new("Normal text", None));
      });
      f.item(Item::new(), |ui| {
         ui.add(Button::new("Click me"));
      });
   });
```

Set `ui.spacing_mut().button_padding` before the row if you want the height difference to be obvious.

### Left / right action row

`Justify::SpaceBetween` spreads the leftover space between the items, so the outer two sit on the row's edges
and everything between them floats. Here the disabled first page pushes the label into the middle:

```rust
use egui_elements::layout::{CrossAlign, Flex, Item, Justify};
use egui_elements::widgets::{Button, Label};
use egui_elements::Theme;

Flex::row("pagination")
   .gap(theme.spacing.sm)
   .justify(Justify::SpaceBetween)
   .align_items(CrossAlign::Center)
   .show(ui, |f| {
      f.item(Item::new(), |ui| {
         ui.add_enabled(false, Button::new("Previous"));
      });
      f.item(Item::new(), |ui| {
         ui.add(Label::new("Page 1 of 25", None));
      });
      f.item(Item::new(), |ui| {
         ui.add(Button::new("Next"));
      });
   });
```

`Justify::SpaceAround` is the other distribution with gutters: equal space around each item instead of only
between them.

---

## 7. Known limits

- **Cells are not clipped.** A cell is a rect, not a clip rect. `Button`'s hover expansion and `Badge`
  overlays paint outside their widget's rect, so they can reach into the gap. Keep gaps at
  `theme.spacing.sm` or wider when a row holds either.
- **`ui.available_width()` inside a child reports the cell width**, not the parent's. That is what makes the
  `.min_size(vec2(ui.available_width(), 0.0))` recipe work, but it also means a child cannot ask "how wide is
  the row".
- **An `Auto` item whose content changes size is corrected on the next frame.** The container compares the
  placed child against its cached size and asks for one repaint; the frame the content changed can show the
  previous cell width.
- **An item's cross size is its measured size** unless the container (or the item) sets
  `CrossAlign::Stretch`. A widget that does not paint its full rect will look un-stretched even when its cell
  is.
- **Not supported:** `flex-wrap`, `order`, `flex-grow` on top of a natural size, percentage sizes, and
  baseline alignment.
