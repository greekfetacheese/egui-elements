
# Particles

Weather and atmosphere effects. An effect is **not** a widget: it keeps its state in `Context` memory
under one `id_source`, paints onto a layer, and takes no space in the layout — so you mount it once and
the rest of the UI lays out as if it were not there.

Each effect has a settings struct with a drop-in settings window. Confine an effect to a panel by feeding
the settings window's rect into the effect's `area`, which is how a host keeps snow inside a hero section
instead of over the whole window.

## Snow

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
controls (no window chrome) inside a `Panel`, `Window`, or `ScrollArea` — wrap it in
an `egui::ScrollArea` when that container can be shorter than the controls.

```rust
let mut settings = SnowSettings::default();

// egui 0.37 panels are shown into a `Ui`, not into the context.
egui::Panel::right("snow").show(ui, |ui| {
    settings.body_ui(ui);
});
```
## Light

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
## Lightning

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
