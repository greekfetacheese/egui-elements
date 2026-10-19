//! Internals shared by the particle effects: RNG, layer resolution and settings chrome.

mod layer;
mod rng;
mod window;

pub use layer::ParticleLayer;
pub(super) use layer::resolve_target;
pub(super) use rng::ParticleRng;
pub(crate) use window::{begin_body, settings_window};

/// Orders two range endpoints, swapping them when the caller inverted them.
pub(super) fn ordered(a: f32, b: f32) -> (f32, f32) {
   if a <= b { (a, b) } else { (b, a) }
}
