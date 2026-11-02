# Motion

Time, in one place: eased value animation and two containers that animate what they hold.

```rust
use egui_elements::motion::{Collapse, Easing, Fade, FadeMode, Lerp, Motion, animate, animate_with};
```

`motion` is re-exported at the crate root as well. Run the proof: `cargo run --example motion`.

---

## 1. What this is

`motion` is the crate's one place for time. `animate` drives a **value** — any `f32` a host wants to ease toward
a target — while `Fade` and `Collapse` drive a **container**, one by opacity and one by its own measured height.
All three take a `Motion`, which is a duration plus an `Easing`, and all three keep their state per `Id` in
`Context` memory, so a host never stores an animation value in its own struct.

That split is the whole API: a value, an opacity, a height. There is no timeline, no keyframe list and no
sequencer.

---

## 2. The CSS → API map

| CSS | here |
|-----|------|
| `transition` | `Motion` (duration + curve) |
| `transition-duration` | `Motion::duration` / `Motion::new(secs)` |
| `transition-timing-function` | `Easing` |
| an `opacity` transition | `Fade` |
| a `max-height` reveal | `Collapse` |
| interpolating two `Color32`s | `Lerp` |
| a `transition` on a hover state | `InteractT` (built into the widgets) |

Durations come from the installed style: `Motion::from_ui(ui)` reads `Style::animation_time`, which every crate
theme sets to `0.3`. A host that already tunes `animation_time` does not configure motion twice. `Motion::new`
takes an explicit duration, and `Motion::instant()` is duration `0.0` — the reduce-motion answer and the
deterministic-test answer in one call.

`Easing` is `Linear`, `EaseIn`, `EaseOut` (the default), `EaseInOut`, or `Custom(fn(f32) -> f32)`. The cubic
family is what web design systems use; `Custom` takes any function mapping `0..=1` to `0..=1`, so an
`egui::emath::easing` curve works without a new variant.

`Lerp` is a trait with one method, `lerp(self, to, t)`, implemented for `f32`, `Vec2`, `Pos2`, `Rect` and
`Color32`. The `Color32` implementation blends in **gamma space** (`Color32::lerp_to_gamma`) because linear-light
blending washes UI mid-tones out — that is the same call epaint makes.

```rust
use eframe::egui::*;
use egui_elements::motion::{Easing, Lerp, Motion};

let mix = theme.colors.bg.lerp(theme.colors.error, 0.375);
let inset = Rect::from_min_size(rect.min, vec2(0.0, 0.0)).lerp(rect, 0.5);
let motion = Motion::new(0.3).easing(Easing::EaseOut);
```

---

## 3. Why not egui's own animation

egui animates **containers, not widgets**, and the two gaps below are measured facts rather than preferences.

`AnimationManager::animate_bool` inserts an unknown id with `last_value = end` and returns that value. A first
sight of `true` therefore returns `1.0` immediately and never fades: any host that wants a fade-in has to tick
closed frames forever, just so the id exists with `false` before the first open. Zeus does exactly that in
`common::fade`.

`AnimationManager::animate_value_with_time` is linear-only. An eased value has no library route at all — the
host either accepts constant speed or writes the curve and the clock itself.

`motion` starts a first sight at `0.0` and applies `Easing`, which is those two gaps closed. A first open fades
in on its own, with no closed-frame ticking, and every curve in `Easing` is available to a value as well as to a
container.

`animate_from` is the same primitive for the opposite need. A *state* that is already true when a widget first
appears must not animate itself on: a checkbox created checked has to be checked on its first frame, not fill
itself in. It takes the first-sight value from the caller — pass `target` for "no entrance animation", `0.0` for
an entrance — and afterwards behaves exactly like `animate_with`: a changed target retargets from the value on
screen.

```rust
// A toggle: land on the state, then animate every later change from what is on screen.
let t = animate_from(ui, response.id.with("check"), target, target, Motion::from_ui(ui));
```

`Checkbox` is the widget that uses it. A fade and a toggle want opposite first frames, and that is the whole
difference between the two functions.

egui's own widgets still swap visual sets instantly through `Style::interact(&response)` — hover and active states
jump rather than transition, and that is the third gap. Every widget in this crate that paints an interaction state
(`Button`, `Frame`, `ComboBox`, `MultiLabel`, `SecureTextEdit`) now blends between them instead, through
[`InteractT`](crate::motion::InteractT):

- **Hover eases** over the `Motion` duration, so the chrome ramps instead of switching.
- **Press snaps** in *and* out. A press is a state flag, not ambience: a hold-to-confirm button has to drop its pressed
  chrome on the frame the gesture fires, and a press that fades in reads as lag on the click.
- **Selection is instant** by design: a selected button changes fill in one step.
- Each widget keeps its own condition pair — a `Frame` counts `contains_pointer()` as hover, a text edit opens on hover
  *or* focus — so *which* state applies is unchanged; only the transition between states animates.

A host that paints its own widget can use the same three calls:

```rust
use egui_elements::motion::{InteractT, Motion};

let across = InteractT::of(ui, response.id, hovered, pressed, Motion::from_ui(ui));
let fill = across.blend(idle, hover, press);
let stroke = across.blend(idle_stroke, hover_stroke, press_stroke);
```

`Motion::instant()` makes every one of them snap, which is the reduce-motion switch and the reason the tests can
assert settled colours deterministically.

---

## 4. The repaint contract

