//! Snow particle effect, ported from [`egui-snow`](https://github.com/LaVashikk/egui-snow)
//! (v0.36.0, MIT License, Copyright (c) 2025 LaVashikk). The port keeps the upstream
//! algorithm, defaults and behavior; randomness uses an internal PRNG instead of `rand`.

mod effect;
mod settings;

pub use effect::Snow;
pub use settings::SnowSettings;
