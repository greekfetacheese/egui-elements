//! Stacked avatars that fan out when the pointer is over them.
//!
//! An [`AvatarGroup`] draws its avatars as a ringed stack — each one overlapping the previous by
//! [`AvatarGroup::overlap`] of its diameter — and spreads them apart on hover so every picture is visible.
//! Only the spacing animates: the diameter, the ring and the fallback plate stay put.
//!
//! The geometry is measured from the reference this was ported from (a React/Tailwind avatar stack,
//! captured at 40 px per avatar, in both states):
//!
//! | measurement | collapsed | expanded |
//! |-------------|-----------|----------|
//! | centre-to-centre step | `0.69 × d` | `1.11 × d` |
//! | neighbour distance | `-0.31 × d` (overlap) | `+0.11 × d` (gap) |
//! | ring, inside the edge | `0.08 × d` | same |
//! | group centre | fixed | fixed |
//!
//! Everything is a ratio of [`AvatarGroup::size`], so one `.size()` scales the stack. The ring is what makes
//! an overlap readable — it separates a front avatar from the one behind it — and it defaults to
//! [`ThemeColors::bg`] (the page), so a stack dropped on `bg` reads like the reference. A group sitting on
//! `frame1` should pass `.visuals(AvatarVisuals { ring: colors.widget_bg, ..AvatarVisuals::from_theme(colors) })`.
//!
//! The widget always reserves the **expanded** width and spreads the stack around a fixed centre, so a
//! neighbour never reflows and nothing paints outside the allocated rect. Use
//! [`AvatarGroup::align`] to anchor the collapsed stack instead of centring it.
//!
//! An avatar is either a picture (clipped to a circle) or a fallback plate with initials:
//!
//! ```
//! # use egui::__run_test_ui;
//! # use egui_elements::widgets::{Avatar, AvatarGroup};
//! # __run_test_ui(|ui| {
//! let accounts = [Avatar::initials("A"), Avatar::initials("S"), Avatar::initials("M")];
//! ui.add(AvatarGroup::new(&accounts));
//! # });
//! ```
//!
//! The fallback plate follows the theme rule for chromatic fills — `accent` with `bg` text — while the
//! reference paints a 35 % tint of the accent with the accent as ink; pass [`Avatar::plate`] and
//! [`Avatar::ink`] (or a whole [`AvatarVisuals`]) for the pastel. Initials shrink to fit when the text is
//! wider than the plate, so two letters never spill out of the circle.
//!
//! Run the proof with: `cargo run --example avatar`
//!
//! [`ThemeColors::bg`]: crate::theme::ThemeColors::bg
//! [`ThemeColors::accent`]: crate::theme::ThemeColors::accent

mod group;

pub use group::AvatarGroup;

use crate::theme::ThemeColors;
use egui::{Color32, Image};

/// Default diameter of one avatar, in points.
pub const DEFAULT_SIZE: f32 = 32.0;

/// Default share of the diameter hidden behind the next avatar while collapsed (`0.31`, the reference's
/// `1 - 0.69`).
pub const DEFAULT_OVERLAP: f32 = 0.31;

/// Default space between two neighbours while expanded, as a share of the diameter (`0.11`).
pub const DEFAULT_GAP: f32 = 0.11;

/// Ring width as a share of the avatar's diameter (`0.08`, measured at roughly 3 px on 38 px avatars).
pub const RING_RATIO: f32 = 0.08;

/// Initials font size as a share of the diameter; the measured cap height is `0.34 × d`.
const INITIALS_RATIO: f32 = 0.48;

/// Widest an initials galley may get, as a share of the diameter, before it is shrunk to fit.
const INITIALS_MAX_WIDTH: f32 = 0.68;

/// Alpha multiplier applied to a disabled group.
const DISABLED_ALPHA: f32 = 0.5;

/// One picture in an [`AvatarGroup`].
///
/// Build it with [`Avatar::image`] for a picture or [`Avatar::initials`] for the fallback plate, then
/// [`Avatar::plate`] / [`Avatar::ink`] to override the fallback's colours for one entry.
#[derive(Clone, Debug)]
pub struct Avatar {
   pub(crate) content: AvatarContent,
   pub(crate) plate: Option<Color32>,
   pub(crate) ink: Option<Color32>,
}

/// What an [`Avatar`] shows.
#[derive(Clone, Debug)]
pub(crate) enum AvatarContent {
   /// A picture, clipped to the avatar's circle.
   Image(Image<'static>),
   /// Text on the fallback plate.
   Initials(String),
}

impl Avatar {
   /// An avatar painted from `image` and clipped to a circle.
   ///
   /// Any [`Image`] works — a texture, a `bytes://`/file URI, or an SVG registered with
   /// `egui_extras::install_image_loaders`. The widget paints it at the avatar's size, so it stays crisp and
   /// keeps its aspect ratio behaviour from the image itself.
   pub fn image(image: Image<'static>) -> Self {
      Self {
         content: AvatarContent::Image(image),
         plate: None,
         ink: None,
      }
   }

   /// An avatar that shows `text` — one or two characters, e.g. the initial of a chain without an icon.
   ///
   /// Longer text is shrunk to fit the plate; at small sizes it still disappears under the next
   /// avatar's ring, so keep it short.
   pub fn initials(text: impl Into<String>) -> Self {
      Self {
         content: AvatarContent::Initials(text.into()),
         plate: None,
         ink: None,
      }
   }

   /// Fill of this avatar's fallback plate. Ignored by [`Avatar::image`].
   ///
   /// Defaults to [`AvatarVisuals::fallback_fill`].
   pub fn plate(mut self, color: Color32) -> Self {
      self.plate = Some(color);
      self
   }

   /// Colour of this avatar's initials. Ignored by [`Avatar::image`].
   ///
   /// Defaults to [`AvatarVisuals::fallback_text`].
   pub fn ink(mut self, color: Color32) -> Self {
      self.ink = Some(color);
      self
   }
}

/// The three colours an [`AvatarGroup`] paints with.
///
/// Resolve the default from the installed theme with [`AvatarVisuals::from_theme`], then override the one
/// field you need — typically `ring` on a surface that is not `bg`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AvatarVisuals {
   /// Ring drawn inside each avatar's edge. This is the separator that makes a stack readable.
   pub ring: Color32,
   /// Fill of the fallback plate.
   pub fallback_fill: Color32,
   /// Colour of the initials on the fallback plate.
   pub fallback_text: Color32,
}

impl AvatarVisuals {
   /// All three colours, explicitly.
   pub const fn new(ring: Color32, fallback_fill: Color32, fallback_text: Color32) -> Self {
      Self {
         ring,
         fallback_fill,
         fallback_text,
      }
   }

   /// The palette-derived default: the ring is the page (`colors.bg`), and the plate is `colors.accent`
   /// carrying `colors.bg` text, like every other chromatic fill in this crate.
   pub fn from_theme(colors: ThemeColors) -> Self {
      Self {
         ring: colors.bg,
         fallback_fill: colors.accent,
         fallback_text: colors.bg,
      }
   }
}
