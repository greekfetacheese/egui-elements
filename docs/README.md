# Documentation

Nine pages, one per area of the crate. Each page mirrors a module under `src/`, so "where does this
belong" has one answer.

| Page | Covers | Module |
|---|---|---|
| [widgets.md](widgets.md) | every widget: what it does, its constructor, a working example | `src/widgets/` |
| [layout.md](layout.md) | `Flex`, `Item`, `Table`, `Column` — plus the measure-pass contract | `src/layout/` |
| [motion.md](motion.md) | `Motion`, `Easing`, `animate`, `Lerp`, `Fade`, `Collapse` — plus the repaint contract | `src/motion/` |
| [particles.md](particles.md) | `Snow`, `Light`, `Lightning` and their settings windows | `src/particles/` |
| [components.md](components.md) | feature-gated composites: credentials form, input fields, QR image, QR scanner | `src/components/` |
| [utils.md](utils.md) | theme switcher, colour mixing, WCAG contrast, HSLA, palette remapping | `src/utils.rs` |
| [editor.md](editor.md) | `ThemeEditor`: panels, save / load, fields | `src/editor.rs` |
| [theme.md](theme.md) | palette specification: slots, surfaces, widget states, contrast policy | `src/theme.rs`, `src/themes/` |
| [gradient.md](gradient.md) | `Fill` / `Gradient` specification: the mesh, the clamp | `src/gradient.rs` |

## How the pages are built

- **A reference entry** is *name · what it does · example*. Code first, prose after; the prose says what
  the code cannot (a default, a gotcha, why a builder exists).
- **Every entry names its proof.** If an entry has an example under `examples/`, the page links it and
  the command to run it. A claim you cannot run is a claim to distrust.
- **Specifications stay separate from usage.** `theme.md`, `gradient.md` and the numbered sections at the
  bottom of `layout.md` / `motion.md` are contracts (slot roles, the measure pass, the repaint rule).
  The reference sections at the top of those pages are the recipes that use them.
- **A new widget adds a row** to the table in [widgets.md](widgets.md) and an example under `examples/` —
  not a section in the root README.

## Reading order

New to the crate? [../README.md](../README.md) → [widgets.md](widgets.md) → the page that matches what
you are building. Changing a palette? [theme.md](theme.md) first, then [editor.md](editor.md).
