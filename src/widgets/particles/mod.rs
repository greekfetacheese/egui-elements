//! Particle effects for egui.
//!
//! Currently one effect: [`Snow`], ported from
//! [`egui-snow`](https://github.com/LaVashikk/egui-snow) (MIT, Copyright (c) 2025
//! LaVashikk, v0.36.0). The port keeps the upstream algorithm, defaults and
//! behavior; randomness uses an internal PRNG instead of `rand`, so the crate
//! gains no dependency.
//!
//! The effect keeps its per-frame state in egui's temporary data storage and
//! paints on an egui layer, so it never disturbs the host layout.
//! [`SnowSettings`] collects the public knobs and draws a themed settings window
//! through [`SnowSettings::show_window`]; [`Snow::from_settings`] turns the
//! knobs into a configured effect.
//!
//! ```no_run
//! use egui::Ui;
//! use egui_elements::widgets::particles::{Snow, SnowSettings};
//!
//! fn snow(ui: &mut Ui, settings: &mut SnowSettings) {
//!    settings.show_window(ui.ctx());
//!    if settings.active {
//!       Snow::from_settings("snow", settings).show(ui);
//!    }
//! }
//! ```

mod rng;
mod settings;
mod snow;

pub use settings::SnowSettings;
pub use snow::{Snow, SnowLayer};
