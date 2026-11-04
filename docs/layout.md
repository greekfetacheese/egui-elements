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
| `justify-self` (content inside a cell) | `Item::align_content(CrossAlign::…)` |
| `flex: <n>` | `Item::grow(n)` (basis 0) |
| `width: Npx` | `Item::fixed(N)` |
| `flex-basis: auto` | `Item::new()` (measured content size) |
| `width: max-content` | `Flex::row_auto(id)` (hugs its items) |
| a stacked cell | `Item::stack()` (children top-down in one cell) |
| `flex-wrap`, `order`, `flex-grow` with `basis: auto`, percentages, baseline alignment | **not supported** |

Gaps come from the theme, never from a literal: `theme.spacing` is `xs` 4, `sm` 8, `md` 12, `lg` 16, `xl` 24.
With no `.gap(...)`, the container falls back to `ui.spacing().item_spacing` on its own axis, which
`Theme::install` already sets from the same scale.

`Item::align_content(CrossAlign::…)` is the one builder that is not `align-self`: it places the content
inside the cell when the cell is wider (row) or taller (column) than the content, so an `Item::grow(1.0)`
middle item with `.align_content(CrossAlign::Center)` centres its label in the space between two fixed
buttons instead of hugging the button before it. `Start` is the default; `Stretch` keeps the full cell box.

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

A row that should hug its content instead of spanning the parent is `Flex::row_auto` — the right
choice for a group nested inside a row cell, where a spanning row would claim the width that belongs to
everything after it.

A cell's child `Ui` continues the container's axis, so a row cell starts `left_to_right` and two labels
inside one cell sit side by side. `Item::stack()` lays that cell's children out top-down instead, without
hand-wrapping the body in `ui.vertical`:

```rust
Flex::row_auto("account").gap(8.0).show(ui, |f| {
   f.item(Item::new().stack(), |ui| {
      ui.label("Mike");
      ui.label("you@example.com");
   });
});
```

Overflow is clipped silently by default. `Flex::debug_overflow(true)` paints a red outline on any cell
whose content is larger than its box, which is the fastest way to find the cell that ate a label.

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

## 6. Tables

`Table` is the grid sibling of `Flex`: one header line and one body row per item, with the same column
extents used by every line. It is data-driven — `Table::new(id, &rows)` takes the slice and iterates it
inside `show`, instead of asking the host to emit a row at a time.

### The column map

| CSS | here |
|-----|------|
| `width: Npx` | `Column::Fixed(n)` |
| `width: max-content` | `Column::Auto` |
| `grid-template-columns: Nfr` | `Column::Fr(n)` |
| `width: auto` (no `fr`) | the table is as wide as its columns, left-aligned |

The last row is the one that surprises people: the table is 100% wide **only** when it has an `Fr` column.
With no `Fr`, `resolve_columns` has nothing to grow, so the table is exactly its fixed columns plus its auto
columns plus the gaps — CSS `width: auto`. Add one `Fr` column when the table should fill its parent.

### Why data-driven

An `auto` column needs every cell of the page measured before the first one is placed: its extent is the
widest cell, and a row that streams into place cannot know that until it has seen the rest. A streaming
`row()` API could only settle over frames, widening on every repaint as a longer cell arrives. Taking the
slice up front lets the table measure all rows in the same sizing pass and place them all in one frame.

### The measure-pass contract

The same contract as `Flex` (§3): on a measuring frame the cell and row closures run **twice** — once
invisibly to measure, once to place. Interactions are inert during the measuring pass, but a closure that
mutates host state outside a widget response applies twice.

**The header must be added before the rows.** The table allocates its whole band — header plus every row —
on the first `header()` or `rows()` call, because the `Auto` column widths are only known after the measure
pass and the header's height feeds the band. Adding the header after `rows()` leaves its cells unplaced.

### `Table::key`

The size cache sees the table id and the **row count**, not the row content. Swap ten rows for ten other
rows and the cached `Auto` width sticks. Pass something derived from the content — a page index, a filter
hash, a sort order — to `.key(..)` and the table measures again on the frame that value changes:

```rust
Table::new("tx_history", &rows).key(page).show(ui, |t| { /* … */ });
```

### Row chrome

`header_rule(true)` reserves one `item_spacing.y` step under the header text and draws the 1px separator at
the bottom of that band, so the rule never touches the header's descenders. For zebra rows or a card band,
use the row hook: `TableRow::fill(color)` paints the row's full-width band before its cells, and
`TableRow::rect()` returns that band so a host can draw its own chrome. `TableRow::index()` is the row's
index in the slice.

**Chrome overhang is the host's to budget.** A cell is a rect, not a clip: a themed `Button` paints about
6 pt above and below its band, and the table cannot measure painted chrome. A table of buttons therefore
needs `row_gap >= 2 × the overhang` — 12 pt, which is `theme.spacing.md` or the default
`item_spacing.y`. `theme.spacing.xs` (4 pt) lets one row's button reach into the next row. This is a
documented host duty, not something the container can infer.

### Recipe: the `tx_history` shape

Five columns (`fr / fr / fr / auto / fixed`), a header with a cell per column — including the empty one over
the Details column — and zebra rows:

```rust
use eframe::egui::*;
use egui_elements::layout::{Column, Table};
use egui_elements::widgets::{Button, Label};
use egui_elements::Theme;

struct Tx {
   wallet: &'static str,
   chain: &'static str,
   action: &'static str,
   age: &'static str,
}

fn tx_history(ui: &mut Ui, theme: &Theme, rows: &[Tx], page: usize) {
   Table::new("tx_history", rows)
      .columns([
         Column::Fr(2.2),
         Column::Fr(1.6),
         Column::Fr(4.0),
         Column::Auto,
         Column::Fixed(100.0),
      ])
      .gap(theme.spacing.md)
      .row_gap(theme.spacing.md)
      .header_rule(true)
      .key(page)
      .show(ui, |t| {
         t.header(|h| {
            for text in ["Wallet", "Chain", "Action", "Age", ""] {
               h.cell(|ui| {
                  ui.add(Label::new(
                     RichText::new(text)
                        .size(theme.typography.normal)
                        .strong()
                        .color(theme.colors.text),
                     None,
                  ));
               });
            }
         });
         t.rows(|r, tx| {
            if r.index() % 2 == 1 {
               r.fill(theme.colors.widget_bg);
            }
            for text in [tx.wallet, tx.chain, tx.action, tx.age] {
               r.cell(|ui| {
                  ui.add(Label::new(
                     RichText::new(text)
                        .size(theme.typography.normal)
                        .color(theme.colors.text),
                     None,
                  ));
               });
            }
            r.cell(|ui| {
               ui.add(Button::new("Details"));
            });
         });
      });
}
```

The `Action` column is the one that needs truncation in a real host — `Label::new(..)
.wrap_mode(TextWrapMode::Truncate)` keeps a long action name inside its `Fr` cell. Gaps come from
`theme.spacing`, sizes from `theme.typography`; no literal pixel values are needed above the `fixed` column
itself.

---

## 7. Recipes

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

## 8. Known limits

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
