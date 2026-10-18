//! Radial glow motes and embers for egui.

use egui::epaint::Mesh;
use egui::{Color32, Context, Id, LayerId, Order, Pos2, Rect, TextureId, Vec2, vec2};
use std::ops::RangeInclusive;

use super::super::shared::{ParticleLayer, ParticleRng, ordered, resolve_target};
use super::settings::LightSettings;

/// Smallest number of segments a glow fan is built from.
const MIN_SEGMENTS: usize = 12;
/// Largest number of segments a glow fan is built from.
const MAX_SEGMENTS: usize = 48;
/// Hot-core ring radius as a fraction of the mote radius.
const CORE_RING: f32 = 0.15;
/// Inner-ring radius as a fraction of the mote radius.
const INNER_RING: f32 = 0.45;
/// Outer-ring (rim) radius as a fraction of the mote radius, fully transparent.
const RIM_RING: f32 = 1.0;
/// Alpha of the hot-core ring, as a fraction of the mote colour.
const CORE_ALPHA: f32 = 0.55;
/// Alpha of the inner ring, as a fraction of the mote colour.
const INNER_ALPHA: f32 = 0.18;
/// Exponent of the size-attenuation curve (peak alpha against mote radius).
const SIZE_ATTENUATION_POWER: f32 = 0.5;
/// Lower clamp of the size-attenuation curve.
const MIN_SIZE_ATTENUATION: f32 = 0.4;
/// Peak wander speed of a drifting mote in pixels per second.
const DRIFT_SPEED: f32 = 20.0;
/// How quickly a drifting mote's velocity chases its wander target, per second.
const DRIFT_RESPONSE: f32 = 6.0;
/// Horizontal sway of a rising ember in pixels per second.
const RISE_SWAY: f32 = 18.0;
/// Upward speed of a rising ember in pixels per second.
const RISE_SPEED: f32 = 120.0;

/// Per-frame state of the [`Light`] effect, kept in egui's temporary data storage.
#[derive(Clone)]
struct LightState {
   motes: Vec<LightMote>,
   rng: ParticleRng,
}

impl LightState {
   fn new(seed: u64) -> Self {
      Self {
         motes: Vec::new(),
         rng: ParticleRng::new(seed),
      }
   }
}

/// One glowing mote.
#[derive(Clone, Copy)]
struct LightMote {
   /// Normalized coordinates (0.0..1.0).
   pos: Pos2,
   /// Velocity in pixels per second.
   vel: Vec2,
   /// Base radius in pixels.
   radius: f32,
   /// Per-mote intensity in 0..=1, already scaled by the settings intensity.
   intensity: f32,
   /// Phase driving flicker and motion, unique per mote.
   phase: f32,
   /// Flicker angular rate in radians per second.
   flicker_rate: f32,
   /// Flicker depth in 0..=1.
   flicker_depth: f32,
   /// Age in seconds.
   age: f32,
   /// Total lifetime in seconds.
   lifetime: f32,
}

impl LightMote {
   fn spawn(
      rng: &mut ParticleRng,
      motion: LightMotion,
      radius_range: &RangeInclusive<f32>,
      lifetime_range: &RangeInclusive<f32>,
      intensity: f32,
      flicker: f32,
   ) -> Self {
      let pos = match motion {
         LightMotion::Drift => Pos2::new(rng.range(0.0, 1.0), rng.range(0.0, 1.0)),
         LightMotion::Rise => Pos2::new(rng.range(0.0, 1.0), 1.0),
      };

      Self {
         pos,
         vel: Vec2::ZERO,
         radius: rng.range(*radius_range.start(), *radius_range.end()),
         intensity: (intensity * rng.range(0.6, 1.0)).clamp(0.0, 1.0),
         phase: rng.range(0.0, std::f32::consts::TAU),
         flicker_rate: rng.range(1.5, 4.0),
         flicker_depth: (flicker * rng.range(0.5, 1.0)).clamp(0.0, 1.0),
         age: 0.0,
         lifetime: rng.range(*lifetime_range.start(), *lifetime_range.end()),
      }
   }

