//! Particle effects for egui.
//!
//! Two effects: [`Snow`], ported from
//! [`egui-snow`](https://github.com/LaVashikk/egui-snow) (MIT, Copyright (c) 2025
//! LaVashikk, v0.36.0), and [`Light`], soft radial glow motes and rising embers.
//! Both keep their per-frame state in egui's temporary data storage and paint on
//! an egui layer, so they never disturb the host layout. Randomness uses an
//! internal PRNG instead of `rand`, so the crate gains no dependency.
//!
//! Each effect ships a settings struct ([`SnowSettings`], [`LightSettings`]) that
//! collects the public knobs and draws a themed settings window through
//! `show_window`, or just the controls through `body_ui` for a host-provided
//! container; `from_settings` turns the knobs into a configured effect.
//!
//! ```no_run
//! use egui::Ui;
//! use egui_elements::widgets::particles::{Light, LightSettings, Snow, SnowSettings};
//!
//! fn particles(ui: &mut Ui, snow: &mut SnowSettings, light: &mut LightSettings) {
//!    snow.show_window(ui.ctx());
//!    light.show_window(ui.ctx());
//!    if snow.active {
//!       Snow::from_settings("snow", snow).show(ui);
//!    }
//!    if light.active {
//!       Light::from_settings("light", light).show(ui);
//!    }
//! }
//! ```

mod light;
mod shared;
mod snow;

pub use light::{Light, LightMotion, LightSettings};
pub use shared::ParticleLayer;
pub use snow::{Snow, SnowSettings};
