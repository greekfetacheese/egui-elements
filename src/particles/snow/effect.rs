//! Snow particle effect, ported from [`egui-snow`](https://github.com/LaVashikk/egui-snow)
//! (v0.36.0, MIT License, Copyright (c) 2025 LaVashikk). The port keeps the upstream
//! algorithm, defaults and behavior; randomness uses an internal PRNG instead of `rand`.

use egui::{Color32, Context, Id, LayerId, Order, Pos2, Rect, Vec2};
use std::ops::RangeInclusive;

use super::super::shared::{ParticleLayer, ParticleRng, ordered, resolve_target};
use super::settings::SnowSettings;

#[derive(Clone)]
struct SnowState {
   flakes: Vec<Snowflake>,
   rng: ParticleRng,
}

impl SnowState {
   fn new(seed: u64) -> Self {
      Self {
         flakes: Vec::new(),
         rng: ParticleRng::new(seed),
      }
   }
}

#[derive(Clone, Copy)]
struct Snowflake {
   /// Normalized coordinates (0.0..1.0)
   normalized_pos: Pos2,
   /// Fall speed (px/sec), unique for each particle
   fall_speed: f32,
   /// Random turbulence on X axis, unique for each particle
   turbulence: f32,
   /// Size of the snowflake
   size: f32,
   /// Oscillation phase
   phase: f32,
}

/// A configurable snow effect for egui.
///
/// `Snow` allows you to add a falling snow animation to any egui layer or area.
/// It maintains its state between frames using egui's temporary data storage.
///
/// Ported from [`egui-snow`](https://github.com/LaVashikk/egui-snow); the algorithm,
/// defaults and builders match upstream.
#[derive(Debug)]
pub struct Snow {
   id: Id,
   color: Color32,
   density: usize,
   layer_order: Order,
   custom_layer: Option<LayerId>,
   custom_area: Option<Rect>,

   size_range: RangeInclusive<f32>,
   speed_range: RangeInclusive<f32>,
   wind: Vec2,
}

impl Snow {
   /// Creates a new snow effect with a unique ID.
   pub fn new(id_source: impl std::hash::Hash + std::fmt::Debug) -> Self {
      Self {
         id: Id::unique(id_source),
         color: Color32::WHITE,
         density: 50,
         layer_order: Order::Foreground,
         custom_layer: None,
         custom_area: None,
         // Default values
         size_range: 0.3..=1.5,
         speed_range: 40.0..=100.0,
         wind: Vec2::ZERO,
      }
   }

   /// Builds a snow effect from [`SnowSettings`].
   ///
   /// Inverted ranges are normalized (swapped) so a host can never hand the
   /// spawner an empty range.
   pub fn from_settings(
      id_source: impl std::hash::Hash + std::fmt::Debug,
      settings: &SnowSettings,
   ) -> Self {
      let (size_min, size_max) = ordered(settings.size_min, settings.size_max);
      let (speed_min, speed_max) = ordered(settings.speed_min, settings.speed_max);
      let snow = Self::new(id_source)
         .color(settings.color)
         .density(settings.density)
         .size(size_min..=size_max)
         .speed(speed_min..=speed_max)
         .wind(settings.wind);

      match settings.layer {
         ParticleLayer::Background => snow.on_background(),
         ParticleLayer::Foreground => snow.on_foreground(),
         ParticleLayer::Top => snow.on_top(),
      }
   }

   /// Sets the color of the snowflakes.
   pub fn color(mut self, color: Color32) -> Self {
      self.color = color;
      self
   }

   /// Sets the number of snowflakes.
   pub fn density(mut self, density: usize) -> Self {
      self.density = density;
      self
   }

   /// Sets the falling speed range (min..=max) in pixels per second.
   pub fn speed(mut self, range: RangeInclusive<f32>) -> Self {
      self.speed_range = range;
      self
   }

   /// Sets the snowflake size range (min..=max).
   pub fn size(mut self, range: RangeInclusive<f32>) -> Self {
      self.size_range = range;
      self
   }

   /// Sets the wind vector.
   pub fn wind(mut self, wind: impl Into<Vec2>) -> Self {
      self.wind = wind.into();
      self
   }

   /// Places the snow on the Debug layer (on top of everything).
   pub fn on_top(mut self) -> Self {
      self.layer_order = Order::Debug;
      self
   }

   /// Places the snow on the Foreground layer.
   pub fn on_foreground(mut self) -> Self {
      self.layer_order = Order::Foreground;
      self
   }

   /// Places the snow on the Background layer.
   pub fn on_background(mut self) -> Self {
      self.layer_order = Order::Background;
      self
   }

   /// Provides full control over the rendering layer.
   ///
   /// This overrides `on_top`, `on_foreground`, and `on_background`.
   pub fn layer(mut self, layer_id: LayerId) -> Self {
      self.custom_layer = Some(layer_id);
      self
   }

   /// Restricts the snow effect to a specific area.
   ///
   /// If not set, it defaults to `ctx.content_rect()`.
   pub fn area(mut self, area: Rect) -> Self {
      self.custom_area = Some(area);
      self
   }