   /// Advances the mote by `dt` seconds inside `area` (never leaving it).
   ///
   /// Returns `true` when a rising ember climbed out of the top edge and must
   /// be recycled; drifting motes always bounce and return `false`.
   fn advance(&mut self, motion: LightMotion, wind: Vec2, time: f32, dt: f32, area: Rect) -> bool {
      match motion {
         LightMotion::Drift => {
            // Damped wander: the velocity smoothly chases a slow phase-driven target.
            let target = Vec2::new(
               (self.phase + time * 0.7).sin(),
               (self.phase * 1.3 + time * 0.5).cos(),
            ) * DRIFT_SPEED
               + wind;
            let blend = (DRIFT_RESPONSE * dt).min(1.0);
            self.vel += (target - self.vel) * blend;
         }
         LightMotion::Rise => {
            self.vel.x = (self.phase + time).sin() * RISE_SWAY + wind.x;
            self.vel.y = -RISE_SPEED + wind.y;
         }
      }

      self.pos.x += self.vel.x * dt / area.width();
      self.pos.y += self.vel.y * dt / area.height();

      // A rising ember that crossed the top edge is recycled by the caller.
      let escaped = matches!(motion, LightMotion::Rise) && self.pos.y < 0.0;

      if matches!(motion, LightMotion::Drift) {
         // Bounce off the area edges so drifting motes stay on screen.
         if self.pos.x < 0.0 {
            self.pos.x = -self.pos.x;
            self.vel.x = self.vel.x.abs();
         } else if self.pos.x > 1.0 {
            self.pos.x = 2.0 - self.pos.x;
            self.vel.x = -self.vel.x.abs();
         }
         if self.pos.y < 0.0 {
            self.pos.y = -self.pos.y;
            self.vel.y = self.vel.y.abs();
         } else if self.pos.y > 1.0 {
            self.pos.y = 2.0 - self.pos.y;
            self.vel.y = -self.vel.y.abs();
         }
      }

      // Clamp away any residual overshoot (e.g. after a spike in `dt`).
      self.pos.x = self.pos.x.clamp(0.0, 1.0);
      self.pos.y = self.pos.y.clamp(0.0, 1.0);

      escaped
   }
}

/// Motion model for the [`Light`] effect.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightMotion {
   /// Slow wandering motes that bounce off the area edges.
   Drift,
   /// Embers that rise from the bottom edge and sway as they climb.
   Rise,
}

impl LightMotion {
   /// Human-readable name, matching the settings window labels.
   pub fn to_str(&self) -> &str {
      match self {
         LightMotion::Drift => "Drift",
         LightMotion::Rise => "Rise",
      }
   }
}

/// A configurable radial-glow effect for egui.
///
/// `Light` paints soft, flickering motes — drifting haze or rising embers — on
/// an egui layer. It maintains its state between frames using egui's temporary
/// data storage and never disturbs the host layout.
///
/// All motes are appended into a single triangle mesh per frame, so stacking
/// order is preserved and the effect costs one shape.
#[derive(Debug)]
pub struct Light {
   id: Id,
   color: Color32,
   density: usize,
   layer_order: Order,
   custom_layer: Option<LayerId>,
   custom_area: Option<Rect>,

   radius_range: RangeInclusive<f32>,
   intensity: f32,
   motion: LightMotion,
   wind: Vec2,
   flicker: f32,
   lifetime_range: RangeInclusive<f32>,
}

impl Light {
   /// Creates a new light effect with a unique ID.
   pub fn new(id_source: impl std::hash::Hash + std::fmt::Debug) -> Self {
      Self {
         id: Id::new(id_source),
         color: Color32::from_rgb(255, 224, 178),
         density: 48,
         layer_order: Order::Foreground,
         custom_layer: None,
         custom_area: None,
         // Neutral defaults: low density, wide soft radii, gentle flicker.
         radius_range: 8.0..=36.0,
         intensity: 0.7,
         motion: LightMotion::Drift,
         wind: Vec2::ZERO,
         flicker: 0.25,
         lifetime_range: 4.0..=9.0,
      }
   }

