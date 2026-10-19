//! Particle effects for egui.
//!
//! Three effects: [`Snow`], ported from
//! [`egui-snow`](https://github.com/LaVashikk/egui-snow) (MIT, Copyright (c) 2025
//! LaVashikk, v0.36.0); [`Light`], soft radial glow motes and rising embers; and
//! [`Lightning`], a jagged bolt with a full-area flash. All keep their per-frame
//! state in egui's temporary data storage and paint on an egui layer, so they
//! never disturb the host layout. Randomness uses an internal PRNG instead of
//! `rand`, so the crate gains no dependency.
//!
//! Each effect ships a settings struct ([`SnowSettings`], [`LightSettings`],
//! [`LightningSettings`]) that collects the public knobs and draws a themed
//! settings window through `show_window`, or just the controls through `body_ui`
//! for a host-provided container; `from_settings` turns the knobs into a
//! configured effect.
//!
//! ```no_run
//! use egui::Ui;
//! use egui_elements::widgets::particles::{
//!    Light, LightSettings, Lightning, LightningSettings, Snow, SnowSettings,
//! };
//!
//! fn particles(
//!    ui: &mut Ui,
//!    snow: &mut SnowSettings,
//!    light: &mut LightSettings,
//!    lightning: &mut LightningSettings,
//! ) {
//!    snow.show_window(ui.ctx());
//!    light.show_window(ui.ctx());
//!    lightning.show_window(ui.ctx());
//!    if snow.active {
//!       Snow::from_settings("snow", snow).show(ui);
//!    }
//!    if light.active {
//!       Light::from_settings("light", light).show(ui);
//!    }
//!    if lightning.active {
//!       Lightning::from_settings(LightningSettings::STRIKE_ID, lightning).show(ui);
//!    }
//! }
//! ```

mod light;
mod lightning;
pub(crate) mod shared;
mod snow;

pub use light::{Light, LightMotion, LightSettings};
pub use lightning::{Lightning, LightningSettings};
pub use shared::ParticleLayer;
pub use snow::{Snow, SnowSettings};
