//! The loader family: the widgets that say "something is happening" without
//! telling you how much is left.
//!
//! Three members, and they are different mechanisms rather than three skins of one:
//!
//! - [`Spinner`] — an orbiting arc on a ring, which reads as progress that never
//!   ends. Same geometry as [`egui::Spinner`], with a glow, a comet tail and a track.
//! - [`Ripple`] — a 3 x 3 grid of coloured cells lit in a wave, ported from
//!   [alexruix / fluffy-starfish-52](https://uiverse.io/alexruix/fluffy-starfish-52) (MIT); it
//!   reads as blocks arriving and carries its own hue ramp.
//! - [`Orb`] — a glowing sphere whose lit rim orbits it while a label pulses letter by
//!   letter, ported from [dexter-st / bright-lizard-8](https://uiverse.io/dexter-st/bright-lizard-8)
//!   (MIT); it is the only member that carries text of its own.
//!
//! All three report [`Role::ProgressIndicator`](egui::Role::ProgressIndicator), all paint from an
//! installed [`Theme`] unless you override the colour, and all expose `paint_at(&Ui, Rect)` so a
//! host can draw one inside a widget it already owns.
//!
//! [`Theme`]: crate::theme::Theme

mod mesh;
mod orb;
mod ripple;
mod spinner;

pub use orb::Orb;
pub use ripple::Ripple;
pub use spinner::{Spinner, SpinnerShadow};
