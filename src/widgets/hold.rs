//! Hold-to-confirm settings: the completion burst and the public knobs for a
//! [`Button::hold`](crate::widgets::Button::hold) action.

use egui::Color32;

/// Completion burst drawn when a hold-to-confirm button reaches 100 %.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HoldBurst {
   /// No completion effect; a completed hold paints nothing extra.
   Off,
   /// An expanding radial glow at the button's centre, fading out.
   #[default]
   Glow,
   /// A lightning bolt and flash struck across the button.
   Lightning,
}

impl HoldBurst {
   /// Human-readable name, used by the settings window's kind selector.
   pub const fn to_str(self) -> &'static str {
      match self {
         Self::Off => "Off",
         Self::Glow => "Glow",
         Self::Lightning => "Lightning",
      }
   }
}

/// Knobs for a hold-to-confirm button: the hold duration, the progressive fill
/// and the completion burst.
///
/// [`Button::hold_settings`](crate::widgets::Button::hold_settings) replaces the
/// whole configuration at once; the per-field shortcuts
/// [`Button::hold`](crate::widgets::Button::hold),
/// [`Button::hold_fill`](crate::widgets::Button::hold_fill) and
/// [`Button::hold_burst`](crate::widgets::Button::hold_burst) compose field by
/// field instead.
///
/// The defaults reproduce the look of a button that configures nothing: a 1 s
/// hold, the theme's accent colour, and a 300 ms [`HoldBurst::Glow`] burst.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HoldSettings {
   /// Settings-window visibility. Default `false` (the example opens it).
   ///
   /// [`Button`](crate::widgets::Button) ignores this field; it exists so a host
   /// can keep one settings value for both the window and the buttons.
   pub open: bool,
   /// Hold duration in seconds; `0.0` or less means "not a hold button".
   ///
   /// Default `1.0`.
   pub seconds: f32,
   /// Sweep colour at the start of the hold, or `None` for the installed
   /// theme's [`colors.accent`](crate::theme::ThemeColors::accent).
   ///
   /// Default `None`.
   pub fill_from: Option<Color32>,
   /// Sweep colour at completion. When `Some`, the sweep shifts from
   /// [`Self::fill_from`] towards this colour as the hold fills; `None` keeps it
   /// solid.
   ///
   /// Default `None`.
   pub fill_to: Option<Color32>,
   /// Completion effect. Default [`HoldBurst::Glow`].
   pub burst: HoldBurst,
   /// Burst colour, or `None` for the resolved fill colour at completion.
   ///
   /// Default `None`.
   pub burst_color: Option<Color32>,
   /// Multiplier over the effect's own profile, in `0.0..=1.0`. `0.0` disables
   /// the burst entirely.
   ///
   /// Default `1.0`.
   pub burst_intensity: f32,
   /// Burst duration in milliseconds. Default `300.0`.
   pub burst_ms: f32,
   /// Number of bolts for a [`HoldBurst::Lightning`] burst; ignored by the other
   /// kinds.
   ///
   /// Default `1`.
   pub burst_bolts: usize,
   /// Branch recursion depth for a [`HoldBurst::Lightning`] burst (`0` draws the
   /// trunk only); ignored by the other kinds.
   ///
   /// Default `1`.
   pub burst_branch_depth: u32,
}

impl Default for HoldSettings {
   fn default() -> Self {
      Self {
         open: false,
         seconds: 1.0,
         fill_from: None,
         fill_to: None,
         burst: HoldBurst::Glow,
         burst_color: None,
         burst_intensity: 1.0,
         burst_ms: 300.0,
         burst_bolts: 1,
         burst_branch_depth: 1,
      }
   }
}

impl HoldSettings {
   /// Knobs back to defaults; `open` is preserved (a reset must not close the window).
   pub fn reset(&mut self) {
      let open = self.open;
      *self = Self::default();
      self.open = open;
   }

   /// The sweep colour at `progress`: [`Self::fill_from`] alone when there is no
   /// [`Self::fill_to`], otherwise a gamma-space shift towards `fill_to`.
   pub(crate) fn fill_color(&self, accent: Color32, progress: f32) -> Color32 {
      let from = self.fill_from.unwrap_or(accent);
      match self.fill_to {
         Some(to) => from.lerp_to_gamma(to, progress.clamp(0.0, 1.0)),
         None => from,
      }
   }

   /// Burst length in seconds; `0.0` for a zero-length burst.
   pub(crate) fn burst_secs(&self) -> f32 {
      (self.burst_ms / 1000.0).max(0.0)
   }

   /// Whether a completed hold should start a burst at all.
   ///
   /// [`HoldBurst::Off`], a zero-length burst and zero intensity all paint
   /// nothing, so the widget must not keep the frame loop awake for them.
   pub(crate) fn burst_visible(&self) -> bool {
      self.burst != HoldBurst::Off && self.burst_secs() > 0.0 && self.burst_intensity > 0.0
   }
}

#[cfg(test)]
mod tests {
   use super::*;

   #[test]
   fn defaults_match_the_original_look() {
      let settings = HoldSettings::default();
      assert!(!settings.open);
      assert_eq!(settings.seconds, 1.0);
      assert_eq!(settings.fill_from, None);
      assert_eq!(settings.fill_to, None);
      assert_eq!(settings.burst, HoldBurst::Glow);
      assert_eq!(settings.burst_color, None);
      assert_eq!(settings.burst_intensity, 1.0);
      assert_eq!(settings.burst_ms, 300.0);
      assert_eq!(settings.burst_bolts, 1);
      assert_eq!(settings.burst_branch_depth, 1);
   }

   #[test]
   fn reset_keeps_open() {
      let mut settings = HoldSettings {
         open: true,
         seconds: 3.0,
         fill_from: Some(Color32::from_rgb(1, 2, 3)),
         fill_to: Some(Color32::from_rgb(4, 5, 6)),
         burst: HoldBurst::Lightning,
         burst_color: Some(Color32::from_rgb(7, 8, 9)),
         burst_intensity: 0.2,
         burst_ms: 900.0,
         burst_bolts: 4,
         burst_branch_depth: 3,
      };

      settings.reset();

      assert!(settings.open, "reset must not close the window");
      assert_eq!(
         settings,
         HoldSettings {
            open: true,
            ..Default::default()
         }
      );
   }

   #[test]
   fn the_fill_is_solid_without_a_second_colour() {
      let start = Color32::from_rgb(10, 20, 30);
      let settings = HoldSettings {
         fill_from: Some(start),
         ..Default::default()
      };
      assert_eq!(
         settings.fill_color(Color32::RED, 0.2),
         start,
         "a fill without `fill_to` must not depend on progress"
      );
      assert_eq!(settings.fill_color(Color32::RED, 0.9), start);
   }

   #[test]
   fn off_and_zero_intensity_are_not_visible_bursts() {
      assert!(
         !HoldSettings {
            burst: HoldBurst::Off,
            ..Default::default()
         }
         .burst_visible()
      );
      assert!(
         !HoldSettings {
            burst_intensity: 0.0,
            ..Default::default()
         }
         .burst_visible()
      );
      assert!(HoldSettings::default().burst_visible());
   }
}