   /// Updates and renders the snow effect.
   ///
   /// This should be called every frame. It automatically requests a repaint.
   pub fn show(self, ctx: &Context) {
      let Some((layer_id, screen_rect)) = resolve_target(
         ctx,
         self.id,
         self.layer_order,
         self.custom_layer,
         self.custom_area,
      ) else {
         return;
      };

      let dt = ctx.input(|i| i.stable_dt).min(0.1);
      let time = ctx.input(|i| i.time);
      let seed = self.id.value() ^ time.to_bits();

      let (mut snowflakes, mut rng) = ctx.data_mut(|data| {
         let state = data.get_temp_mut_or_insert_with(self.id, || SnowState::new(seed));
         (std::mem::take(&mut state.flakes), state.rng)
      });

      // Spawn logic
      if snowflakes.len() < self.density {
         let needed = self.density - snowflakes.len();
         for _ in 0..needed {
            snowflakes.push(Snowflake::spawn(
               &mut rng,
               true,
               &self.size_range,
               &self.speed_range,
            ));
         }
      } else if snowflakes.len() > self.density {
         snowflakes.truncate(self.density);
      }

      let painter = ctx.layer_painter(layer_id);

      // Cache values for alpha calculation
      let min_size = *self.size_range.start();
      let size_diff = (*self.size_range.end() - min_size).max(0.0001);

      // Update & Render logic
      for flake in &mut snowflakes {
         // --- Update ---
         // Y: Individual speed + vertical wind
         let pixel_dy = (flake.fall_speed + self.wind.y) * dt;

         // X: Individual turbulence + global wind + sine wave
         let sway = ((time as f32 + flake.phase).sin()) * 5.0;
         let pixel_dx = (flake.turbulence + self.wind.x + sway) * dt;

         flake.normalized_pos.x += pixel_dx / screen_rect.width();
         flake.normalized_pos.y += pixel_dy / screen_rect.height();

         // Respawn (Wrap Y)
         // If it flew down
         if flake.normalized_pos.y > 1.0 {
            *flake = Snowflake::spawn(
               &mut rng,
               false,
               &self.size_range,
               &self.speed_range,
            );
         }
         // If it flew up (strong upward wind)
         else if flake.normalized_pos.y < -0.05 {
            flake.normalized_pos.y = 1.0;
            flake.normalized_pos.x = rng.range(0.0, 1.0);
            flake.size = rng.range(*self.size_range.start(), *self.size_range.end());
            flake.fall_speed = rng.range(*self.speed_range.start(), *self.speed_range.end());
         }

         // Wrap X (seamless horizontal transition)
         if flake.normalized_pos.x > 1.0 {
            flake.normalized_pos.x -= 1.0;
         } else if flake.normalized_pos.x < 0.0 {
            flake.normalized_pos.x += 1.0;
         }

         // --- Render ---
         let pixel_pos = Pos2::new(
            screen_rect.min.x + flake.normalized_pos.x * screen_rect.width(),
            screen_rect.min.y + flake.normalized_pos.y * screen_rect.height(),
         );

         // Calculate alpha based on size (simulating depth)
         // Smaller particles are more transparent
         let depth_factor = (flake.size - min_size) / size_diff;
         let alpha_mult = 0.4 + (0.6 * depth_factor); // Min. transparency 40%
         let final_color = self.color.gamma_multiply(alpha_mult);

         painter.circle_filled(pixel_pos, flake.size, final_color);
      }

      // Return snowflakes to state
      ctx.data_mut(|data| {
         let state = data.get_temp_mut_or_insert_with(self.id, || SnowState::new(seed));
         state.flakes = snowflakes;
         state.rng = rng;
      });

      ctx.request_repaint();
   }
}

impl Snowflake {
   fn spawn(
      rng: &mut ParticleRng,
      random_y: bool,
      size_range: &RangeInclusive<f32>,
      speed_range: &RangeInclusive<f32>,
   ) -> Self {
      Self {
         normalized_pos: Pos2::new(
            rng.range(0.0, 1.0),
            if random_y { rng.range(0.0, 1.0) } else { -0.05 },
         ),
         fall_speed: rng.range(*speed_range.start(), *speed_range.end()),
         turbulence: rng.range(-20.0, 20.0),
         size: rng.range(*size_range.start(), *size_range.end()),
         phase: rng.range(0.0, std::f32::consts::PI * 2.0),
      }
   }
}

#[cfg(test)]
mod tests {
   use super::*;
   use egui::__run_test_ctx;
   use egui::{Pos2, Rect, vec2};

   #[test]
   fn from_settings_swaps_inverted_ranges() {
      let settings = SnowSettings {
         size_min: 2.0,
         size_max: 0.5,
         speed_min: 100.0,
         speed_max: 10.0,
         ..Default::default()
      };

      let snow = Snow::from_settings("inverted", &settings);
      assert_eq!(*snow.size_range.start(), 0.5);
      assert_eq!(*snow.size_range.end(), 2.0);
      assert_eq!(*snow.speed_range.start(), 10.0);
      assert_eq!(*snow.speed_range.end(), 100.0);
   }

   #[test]
   fn from_settings_maps_layer() {
      let settings = SnowSettings {
         layer: ParticleLayer::Top,
         color: Color32::from_rgb(1, 2, 3),
         density: 12,
         ..Default::default()
      };

      let snow = Snow::from_settings("top", &settings);
      assert_eq!(snow.layer_order, Order::Debug);
      assert_eq!(snow.color, Color32::from_rgb(1, 2, 3));
      assert_eq!(snow.density, 12);

      let background = Snow::from_settings(
         "background",
         &SnowSettings {
            layer: ParticleLayer::Background,
            ..Default::default()
         },
      );
      assert_eq!(background.layer_order, Order::Background);
   }

   #[test]
   fn show_spawns_flakes() {
      __run_test_ctx(|ctx| {
         let area = Rect::from_min_size(Pos2::ZERO, vec2(64.0, 64.0));
         Snow::new("test_snow").density(7).area(area).show(ctx);

         let state = ctx
            .data(|d| d.get_temp::<SnowState>(Id::unique("test_snow")))
            .expect("SnowState should exist after show");
         assert_eq!(state.flakes.len(), 7);
      });
   }
}
