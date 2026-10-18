//! [`LightSettings`] — public knobs for the light effect plus a themed settings window.

use egui::{Color32, Context, DragValue, Slider, Ui, Vec2};

use super::super::shared::{ParticleLayer, begin_body, settings_window};
use super::LightMotion;

/// User-facing knobs for [`Light`](super::Light).
///
/// The defaults are deliberately neutral — a warm white, low density, wide soft
/// radii, gentle flicker and multi-second lifetimes — so the effect can be
/// tuned to a particular screen instead of shipping one screen's look.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct LightSettings {
   /// Settings-window visibility. Default `false` (the example opens it).
   pub open: bool,
   /// Whether the effect is drawn at all.
   pub active: bool,
   /// Mote color.
   pub color: Color32,
   /// Number of motes.
   pub density: usize,
   /// Minimum mote radius in pixels.
   pub radius_min: f32,
   /// Maximum mote radius in pixels.
   pub radius_max: f32,
   /// Global intensity multiplier in `0.0..=1.0`.
   pub intensity: f32,
   /// Motion model.
   pub motion: LightMotion,
   /// Global wind vector (x = horizontal, y = vertical).
   pub wind: Vec2,
   /// Flicker depth in `0.0..=1.0` (`0` disables flicker).
   pub flicker: f32,
   /// Minimum mote lifetime in seconds.
   pub lifetime_min: f32,
   /// Maximum mote lifetime in seconds.
   pub lifetime_max: f32,
   /// Layer the effect is painted on.
   pub layer: ParticleLayer,
}

impl Default for LightSettings {
   fn default() -> Self {
      Self {
         open: false,
         active: true,
         color: Color32::from_rgb(255, 224, 178),
         density: 48,
         radius_min: 8.0,
         radius_max: 36.0,
         intensity: 0.7,
         motion: LightMotion::Drift,
         wind: Vec2::ZERO,
         flicker: 0.25,
         lifetime_min: 4.0,
         lifetime_max: 9.0,
         layer: ParticleLayer::Foreground,
      }
   }
}

impl LightSettings {
   /// Knobs back to defaults; `open` is preserved (a reset must not close the window).
   pub fn reset(&mut self) {
      let open = self.open;
      *self = Self::default();
      self.open = open;
   }

   /// Themed floating settings window; returns the window's outer `Response` when open.
   ///
   /// Window chrome plus a vertical scroll area around [`Self::body_ui`], so a
   /// viewport shorter than the controls scrolls instead of clipping them.
   pub fn show_window(&mut self, ctx: &Context) -> Option<egui::Response> {
      // Local copy — `settings_window` borrows `open` while the body borrows `self`.
      let mut open = self.open;
      let resp = settings_window(ctx, "Light Settings", &mut open, |ui| {
         self.body_ui(ui)
      });
      self.open = open;
      resp
   }

   /// Draw the settings controls into `ui` — no window chrome.
   ///
   /// Render them in your own container (`egui::Window`, `SidePanel`, `ScrollArea`, …)
   /// instead of using [`Self::show_window`]. If that container can be shorter than the
   /// controls, wrap it in an `egui::ScrollArea`.
   pub fn body_ui(&mut self, ui: &mut Ui) {
      let theme = begin_body(ui);

      ui.heading("General");
      ui.checkbox(&mut self.active, "Effect Active");
      ui.horizontal(|ui| {
         ui.label("Color:");
         ui.color_edit_button_srgba(&mut self.color);
      });

      ui.separator();
      ui.heading("Motion");
      ui.horizontal(|ui| {
         ui.label("Motion:");
         ui.selectable_value(
            &mut self.motion,
            LightMotion::Drift,
            LightMotion::Drift.to_str(),
         )
         .on_hover_text("Slow wandering motes that bounce off the area edges");
         ui.selectable_value(
            &mut self.motion,
            LightMotion::Rise,
            LightMotion::Rise.to_str(),
         )
         .on_hover_text("Embers that rise from the bottom edge and sway as they climb");
      });

      ui.separator();
      ui.heading("Particles");

      ui.horizontal(|ui| {
         ui.label("Density (Count):");
         ui.add(Slider::new(&mut self.density, 0..=2000).logarithmic(true));
      });

      ui.horizontal(|ui| {
         ui.label("Radius (px):");
         ui.add(
            DragValue::new(&mut self.radius_min)
               .speed(0.5)
               .range(1.0..=200.0)
               .prefix("Min: "),
         );
         ui.add(
            DragValue::new(&mut self.radius_max)
               .speed(0.5)
               .range(1.0..=200.0)
               .prefix("Max: "),
         );
      });
      // Ensure min does not exceed max
      if self.radius_min > self.radius_max {
         self.radius_min = self.radius_max;
      }

      ui.horizontal(|ui| {
         ui.label("Intensity:");
         ui.add(Slider::new(&mut self.intensity, 0.0..=1.0));
      });

      ui.horizontal(|ui| {
         ui.label("Flicker:");
         ui.add(Slider::new(&mut self.flicker, 0.0..=1.0));
      });

      ui.separator();
      ui.heading("Physics");

      ui.label("Wind Vector:");
      ui.horizontal(|ui| {
         ui.add(Slider::new(&mut self.wind.x, -200.0..=200.0).text("Horizontal"));
      });
      ui.horizontal(|ui| {
         ui.add(Slider::new(&mut self.wind.y, -100.0..=100.0).text("Vertical"));
      });

      ui.horizontal(|ui| {
         ui.label("Lifetime (s):");
         ui.add(
            DragValue::new(&mut self.lifetime_min)
               .speed(0.1)
               .range(0.5..=60.0)
               .prefix("Min: "),
         );
         ui.add(
            DragValue::new(&mut self.lifetime_max)
               .speed(0.1)
               .range(0.5..=60.0)
               .prefix("Max: "),
         );
      });
      if self.lifetime_min > self.lifetime_max {
         self.lifetime_min = self.lifetime_max;
      }

      ui.separator();
      ui.heading("Rendering");
      ui.horizontal(|ui| {
         ui.label("Layer:");
         ui.selectable_value(
            &mut self.layer,
            ParticleLayer::Background,
            ParticleLayer::Background.to_str(),
         )
         .on_hover_text("Behind everything (Order::Background)");
         ui.selectable_value(
            &mut self.layer,
            ParticleLayer::Foreground,
            ParticleLayer::Foreground.to_str(),
         )
         .on_hover_text("In front of windows (Order::Foreground)");
         ui.selectable_value(
            &mut self.layer,
            ParticleLayer::Top,
            ParticleLayer::Top.to_str(),
         )
         .on_hover_text("On top of everything including debug/tooltips (Order::Debug)");
      });

      ui.add_space(theme.spacing.sm);
      if ui.button("Reset Defaults").clicked() {
         self.reset();
      }
   }
}

