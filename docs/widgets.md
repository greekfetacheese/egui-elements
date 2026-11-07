# Widgets

Everything under `src/widgets/`. Each entry says what the type does and how it is built; the example
column is the proof that ships with the crate, so every claim below can be run.

```sh
cargo run --example demo --features full   # the whole gallery
```

| Entry | Example |
|---|---|
| [Button](#button) | `cargo run --example demo` |
| [Hold-to-confirm buttons](#hold-to-confirm-buttons) | `cargo run --example hold_to_confirm` |
| [Checkboxes](#checkboxes) | `cargo run --example checkbox` |
| [Radios](#radios) | `cargo run --example radio` |
| [Switches](#switches) | `cargo run --example switch` |
| [Sliders](#sliders) | `cargo run --example slider` |
| [ComboBox](#combobox) | `cargo run --example combo_box` |
| [TextEdit / SecureTextEdit](#textedit--securetextedit) | `cargo run --example text_edit` |
| [Tabs](#tabs) | `cargo run --example tabs` |
| [Chips](#chips) | `cargo run --example demo` |
| [Label](#label) | `cargo run --example label_sections` |
| [MultiLabel](#multilabel) | `cargo run --example demo` |
| [Badges](#badges) | `cargo run --example badge` |
| [Avatar groups](#avatar-groups) | `cargo run --example avatar` |
| [Indicators](#indicators) | `cargo run --example indicator` |
| [FPS meters](#fps-meters) | `cargo run --example fps_meter` |
| [Frame](#frame) | `cargo run --example email` |
| [Surfaces](#surfaces) | `cargo run --example email` |
| [Gradient fills](#gradient-fills) | `cargo run --example gradient` |
| [Scroll areas and dividers](#scroll-areas-and-dividers) | `cargo run --example demo` |
| [Window](#window) | `cargo run --example demo` |
| [Modal](#modal) | `cargo run --example demo` |
| [Popover](#popover) | `cargo run --example popover` |
| [Toasts](#toasts) | `cargo run --example toast` |
| [Spinners](#spinners) | `cargo run --example spinner` |
| [Ripples](#ripples) | `cargo run --example ripples` |
| [Orbs](#orbs) | `cargo run --example orb` |

---

## The gallery in one go

The smallest complete surface: a theme installed once, then the widgets a page reaches for.

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

## Button

The crate's button: themed chrome read from the installed theme, egui atoms for the label, and two
crate extras — a corner `Badge` and hold-to-confirm.

**Signature** `Button::new(atoms)` · `Button::selectable(selected, atoms)` · `Button::image(img)` ·
`Button::image_and_text(img, text)`

```rust
use egui::Align;
use egui_elements::{Badge, Button};

ui.add(Button::new("Save"));
ui.add(Button::new("Inbox").badge(Badge::new("3")));
ui.add(Button::selectable(selected, "All"));

// A chromatic fill needs its own ink, or the label keeps `colors.text`:
// `filled` sets it to `colors.ink_on(fill)` and mixes hover/press toward `bg`.
let danger = theme.button_visuals().filled(theme.colors.error, theme.colors);
ui.add(Button::new("Delete").visuals(danger));
```

`Button::new` takes `impl IntoAtoms` — a `&str`, a `WidgetText`, an `Image`, or an `(image, text)`
tuple. `.min_size(vec2(width, 0.0))` fixes the width and needs `.label_align(Align::Center)`, or a short
caption sits left-aligned next to a centred sibling. `.small()` shrinks the chrome, `.selected(true)`
paints the `highlight` state, `.visuals(ButtonVisuals)` overrides one slot without touching the theme.

`.bg_color(fill)` replaces the fill and leaves the ink at `colors.text` — right for a surface-coloured
button, wrong for a chromatic one. Whenever the fill is not a surface, use
`.visuals(theme.button_visuals().filled(fill, theme.colors))`: the label becomes `colors.ink_on(fill)`,
which clears AA on every built-in palette, and hover/press mix the fill toward `bg`.

Hold-to-confirm is a button mode, not a separate widget — see
[Hold-to-confirm buttons](#hold-to-confirm-buttons).

`Button` shadows `egui::Button`: import it after `use egui::*` or alias one of the two.

## Hold-to-confirm buttons

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

## Checkboxes

`Checkbox` is a themed checkbox whose fill, border, tick and shadow animate on
toggle. Geometry follows the reference (a 16 px box): a 0.17·size corner radius,
a 1 pt border and a three-point tick with round caps — all ratios of one size, so
`.size()` scales the whole widget.

```rust
use egui_elements::motion::Motion;
use egui_elements::widgets::{Checkbox, CheckboxShadow, CheckboxVisuals};

let mut two_factor = false;
let mut flag = false;

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

## Radios

`Radio` shares the checkbox's plate — the same row, colours, shadow and clock — with
a circular outline and a dot for its mark, so `.size()` scales both the same way.

A group is **every radio bound to the same `&mut value`**: selecting one sets it and
the rest of the group follows, while clicking the selected radio changes nothing.

```rust
use egui_elements::motion::Motion;
use egui_elements::widgets::{Radio, ToggleShadow, ToggleVisuals};

#[derive(PartialEq)]
enum Mode {
    Fast,
    Slow,
}

let mut mode = Mode::Fast;

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

## Switches

`Switch` draws the checkbox's plate as a track with a sliding knob: the same
`ToggleVisuals`, `ToggleShadow` and clock, so it sits beside a checkbox without
changing the language. The track is 1.9 times its height and the knob 0.82 of it,
so one `.size()` scales both. Off, the track is an outline and the knob takes the
border colour; on, the track fills with `accent` and the knob takes `mark`, exactly
like the checkbox's tick.

```rust
use egui_elements::motion::Motion;
use egui_elements::widgets::{Switch, ToggleShadow, ToggleVisuals};

let mut dark_mode = false;
let mut flag = false;

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

## Sliders

`Slider` is a themed slider ported from
[Web Awesome's `wa-slider`](https://webawesome.com/docs/components/slider/): a fully rounded
rail, a circular thumb with a thin surface ring, a label above, optional reference captions
below and an optional formatted value beside the rail. Every measurement is a ratio of
`.size()` (the thumb's diameter), so one knob scales the rail, the ring and the step markers
together.

```rust
use egui_elements::widgets::Slider;

let mut users = 0.0f32;
let mut volume = 0.0f32;
let mut level = 0.0f32;
let mut price = 0.0f32;

ui.add(
    Slider::new(&mut users, 0.0..=6.0)
        .text("Number of users")
        .step_by(1.0)
        .markers(true)
        .tooltip(true)
        .references(["Less", "More"])
        .hint("Limit six per team"),
);

ui.add(Slider::new(&mut volume, 0.0..=100.0).text("Volume").fixed_decimals(1));
ui.add(Slider::new(&mut level, 0.0..=100.0).vertical().height(180.0));
ui.add(
    Slider::new(&mut price, 0.0..=100.0)
        .custom_formatter(|value, _| format!("${value:.0}")),
);
```

The value cell is an editable `DragValue` (drag it or click it to type) placed by a `Flex`
row, so it stays centred on the rail; `.show_value(false)` drops it. `.custom_formatter`
drives both the number and the tooltip, and the step decides the written precision unless
`.min_decimals` / `.max_decimals` / `.fixed_decimals` pin it. A focused slider takes the
arrow keys (Home and End jump to the ends) and `Response::changed` fires on the frame that
moves the value. `.visuals(SliderVisuals { .. })` replaces the colours for one slider — start
from `SliderVisuals::from_theme(theme.colors)` to keep the rest.

Proof: `cargo run --example slider`.

## ComboBox

A themed dropdown: a button-shaped selected value plus a popup list.

**Signature** `ComboBox::new(id_salt, selected_label).width(w).show_ui(ui, |ui| …)`

```rust
use egui::Sense;
use egui_elements::{ComboBox, Label};

let current = Label::new(text.as_str(), None);
ComboBox::new("item", current).width(200.0).show_ui(ui, |ui| {
    for item in ["Alpha", "Beta", "Gamma"] {
        let row = Label::new(item, None).sense(Sense::click()).fill_width(true).expand(Some(4.0));
        if ui.add(row).clicked() {
            text = item.to_owned();
        }
    }
});
```

- Every `ComboBox` on screen needs its own `id_salt`. A gallery rendered on `bg`, `frame1` **and** a
  `Window` must use distinct salts, or the open state and the selection are shared.
- `show_rows_ui(ui, row_height, total_rows, |ui, range| …)` builds only the visible rows for long
  lists; every row must lay out at exactly `row_height` (egui adds `item_spacing.y` itself).
- `.popup_gap(points)` (default `ThemeSpacing::xs`) and `.popup_content_margin(points)` (default `5`)
  tune the popup chrome; `.label("…")` puts a caption above the button; `.popup_max_height(..)` caps it.
- It paints a real button, so its height is `inner_height.max(icon_width) + 2 × button_padding` —
  about 34 pt with default padding. A host row sized to `interact_size.y` (20) will be overflowed.
- Proof: `examples/combo_box.rs` — `cargo run --example combo_box`.

## TextEdit / SecureTextEdit

One themed text box under two names: `TextEdit` for ordinary buffers, `SecureTextEdit` for
`secure-types` buffers and masking. Both drive egui's `TextBuffer`, so a `String` works everywhere.

**Signature** `TextEdit::singleline(&mut buffer)` · `TextEdit::multiline(&mut buffer)` (alias of
`SecureTextEdit`)

```rust
use egui_elements::{SecureTextEdit, TextEdit};

let mut name = String::from("Rey");
let mut notes = String::new();

ui.add(TextEdit::singleline(&mut name).hint_text("Name"));
ui.add(TextEdit::multiline(&mut notes));
ui.add(SecureTextEdit::singleline(&mut password).password(true));
```

- `TextEdit` is available without any feature; `SecureTextEdit` needs `secure-types` for a
  `SecureString` buffer, and `.password(true)` masks it.
- Leaving `.visuals(..)` unset falls back to the theme's text-edit chrome read from the context, so the
  field follows the installed theme without help.
- `.hint_text(..)` is the placeholder drawn while the buffer is empty.
- Proof: `examples/text_edit.rs` — `cargo run --example text_edit`.

## Tabs

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

let mut selected = 0usize;

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

let mut selected = 0usize;

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

## Chips

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

## Label

The crate's text primitive: an optional leading image, click sense, hover/selection chrome, and a
mixed-style variant that wraps like a paragraph.

**Signature** `Label::new(text, image)` · `Label::sections([RichText, …], image)`

```rust
use egui::{RichText, Sense};
use egui_elements::Label;

ui.add(Label::new("Caption", None));

ui.add(Label::new("Open details", None).sense(Sense::click()).expand(Some(4.0)).fill_width(true));

ui.add(
    Label::sections(
        vec![RichText::new("Draft").strong(), RichText::new(" · unsaved")],
        None,
    )
    .wrap()
    .fill_width(true),
);
```

- `Label::sections` builds **one wrapping galley** of mixed styles — store a `Vec<RichText>` and join at
  layout time, because a `Style` is not known in the constructor. `RichText::strong()` is a stronger
  colour, not a bold weight (the default fonts have no bold face).
- `.size(f32)` overrides every span's size. `.interactive(false)` (the default) paints no chrome;
  `.sense(Sense::click())` + `.expand(Some(n))` give a row hover/selection chrome around a clickable
  label. `.selected(bool)` paints the chosen state.
- `.fill_width(true)` stretches the label to the available width; `.wrap()` / `.wrap_mode()` control
  wrapping. A label with an image must size it through the image's fitted size, not its native texture
  size — the crate does this for you when the image is passed to the constructor.
- Proof: `examples/label_sections.rs` — `cargo run --example label_sections`.

## MultiLabel

Several [`Label`](#label)s on one horizontal line, sharing a vertical centre and one spacing value — a
status row (avatar, name, tag) without a nested `Ui` per item.

**Signature** `MultiLabel::new(vec![Label, …]).inter_spacing(f32).sense(Sense)`

```rust
use egui_elements::{Label, MultiLabel};

ui.add(
    MultiLabel::new(vec![
        Label::new("ada", None),
        Label::new("·", None),
        Label::new("admin", None).selected(true),
    ])
    .inter_spacing(8.0),
);
```

`MultiLabel` lays out horizontally and does **not** wrap a paragraph — for mixed styles in running text
use `Label::sections` instead.

## Badges

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

let episodes = 3;
let new_episode = true;

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

## Avatar groups

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

## Indicators

`Indicator` is a small status light with three states: `On` (a filled success dot
inside a soft glow), `Connecting` (a warning ring) and `Off` (an error ring with a
bar through it).

```rust
use egui_elements::widgets::{BadgePulse, Indicator, IndicatorState};

ui.add(Indicator::new(IndicatorState::On));
ui.add(Indicator::new(IndicatorState::Connecting).size(14.0));
ui.add(Indicator::new(IndicatorState::Off).size(14.0));
```

Any state can pulse with the badge's loop — the same `BadgePulse` parameters, so
`period`, `scale`, `alpha` and the easing behave exactly as they do on a badge:

```rust
ui.add(Indicator::new(IndicatorState::On).pulse(BadgePulse::new()));
ui.add(Indicator::new(IndicatorState::Connecting).pulse(BadgePulse::new().period(2.0)));
```

The phase is derived from each light's id, so siblings in one row are staggered
instead of beating in lockstep. The glow and the pulse paint outside the allocated
square, exactly like the badge halo — leave a gap in a tight layout. A pulsing light
repaints every frame; drop `.pulse(..)` when the state is static. Proof:
`cargo run --example indicator`.

## FPS meters

`FpsMeter` measures the frame rate and the CPU time the host reports for a frame. Call
`frame` once per frame with the host's own `Instant::elapsed()`; the meter times the
interval between calls itself, so the two readings stay independent.

```rust
use std::time::Instant;
use egui_elements::widgets::FpsMeter;

let mut meter = FpsMeter::new();

// once per frame, at the end of the host's work
let start = Instant::now();
meter.frame(start.elapsed());

// a themed floating window …
meter.show_window(ctx);

// … or the chrome-free body inside any container
meter.body_ui(ui);

// the meter never schedules frames: repaint from the host
ctx.request_repaint();

// the title bar's close button hides the window; a toolbar can restore it
if !meter.is_open() {
    meter.set_open(true);
}
```

`max_fps()` is the ceiling the frame work allows — `1000 / cpu_ms()` — so a vsync-capped
60 Hz display still reports its headroom (a 4 ms frame reads 250). `fps()` keeps the
measured rate: the mean over the sample window (120 frames by default, about two seconds
at 60 fps). `stats()` returns the min / mean / max of both the frame interval and the CPU
time, as a `Spread`. An empty window paints an em dash. The body never schedules a
repaint: the host owns the frame loop, and `ctx.request_repaint()` is the live-metric call
when the app is otherwise idle. The window floats and can be dragged; `anchor` pins it to a screen corner
instead (a pinned window cannot be dragged). Its title bar closes it: `is_open` reads that
state and `set_open` restores it from a toolbar toggle, and nothing is painted while closed.
Two meters need distinct window titles, because the title is the window id. `with_capacity`
resizes the window and `clear` restarts the measurement.
Proof: `cargo run --example fps_meter`.

## Frame

The crate's frame: `egui::Frame`'s builder shape plus a per-state fill and stroke, a shadow, optional
hover/click/selection behaviour, and a `Surface` shortcut.

**Signature** `Frame::new()` · `Frame::from_egui(egui::Frame)` · `Frame::surface(&theme, Surface::Panel)`

```rust
use egui_elements::layout::Flex;
use egui_elements::{Frame, Label, Surface};

Frame::surface(&theme, Surface::Panel).show(ui, |ui| {
    ui.add(Label::new("Card", None));
});

let row = Frame::new()
    .inner_margin(egui::Margin::same(8))
    .interactive(true)
    .selected(is_current)
    .show(ui, |ui| {
        ui.add(Label::new("Inbox", None));
    });
```

- `Frame::new()` is `Frame::NONE`. Builders: `.fill()` (a `Fill`, so a gradient works), `.stroke()`,
  `.corner_radius()`, `.shadow()`, `.inner_margin()` / `.outer_margin()`, `.fill_width(bool)`,
  `.fit()`, `.sense()`.
- `interactive(false)` (the default) always paints the idle fill; hover and press only paint with
  `.interactive(true)`. `selected(true)` paints `FrameVisuals::bg_selected` (`highlight`) — do not fake
  selection with `.fill()`.
- `show(ui, |ui| …)` returns an `InnerResponse`. `.begin(ui)` hands back the content `Ui` plus a
  `Prepared` when a widget has to paint its own corner chrome (badges, custom rows).
- Plain chrome stays on `theme.frame1` / `theme.window_frame` (an `egui::Frame`) where no state is
  wanted; `Frame` is the interactive, stateful one.

## Surfaces

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

## Gradient fills

A fill slot holds a `Fill`: a solid `Color32` or a two-stop `Gradient`. egui paints a rect with
one colour, so a gradient goes through a vertex-coloured mesh; the ramp is exact because the shader
interpolates the two stops across a convex rounded rect, and the mesh carries a feathered rim so it
keeps the anti-aliasing of a painted rect.

```rust
use egui_elements::gradient::{Fill, Gradient};
use egui_elements::visuals::ButtonVisuals;

let button_visuals = theme.button_visuals();

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

`theme.gradient_dir` picks the axis (`TopDown` by default), and `cargo run --example gradient` shows
all four `GradientDir` axes on the same widget set.

Full contract — the mesh, the clamp, and what stays solid — in [`gradient.md`](gradient.md).

## Scroll areas and dividers

The installed theme pins the stock scroll-bar geometry — a 6-point bar, a 2-point inner margin
and a 4-point outer margin — so every `egui::ScrollArea` in the app draws the same thin bar,
with the handle colours still coming from the theme. Use `ScrollArea` directly;
`scroll_bar_visibility` hides the bar for panes that scroll with the wheel only.

```rust
use egui::scroll_area::ScrollBarVisibility;
use egui::ScrollArea;

ScrollArea::vertical()
    .id_salt("reading")
    .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
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

## Window

`egui::Window` with a title fill that stays put. Stock egui overwrites the title bar fill
(`widgets.open.weak_bg_fill`) whenever the window holds focus, so opening a second window changes the
colour of the first one's title.

**Signature** `Window::new(title).open(&mut open).frame(theme.window_frame).show(ctx, |ui| …)`

```rust
use egui::Stroke;
use egui_elements::Window;

Window::new("Settings")
    .open(&mut open)
    .frame(theme.window_frame)
    .title_frame(theme.window_frame.stroke(Stroke::NONE))
    .show(ctx, |ui| {
        ui.label("…");
    });
```

The rest of the builder is forwarded from `egui::Window`: `.resizable(false)`, `.default_pos(..)`,
`.collapsible(..)`, `.anchor(..)` all behave as they do upstream. Only the focus-driven title fill is
pinned (and restored when `show` returns), so a themed title bar does not flicker between states.

## Modal

A centered themed card over a dimmed backdrop that swallows clicks: heading, optional subtitle and
header icon, optional footer, and three ways out (`Esc`, the ×, the backdrop).

**Signature** `Modal::new(id, &mut open).heading(..).subtitle(..).footer(..).show(ctx, |ui| …) -> Option<R>`

```rust
use egui_elements::{Button, Label, Modal};

let mut cancelled = false;
Modal::new("confirm", &mut open)
    .heading("Discard changes?")
    .subtitle("This cannot be undone.")
    .max_width(420.0)
    .footer(|ui| {
        if ui.add(Button::new("Cancel")).clicked() {
            cancelled = true;
        }
    })
    .show(ctx, |ui| {
        ui.add(Label::new("Three files are modified.", None));
    });
```

- `id` accepts `Id`, `&str` or `String` (the crate's `IntoModalId`). `show` returns `Some` only while
  the modal is open, so `if let Some(..)` gates work on the body's return value.
- Defaults: left-aligned title and icon, separators on, **no maximum width** — a long body stretches
  the card unless `.max_width(..)` or `.width(..)` is set.
- `.center_header(true)` centres the title, `header_icon(egui::Image)` takes an image (a Lucide SVG,
  not a glyph), `.closable(false)`, `.close_on_backdrop(false)` and `.close_on_escape(false)` remove
  exits, and `.alert(true)` marks the dialog as an alert for assistive tech.

## Popover

An anchored overlay panel — picker, menu, small inspector — that owns the surface, the placement and
the two dismissal rules. The caller keeps the open flag and the anchor rect.

**Signature** `Popover::new(id_salt, &mut open).side(PopoverSide::Below).show(ctx, anchor, |ui| …)`

```rust
use egui_elements::{Label, Popover, PopoverSide};

let anchor = ui.max_rect();
Popover::new("emoji", &mut open)
    .side(PopoverSide::Below)
    .gap(4.0)
    .width(220.0)
    .show(ctx, anchor, |ui| {
        ui.add(Label::new("…", None));
    });
```

- `PopoverSide::{Below (default), Above, Right, Left}`, plus `.gap(points)` and `.width(points)`.
- Closes on a click outside and on `Esc` by default; `.close_on_click_outside(false)` and
  `.close_on_escape(false)` keep it up.
- The surface is `Surface::Overlay` (the theme's window frame) unless `.frame(Frame)` overrides it, and
  `.order(Order)` controls the layer.
- It never flips itself: a host that needs side selection computes it from its own layout.
- Proof: `examples/popover.rs` — `cargo run --example popover`.

## Toasts

`Toast` enqueues a notification; `Toasts` draws the stack. The builder takes only
`&Context`, so any callback that holds the context can fire one, and the renderer is
a single call in your top-level update:

```rust
use egui_elements::widgets::{ChipRole, Toast, Toasts};

// Once per frame, after the panels:
Toasts::new().render(ctx);

// From a button handler, a completion callback, …
Toast::new("Deploy complete")
    .tone(ChipRole::Success)
    .description("Rolled out to us-east-1")
    .show(ctx);
```

`Toast::new("…")` draws the title with a left accent bar tinted by the tone — a
`ChipRole`, the same semantic role enum `Chip` uses: `Success`, `Error`, `Warning`,
`Info` and `Neutral` (`Accent` also works). `.description(…)` adds a second, muted
line, `.duration(Duration::from_secs(8))` overrides the four-second auto-dismiss, and
`.persistent()` keeps the toast until the user clicks ×.

`Toasts::new()` configures the renderer, and every knob is a builder:

```rust
use egui::Align2;

Toasts::new()
    .anchor(Align2::RIGHT_BOTTOM)   // the default corner
    .offset((12.0, 12.0))           // points from that corner
    .width(320.0)                   // card width
    .max_visible(5)                 // oldest toasts are dropped past the cap
    .clear_all_button(true)         // "Clear all" pill once two pile up
    .render(ctx);
```

The stack fades out only (never in), hovering a toast freezes its auto-dismiss
countdown so it can be read or copied, and a settled stack requests no repaint.
Clicking × starts the fade immediately; the "Clear all" pill does the same for every
visible toast. Danger and warning tones are announced as ARIA alerts, the rest as
status messages. Proof: `cargo run --example toast`.

## Spinners

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

## Ripples

`Ripple` is the second loader: a 3 x 3 grid of coloured cells lit in reading order,
one every 100 ms, each fading over 30 % of a 1.5 s loop and dark for the remaining
40 %. It reads as blocks arriving rather than as progress, which is what the
spinner already says.

Ported from [alexruix / fluffy-starfish-52](https://uiverse.io/alexruix/fluffy-starfish-52) (MIT).

```rust
use egui_elements::widgets::Ripple;

let colours = [colors.accent; 9];

ui.add(Ripple::new());                                 // theme ramp, 2.6 x interact_size.y
ui.add(Ripple::new().size(64.0));                      // square grid, cells scale with it
ui.add(Ripple::new().period(1.0).delay(0.05));         // faster wave, tighter step
ui.add(Ripple::new().color(Color32::WHITE));           // one flat colour
ui.add(Ripple::new().cells(colours));                  // your own nine, in reading order
```

The nine colours default to a ramp derived from `ThemeColors::accent`: the accent's
hue walked by 35° at full saturation, lightness from 50 % to 69 %. Only the hue is
taken from the accent — a muted accent would make the wave read as a brightness
flicker instead of the ported hue ramp. `.color()` repaints every cell with one
colour, `.cells()` replaces the ramp whole, `.period()` and `.delay()` retime it,
and `.easing()` replaces the measured CSS `ease` curve. `paint_at(&Ui, Rect)` draws
it inside a widget you already own.

## Orbs

`Orb` is the third loader, and the only one that carries text of its own: a glowing
sphere whose lit rim orbits it once every 2 s while a label inside pulses letter by
letter, each letter 100 ms behind the one before it.

Ported from [dexter-st / bright-lizard-8](https://uiverse.io/dexter-st/bright-lizard-8) (MIT).

```rust
use egui_elements::widgets::Orb;

ui.add(Orb::new().text("Generating"));                    // accent shell, sized for the label
ui.add(Orb::new().size(64.0));                            // ball only
ui.add(Orb::new().text("Loading").color(Color32::from_rgb(0xad, 0x5f, 0xff))); // retint
ui.add(Orb::new().period(1.0).letter_delay(0.05));        // faster, tighter stagger
ui.add(Orb::new().text("Saving").text_size(13.0));        // a smaller label
```

The reference is one round element carrying three `inset` box-shadows that rotate
while their colours cross-fade, which egui cannot express: there is no inset shadow.
The port rebuilds each shadow from its geometry per vertex of one radial mesh: a
circle of the ball's radius shifted away from the light, blurred by the shadow's
radius, painted over everything outside it. The three crescents therefore have
circular edges, which is what reads as an inner sphere turning inside the ball
instead of a half-moon sweeping around it. Only the accent's hue comes from the
palette (`Theme::colors_from_ctx`): each crescent keeps the reference's saturation
and brightness, so the ball holds the source's light-and-dark structure and a muted
accent still reads as a light rather than a grey ball.

`.color()` retints the whole shell (every shell is a hue offset from it, so a pure
grey paints a silver ball), `.period()` retimes both the turn and the letter
pulse, `.letter_delay(0.0)` pulses every letter at once, `.text_size()` overrides the
label's 1.5 x body default, and `.size()` pins the square — without it the ball grows
so the label keeps 60 % of the diameter, which is the reference's 108 pt of text in
180 pt. An empty `.text("")` is the same as no label. `paint_at(&Ui, Rect)` draws it
inside a widget you already own.