   /// Builds a light effect from [`LightSettings`].
   ///
   /// Inverted ranges are normalized (swapped) so a host can never hand the
   /// spawner an empty range.
   pub fn from_settings(
      id_source: impl std::hash::Hash + std::fmt::Debug,
      settings: &LightSettings,
   ) -> Self {
      let (radius_min, radius_max) = ordered(settings.radius_min, settings.radius_max);
      let (lifetime_min, lifetime_max) = ordered(settings.lifetime_min, settings.lifetime_max);
      let light = Self::new(id_source)
         .color(settings.color)
         .density(settings.density)
         .radius(radius_min..=radius_max)
         .intensity(settings.intensity)
         .motion(settings.motion)
         .wind(settings.wind)
         .flicker(settings.flicker)
         .lifetime(lifetime_min..=lifetime_max);

      match settings.layer {
         ParticleLayer::Background => light.on_background(),
         ParticleLayer::Foreground => light.on_foreground(),
         ParticleLayer::Top => light.on_top(),
      }
   }

   /// Sets the mote color.
   pub fn color(mut self, color: Color32) -> Self {
      self.color = color;
      self
   }

   /// Sets the number of motes.
   pub fn density(mut self, density: usize) -> Self {
      self.density = density;
      self
   }

   /// Sets the mote radius range (min..=max) in pixels.
   pub fn radius(mut self, range: RangeInclusive<f32>) -> Self {
      self.radius_range = range;
      self
   }

   /// Sets the global intensity multiplier in `0.0..=1.0`.
   pub fn intensity(mut self, intensity: f32) -> Self {
      self.intensity = intensity.clamp(0.0, 1.0);
      self
   }

   /// Sets the motion model.
   pub fn motion(mut self, motion: LightMotion) -> Self {
      self.motion = motion;
      self
   }

   /// Sets the wind vector.
   pub fn wind(mut self, wind: impl Into<Vec2>) -> Self {
      self.wind = wind.into();
      self
   }

   /// Sets the flicker depth in `0.0..=1.0` (`0` disables flicker).
   pub fn flicker(mut self, flicker: f32) -> Self {
      self.flicker = flicker.clamp(0.0, 1.0);
      self
   }

   /// Sets the mote lifetime range (min..=max) in seconds.
   pub fn lifetime(mut self, range: RangeInclusive<f32>) -> Self {
      self.lifetime_range = range;
      self
   }

   /// Places the light on the Debug layer (on top of everything).
   pub fn on_top(mut self) -> Self {
      self.layer_order = Order::Debug;
      self
   }

   /// Places the light on the Foreground layer.
   pub fn on_foreground(mut self) -> Self {
      self.layer_order = Order::Foreground;
      self
   }

   /// Places the light on the Background layer.
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

   /// Restricts the light effect to a specific area.
   ///
   /// If not set, it defaults to `ctx.content_rect()`.
   pub fn area(mut self, area: Rect) -> Self {
      self.custom_area = Some(area);
      self
   }

