# egui-elements

Themes, widgets, and components for [egui](https://github.com/emilk/egui).

The goal of this crate is to make it easy to create nice
looking apps with egui.

egui-elements is far from perfect and there may be breaking changes to the color palettes as egui evolves.

This crate is **not** a drop-in replacement for stock egui widgets. It ships:

- A widget set (`Button`, `Label`, `ComboBox`, `SecureTextEdit`, `Modal`, `MultiLabel`, `Spinner`, `Checkbox`, `Radio`, `Switch`, `Tabs`, `Chip`, `Divider`, `Scroll`, `Window`) that reads visuals from an installed theme, corner `Badges` for buttons, hold-to-confirm buttons with a configurable progress sweep and completion burst, an animated checkbox, segmented and underlined tabs whose indicator slides between segments, and `AvatarGroup` stacks that fan out on hover
- **Layout** — a CSS-flexbox subset (`Flex::row` / `Flex::column`, `gap`, `justify`, `align_items`, `flex: n` items) and a data-driven `Table` with `fixed` / `auto` / `fr` columns; see [`docs/layout.md`](docs/layout.md), run `cargo run --example layout`
- **Motion** — eased value animation (`animate`, `Motion`, `Easing`), colour/rect interpolation (`Lerp`) and two containers: `Fade` (opacity) and `Collapse` (a body revealed by its own measured height); see [`docs/motion.md`](docs/motion.md), run `cargo run --example motion`
- `Snow` and `Light` particle effects with drop-in, themed settings windows (`Snow` ported from [egui-snow](https://github.com/LaVashikk/egui-snow), MIT)
- Seven built-in palettes (`ThemeKind`) plus a live `ThemeEditor`: palette, frame, widget, typography and spacing editors, a contrast audit, a live preview, theme naming/detaching, and host-wired Save / Load to JSON
- Feature-gated composites: credentials form, QR image, Linux QR scanner

`Button`, `ComboBox`, `Window`, `Spinner` and `Checkbox` have the same names as the egui types. Import this crate's widgets *after* `use eframe::egui::*`, or alias one of them.

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

A fill that is not a surface — a primary button in `accent`, a destructive one in `error` — needs its
own ink. `ButtonVisuals::filled(fill, colors)` supplies it: the label becomes `colors.ink_on(fill)`, so
the copy is AA on every theme without a second palette.

```rust
let primary = theme.button_visuals().filled(theme.colors.accent, theme.colors);
let danger = theme.button_visuals().filled(theme.colors.error, theme.colors);
ui.add(Button::new("Send").visuals(primary));
ui.add(Button::new("Delete").visuals(danger));
```

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

The editor can rename a theme (which detaches a built-in as
`ThemeKind::Custom`), reset it to its built-in palette, revert edits and preview
the result live. Save / Load buttons appear once the host wires
`ThemeEditor::set_io`; the crate ships no dialog or serializer dependency, so
the demo wires `rfd` + `serde_json` itself (see `examples/demo.rs`).

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

Long option lists can be virtualized: `show_rows_ui(ui, row_height, total_rows, |ui, range| …)` builds only the rows in the viewport, and every row must lay out at exactly `row_height` (without `item_spacing.y`, which egui adds itself). The popup chrome is adjustable with `.popup_gap(points)` (default `ThemeSpacing::xs`) and `.popup_content_margin(points)` (default `5`), and its surface is themed through `ComboBoxVisuals::popup_bg` / `popup_border`. See `cargo run --example combo_box`.

### Surfaces

Theme chrome has four named layers, so nesting is described by role instead of remembering
which of `frame1` / `frame2` goes where.

| Surface | Fill | Use |
|---|---|---|
| `Canvas` | `colors.bg` | page background |
| `Panel` | `frame1` (`widget_bg`) | major card on the canvas |
| `Well` | `frame2` (`bg`) | recessed area inside a panel |
| `Overlay` | `window_frame` | windows and modals |

`Surface::Panel.card()` is `Well` and `Surface::Well.card()` is `Panel`, so a card always
contrasts with the surface under it. `Frame::surface` returns the crate frame with that
surface's hover / click visuals already applied.

```rust
use egui_elements::{Frame, Surface};

let card = Frame::surface(&theme, Surface::Panel);
let well = Frame::surface(&theme, Surface::Panel.card());
```

### Gradient fills

A fill slot holds a `Fill`: a solid `Color32` or a two-stop `Gradient`. egui paints a rect with
one colour, so a gradient goes through a vertex-coloured mesh; the ramp is exact because the shader
interpolates the two stops across a convex rounded rect, and the mesh carries a feathered rim so it
keeps the anti-aliasing of a painted rect.

```rust
use egui_elements::gradient::{Fill, Gradient};

ui.add(Label::new("Solid", None).visuals(button_visuals.filled(colors.accent, colors)));

let brand = Fill::horizontal(
   Color32::from_rgb(0x00, 0xc9, 0x96),
   Color32::from_rgb(0x00, 0x3d, 0x4d),
);
brand.show_rounded(ui, vec2(180.0, 64.0), 8);

// Or paint any widget with it, through the visuals it already takes.
let visuals = ButtonVisuals {
   bg: brand,
   ..theme.button_visuals()
};
ui.add(Button::new("Branded").visuals(visuals));
```

`ThemeVariant` paints every themed fill as a gradient without touching a widget:
`theme.variant = ThemeVariant::Gradient` turns each fill into a soft bevel of its own colour, and
the bevel *clamps* itself by contrast on every one of the seven palettes — a label never loses
`CONTRAST_AA_TEXT` because its button became a ramp. `theme.gradient_strength` (default `0.1`) is
how far the ramp may travel toward white / black before the clamp applies.

```rust
theme.set_variant(ThemeVariant::Gradient);
theme.install(ctx);
```

The editor's **Fill** section flips the same field and shows the bevel depth slider. Panels,
windows, popups and the modal card stay flat: an `egui::Frame` paints them and it takes one colour.

Full contract — the mesh, the clamp, and what stays solid — in [`docs/gradient.md`](docs/gradient.md).

### Chips

`Chip` is an auto-width pill for one short label — tags, filters, statuses. It sizes itself to
the text (unlike `Badge`, whose circle grows with the label) and takes its fill and ink from
`ThemeColors::tint`, so the label keeps AA contrast in every theme.

```rust
use egui_elements::widgets::{Chip, ChipRole};

ui.add(Chip::new("Design").role(ChipRole::Info));
ui.add(Chip::new("Needs reply").role(ChipRole::Warning).dot(true));
ui.add(Chip::new("Archived").outline(true));
```

`ChipRole` picks the palette slot (`Neutral`, `Accent`, `Info`, `Success`, `Warning`,
`Error`). `.dot(true)` prefixes a status dot in the role colour; `.outline(true)` swaps the
tinted fill for a role-coloured border with theme text. Sizing knobs: `.padding(Vec2)`,
`.corner_radius(f32)` (default is a pill) and `.text_style(TextStyle)`.

### Scroll areas and dividers

`Scroll` is a `ScrollArea` with desktop-panel geometry: a 6-point bar, a 2-point inner margin
and a 4-point outer margin, with the handle colours still coming from the theme. `.bars(...)`
hides it for panes that scroll with the wheel only.

```rust
use egui::scroll_area::ScrollBarVisibility;
use egui_elements::widgets::Scroll;

Scroll::vertical("reading")
    .bars(ScrollBarVisibility::AlwaysHidden)
    .show(ui, |ui| {
        ui.label("Wheel-scrolled body");
    });
```

`Divider` is a one-pixel rule in `colors.border` — horizontal across the available width, or a
short vertical one between toolbar items. `.inset(f32)` skips space at the start edge.

```rust
use egui_elements::widgets::Divider;

ui.add(Divider::new().inset(8.0));
ui.add(Divider::vertical().length(14.0));
```

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

A badge can hang off the corner instead of sitting inside it, and can animate:

```rust
use egui_elements::widgets::{
    Badge, BadgeBounce, BadgeBounceTrigger, BadgePulse,
};

// The circle's centre on the button's top-right corner.
ui.add(Button::new("Messages").badge(Badge::new("3").straddle()));

// A looping radar-like halo.
ui.add(Button::new("Inbox").badge(Badge::new("12").straddle().pulse(BadgePulse::new())));

// Hop when the count changes (the default trigger).
ui.add(Button::new("Episodes").badge(
    Badge::new(episodes.to_string()).straddle().bounce(BadgeBounce::new()),
));

// Hop only when the host says so.
let badge = Badge::new("1")
    .id("sith")
    .straddle()
    .bounce(BadgeBounce::new().trigger(BadgeBounceTrigger::Manual));
ui.add(Button::new("Sith").badge(badge));
if new_episode {
    Badge::bounce_now(ui.ctx(), "sith");
}
```

`.straddle()` centres the badge on the host's corner (`.anchor(BadgeAnchor::Corner)` with no explicit inset);
`.inset(..)` then pushes it further out. `BadgePulse` is the loop — `scale`, `alpha` and `period` default to
the measured `1.9 x`, `0.50` and `1.5 s`. `BadgeBounce` is the hop, `0.5 x` the diameter at peak, triggered by
`OnChange` (the default), `Manual` (via `Badge::bounce_now`) or `Loop`.

A badge can also be rendered on its own, with no host widget: `ui.add(Badge::new("3"))` allocates its own
square. Both the halo and the hop paint outside the badge's rect, so a badge animating inside a clipped parent
is clipped with it.

### Spinners

`Spinner` has the same geometry as egui's spinner — a rotating arc whose sweep is
modulated by `sin(time)` — drawn with the theme accent and three colour effects
that make it read as a light orbiting the ring: a glow, a tail-to-head alpha ramp
on the arc, and a faint track ring.

```rust
use egui_elements::widgets::{Spinner, SpinnerShadow};

ui.add(Spinner::new());                              // accent, interact_size.y square
ui.add(Spinner::new().size(48.0));                   // square, proportional stroke
ui.add(Spinner::new().color(Color32::WHITE));        // glow re-derives from the override
ui.add(Spinner::new().shadow(SpinnerShadow::NONE));  // no glow
ui.add(Spinner::new().comet(0.0).track_strength(0.0)); // flat arc, no track
```

The colour comes from `Theme::colors_from_ctx`, so an installed theme is enough.
The track (`.track_strength`, 0.16 of the arc colour) and the glow
(`.shadow(SpinnerShadow::new(color))`, a reach of 0.22·size at 0.30 peak alpha)
are both fractions of the *final* arc colour, so overriding `.color()` repaints
the whole spinner coherently. `.comet()` ramps the arc from a quarter-strength
tail to a full-strength head (`0.0` paints a flat arc), `.thickness()` overrides
the stroke (default 10 % of the size, clamped 2–6 pt), and
`paint_at(&Ui, Rect)` draws it inside another widget. As with any shadow, the glow
spills past the allocated square by `blur_ratio · size`.

### Checkboxes

`Checkbox` is a themed checkbox whose fill, border, tick and shadow animate on
toggle. Geometry follows the reference (a 16 px box): a 0.17·size corner radius,
a 1 pt border and a three-point tick with round caps — all ratios of one size, so
`.size()` scales the whole widget.

```rust
use egui_elements::motion::Motion;
use egui_elements::widgets::{Checkbox, CheckboxShadow, CheckboxVisuals};

ui.add(Checkbox::new(&mut two_factor, "SMS verification"));
ui.add(Checkbox::without_text(&mut flag));              // box only, for a table cell
ui.add(Checkbox::new(&mut flag, "Large").size(28.0));   // every ratio scales
ui.add(Checkbox::new(&mut flag, "Flat").shadow(CheckboxShadow::NONE));
ui.add(Checkbox::new(&mut flag, "Instant").motion(Motion::instant()));
```

Every colour derives from the installed theme: the checked fill is `colors.accent`,
the tick is `colors.bg`, and the idle box is `colors.border` over a transparent
fill, so the same widget reads on `bg`, `frame1` and `window`. On a click the
fill, the border and the shadow's alpha follow one clock while the tick pops in
with a slight overshoot, and a press scales the box to 0.93. The duration is the
style's `animation_time`; `.motion(..)` overrides it and `Motion::instant()` snaps.

`.visuals(CheckboxVisuals { .. })` replaces the colours for one widget — start from
`CheckboxVisuals::from_theme(theme.colors)` to keep the rest. A click reports
`Response::changed`, Space/Enter work while the box is focused, and
`ui.add_enabled(false, Checkbox::new(..))` mutes the whole row.

### Radios

`Radio` shares the checkbox's plate — the same row, colours, shadow and clock — with
a circular outline and a dot for its mark, so `.size()` scales both the same way.

A group is **every radio bound to the same `&mut value`**: selecting one sets it and
the rest of the group follows, while clicking the selected radio changes nothing.

```rust
use egui_elements::motion::Motion;
use egui_elements::widgets::{Radio, ToggleShadow, ToggleVisuals};

ui.add(Radio::new(&mut mode, Mode::Fast, "Fast"));
ui.add(Radio::without_text(&mut mode, Mode::Slow));    // plate only, for a table cell
ui.add(Radio::new(&mut mode, Mode::Fast, "Large").size(28.0));
ui.add(Radio::new(&mut mode, Mode::Fast, "Flat").shadow(ToggleShadow::NONE));
ui.add(Radio::new(&mut mode, Mode::Fast, "Instant").motion(Motion::instant()));
```

The shared knobs take the same types as the checkbox: `.visuals(ToggleVisuals)` and
`.shadow(ToggleShadow)` replace the palette-derived colours and lift for one widget,
and `.motion(Motion)` overrides the style's `animation_time`. A click reports
`Response::changed`, Space selects a focused radio, and
`ui.add_enabled(false, Radio::new(..))` mutes the whole row.

### Switches

`Switch` draws the checkbox's plate as a track with a sliding knob: the same
`ToggleVisuals`, `ToggleShadow` and clock, so it sits beside a checkbox without
changing the language. The track is 1.9 times its height and the knob 0.82 of it,
so one `.size()` scales both. Off, the track is an outline and the knob takes the
border colour; on, the track fills with `accent` and the knob takes `mark`, exactly
like the checkbox's tick.

```rust
use egui_elements::motion::Motion;
use egui_elements::widgets::{Switch, ToggleShadow, ToggleVisuals};

ui.add(Switch::new(&mut dark_mode, "Dark mode"));
ui.add(Switch::without_text(&mut flag));               // track only, for a table cell
ui.add(Switch::new(&mut flag, "Large").size(28.0));    // height; track and knob scale
ui.add(Switch::new(&mut flag, "Flat").shadow(ToggleShadow::NONE));
ui.add(Switch::new(&mut flag, "Instant").motion(Motion::instant()));
```

`.visuals(ToggleVisuals { .. })` replaces the colours for one switch — start from
`ToggleVisuals::from_theme(theme.colors)` to keep the rest. A click anywhere in the
row reports `Response::changed`, Space toggles a focused switch, and
`ui.add_enabled(false, Switch::new(..))` mutes the whole row.

### Tabs

`Tabs` is a segmented control: one pill indicator on a fully rounded track, and
the pill *slides* — travelling and stretching — to whichever segment you pick,
instead of a cell lighting up in place. The selected segment carries the palette's
`highlight` fill with `ink_on` ink, hover eases the segment's ink toward `text`
over a `hover` wash, and the pill rides on the toggle family's `ToggleShadow`, so
a strip and a switch on one card cast the same kind of shadow. The timing is the
style's `animation_time`, like every other crate widget.

```rust
use egui_elements::motion::Motion;
use egui_elements::widgets::{Tabs, TabsVisuals, ToggleShadow};

Tabs::new("filter", &mut selected, ["All", "Media", "Files"]).show(ui);
Tabs::new("range", &mut selected, ["Day", "Week"]).fill_width(false).show(ui);
Tabs::new("tall", &mut selected, ["A", "B"]).height(44.0).show(ui);
Tabs::new("flat", &mut selected, ["A", "B"]).shadow(ToggleShadow::NONE).show(ui);
Tabs::new("instant", &mut selected, ["A", "B"]).motion(Motion::instant()).show(ui);
```

`.visuals(TabsVisuals { .. })` replaces the colours for one strip — start from
`TabsVisuals::from_theme(theme.colors)` to keep the rest. A first-sight strip is
still (the indicator is seeded at the selection), and a click reports
`Response::changed`.

`.style(TabsStyle::Underline)` paints the page-tab shape instead: fitted tabs on a
one-point `border` baseline that runs across the row, the selected tab's ink in
`accent`, and a two-point `accent` bar sitting on the baseline under that tab,
travelling to its neighbour exactly as the pill does. Underline tabs are always
fitted to their labels; `.fill_width(false)` only stops the baseline at the last
tab. Its defaults come from `TabsVisuals::underline_from_theme(theme.colors)`.

```rust
use egui_elements::widgets::{Tabs, TabsStyle};

Tabs::new("page", &mut selected, ["Banners", "Posts", "Members"])
   .style(TabsStyle::Underline)
   .show(ui);
Tabs::new("page_fitted", &mut selected, ["Overview", "Activity"])
   .style(TabsStyle::Underline)
   .fill_width(false)
   .show(ui);
```

In both styles the segment you click keeps its hover (or press) wash until the
indicator has landed on it, so a click never flashes the bare surface while the
motion is still running.

### Avatar groups

`AvatarGroup` stacks avatars — each one overlapping the previous by a share of its diameter — and fans
them out while the pointer is over the stack, so every picture becomes visible. Only the spacing
animates, on the style's `animation_time`; the diameter, the ring and the fallback plate stay put.

```rust
use egui_elements::widgets::{Avatar, AvatarGroup};

let chains = [
    Avatar::initials("E"),
    Avatar::initials("A").plate(theme.colors.info),
    Avatar::initials("O"),
    Avatar::initials("P"),
];
ui.add(AvatarGroup::new(&chains).size(28.0));

// A picture clips to the circle; `Image` accepts textures, URIs and SVGs.
ui.add(AvatarGroup::new(&[Avatar::image(Image::new("bytes://icon.png"))]));
```

Geometry is measured from the reference this was ported from and expressed as ratios of `.size()`
(default 32): the collapsed step is `0.69 × d` (an `overlap` of `0.31`), the expanded step `1.11 × d`
(a `gap` of `0.11`), and the ring `0.08 × d`, painted inside each avatar's edge. The ring defaults to
`colors.bg` — the page — because it is what separates a front avatar from the one behind it; on a
`frame1` surface pass
`.visuals(AvatarVisuals { ring: colors.widget_bg, ..AvatarVisuals::from_theme(colors) })`.

The group always reserves the **expanded** width and spreads the stack around a fixed centre, so a
neighbour never reflows and nothing paints outside the allocated rect. `.align(Align::Min)` pins the
leftmost avatar instead and lets the fan grow rightwards, which is what a row wants. `.motion(Motion)`
overrides the clock, `Motion::instant()` snaps, and `.expand(false)` keeps a group permanently stacked.
`Avatar::initials` falls back to the accent plate with `bg` text (the theme rule for chromatic fills);
`Avatar::plate` / `.ink` override one entry. Keep initials to one or two characters: longer text is
shrunk to fit the plate, and at small sizes it still ends up under the next avatar's ring.

Run the proof with `cargo run --example avatar`.

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

Anchored popover proof (a symbol picker, an action menu and an above-side inspector):

```sh
cargo run --example popover
```

Product-framed reference client (mail UI):

```sh
cargo run --example email
```

Telegram Desktop replica: a two-pane `Flex` shell (chat list, conversation, overlays and settings) built from the widget set, with every `ThemeColors` slot in use. The title strip opens a live theme-audit panel that names the installed palette, shows each slot and its contrast checks, and probes the same gallery on `bg`, `frame1` and `frame2`:

```sh
cargo run --features secure-types --example telegram
```

Segmented tabs, filled and fitted:

```sh
cargo run --example tabs
```

Themed text edits — hint text, character limit, multiline:

```sh
cargo run --example text_edit
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

Avatar groups: the reference stack, live geometry knobs and a multi-chain balance row:

```sh
cargo run --example avatar
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
| `Custom(String)` | Runtime-defined: the persisted `Theme` is the definition, the string is its name |

A customized theme keeps its own name instead of pretending to be a built-in.
Retag it, persist the whole theme, and load it back with its own kind:

```rust
let mut theme = Theme::new(ThemeKind::TokyoNight);
theme.colors.accent = egui::Color32::from_rgb(255, 91, 103);
theme.rename("Sunset");

let json = serde_json::to_string(&theme)?; // "kind":{"Custom":"Sunset"}
let mut restored: Theme = serde_json::from_str(&json)?;
restored.install(&ctx);
```

`ThemeKind::Custom` is an identity only: `Theme::new` cannot rebuild a palette
from a name (it falls back to Tokyo Night; `Theme::try_new` returns `None`).
Requires the `serde` feature.

Palette slot roles (`bg` vs `widget_bg`, `hover` vs `highlight`, …) are documented in [`docs/theme.md`](docs/theme.md).

## License

MIT. Icons from the `lucide` feature are [Lucide](https://lucide.dev) (ISC).

The Snow particle effect is ported from [egui-snow](https://github.com/LaVashikk/egui-snow) (MIT, © 2025 LaVashikk).
