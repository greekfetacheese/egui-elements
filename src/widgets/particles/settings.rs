//! [`SnowSettings`] — public knobs for the snow effect plus a themed settings window.

use egui::{Color32, Context, Slider, Vec2};

use super::snow::SnowLayer;
use crate::widgets::Window;

/// User-facing knobs for [`Snow`](super::Snow).
///
/// The defaults match the upstream `egui-snow` demo, so
/// `Snow::from_settings("snow", &SnowSettings::default())` reproduces the
/// upstream look.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct SnowSettings {
   /// Settings-window visibility. Default `false` (the example opens it).
   pub open: bool,
   /// Whether the effect is drawn at all.
   pub active: bool,
   /// Snowflake color.
   pub color: Color32,
   /// Number of snowflakes.
   pub density: usize,
   /// Minimum snowflake size in pixels.
   pub size_min: f32,
   /// Maximum snowflake size in pixels.
   pub size_max: f32,
   /// Minimum fall speed in pixels per second.
   pub speed_min: f32,
   /// Maximum fall speed in pixels per second.
   pub speed_max: f32,
   /// Global wind vector (x = horizontal, y = vertical).
   pub wind: Vec2,
   /// Layer the effect is painted on.
   pub layer: SnowLayer,
}

impl Default for SnowSettings {
   fn default() -> Self {
      Self {
         open: false,
         active: true,
         color: Color32::from_white_alpha(200),
         density: 400,
         size_min: 0.3,
         size_max: 1.0,
         speed_min: 40.0,
         speed_max: 120.0,
         wind: Vec2::ZERO,
         layer: SnowLayer::Foreground,
      }
   }
}

impl SnowSettings {
   /// Knobs back to defaults; `open` is preserved (a reset must not close the window).
   pub fn reset(&mut self) {
      let open = self.open;
      *self = Self::default();
      self.open = open;
   }

   /// Themed floating settings window; returns the window's outer `Response` when open.
   pub fn show_window(&mut self, ctx: &Context) -> Option<egui::Response> {
      let theme = crate::theme::Theme::current(ctx);
      let frame = theme.window_frame;
      let title_frame = frame.stroke(egui::Stroke::NONE);
      let mut open = self.open; // local copy — `.open()` borrows, `self` is used inside the closure
      let resp = Window::new("Snow Settings")
         .open(&mut open)
         .default_width(300.0)
         .frame(frame)
         .title_frame(title_frame)
         .show(ctx, |ui| {
            ui.heading("General");
            ui.checkbox(&mut self.active, "Effect Active");
            ui.horizontal(|ui| {
               ui.label("Color:");
               ui.color_edit_button_srgba(&mut self.color);
            });

            ui.separator();
            ui.heading("Particles");

            ui.label("Density (Count):");
            ui.add(Slider::new(&mut self.density, 0..=20000).logarithmic(true));

            ui.add_space(5.0);
            ui.label("Size Range (px):");
            ui.horizontal(|ui| {
               ui.add(
                  egui::DragValue::new(&mut self.size_min)
                     .speed(0.1)
                     .range(0.1..=20.0)
                     .prefix("Min: "),
               );
               ui.add(
                  egui::DragValue::new(&mut self.size_max)
                     .speed(0.1)
                     .range(0.1..=20.0)
                     .prefix("Max: "),
               );
            });
            // Ensure min does not exceed max
            if self.size_min > self.size_max {
               self.size_min = self.size_max;
            }

            ui.separator();
            ui.heading("Physics");

            ui.label("Fall Speed Range (px/s):");
            ui.horizontal(|ui| {
               ui.add(
                  egui::DragValue::new(&mut self.speed_min)
                     .speed(1.0)
                     .range(0.0..=500.0)
                     .prefix("Min: "),
               );
               ui.add(
                  egui::DragValue::new(&mut self.speed_max)
                     .speed(1.0)
                     .range(0.0..=500.0)
                     .prefix("Max: "),
               );
            });
            if self.speed_min > self.speed_max {
               self.speed_min = self.speed_max;
            }

            ui.label("Wind Vector:");
            ui.horizontal(|ui| {
               ui.add(Slider::new(&mut self.wind.x, -200.0..=200.0).text("Horizontal"));
            });
            ui.horizontal(|ui| {
               ui.add(Slider::new(&mut self.wind.y, -100.0..=100.0).text("Vertical"));
            });

            ui.separator();
            ui.heading("Rendering");
            ui.horizontal(|ui| {
               ui.label("Layer:");
               ui.selectable_value(
                  &mut self.layer,
                  SnowLayer::Background,
                  SnowLayer::Background.to_str(),
               )
               .on_hover_text("Behind everything (Order::Background)");
               ui.selectable_value(
                  &mut self.layer,
                  SnowLayer::Foreground,
                  SnowLayer::Foreground.to_str(),
               )
               .on_hover_text("In front of windows (Order::Foreground)");
               ui.selectable_value(
                  &mut self.layer,
                  SnowLayer::Top,
                  SnowLayer::Top.to_str(),
               )
               .on_hover_text("On top of everything including debug/tooltips (Order::Debug)");
            });

            ui.add_space(10.0);
            if ui.button("Reset Defaults").clicked() {
               self.reset();
            }
         });

      self.open = open;
      resp.map(|r| r.response)
   }
}

#[cfg(test)]
mod tests {
   use super::*;
   use egui::__run_test_ctx;

   #[test]
   fn defaults_match_upstream_demo() {
      let settings = SnowSettings::default();
      assert!(!settings.open);
      assert!(settings.active);
      assert_eq!(settings.color, Color32::from_white_alpha(200));
      assert_eq!(settings.density, 400);
      assert_eq!(settings.size_min, 0.3);
      assert_eq!(settings.size_max, 1.0);
      assert_eq!(settings.speed_min, 40.0);
      assert_eq!(settings.speed_max, 120.0);
      assert_eq!(settings.wind, Vec2::ZERO);
      assert_eq!(settings.layer, SnowLayer::Foreground);
   }

   #[test]
   fn reset_keeps_open() {
      let mut settings = SnowSettings {
         open: true,
         active: false,
         density: 3,
         size_min: 9.0,
         size_max: 11.0,
         speed_min: 5.0,
         speed_max: 6.0,
         wind: Vec2::new(7.0, 8.0),
         layer: SnowLayer::Top,
         color: Color32::from_rgb(9, 9, 9),
         ..Default::default()
      };

      settings.reset();

      assert!(settings.open, "reset must not close the window");
      assert_eq!(
         settings,
         SnowSettings {
            open: true,
            ..Default::default()
         }
      );
   }

   #[test]
   fn show_window_returns_response_only_when_open() {
      __run_test_ctx(|ctx| {
         let mut closed = SnowSettings::default();
         assert!(closed.show_window(ctx).is_none());
      });

      __run_test_ctx(|ctx| {
         let mut open = SnowSettings {
            open: true,
            ..Default::default()
         };
         assert!(open.show_window(ctx).is_some());
      });
   }
}
