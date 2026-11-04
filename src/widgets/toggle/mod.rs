//! The toggle family: the shared plate, its glyphs, and the widgets built on them.
//!
//! A plate is one outlined shape with a glyph inside it, a label beside it and one clickable row.
//! Everything that is not the glyph lives here: the row allocation, the interaction weights, the press
//! scale and the lift. Checkbox pairs a rounded box with a tick, Radio pairs a circle with a dot, and
//! Switch draws the same plate as a pill whose knob slides from end to end, so the same row, clock and
//! palette serve all three and only the outline and the mark differ. The plate itself is crate-private;
//! the public knobs it exposes, [`ToggleVisuals`] and [`ToggleShadow`], are
//! re-exported at the crate root, and [`Checkbox`] keeps its historic `CheckboxVisuals`/`CheckboxShadow`
//! aliases.

mod checkbox;
mod glyph;
mod plate;
mod radio;
mod switch;

pub use checkbox::{Checkbox, CheckboxShadow, CheckboxVisuals};
pub(crate) use plate::{DEFAULT_SIZE, PlateGlyph, PlateShape, Row, TRACK_RATIO};
pub use radio::Radio;
pub use switch::Switch;

use egui::{Color32, Shadow};

use crate::theme::ThemeColors;

/// Shadow alpha floor while unchecked; `1.0` at rest is the configured strength.
const SHADOW_T_FLOOR: f32 = 0.30;

/// The lift under a plate, in ratios of the plate so it scales with the toggle's size.
///
/// [`ToggleShadow::derived`] is the default (the checked fill at 42 %, under a 0.34·size blur, offset
/// 0.055·size down); [`ToggleShadow::new`] keeps that geometry in a colour of your own;
/// [`ToggleShadow::NONE`] turns the lift off.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ToggleShadow {
   /// Shadow colour. `None` = the checked fill, so overriding the plate colour repaints the lift with it.
   pub color: Option<Color32>,
   /// Vertical offset, as a fraction of the plate size.
   pub offset_y: f32,
   /// Blur reach, as a fraction of the plate size.
   pub blur: f32,
   /// Spread, as a fraction of the plate size.
   pub spread: f32,
   /// Peak alpha, `0.0..=1.0`.
   pub strength: f32,
}

impl ToggleShadow {
   /// No lift. The plate still feathers its own edges.
   pub const NONE: Self = Self {
      color: None,
      offset_y: 0.0,
      blur: 0.0,
      spread: 0.0,
      strength: 0.0,
   };

   /// The default lift: the checked fill at 42 %, offset 0.055·size, blur 0.34·size.
   pub fn derived() -> Self {
      Self {
         color: None,
         offset_y: 0.055,
         blur: 0.34,
         spread: 0.0,
         strength: 0.42,
      }
   }

   /// The derived lift in `color`.
   pub fn new(color: Color32) -> Self {
      Self {
         color: Some(color),
         ..Self::derived()
      }
   }

   /// Resolve the ratios against `size`. `t` (the toggle progress) lifts the alpha from
   /// `0.30 × strength` while unchecked to `strength` when checked, so the shadow blooms with the fill.
   pub fn to_shadow(self, size: f32, checked_fill: Color32, t: f32) -> Shadow {
      if self.strength <= 0.0 {
         return Shadow::NONE;
      }
      let alpha = self.strength * (SHADOW_T_FLOOR + (1.0 - SHADOW_T_FLOOR) * t);
      let color = self.color.unwrap_or(checked_fill).gamma_multiply(alpha);
      Shadow {
         offset: [0, (self.offset_y * size).round() as i8],
         blur: (self.blur * size).round().clamp(0.0, 255.0) as u8,
         spread: (self.spread * size).round().clamp(0.0, 255.0) as u8,
         color,
      }
   }
}

impl Default for ToggleShadow {
   fn default() -> Self {
      Self::NONE
   }
}

/// Per-widget colours for a plate toggle.
///
/// [`Default`] is inert (transparent, no border, no shadow). The widget paints with
/// [`ToggleVisuals::from_theme`] unless you pass your own set — start there to override one slot.
///
/// ```no_run
/// # use egui::{__run_test_ui, Color32};
/// # use egui_elements::theme::{Theme, ThemeKind};
/// # use egui_elements::widgets::{Checkbox, CheckboxVisuals};
/// # let colors = Theme::new(ThemeKind::TokyoNight).colors;
/// # __run_test_ui(|ui| {
/// # let mut on = false;
/// let visuals = CheckboxVisuals {
///    bg_checked: Color32::WHITE,
///    ..CheckboxVisuals::from_theme(colors)
/// };
/// ui.add(Checkbox::new(&mut on, "Lit").visuals(visuals));
/// # });
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ToggleVisuals {
   /// Idle (unchecked) fill. Default `TRANSPARENT`, so the plate picks up its surface.
   pub bg: Color32,
   /// Fill while hovered.
   pub bg_hover: Color32,
   /// Fill when checked.
   pub bg_checked: Color32,
   /// Idle border.
   pub border: Color32,
   /// Border while hovered.
   pub border_hover: Color32,
   /// Border when checked.
   pub border_checked: Color32,
   /// Glyph colour: the tick's line, or the radio's dot.
   pub mark: Color32,
   /// The lift under the plate.
   pub shadow: ToggleShadow,
}

impl ToggleVisuals {
   /// The defaults a plate toggle derives from an installed palette.
   ///
   /// `bg` is transparent so the plate sits on any surface; the checked fill is the accent and the mark is
   /// [`ThemeColors::ink_on`]`(accent)` — the crate's rule for text on a chromatic fill.
   pub fn from_theme(colors: ThemeColors) -> Self {
      Self {
         bg: Color32::TRANSPARENT,
         bg_hover: colors.hover,
         bg_checked: colors.accent,
         border: colors.border,
         border_hover: colors.border.lerp_to_gamma(colors.accent, 0.5),
         border_checked: colors.accent,
         mark: colors.ink_on(colors.accent),
         shadow: ToggleShadow::derived(),
      }
   }
}