   /// Updates and renders the light effect.
   ///
   /// This should be called every frame. It automatically requests a repaint.
   pub fn show(self, ctx: &Context) {
      let Some((layer_id, area)) = resolve_target(
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

      let (mut motes, mut rng) = ctx.data_mut(|data| {
         let state = data.get_temp_mut_or_insert_with(self.id, || LightState::new(seed));
         (std::mem::take(&mut state.motes), state.rng)
      });

      // Spawn logic
      if motes.len() < self.density {
         let needed = self.density - motes.len();
         for _ in 0..needed {
            motes.push(LightMote::spawn(
               &mut rng,
               self.motion,
               &self.radius_range,
               &self.lifetime_range,
               self.intensity,
               self.flicker,
            ));
         }
      } else if motes.len() > self.density {
         motes.truncate(self.density);
      }

      let painter = ctx.layer_painter(layer_id);
      let mut mesh = Mesh::default();

      for mote in &mut motes {
         // --- Update ---
         mote.age += dt;
         let escaped = mote.advance(self.motion, self.wind, time as f32, dt, area);
         // Recycle expired motes, and rising embers that climbed out of the top
         // edge, so no glow is painted outside the effect area.
         if mote.age >= mote.lifetime || escaped {
            *mote = LightMote::spawn(
               &mut rng,
               self.motion,
               &self.radius_range,
               &self.lifetime_range,
               self.intensity,
               self.flicker,
            );
         }

         // --- Render ---
         let u = (mote.age / mote.lifetime).clamp(0.0, 1.0);
         let envelope = envelope(u);
         // Radius and intensity shrink with age for rising embers.
         let shrink = match self.motion {
            LightMotion::Drift => 1.0,
            LightMotion::Rise => 1.0 - 0.5 * u,
         };
         let flicker = 1.0
            - mote.flicker_depth
               * 0.5
               * (1.0 + (mote.phase + time as f32 * mote.flicker_rate).sin());
         // A big disc must not dump more light than a small one, so scale its
         // peak alpha down with its radius.
         let attenuation = size_attenuation(
            mote.radius,
            *self.radius_range.start(),
            *self.radius_range.end(),
         );
         let alpha = (mote.intensity * shrink * envelope * flicker * attenuation).clamp(0.0, 1.0);

         let pixel_pos = Pos2::new(
            area.min.x + mote.pos.x * area.width(),
            area.min.y + mote.pos.y * area.height(),
         );
         let color = self.color.gamma_multiply(alpha);
         mesh.append(glow_mesh(pixel_pos, mote.radius * shrink, color));
      }

      // Return motes to state
      ctx.data_mut(|data| {
         let state = data.get_temp_mut_or_insert_with(self.id, || LightState::new(seed));
         state.motes = motes;
         state.rng = rng;
      });

      painter.add(egui::Shape::mesh(mesh));
      ctx.request_repaint();
   }
}

/// Builds one radial glow as a triangle fan with a hot core and a long tail.
///
/// The centre is painted at full `color`, then three rings close the fan: a hot
/// core at `CORE_RING * radius` carrying `CORE_ALPHA` of the colour, an inner
/// ring at `INNER_RING * radius` carrying `INNER_ALPHA`, and a transparent rim
/// at `RIM_RING * radius`. The steep centre-to-core drop gives the point its
/// hot core and the long, faint core-to-rim tail gives it reach, which is what
/// reads as a light. A single linear two-ring fan instead fades at one constant
/// rate from centre to rim, and the eye reads that flat profile as a milky disc
/// rather than a glowing source.
///
/// The mesh uses the font texture (white at `WHITE_UV`), so the fan fades out
/// instead of tinting; the rim must stay premultiplied zero for that reason.
fn glow_mesh(center: Pos2, radius: f32, color: Color32) -> Mesh {
   let segments = ((radius / 4.0).clamp(MIN_SEGMENTS as f32, MAX_SEGMENTS as f32)) as usize;
   let mut mesh = Mesh::default();
   mesh.texture_id = TextureId::default();

   mesh.colored_vertex(center, color);

   // Ring radii and alphas, innermost first; the rim stays fully transparent.
   let rings = [
      (
         CORE_RING * radius,
         color.gamma_multiply(CORE_ALPHA),
      ),
      (
         INNER_RING * radius,
         color.gamma_multiply(INNER_ALPHA),
      ),
      (RIM_RING * radius, Color32::TRANSPARENT),
   ];
   for (ring_radius, ring_color) in rings {
      for i in 0..segments {
         let (sin, cos) = (i as f32 / segments as f32 * std::f32::consts::TAU).sin_cos();
         mesh.colored_vertex(center + vec2(cos, sin) * ring_radius, ring_color);
      }
   }

   for i in 0..segments {
      let next = (i + 1) % segments;

      // Centre fan onto the hot-core ring, then a quad strip between each
      // successive pair of rings out to the rim.
      mesh.add_triangle(0, 1 + i as u32, 1 + next as u32);
      for ring in 0..(rings.len() as u32 - 1) {
         let inner = 1 + ring * segments as u32 + i as u32;
         let inner_next = 1 + ring * segments as u32 + next as u32;
         let outer = 1 + (ring + 1) * segments as u32 + i as u32;
         let outer_next = 1 + (ring + 1) * segments as u32 + next as u32;

         mesh.add_triangle(inner, outer, outer_next);
         mesh.add_triangle(inner, outer_next, inner_next);
      }
   }

   mesh
}

/// Peak-alpha multiplier that stops a large mote from outshining a small one.
///
/// A glow's peak is its centre, so shrinking that peak by size changes how much
/// light a mote dumps into the frame while leaving its shape alone. The curve is
/// `(min_radius / radius).powf(SIZE_ATTENUATION_POWER)` clamped to
/// `MIN_SIZE_ATTENUATION..=1.0`. With the crate's default `8.0..=36.0` radius
/// range the exponent lands near `0.47` at `max_radius`: a disc 4.5x larger
/// peaks at roughly half the alpha, which is enough to even out the frame
/// without making the big motes read as dim. A gentler exponent such as `0.35`
/// only reaches ~`0.59` over that range and leaves the largest motes dominant,
/// which is the complaint this curve answers.
///
/// Monotonic non-increasing in `radius`, exactly `1.0` at `min_radius`, and
/// never leaves `0.4..=1.0`.
fn size_attenuation(radius: f32, min_radius: f32, max_radius: f32) -> f32 {
   // Treat the smaller endpoint as the floor and never divide by zero, so an
   // inverted or zero-width range cannot produce a NaN alpha.
   let floor = min_radius.min(max_radius).max(f32::MIN_POSITIVE);
   (floor / radius.max(floor))
      .powf(SIZE_ATTENUATION_POWER)
      .clamp(MIN_SIZE_ATTENUATION, 1.0)
}

/// Life envelope over `u = age / lifetime`.
///
/// Fades in over the first 20 % and out over the last 30 %; exactly `0` at both
/// ends so nothing pops in or out.
fn envelope(u: f32) -> f32 {
   let u = u.clamp(0.0, 1.0);
   if u < 0.2 {
      u / 0.2
   } else if u > 0.7 {
      (1.0 - u) / 0.3
   } else {
      1.0
   }
}

#[cfg(test)]
mod tests {
   use super::*;
   use egui::__run_test_ctx;

