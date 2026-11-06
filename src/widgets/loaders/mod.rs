//! The loader family: the widgets that say "something is happening" without
//! telling you how much is left.
//!
//! Two members, and they are different mechanisms rather than two skins of one:
//!
//! - [`Spinner`] — an orbiting arc on a ring, which reads as progress that never
//!   ends. Same geometry as [`egui::Spinner`], with a glow, a comet tail and a track.
//! - [`Ripple`] — a 3 x 3 grid of coloured cells lit in a wave, ported from
//!   [alexruix / fluffy-starfish-52](https://uiverse.io/alexruix/fluffy-starfish-52) (MIT); it
//!   reads as blocks arriving and carries its own hue ramp.
//!
//! Both report [`Role::ProgressIndicator`](egui::Role::ProgressIndicator), both paint from an installed [`Theme`] unless you
//! override the colour, and both expose `paint_at(&Ui, Rect)` so a host can draw one inside a
//! widget it already owns.
//!
//! [`Theme`]: crate::theme::Theme

mod mesh;
mod ripple;
mod spinner;

pub use ripple::Ripple;
pub use spinner::{Spinner, SpinnerShadow};