An animation requests a repaint **only while `t < 1.0`**. A settled one requests none, and `Motion::instant()`
(duration `0.0`) returns the target immediately, so it never requests one at all.

That is what makes the module usable in a battery-sensitive host: a page of settled panels does not keep the
frame loop awake. It is also how the contract is tested — `Context::has_requested_repaint()` read inside the
frame body, right after the call, after a few warm-up frames so egui's own first-frame repaint does not mask the
result.

The one rule a host has to keep: call the animation from a place that runs every frame. `FadeMode::In` does not
need closed-frame ticking, but a host that stops drawing a `Fade` entirely still pays nothing — the next open
starts from `0.0` again because the state is keyed per `Id`.

---

## 5. Ids

Animation state lives per `Id` in `Context` memory. Two rules follow, and both bite silently when broken:

- Pass a **stable** id per animation. An id derived from the frame (a counter, a timestamp, a value that
  changes) animates nothing, because every frame is a first sight at `0.0`.
- Never pass the same id to two independent animations. The second one reads and writes the first one's state,
  so the two fight over one value.

`Collapse` derives `id.with("t")` for the progress and `id.with("collapse")` for the measured height
internally, so a host can use the bare id for a value of its own in the same frame without a collision.
`Fade` uses the id directly.

---

## 6. `Collapse` behaviour

The natural height is measured **once, on the first open**, by an invisible probe run of the body. That frame
allocates `0.0` and the body runs once — not twice, unlike a measure-pass container. The measured height is
stored per `Id`, and afterwards the height follows the last measured value.

That trade has a visible edge: a body that changes size animates one frame behind, because the height it follows
is last frame's measurement. Re-measuring every frame would cost a probe pass per frame instead.

The band is the animated height. The body is laid out at its **natural** height and clipped to the band, and the
cursor advances by the band alone. Two consequences:

- A closed collapse allocates nothing, so it leaves **no gap** behind it — the rows below close up with no
  spacer to tune.
- Whatever follows the collapse — a label, a button, the next section — rides the animation without knowing it
  exists. That is the point: siblings follow for free.

The symmetry with `layout` is worth naming: a **cell rect is not a clip**. A `Flex` cell places content in a
rect but never cuts it, which is correct for a layout engine and would be wrong here. A `Collapse` intersects
its band with the parent clip explicitly, so the body can only be seen inside the animated height.

```rust
use egui_elements::motion::{Collapse, Motion};

let total = Collapse::new("details", open).motion(motion).show(ui, |ui| {
   ui.label("The body is laid out here, inside the closure.");
});
ui.label(format!("the cursor advanced {total:.1} pt"));
```

---

## 7. Host migration

| Today | `motion` |
|-------|----------|
| `common::fade::panel_fade` / `show_with_fade` | `Fade` with `FadeMode::In` |
| the account panel's `body_heights[]` + `set_clip_rect` + `set_min_height` + `PANEL_CLIP_SLACK` block | `Collapse` — the height cache, the manual clip and the slack constant all disappear |
| `tx/modals.rs`'s reveal + height pair | `Fade` + `Collapse` |

The account panel is the largest win: it caches one height per panel, clamps the clip to a slack constant and
sets a minimum height so the layout below does not jump. `Collapse` measures the body itself, clips to its own
animated band and advances the cursor by that band, so all four pieces go.

---

## 8. Proof

```sh
cargo run --example motion
```

Six sections, one claim each:

- **A. Value animation** — `animate` eases a number toward a target; the number in the caption is the value on
  screen and stops changing without repaints.
- **B. Interpolation** — nine swatches at `t = i / 8` show `Lerp` on a colour with no animation at all.
- **C. Fade** — the same toggle on two panels: `FadeMode::In` closes instantly, `FadeMode::Both` fades out.
- **D. Collapse** — three bodies of different heights; the label after the accordion sums the three allocated
  heights and moves while a band animates.
- **E. Timing** — duration and easing controls retime every section above, including `Instant` (duration `0.0`,
  no repaint).
- **F. Widget transitions** — hover one of the three buttons, the interactive frame and the `MultiLabel`: the chrome
  ramps, and `Instant` in section E makes it snap.

---

## 9. A loop is not a leg

Two animations in this crate do not fit `animate`'s "ease a value toward a target" shape, and the difference
decides where their state lives.

A **leg** has a target: the hop a `Badge` plays when its count changes runs once, from `0` to `1`, then it is
settled and asks for no repaint. Its trigger is an enum — the badge's text changing (`OnChange`), the host
raising it by name (`Manual`, via `Badge::bounce_now`), or a period (`Loop`) — and the state that remembers
"this badge was showing 2, now it shows 3" lives in `ctx` temp data keyed on the badge's id, exactly like
`AnimState`. First sight never hops: a badge that appears already showing `3` must be still.

A **loop** has no target: the badge's pulse grows a copy of its circle from `1.0` to `1.9 x` and fades it, once
per period. It keeps no state at all — the phase is `input.time / period`, so the same frame paints the same
picture, and a restart of the host's clock does not restart the animation. A loop is never settled, so it
requests a repaint for every visible frame it paints; that is the one place in this crate where an
unconditional `request_repaint` is correct.

| | the hop | the pulse |
|---|---|---|
| progress | one eased leg | `input.time / period`, wrapped |
| state | `ctx` temp data, keyed on the badge id | none |
| repaint | while `q < 1`, and always for `Loop` | every visible frame |
| measured ratios | `amplitude = 0.5 x d` | `scale = 1.9`, `alpha = 0.50`, `period 1.5 s` |