   #[test]
   fn glow_mesh_vertex_and_index_counts() {
      let mesh = glow_mesh(Pos2::ZERO, 100.0, Color32::WHITE);
      // radius / 4 == 25 segments -> 1 centre + 3 * 25 ring vertices.
      assert_eq!(mesh.vertices.len(), 76);
      // 3 indices for the core fan + 12 for the two halo quad strips, per segment.
      assert_eq!(mesh.indices.len(), 375);
      assert_eq!(mesh.texture_id, TextureId::default());
   }

   #[test]
   fn glow_mesh_ring_alphas_fall_off() {
      let segments = 25;
      let mesh = glow_mesh(Pos2::ZERO, 100.0, Color32::from_white_alpha(255));
      let center = mesh.vertices[0].color;
      let core = mesh.vertices[1].color;
      let inner = mesh.vertices[1 + segments].color;
      let rim = mesh.vertices[1 + 2 * segments].color;

      assert!(
         center.a() > core.a(),
         "centre must be brighter than the hot-core ring"
      );
      assert!(
         core.a() > inner.a(),
         "hot-core ring must be brighter than the inner ring"
      );
      assert!(
         inner.a() > rim.a(),
         "inner ring must be brighter than the rim"
      );
      assert_eq!(rim, Color32::TRANSPARENT);
   }

   #[test]
   fn glow_mesh_center_tracks_intensity() {
      let dim = glow_mesh(Pos2::ZERO, 20.0, Color32::from_white_alpha(60));
      let bright = glow_mesh(Pos2::ZERO, 20.0, Color32::from_white_alpha(220));
      assert_eq!(dim.vertices[0].color.a(), 60);
      assert_eq!(bright.vertices[0].color.a(), 220);
   }

