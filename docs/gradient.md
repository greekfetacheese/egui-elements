# Gradient fills

A fill slot in this crate holds a `Fill`: a solid `Color32` or a two-stop `Gradient`. That is the
whole abstraction. A widget asks for its fill and paints it; it never asks whether the theme is in
solid or gradient mode, and a host can override any one widget with either.

```rust
use egui_elements::gradient::{Fill, Gradient, GradientDir};

let solid = Fill::solid(colors.widget_bg);
let ramp = Fill::horizontal(colors.accent, colors.info);
let bevel = Fill::graded(solid, colors.text, 0.1); // contrast-clamped
```

## The primitive

| Type | Meaning |
|---|---|
| `Fill` | `Solid(Color32)` or `Gradient(Gradient)`; `From<Color32>` and `From<Gradient>` |
| `Gradient` | `from`, `to`, `dir`; `Copy`, `Eq`, `Hash`, serde |
| `GradientDir` | `TopDown` (default), `BottomUp`, `LeftToRight`, `RightToLeft` |

`Fill` implements `Lerp`, so a hover or press transition eases the two stops instead of snapping
to one colour. A solid blends into a flat ramp, which is why a widget that suddenly gains a gradient
does not pop.

Serde is `untagged`: a solid saves as the `Color32` it always was, so theme JSON written before
gradients still loads, and a gradient saves as a `{ from, to, dir }` object.

## The mesh

egui paints a rect with a single colour — `Shape::Rect` carries one `Color32` — so a gradient has
to be a vertex-coloured `Mesh`. Cardinal directions sample every vertex in the same space, so
barycentric interpolation across a convex shape is an exact linear ramp: the two stops are the only
places a colour is chosen, and there is nothing to band.

`Gradient::mesh_rounded` builds **two rings**:

- an inner outline, inset by one point, sampled in the gradient and fanned into a solid interior;
- an outer outline on the rect, left fully transparent, whose triangles are the anti-aliased rim.

Both rings use **one** corner step count, taken from the outer radii, so vertex `i` lines up with
vertex `i` and every rim triangle stays a thin slice at the edge. A corner whose inset radius
collapses to zero keeps its slot by repeating the sharp point — without that, the inset radius can
fall into a lower segment bucket (3 pt → 2 pt is enough) and the rings end up different lengths,
which draws the rim as long triangles across the fill. `the_inset_ring_pairs_with_the_outer_ring_at_every_radius`
pins it.

epaint feathers its own rects, so a mesh without a rim reads as a hard, jaggied edge next to them —
that is the one piece of work a gradient fill cannot skip.

A 4 pt corner costs ~40 vertices; a pill costs ~130. That is the same order as the tessellated
rounded rect it replaces, and there is no cache: a mesh is rebuilt per paint, like every other
shape egui lays out.

## Contrast: the rule that makes it safe

A ramp that runs *through* a colour close to the label ink makes text unreadable even when both ends
look fine. `Gradient::bevel(base, ink, strength)` is the one derivation that prevents it:

1. one end is lifted toward white and the other dropped toward black, each by at most `strength`;
2. each end is **clamped** by contrast — a bisection stops the ramp the moment `ink` would fall
   under `CONTRAST_AA_TEXT` (with a small margin for `u8` rounding);
3. a fill with no headroom at all stays flat, so it is painted as a plain rect, not a mesh.

Because clipping happens on both ends, the whole ramp stays on the ink's far side: the mid-point of
a bevel is always between `base` and a stop that already clears AA.

`Fill::min_contrast(ink)` samples 16 points along the ramp and returns the worst one, which is what
the guarantee test asserts:

```text
gradient_fills_keep_their_ink_readable_on_every_builtin_theme
  every ThemeKind × every themed fill → min_contrast(ink) >= CONTRAST_AA_TEXT
```

The clamp uses `utils::mix` (unpremultiplied sRGB) and `utils::contrast_ratio` (WCAG 2.2). Contrast
ignores alpha, so a translucent fill must be composited over its surface before it is judged.

## `ThemeVariant`

`Theme::variant` decides how the themed fills are painted:

| Variant | Paints |
|---|---|
| `Solid` (default) | the palette colour, exactly as before |
| `Gradient` | a `Fill::bevel` of that colour, clamped against the widget's ink |

Fills are **stored solid** on `ThemeVisuals`; the variant is applied by the `Theme::*_visuals()`
getters as a widget reads them. So:

- switching variant is one field, and the next frame paints it — nothing to rebuild;
- the editor keeps editing the same solid colours;
- the palette-switch `remap` keeps working, because it still sees slots, not derived stops;
- `Theme::install`, `.visuals(theme.button_visuals())` and every other read go through the same
  graded value.

`theme.gradient_strength` (default `DEFAULT_GRADIENT_STRENGTH`, `0.1`) is how far a ramp may
travel. A tight palette simply gets a subtler ramp — never an unreadable one.

```rust
theme.set_variant(ThemeVariant::Gradient);
theme.install(ctx);
```

The ink each fill is clamped against: `ButtonVisuals::text`, `TextEditVisuals::text`,
`ThemeColors::text` for the combo box and the frames (neither carries an ink slot of its own).

A fill that is **already a gradient** passes through untouched: an explicit gradient outranks the
variant.

## What stays flat

`egui::Frame`-painted surfaces take one colour, so they stay solid by design: panels
(`Theme::frame1` / `frame2`), windows (`window_frame`), the combo-box popup (`popup_bg`), the modal
card, and `utils::frame` chrome. The crate's own `widgets::Frame`, `Button`, `Label`, `ComboBox`,
`TextEdit` and the toggle plates paint through `visuals::paint_shape`, so they all take a `Fill`
today. Turning a panel into a gradient means painting it through the crate `Frame` (or a
`ShapeIdx` reserve/set, as the hold sweep does), not through `egui::Frame`.

## Host gradients and the audit

A host-authored gradient (a brand ramp, a `uigradients` pair) is *its own* contrast responsibility:
the themed clamp only guards the bevels it derives. Reach for `Fill::min_contrast(ink)` and
`theme.colors.ink_on(fill)` in tests — a dark ramp behind white text is fine, a saturated mid-tone
behind white is not.

## Notes

- **Gamma space.** `Gradient::sample` interpolates with `Color32::lerp_to_gamma`, which is what
  epaint does with vertex colours; sampling in linear light would wash every mid-tone out.
- **Opaque stops.** Contrast is measured on the stops as they are; composite first if they carry
  alpha.
- **Mesh cost.** Rebuilt per paint, no cache, no state; roughly two vertices per outline point.
- **Breaking change.** The four visuals structs and `bg_from_res` return `Fill` where they returned
  `Color32`. `Fill: From<Color32>` keeps literals short (`Fill::solid(c)`), and `Fill::to_solid()`
  is the way back for a path that cannot paint a mesh.

