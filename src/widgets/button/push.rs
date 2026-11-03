//! [`PushStyle`] — the raised base under a [`Button`](crate::widgets::Button)'s
//! face, and the travel that collapses it while the button is pressed.

use egui::Color32;

/// Knobs for a raised button: how high the face sits above its base, and what
/// the base is painted with.
///
/// The base is a copy of the face's rounded rectangle, shifted down by
/// [`Self::depth`] and painted under it, so the exposed band reads as a solid
/// extruded edge rather than a blurred shadow. While the button is pressed the
/// face travels down by the same `depth` and lands exactly on the base, so the
/// button looks pushed flush.
///
/// [`Button::push_style`](crate::widgets::Button::push_style) replaces the whole
/// configuration; [`Button::push`](crate::widgets::Button::push) and
/// [`Button::push_color`](crate::widgets::Button::push_color) set one field each
/// and compose in call order. A depth of `0.0` or less leaves the button flat,
/// which is also why `push_color` alone does not arm the effect.
///
/// With the `serde` feature the whole struct is serializable, like
/// [`HoldSettings`](crate::widgets::HoldSettings). Deserialization ignores
/// unknown fields and fills missing ones from [`Self::default`].
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PushStyle {
   /// Height of the base below the face, in points, and the distance the face
   /// travels down while pressed.
   ///
   /// Default `4.0`.
   pub depth: f32,
   /// Base colour, or `None` for the installed theme's
   /// [`colors.accent`](crate::theme::ThemeColors::accent).
   ///
   /// Default `None`.
   pub color: Option<Color32>,
}

impl Default for PushStyle {
   fn default() -> Self {
      Self {
         depth: 4.0,
         color: None,
      }
   }
}