   #[test]
   fn size_attenuation_is_one_at_min_radius() {
      assert_eq!(size_attenuation(8.0, 8.0, 36.0), 1.0);
   }

   #[test]
   fn size_attenuation_is_non_increasing_across_the_range() {
      let (min_radius, max_radius) = (8.0, 36.0);
      let mut previous = f32::INFINITY;
      for step in 0..=64 {
         let radius = min_radius + (max_radius - min_radius) * step as f32 / 64.0;
         let value = size_attenuation(radius, min_radius, max_radius);
         assert!(
            value <= previous,
            "attenuation grew from {previous} to {value} at radius {radius}"
         );
         previous = value;
      }
   }

   #[test]
   fn size_attenuation_stays_within_bounds() {
      let (min_radius, max_radius) = (8.0, 36.0);
      for step in 0..=64 {
         let radius = min_radius + (max_radius - min_radius) * step as f32 / 64.0;
         let value = size_attenuation(radius, min_radius, max_radius);
         assert!(
            (0.4..=1.0).contains(&value),
            "attenuation {value} left 0.4..=1.0 at radius {radius}"
         );
      }

      let at_max = size_attenuation(max_radius, min_radius, max_radius);
      assert!(
         (0.45..=0.5).contains(&at_max),
         "attenuation at max radius {at_max} must read about 0.45..=0.5"
      );
   }

   #[test]
   fn peak_alpha_does_not_grow_with_radius() {
      let (min_radius, max_radius) = (8.0, 36.0);
      // Mirrors the peak of `Light::show`'s alpha: the envelope and flicker
      // both reach 1.0 there, and the settings are identical apart from the
      // mote radius, so the size attenuation is the only difference.
      let peak_alpha = |radius: f32| {
         let intensity = 0.8;
         let shrink = 1.0;
         let envelope = 1.0;
         let flicker = 1.0;
         (intensity
            * shrink
            * envelope
            * flicker
            * size_attenuation(radius, min_radius, max_radius))
         .clamp(0.0, 1.0)
      };

      let small = peak_alpha(min_radius);
      let large = peak_alpha(max_radius);
      assert!(
         large <= small,
         "a mote at radius_max must not outshine one at radius_min ({large} > {small})"
      );
      assert!(
         small > large,
         "the peak alpha must actually fall with radius ({small} == {large})"
      );
   }

   #[test]
   fn from_settings_maps_color_density_motion_layer() {
      let settings = LightSettings {
         color: Color32::from_rgb(1, 2, 3),
         density: 13,
         motion: LightMotion::Rise,
         layer: ParticleLayer::Top,
         ..Default::default()
      };

      let light = Light::from_settings("light_map", &settings);
      assert_eq!(light.color, Color32::from_rgb(1, 2, 3));
      assert_eq!(light.density, 13);
      assert_eq!(light.motion, LightMotion::Rise);
      assert_eq!(light.layer_order, Order::Debug);

      let background = Light::from_settings(
         "light_map_bg",
         &LightSettings {
            layer: ParticleLayer::Background,
            ..Default::default()
         },
      );
      assert_eq!(background.layer_order, Order::Background);
   }

   #[test]
   fn from_settings_swaps_inverted_ranges() {
      let settings = LightSettings {
         radius_min: 40.0,
         radius_max: 5.0,
         lifetime_min: 20.0,
         lifetime_max: 2.0,
         ..Default::default()
      };

      let light = Light::from_settings("light_inverted", &settings);
      assert_eq!(*light.radius_range.start(), 5.0);
      assert_eq!(*light.radius_range.end(), 40.0);
      assert_eq!(*light.lifetime_range.start(), 2.0);
      assert_eq!(*light.lifetime_range.end(), 20.0);
   }

   #[test]
   fn show_spawns_density_motes() {
      __run_test_ctx(|ctx| {
         let area = Rect::from_min_size(Pos2::ZERO, vec2(64.0, 64.0));
         Light::new("test_light").density(7).area(area).show(ctx);

         let state = ctx
            .data(|d| d.get_temp::<LightState>(Id::new("test_light")))
            .expect("LightState should exist after show");
         assert_eq!(state.motes.len(), 7);
      });
   }