#[cfg(test)]
mod tests {
   use super::*;
   use egui::__run_test_ctx;

   #[test]
   fn defaults_are_sane() {
      let settings = LightSettings::default();
      assert!(!settings.open);
      assert!(settings.active);
      assert!(settings.density > 0);
      assert!(settings.radius_min <= settings.radius_max);
      assert!(settings.lifetime_min <= settings.lifetime_max);
      assert!(
         settings.lifetime_min > 1.0,
         "lifetimes are multi-second"
      );
      assert_eq!(settings.motion, LightMotion::Drift);
      assert_eq!(settings.layer, ParticleLayer::Foreground);
   }

   #[test]
   fn reset_keeps_open() {
      let mut settings = LightSettings {
         open: true,
         active: false,
         density: 3,
         radius_min: 90.0,
         radius_max: 91.0,
         intensity: 0.1,
         motion: LightMotion::Rise,
         wind: Vec2::new(7.0, 8.0),
         flicker: 0.9,
         lifetime_min: 1.0,
         lifetime_max: 2.0,
         layer: ParticleLayer::Top,
         color: Color32::from_rgb(9, 9, 9),
      };

      settings.reset();

      assert!(settings.open, "reset must not close the window");
      assert_eq!(
         settings,
         LightSettings {
            open: true,
            ..Default::default()
         }
      );
   }

   /// Lay the open settings window out on `screen` twice and return the second
   /// pass's outer window height (window rects settle on the second pass).
   ///
   /// Unlike [`egui::__run_test_ctx`] this keeps the real fonts: empty fonts
   /// make every galley 0 tall and the measured height meaningless.
   fn window_height_on(screen: Vec2) -> f32 {
      let ctx = Context::default();
      // Mirror the demo: install the theme once so spacing/`item_spacing`
      // match the real app instead of egui's tighter default style.
      let mut theme = crate::theme::Theme::new(crate::theme::ThemeKind::TokyoNight);
      theme.install(&ctx);
      let mut settings = LightSettings {
         open: true,
         ..Default::default()
      };
      let mut size = Vec2::ZERO;
      for _ in 0..2 {
         let output = ctx.run_ui(
            egui::RawInput {
               screen_rect: Some(egui::Rect::from_min_size(
                  egui::Pos2::ZERO,
                  screen,
               )),
               ..Default::default()
            },
            |ui| {
               if let Some(resp) = settings.show_window(ui.ctx()) {
                  size = resp.rect.size();
               }
            },
         );
         output.drop_without_applying_deltas();
      }
      size.y
   }

   #[test]
   fn settings_body_fits_the_demo_root() {
      let height = window_height_on(Vec2::new(900.0, 2000.0));
      assert!(
         height <= 650.0,
         "settings window is {height} tall, root is 650"
      );
   }

   #[test]
   fn body_ui_renders_without_a_window() {
      let mut settings = LightSettings::default();
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
      __run_test_ctx(|ctx| {
         let mut closed = LightSettings::default();
         assert!(closed.show_window(ctx).is_none());
      });

      __run_test_ctx(|ctx| {
         let mut open = LightSettings {
            open: true,
            ..Default::default()
         };
         assert!(open.show_window(ctx).is_some());
      });
   }
}
