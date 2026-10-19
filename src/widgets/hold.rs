//! Hold-to-confirm settings: the completion burst and the public knobs for a
//! [`Button::hold`](crate::widgets::Button::hold) action.

use egui::{Color32, Context, Slider, Ui};

use crate::widgets::button::{HOLD_GLOW_END, HOLD_GLOW_START};
use crate::widgets::particles::LightningSettings;
use crate::widgets::particles::shared::{begin_body, settings_window};

/// Softness that reproduces the shared glow fan exactly.
const DEFAULT_GLOW_SOFTNESS: f32 = 1.0;

/// Completion burst drawn when a hold-to-confirm button reaches 100 %.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
///
/// With the `serde` feature enabled the whole struct is serializable, so a host
/// can persist it as part of its own settings instead of copying each knob.
/// Deserialization ignores unknown fields (serde's default) and fills missing
/// ones from [`HoldSettings::default`], so a file written before a knob existed
/// still loads and that knob takes its default — which is what lets a host
/// persist this type across crate upgrades. [`Self::open`] is serialized like
/// every other field; a host that owns the settings window sets it to `false`
/// after loading.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
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
   /// Core stroke thickness of a [`HoldBurst::Lightning`] burst, in pixels;
   /// ignored by the other kinds.
   ///
   /// Default `2.0`, the [`LightningSettings::default`] value (see
   /// `DEFAULT_THICKNESS` in `widgets/particles/lightning/effect.rs`).
   pub burst_thickness: f32,
   /// Midpoint-displacement jitter of a [`HoldBurst::Lightning`] burst, as a
   /// fraction of each segment's length; ignored by the other kinds.
   ///
   /// Default `0.18`, the [`LightningSettings::default`] value (see
   /// `DEFAULT_JITTER` in `widgets/particles/lightning/effect.rs`).
   pub burst_jitter: f32,
   /// Radius of a [`HoldBurst::Glow`] at the start of the burst, as a multiple
   /// of the button's shorter side; ignored by the other kinds.
   ///
   /// Default `1.0` (`HOLD_GLOW_START`).
   pub burst_glow_from: f32,
   /// Radius of a [`HoldBurst::Glow`] at the end of the burst, as a multiple of
   /// the button's shorter side; ignored by the other kinds.
   ///
   /// At paint time this is clamped to at least [`Self::burst_glow_from`], so a
   /// pair that would shrink (or a negative one) paints a constant radius
   /// instead of a shrinking band.
   ///
   /// Default `2.2` (`HOLD_GLOW_END`).
   pub burst_glow_to: f32,
   /// Falloff softness of a [`HoldBurst::Glow`]; ignored by the other kinds.
   ///
   /// The shared fan keeps its hot-core ring at `0.15` of the radius and its
   /// inner ring at `0.45`, carrying `0.55` and `0.18` of the colour. This knob
   /// maps each interior ring fraction `r` to `r.powf(1.0 / softness)`; the rim
   /// and every alpha stay put. Values above `1.0` push the two interior rings
   /// outward, spreading the colour over a longer run to the transparent rim (a
   /// gentler falloff); values below `1.0` pull them in for a tighter, harder
   /// core. Exactly `1.0` reproduces the shared profile vertex for vertex.
   ///
   /// Default `1.0`.
   pub burst_glow_softness: f32,
}