   #[test]
   fn envelope_is_zero_at_both_ends() {
      assert!(
         envelope(0.0).abs() < 1e-6,
         "envelope must start at zero"
      );
      assert!(
         envelope(1.0).abs() < 1e-6,
         "envelope must end at zero"
      );
      assert_eq!(envelope(0.2), 1.0);
      assert_eq!(envelope(0.5), 1.0);
      assert_eq!(envelope(0.7), 1.0);
   }

   #[test]
   fn large_dt_keeps_motes_contained() {
      __run_test_ctx(|ctx| {
         let area = Rect::from_min_size(Pos2::ZERO, vec2(64.0, 64.0));

         for frame in 0..120 {
            ctx.input_mut(|i| {
               i.stable_dt = 30.0;
               i.time = f64::from(frame);
            });
            Light::new("test_light_large_dt").density(9).area(area).show(ctx);
         }

         let state = ctx
            .data(|d| d.get_temp::<LightState>(Id::new("test_light_large_dt")))
            .expect("LightState should exist after show");
         assert_eq!(state.motes.len(), 9);
         for mote in &state.motes {
            assert!(
               mote.age <= mote.lifetime,
               "age {} exceeded lifetime {}",
               mote.age,
               mote.lifetime
            );
            assert!(
               (0.0..=1.0).contains(&mote.pos.x),
               "x {} left the area",
               mote.pos.x
            );
            assert!(
               (0.0..=1.0).contains(&mote.pos.y),
               "y {} left the area",
               mote.pos.y
            );
         }
      });
   }

   #[test]
   fn large_dt_keeps_rising_motes_contained() {
      __run_test_ctx(|ctx| {
         let area = Rect::from_min_size(Pos2::ZERO, vec2(64.0, 64.0));
         let id = Id::new("test_light_rise_large_dt");

         // Smallest `y` any ember reached. It proves the run actually drove
         // embers into the top edge, and stays above zero only when they are
         // recycled instead of piling up on it.
         let mut min_y = f32::INFINITY;

         for frame in 0..120 {
            ctx.input_mut(|i| {
               i.stable_dt = 30.0;
               i.time = f64::from(frame);
            });
            Light::new("test_light_rise_large_dt")
               .density(9)
               .motion(LightMotion::Rise)
               .area(area)
               .show(ctx);

            let state = ctx
               .data(|d| d.get_temp::<LightState>(id))
               .expect("LightState should exist after show");
            for mote in &state.motes {
               min_y = min_y.min(mote.pos.y);
            }
         }

         let state = ctx
            .data(|d| d.get_temp::<LightState>(id))
            .expect("LightState should exist after show");
         assert_eq!(state.motes.len(), 9);
         for mote in &state.motes {
            assert!(
               mote.age <= mote.lifetime,
               "age {} exceeded lifetime {}",
               mote.age,
               mote.lifetime
            );
            assert!(
               (0.0..=1.0).contains(&mote.pos.x),
               "x {} left the area",
               mote.pos.x
            );
            assert!(
               (0.0..=1.0).contains(&mote.pos.y),
               "y {} left the area",
               mote.pos.y
            );
         }

         assert!(
            min_y < 0.1,
            "no rising ember reached the top edge (min y {min_y})"
         );
         assert!(
            min_y > 0.0,
            "an ember was left resting on the top edge (min y {min_y})"
         );
      });
   }

   #[test]
   fn degenerate_area_is_a_noop() {
      __run_test_ctx(|ctx| {
         let area = Rect::from_min_size(Pos2::ZERO, vec2(0.0, 64.0));
         Light::new("test_light_degenerate").density(5).area(area).show(ctx);
         assert!(
            ctx.data(|d| d.get_temp::<LightState>(Id::new("test_light_degenerate")))
               .is_none(),
            "a degenerate area must not create state"
         );
      });
   }
}
