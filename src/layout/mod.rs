//! A CSS-flexbox subset for egui: rows and columns of known-size items.
//!
//! The vocabulary is deliberately CSS: [`Axis`] is `flex-direction` (`row` /
//! `column`), `gap` is the space between items, [`Justify`] is
//! `justify-content`, [`CrossAlign`] is `align-items` / `align-self`, and
//! [`ItemSize::Grow`] is `flex: <n>` with a basis of zero. Anyone who knows
//! flexbox can map that knowledge onto this module without learning a new
//! layout language.
//!
//! # Contract
//!
//! A container measures its items first, then places them into exact cell
//! rects. Measuring is the job of the `Flex` container (which talks to `Ui`);
//! [`solver::resolve`] is the pure placement math. It takes natural sizes in
//! and returns one [`Rect`](egui::Rect) per item relative to the container's
//! top-left, plus the container's total [`Vec2`](egui::Vec2). It holds no `Ui`
//! state, allocates no widgets, and can be unit-tested with plain numbers.
//!
//! # Cross-axis sizing is content-driven
//!
//! The container's cross extent is the tallest (row) or widest (column) item,
//! never the parent's available height. That is deliberate, and it is the bug
//! this module exists to fix: seeding a row's height from egui's guess makes a
//! tall button overflow the row or leaves a short row floating in empty space.
//!
//! # Out of scope
//!
//! `flex-wrap`, `order`, content-basis grow (`flex-grow` with `basis: auto`),
//! percentages, and baseline alignment. [`ItemSize::Grow`] is `flex-basis: 0`,
//! i.e. CSS `flex: <n>`; there is no `flex-grow` on top of a natural size.

pub mod solver;

pub use solver::{Axis, CrossAlign, ItemSize, ItemSpec, Justify, LayoutOut, resolve};