impl Default for HoldSettings {
   fn default() -> Self {
      // The burst's strike knobs default to the effect's own, so a hold burst
      // paints exactly like the same `Lightning` built without those calls.
      let lightning = LightningSettings::default();
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
         burst_thickness: lightning.thickness,
         burst_jitter: lightning.jitter,
         burst_glow_from: HOLD_GLOW_START,
         burst_glow_to: HOLD_GLOW_END,
         burst_glow_softness: DEFAULT_GLOW_SOFTNESS,
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

   /// The [`Self::burst_glow_from`]..[`Self::burst_glow_to`] radius band, with
   /// both ends non-negative and `to` never below `from`.
   pub(crate) fn glow_radius_range(&self) -> (f32, f32) {
      let from = self.burst_glow_from.max(0.0);
      (from, self.burst_glow_to.max(from))
   }

   /// Whether a completed hold should start a burst at all.
   ///
   /// [`HoldBurst::Off`], a zero-length burst and zero intensity all paint
   /// nothing, so the widget must not keep the frame loop awake for them.
   pub(crate) fn burst_visible(&self) -> bool {
      self.burst != HoldBurst::Off && self.burst_secs() > 0.0 && self.burst_intensity > 0.0
   }

   /// Themed floating settings window; returns the window's outer `Response` when open.
   ///
   /// Window chrome plus a vertical scroll area around [`Self::body_ui`], so a
   /// viewport shorter than the controls scrolls instead of clipping them.
   pub fn show_window(&mut self, ctx: &Context) -> Option<egui::Response> {
      // Local copy — `settings_window` borrows `open` while the body borrows `self`.
      let mut open = self.open;
      let resp = settings_window(ctx, "Hold Settings", &mut open, |ui| {
         self.body_ui(ui)
      });
      self.open = open;
      resp
   }

   /// Draw the settings controls into `ui` — no window chrome.
   ///
   /// Render them in your own container (`egui::Window`, `SidePanel`, `ScrollArea`, …)
   /// instead of using [`Self::show_window`]. If that container can be shorter than the
   /// controls, wrap it in an `egui::ScrollArea`. This method never touches
   /// [`Self::open`].
   pub fn body_ui(&mut self, ui: &mut Ui) {
      let theme = begin_body(ui);

      ui.heading("Hold");
      ui.horizontal(|ui| {
         ui.label("Duration (s):");
         ui.add(Slider::new(&mut self.seconds, 0.2..=5.0));
      });

      ui.separator();
      ui.heading("Fill");
      ui.horizontal(|ui| {
         let mut theme_accent = self.fill_from.is_none();
         if ui.checkbox(&mut theme_accent, "Theme accent").changed() {
            self.fill_from = if theme_accent {
               None
            } else {
               Some(theme.colors.accent)
            };
         }
         if let Some(color) = self.fill_from.as_mut() {
            ui.color_edit_button_srgba(color);
         }
      });
      ui.horizontal(|ui| {
         let mut shifts = self.fill_to.is_some();
         if ui.checkbox(&mut shifts, "Shift to a second colour").changed() {
            self.fill_to = if shifts {
               Some(self.fill_from.unwrap_or(theme.colors.accent))
            } else {
               None
            };
         }
         if let Some(color) = self.fill_to.as_mut() {
            ui.color_edit_button_srgba(color);
         }
      });

      ui.separator();
      ui.heading("Burst");
      ui.horizontal(|ui| {
         ui.label("Kind:");
         for kind in [HoldBurst::Off, HoldBurst::Glow, HoldBurst::Lightning] {
            ui.selectable_value(&mut self.burst, kind, kind.to_str());
         }
      });
      if self.burst != HoldBurst::Off {
         ui.horizontal(|ui| {
            let mut same_as_fill = self.burst_color.is_none();
            if ui.checkbox(&mut same_as_fill, "Same as the fill").changed() {
               self.burst_color = if same_as_fill {
                  None
               } else {
                  Some(self.fill_from.unwrap_or(theme.colors.accent))
               };
            }
            if let Some(color) = self.burst_color.as_mut() {
               ui.color_edit_button_srgba(color);
            }
         });
         ui.horizontal(|ui| {
            ui.label("Intensity:");
            ui.add(Slider::new(&mut self.burst_intensity, 0.0..=1.0));
         });
         ui.horizontal(|ui| {
            ui.label("Duration (ms):");
            ui.add(Slider::new(&mut self.burst_ms, 80.0..=2000.0));
         });
         if self.burst == HoldBurst::Glow {
            ui.horizontal(|ui| {
               ui.label("Size from:");
               ui.add(Slider::new(&mut self.burst_glow_from, 0.2..=4.0))
                  .on_hover_text("Start radius as a multiple of the button's shorter side");
            });
            ui.horizontal(|ui| {
               ui.label("Size to:");
               ui.add(Slider::new(&mut self.burst_glow_to, 0.2..=6.0))
                  .on_hover_text("End radius as a multiple of the button's shorter side");
            });
            ui.horizontal(|ui| {
               ui.label("Softness:");
               ui.add(Slider::new(
                  &mut self.burst_glow_softness,
                  0.2..=3.0,
               ))
               .on_hover_text("1.0 reproduces the shared glow profile exactly");
            });
         }
         if self.burst == HoldBurst::Lightning {
            ui.horizontal(|ui| {
               ui.label("Bolts:");
               ui.add(Slider::new(&mut self.burst_bolts, 1..=5));
            });
            ui.horizontal(|ui| {
               ui.label("Branch depth:");
               ui.add(Slider::new(&mut self.burst_branch_depth, 0..=3));
            });
            ui.horizontal(|ui| {
               ui.label("Thickness (px):");
               ui.add(Slider::new(&mut self.burst_thickness, 0.5..=12.0));
            });
            ui.horizontal(|ui| {
               ui.label("Jitter:");
               ui.add(Slider::new(&mut self.burst_jitter, 0.0..=0.6))
                  .on_hover_text("Midpoint displacement as a fraction of segment length");
            });
         }
      }

      ui.add_space(theme.spacing.sm);
      if ui.button("Reset").clicked() {
         self.reset();
      }
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
      let lightning = LightningSettings::default();
      assert_eq!(
         settings.burst_thickness, lightning.thickness,
         "the burst's thickness default must be the effect's own"
      );
      assert_eq!(
         settings.burst_jitter, lightning.jitter,
         "the burst's jitter default must be the effect's own"
      );
      assert_eq!(settings.burst_glow_from, 1.0);
      assert_eq!(settings.burst_glow_to, 2.2);
      assert_eq!(settings.burst_glow_softness, 1.0);
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
         burst_thickness: 7.0,
         burst_jitter: 0.5,
         burst_glow_from: 0.4,
         burst_glow_to: 3.0,
         burst_glow_softness: 2.0,
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
   fn the_glow_band_never_shrinks() {
      let inverted = HoldSettings {
         burst_glow_from: 2.5,
         burst_glow_to: 0.8,
         ..Default::default()
      };
      assert_eq!(
         inverted.glow_radius_range(),
         (2.5, 2.5),
         "an end radius below the start must clamp up to it"
      );

      let negative = HoldSettings {
         burst_glow_from: -1.0,
         burst_glow_to: -3.0,
         ..Default::default()
      };
      assert_eq!(
         negative.glow_radius_range(),
         (0.0, 0.0),
         "a negative band must clamp to zero"
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
   fn body_ui_renders_without_a_window() {
      let mut settings = HoldSettings::default();
      assert!(!settings.open);
      egui::__run_test_ui(|ui| {
         settings.body_ui(ui);
      });
      assert!(
         !settings.open,
         "body_ui must not touch the window flag"
      );
   }

   #[test]
   fn show_window_returns_response_only_when_open() {
      egui::__run_test_ctx(|ctx| {
         let mut closed = HoldSettings::default();
         assert!(closed.show_window(ctx).is_none());
      });

      egui::__run_test_ctx(|ctx| {
         let mut open = HoldSettings {
            open: true,
            ..Default::default()
         };
         assert!(open.show_window(ctx).is_some());
      });
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

   #[cfg(feature = "serde")]
   #[test]
   fn hold_settings_round_trip() -> Result<(), serde_json::Error> {
      let settings = HoldSettings {
         open: true,
         seconds: 2.5,
         fill_from: Some(Color32::from_rgb(10, 20, 30)),
         fill_to: Some(Color32::from_rgb(40, 50, 60)),
         burst: HoldBurst::Lightning,
         burst_color: Some(Color32::from_rgb(70, 80, 90)),
         burst_intensity: 0.35,
         burst_ms: 750.0,
         burst_bolts: 3,
         burst_branch_depth: 2,
         burst_thickness: 5.5,
         burst_jitter: 0.42,
         burst_glow_from: 0.6,
         burst_glow_to: 3.4,
         burst_glow_softness: 2.25,
      };

      let json = serde_json::to_string(&settings)?;
      let restored: HoldSettings = serde_json::from_str(&json)?;

      assert_eq!(restored, settings);
      Ok(())
   }

   #[cfg(feature = "serde")]
   #[test]
   fn missing_fields_take_their_defaults() -> Result<(), serde_json::Error> {
      let restored: HoldSettings = serde_json::from_str(r#"{"seconds": 0.6}"#)?;

      assert_eq!(
         restored,
         HoldSettings {
            seconds: 0.6,
            ..HoldSettings::default()
         }
      );
      Ok(())
   }

   #[cfg(feature = "serde")]
   #[test]
   fn unknown_fields_are_ignored() -> Result<(), serde_json::Error> {
      let restored: HoldSettings =
         serde_json::from_str(r#"{"seconds": 0.6, "some_knob_from_the_future": 3}"#)?;

      assert_eq!(
         restored,
         HoldSettings {
            seconds: 0.6,
            ..HoldSettings::default()
         }
      );
      Ok(())
   }

   #[cfg(feature = "serde")]
   #[test]
   fn every_burst_kind_round_trips() -> Result<(), serde_json::Error> {
      for burst in [HoldBurst::Off, HoldBurst::Glow, HoldBurst::Lightning] {
         let settings = HoldSettings {
            burst,
            ..HoldSettings::default()
         };

         let json = serde_json::to_string(&settings)?;
         let restored: HoldSettings = serde_json::from_str(&json)?;

         assert_eq!(
            restored, settings,
            "burst kind {burst:?} must survive a round trip"
         );
      }
      Ok(())
   }
}
