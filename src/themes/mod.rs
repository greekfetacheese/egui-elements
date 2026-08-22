//! Built-in palettes.
//!
//! Each module exposes the same surface: [`theme()`](tokyo_night::theme)
//! plus frames, widget visuals, and [`style()`](tokyo_night::style). Prefer
//! [`Theme::new`](crate::theme::Theme::new) / [`ThemeKind`](crate::theme::ThemeKind)
//! over calling these constructors directly.
//!
//! Slot roles (`bg` vs `widget_bg`, `hover` vs `highlight`, …) are defined
//! in the crate `docs/theme.md`. Hex values may change; jobs must not.

pub mod mclaren_650gts_gt3;
pub mod reverie;
pub mod shade_sanctuary;
pub mod tokyo_night;
pub mod tokyo_night_light;
pub mod wasp;
pub mod wasp_light;
