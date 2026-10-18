//! Lightning bolts and a full-area flash for egui.

use egui::{Color32, Context, Id, LayerId, Order, Pos2, Rect, Shape, Stroke, Vec2, vec2};
use std::time::Duration;

use super::super::shared::{ParticleLayer, ParticleRng, resolve_target};
use super::settings::LightningSettings;

/// Recursion levels used to displace a bolt's main trunk.
const BOLT_LEVELS: u32 = 6;
/// Deepest branch recursion the settings may ask for (bounds the recursion).
const MAX_BRANCH_DEPTH: u32 = 8;
/// Horizontal spawn range of a bolt's endpoints in normalized space.
const BOLT_X_MIN: f32 = 0.15;
/// Horizontal spawn range of a bolt's endpoints in normalized space.
const BOLT_X_MAX: f32 = 0.85;

/// Fraction of the parent a branch may attach at (lower end).
const BRANCH_START_MIN: f32 = 0.45;
/// Fraction of the parent a branch may attach at (upper end).
const BRANCH_START_MAX: f32 = 0.95;
/// Branch length as a fraction of its parent's length (shortest).
const BRANCH_LENGTH_MIN: f32 = 0.25;
/// Branch length as a fraction of its parent's length (longest).
const BRANCH_LENGTH_MAX: f32 = 0.55;
/// Smallest angle a branch leaves its parent at, in radians.
const BRANCH_ANGLE_MIN: f32 = 0.5;
/// Largest angle a branch leaves its parent at, in radians.
const BRANCH_ANGLE_MAX: f32 = 1.2;

/// Start of each sub-flash, as a fraction of the strike duration.
const SUB_FLASH_STARTS: [f32; 4] = [0.0, 0.12, 0.22, 0.35];
/// Relative height of each sub-flash before the per-strike jitter.
const SUB_FLASH_HEIGHTS: [f32; 4] = [1.0, 0.55, 0.8, 0.35];
/// Fast attack of a sub-flash, as a fraction of the strike duration.
const SUB_FLASH_ATTACK: f32 = 0.02;
/// Exponential decay time constant of a sub-flash, as a fraction of the duration.
const SUB_FLASH_DECAY: f32 = 0.08;
/// Lower bound of the per-strike sub-flash height jitter.
const SUB_FLASH_JITTER_MIN: f32 = 0.7;
/// Upper bound of the per-strike sub-flash height jitter.
const SUB_FLASH_JITTER_MAX: f32 = 1.0;

/// Halo stroke width as a multiple of the core thickness.
const HALO_WIDTH: f32 = 6.0;
/// Halo stroke alpha at a full envelope.
const HALO_ALPHA: f32 = 0.10;
/// Mid stroke width as a multiple of the core thickness.
const MID_WIDTH: f32 = 2.5;
/// Mid stroke alpha at a full envelope.
const MID_ALPHA: f32 = 0.35;

/// Random spread of the ambient interval: ±50 %.
const AMBIENT_JITTER: f32 = 0.5;

/// Default bolt colour: a cold white-blue.
pub(super) const DEFAULT_COLOR: Color32 = Color32::from_rgb(190, 215, 255);
/// Default number of bolts per strike.
pub(super) const DEFAULT_BOLT_COUNT: usize = 1;
/// Default branch recursion depth.
pub(super) const DEFAULT_BRANCH_DEPTH: u32 = 2;
/// Default core stroke thickness in pixels.
pub(super) const DEFAULT_THICKNESS: f32 = 2.0;
/// Default midpoint-displacement jitter, as a fraction of segment length.
pub(super) const DEFAULT_JITTER: f32 = 0.18;
/// Default peak alpha of the full-area flash.
pub(super) const DEFAULT_FLASH_INTENSITY: f32 = 0.35;
/// Default strike duration in milliseconds.
pub(super) const DEFAULT_DURATION_MS: f32 = 420.0;
/// Default ambient interval in seconds (`0.0` disables the scheduler).
pub(super) const DEFAULT_AMBIENT_INTERVAL_SECS: f32 = 0.0;

/// Immutable knobs needed to build one strike's geometry and timing.
#[derive(Clone, Copy)]
struct StrikeConfig {
   bolt_count: usize,
   branch_depth: u32,
   jitter: f32,
   duration: f32,
}

/// Per-frame state of the [`Lightning`] effect, kept in egui's temporary data storage.
#[derive(Clone)]
struct LightningState {
   /// Drives the ambient schedule (the strike geometry has its own seed).
   rng: ParticleRng,
   /// Strikes currently animating.
   strikes: Vec<ActiveStrike>,
   /// Monotonic counter mixed into each strike's seed.
   strike_index: u64,
   /// Ambient cursor in `ctx` time; `None` until the scheduler first runs.
   next_strike_at: Option<f64>,
}

impl LightningState {
   fn new(seed: u64) -> Self {
      Self {
         rng: ParticleRng::new(seed),
         strikes: Vec::new(),
         strike_index: 0,
         next_strike_at: None,
      }
   }
}

/// One bolt strike: its geometry, age and flicker schedule.
#[derive(Clone, Debug)]
struct ActiveStrike {
   /// Every polyline of the bolt (trunk first, then branches), normalized.
   polylines: Vec<Vec<Pos2>>,
   /// Seconds since the strike was queued.
   age: f32,
   /// Total duration in seconds.
   duration: f32,
   /// Flicker pulses making up the flash envelope.
   sub_flashes: Vec<SubFlash>,
}

impl ActiveStrike {
   /// Builds a strike from `config`, drawing all randomness from `rng`.
   fn new(rng: &mut ParticleRng, config: StrikeConfig) -> Self {
      let mut polylines = Vec::new();
      for _ in 0..config.bolt_count {
         let from = Pos2::new(rng.range(BOLT_X_MIN, BOLT_X_MAX), 0.0);
         let to = Pos2::new(rng.range(BOLT_X_MIN, BOLT_X_MAX), 1.0);
         polylines.extend(generate_bolt_with_branches(
            rng,
            from,
            to,
            BOLT_LEVELS,
            config.jitter,
            config.branch_depth,
         ));
      }

      Self {
         polylines,
         age: 0.0,
         duration: config.duration,
         sub_flashes: SubFlash::pulses(rng),
      }
   }
}

/// One flicker pulse of a strike's flash.
#[derive(Clone, Copy, Debug)]
struct SubFlash {
   /// Start of the pulse, as a fraction of the strike duration.
   start: f32,
   /// Peak brightness of the pulse in `0.0..=1.0`.
   height: f32,
}

impl SubFlash {
   /// The named pulses for one strike, with jittered heights.
   fn pulses(rng: &mut ParticleRng) -> Vec<Self> {
      SUB_FLASH_STARTS
         .iter()
         .zip(SUB_FLASH_HEIGHTS)
         .map(|(&start, height)| Self {
            start,
            height: (height * rng.range(SUB_FLASH_JITTER_MIN, SUB_FLASH_JITTER_MAX))
               .clamp(0.0, 1.0),
         })
         .collect()
   }

   /// Pulse value at `u = age / duration`: fast attack, exponential decay.
   fn value(self, u: f32) -> f32 {
      if u <= self.start {
         0.0
      } else if u < self.start + SUB_FLASH_ATTACK {
         (u - self.start) / SUB_FLASH_ATTACK
      } else {
         (-(u - self.start - SUB_FLASH_ATTACK) / SUB_FLASH_DECAY).exp()
      }
   }
}

/// A configurable lightning effect for egui.
///
/// `Lightning` paints a jagged bolt plus a full-area flash — the "room lighting
/// up" — on an egui layer. It maintains its state between frames using egui's
/// temporary data storage and never disturbs the host layout.
///
/// A strike is episodic: it lives for [`duration_ms`](Self::duration_ms) and
/// flickers through several sub-flashes. Queue one immediately with
/// [`strike`](Self::strike) (the tuned host-triggered path) or let the ambient
/// scheduler fire on its own through [`ambient_interval_secs`](Self::ambient_interval_secs).
#[derive(Debug)]
pub struct Lightning {
   id: Id,
   color: Color32,
   bolt_count: usize,
   branch_depth: u32,
   thickness: f32,
   jitter: f32,
   flash_intensity: f32,
   duration: f32,
   ambient_interval: f32,
   layer_order: Order,
   custom_layer: Option<LayerId>,
   custom_area: Option<Rect>,
}

impl Lightning {
   /// Creates a new lightning effect with a unique ID.
   pub fn new(id_source: impl std::hash::Hash + std::fmt::Debug) -> Self {
      Self {
         id: Id::new(id_source),
         color: DEFAULT_COLOR,
         bolt_count: DEFAULT_BOLT_COUNT,
         branch_depth: DEFAULT_BRANCH_DEPTH,
         thickness: DEFAULT_THICKNESS,
         jitter: DEFAULT_JITTER,
         flash_intensity: DEFAULT_FLASH_INTENSITY,
         duration: DEFAULT_DURATION_MS / 1000.0,
         ambient_interval: DEFAULT_AMBIENT_INTERVAL_SECS,
         layer_order: Order::Foreground,
         custom_layer: None,
         custom_area: None,
      }
   }

   /// Builds a lightning effect from [`LightningSettings`].
   ///
   /// Use the same `id_source` for [`show`](Self::show) and
   /// [`strike`](Self::strike) so both address one state.
   pub fn from_settings(
      id_source: impl std::hash::Hash + std::fmt::Debug,
      settings: &LightningSettings,
   ) -> Self {
      let lightning = Self::new(id_source)
         .color(settings.color)
         .bolt_count(settings.bolt_count)
         .branch_depth(settings.branch_depth)
         .thickness(settings.thickness)
         .jitter(settings.jitter)
         .flash_intensity(settings.flash_intensity)
         .duration_ms(settings.duration_ms)
         .ambient_interval_secs(settings.ambient_interval_secs);

      match settings.layer {
         ParticleLayer::Background => lightning.on_background(),
         ParticleLayer::Foreground => lightning.on_foreground(),
         ParticleLayer::Top => lightning.on_top(),
      }
   }

   /// Sets the bolt and flash color.
   pub fn color(mut self, color: Color32) -> Self {
      self.color = color;
      self
   }

   /// Sets how many bolts a strike draws.
   pub fn bolt_count(mut self, bolt_count: usize) -> Self {
      self.bolt_count = bolt_count;
      self
   }

   /// Sets the branch recursion depth (`0` draws the trunk only).
   pub fn branch_depth(mut self, branch_depth: u32) -> Self {
      self.branch_depth = branch_depth;
      self
   }

   /// Sets the bright core stroke thickness in pixels.
   pub fn thickness(mut self, thickness: f32) -> Self {
      self.thickness = thickness.max(0.0);
      self
   }

   /// Sets the midpoint-displacement jitter as a fraction of segment length.
   pub fn jitter(mut self, jitter: f32) -> Self {
      self.jitter = jitter.max(0.0);
      self
   }

   /// Sets the peak alpha of the full-area flash in `0.0..=1.0`.
   pub fn flash_intensity(mut self, intensity: f32) -> Self {
      self.flash_intensity = intensity.clamp(0.0, 1.0);
      self
   }

   /// Sets the strike duration in milliseconds.
   pub fn duration_ms(mut self, duration_ms: f32) -> Self {
      self.duration = (duration_ms / 1000.0).max(0.0);
      self
   }

   /// Sets the ambient interval in seconds (`0.0` disables the scheduler).
   pub fn ambient_interval_secs(mut self, secs: f32) -> Self {
      self.ambient_interval = secs.max(0.0);
      self
   }

   /// Places the lightning on the Debug layer (on top of everything).
   pub fn on_top(mut self) -> Self {
      self.layer_order = Order::Debug;
      self
   }

   /// Places the lightning on the Foreground layer.
   pub fn on_foreground(mut self) -> Self {
      self.layer_order = Order::Foreground;
      self
   }

   /// Places the lightning on the Background layer.
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

   /// Restricts the lightning effect to a specific area.
   ///
   /// If not set, it defaults to `ctx.content_rect()`.
   pub fn area(mut self, area: Rect) -> Self {
      self.custom_area = Some(area);
      self
   }

   /// Queues a strike immediately and requests a repaint.
   ///
   /// This is the host-triggered path; `Lightning::from_settings(id, &settings).strike(ctx)`
   /// is the tuned one. `strike` uses the same `id_source` as [`show`](Self::show),
   /// so both must be constructed with the same id for the queued strike to be
   /// painted. A degenerate area is a no-op.
   pub fn strike(self, ctx: &Context) {
      if resolve_target(
         ctx,
         self.id,
         self.layer_order,
         self.custom_layer,
         self.custom_area,
      )
      .is_none()
      {
         return;
      }

      let id = self.id;
      let config = self.strike_config();
      ctx.data_mut(|data| {
         let state = data.get_temp_mut_or_insert_with(id, || LightningState::new(id.value()));
         queue_strike(
            &mut state.strikes,
            &mut state.strike_index,
            id,
            config,
         );
      });
      ctx.request_repaint();
   }

   /// Updates and renders the lightning effect.
   ///
   /// Call this every frame. While a strike animates it requests a repaint;
   /// when idle it requests nothing (ambient off) or wakes exactly for the next
   /// ambient strike, so an idle effect costs the host no frames.
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
      let now = ctx.input(|i| i.time);
      let id = self.id;
      let config = self.strike_config();

      let (mut strikes, mut rng, mut strike_index, mut next_strike_at) = ctx.data_mut(|data| {
         let state = data.get_temp_mut_or_insert_with(id, || LightningState::new(id.value()));
         (
            std::mem::take(&mut state.strikes),
            state.rng,
            state.strike_index,
            state.next_strike_at,
         )
      });

      // Advance and retire the active strikes.
      for strike in &mut strikes {
         strike.age += dt;
      }
      strikes.retain(|strike| strike.age < strike.duration);

      // Ambient scheduler: wake exactly when the next strike is due.
      let mut ambient_delay = None;
      if self.ambient_interval > 0.0 {
         let interval = f64::from(self.ambient_interval);
         if next_strike_at.is_none() {
            next_strike_at = Some(now + jittered_interval(&mut rng, interval));
         }
         if next_strike_at.is_some_and(|at| now >= at) {
            queue_strike(&mut strikes, &mut strike_index, id, config);
            next_strike_at = Some(now + jittered_interval(&mut rng, interval));
         }
         if let Some(at) = next_strike_at {
            ambient_delay = Some(Duration::from_secs_f64((at - now).max(0.0)));
         }
      }

      // The flash goes down first so the bolt reads as being lit by it.
      let painter = ctx.layer_painter(layer_id);
      for strike in &strikes {
         let envelope = flash_envelope(strike.age, strike.duration, &strike.sub_flashes);
         if envelope <= 0.0 {
            continue;
         }

         let flash_alpha = (self.flash_intensity * envelope).clamp(0.0, 1.0);
         painter.add(Shape::rect_filled(
            area,
            0.0,
            self.color.gamma_multiply(flash_alpha),
         ));

         let halo_color = self.color.gamma_multiply((HALO_ALPHA * envelope).clamp(0.0, 1.0));
         let mid_color = self.color.gamma_multiply((MID_ALPHA * envelope).clamp(0.0, 1.0));
         let core_color = self.color.gamma_multiply(envelope.clamp(0.0, 1.0));

         for polyline in &strike.polylines {
            let points: Vec<Pos2> = polyline.iter().map(|p| to_pixels(*p, area)).collect();
            painter.add(Shape::line(
               points.clone(),
               Stroke::new(self.thickness * HALO_WIDTH, halo_color),
            ));
            painter.add(Shape::line(
               points.clone(),
               Stroke::new(self.thickness * MID_WIDTH, mid_color),
            ));
            painter.add(Shape::line(
               points,
               Stroke::new(self.thickness, core_color),
            ));
         }
      }

      let has_active = !strikes.is_empty();

      ctx.data_mut(|data| {
         let state = data.get_temp_mut_or_insert_with(id, || LightningState::new(id.value()));
         state.strikes = strikes;
         state.rng = rng;
         state.strike_index = strike_index;
         state.next_strike_at = next_strike_at;
      });

      // Repaint policy: animate while a strike is live, wake exactly for the
      // next ambient strike, and request nothing at all when idle.
      if has_active {
         ctx.request_repaint();
      } else if let Some(delay) = ambient_delay {
         ctx.request_repaint_after(delay);
      }
   }

   /// The immutable per-strike knobs, with the branch depth bounded.
   fn strike_config(&self) -> StrikeConfig {
      StrikeConfig {
         bolt_count: self.bolt_count,
         branch_depth: self.branch_depth.min(MAX_BRANCH_DEPTH),
         jitter: self.jitter,
         duration: self.duration,
      }
   }
}

/// Queues one strike, seeding its geometry from `id ^ strike_index`.
fn queue_strike(
   strikes: &mut Vec<ActiveStrike>,
   strike_index: &mut u64,
   id: Id,
   config: StrikeConfig,
) {
   let seed = id.value() ^ *strike_index;
   *strike_index = strike_index.wrapping_add(1);
   let mut rng = ParticleRng::new(seed);
   strikes.push(ActiveStrike::new(&mut rng, config));
}

/// The next ambient interval, jittered by ±[`AMBIENT_JITTER`].
fn jittered_interval(rng: &mut ParticleRng, interval: f64) -> f64 {
   let spread = 1.0 + f64::from(rng.range(-AMBIENT_JITTER, AMBIENT_JITTER));
   (interval * spread).max(0.0)
}

/// The flash envelope over `age` for a strike of `duration`.
///
/// Sums the strike's sub-flashes and clamps to `0.0..=1.0`; exactly `0` at both
/// ends so nothing pops in or out.
fn flash_envelope(age: f32, duration: f32, sub_flashes: &[SubFlash]) -> f32 {
   if duration <= 0.0 {
      return 0.0;
   }

   let u = (age / duration).clamp(0.0, 1.0);
   if u <= 0.0 || u >= 1.0 {
      return 0.0;
   }

   let sum: f32 = sub_flashes.iter().map(|pulse| pulse.height * pulse.value(u)).sum();
   sum.clamp(0.0, 1.0)
}

/// Maps a normalized point into `area` pixels.
fn to_pixels(point: Pos2, area: Rect) -> Pos2 {
   Pos2::new(
      area.min.x + point.x * area.width(),
      area.min.y + point.y * area.height(),
   )
}

/// Recursive midpoint displacement from `from` to `to`.
///
/// Each level splits every segment and pushes its midpoint perpendicular to the
/// segment by `jitter * segment_length`; the displacement therefore halves each
/// level because the segment length does. Points are clamped into `0.0..=1.0`
/// on both axes so no bolt or branch can leave the effect area.
fn generate_bolt(
   rng: &mut ParticleRng,
   from: Pos2,
   to: Pos2,
   levels: u32,
   jitter: f32,
) -> Vec<Pos2> {
   let mut points = vec![from, to];

   for _ in 0..levels {
      let mut next = Vec::with_capacity(points.len() * 2 - 1);
      for pair in points.windows(2) {
         let a = pair[0];
         let b = pair[1];
         next.push(a);

         let delta = b - a;
         let length = delta.length();
         let midpoint = a + delta * 0.5;
         if length > 0.0 {
            let perpendicular = vec2(-delta.y, delta.x) / length;
            let amplitude = jitter * length;
            next.push(midpoint + perpendicular * rng.range(-amplitude, amplitude));
         } else {
            next.push(midpoint);
         }
      }
      next.push(*points.last().expect("a bolt has at least two points"));
      points = next;
   }

   for point in &mut points {
      point.x = point.x.clamp(0.0, 1.0);
      point.y = point.y.clamp(0.0, 1.0);
   }

   points
}

/// The trunk plus every branch, all as normalized polylines.
fn generate_bolt_with_branches(
   rng: &mut ParticleRng,
   from: Pos2,
   to: Pos2,
   levels: u32,
   jitter: f32,
   branch_depth: u32,
) -> Vec<Vec<Pos2>> {
   let trunk = generate_bolt(rng, from, to, levels, jitter);
   let mut polylines = vec![trunk.clone()];
   spawn_branches(
      rng,
      &trunk,
      levels,
      jitter,
      branch_depth,
      &mut polylines,
   );
   polylines
}

/// Recursively forks a branch off `parent`, bounded by `remaining`.
fn spawn_branches(
   rng: &mut ParticleRng,
   parent: &[Pos2],
   levels: u32,
   jitter: f32,
   remaining: u32,
   out: &mut Vec<Vec<Pos2>>,
) {
   if remaining == 0 || parent.len() < 2 {
      return;
   }

   let branch = spawn_branch(rng, parent, levels, jitter);
   out.push(branch.clone());
   spawn_branches(
      rng,
      &branch,
      levels.saturating_sub(1),
      jitter,
      remaining - 1,
      out,
   );
}

/// One branch: shorter than its parent, angled away and jittered.
fn spawn_branch(rng: &mut ParticleRng, parent: &[Pos2], levels: u32, jitter: f32) -> Vec<Pos2> {
   let from = parent[0];
   let to = parent[parent.len() - 1];
   let length = (to - from).length();
   let direction = if length > 0.0 {
      (to - from) / length
   } else {
      vec2(0.0, 1.0)
   };

   // Attach somewhere along the lower part of the parent.
   let lower = (parent.len() as f32 * BRANCH_START_MIN) as usize;
   let upper = (parent.len() as f32 * BRANCH_START_MAX) as usize;
   let index = (rng.range(lower as f32, (upper + 1) as f32) as usize).min(parent.len() - 1);
   let start = parent[index];

   let sign = if rng.f32() < 0.5 { -1.0 } else { 1.0 };
   let angle = sign * rng.range(BRANCH_ANGLE_MIN, BRANCH_ANGLE_MAX);
   let branch_length = length * rng.range(BRANCH_LENGTH_MIN, BRANCH_LENGTH_MAX);
   let end = start + rotate(direction, angle) * branch_length;

   generate_bolt(
      rng,
      start,
      end,
      levels.saturating_sub(1).max(1),
      jitter,
   )
}

/// Rotates `v` by `angle` radians.
fn rotate(v: Vec2, angle: f32) -> Vec2 {
   let (sin, cos) = angle.sin_cos();
   vec2(v.x * cos - v.y * sin, v.x * sin + v.y * cos)
}

#[cfg(test)]
mod tests {
   use super::*;
   use egui::__run_test_ctx;
   use egui::{ViewportId, vec2};

   /// The named pulses without the per-strike height jitter.
   fn sample_sub_flashes() -> Vec<SubFlash> {
      SUB_FLASH_STARTS
         .iter()
         .zip(SUB_FLASH_HEIGHTS)
         .map(|(&start, height)| SubFlash { start, height })
         .collect()
   }

   /// Perpendicular distance from `point` to the line through `from` and `to`.
   fn distance_to_line(point: Pos2, from: Pos2, to: Pos2) -> f32 {
      let delta = to - from;
      let length = delta.length();
      if length <= 0.0 {
         return (point - from).length();
      }
      ((point.x - from.x) * delta.y - (point.y - from.y) * delta.x).abs() / length
   }

   #[test]
   fn same_seed_generates_identical_bolts() {
      let from = Pos2::new(0.3, 0.0);
      let to = Pos2::new(0.7, 1.0);
      let mut a = ParticleRng::new(42);
      let mut b = ParticleRng::new(42);
      assert_eq!(
         generate_bolt(&mut a, from, to, BOLT_LEVELS, DEFAULT_JITTER),
         generate_bolt(&mut b, from, to, BOLT_LEVELS, DEFAULT_JITTER)
      );
   }

   #[test]
   fn different_seeds_generate_different_bolts() {
      let from = Pos2::new(0.3, 0.0);
      let to = Pos2::new(0.7, 1.0);
      let mut a = ParticleRng::new(1);
      let mut b = ParticleRng::new(2);
      assert_ne!(
         generate_bolt(&mut a, from, to, BOLT_LEVELS, DEFAULT_JITTER),
         generate_bolt(&mut b, from, to, BOLT_LEVELS, DEFAULT_JITTER)
      );
   }

   #[test]
   fn every_bolt_point_stays_normalized() {
      let mut rng = ParticleRng::new(7);
      let polylines = generate_bolt_with_branches(
         &mut rng,
         Pos2::new(0.5, 0.0),
         Pos2::new(0.5, 1.0),
         BOLT_LEVELS,
         0.3,
         3,
      );

      for polyline in &polylines {
         for point in polyline {
            assert!(
               (0.0..=1.0).contains(&point.x),
               "x {} left the normalized area",
               point.x
            );
            assert!(
               (0.0..=1.0).contains(&point.y),
               "y {} left the normalized area",
               point.y
            );
         }
      }
   }

   #[test]
   fn branch_count_is_bounded_by_depth() {
      let mut rng = ParticleRng::new(3);
      for depth in 0..=4u32 {
         let polylines = generate_bolt_with_branches(
            &mut rng,
            Pos2::new(0.5, 0.0),
            Pos2::new(0.5, 1.0),
            BOLT_LEVELS,
            DEFAULT_JITTER,
            depth,
         );
         assert!(
            polylines.len() <= 1 + depth as usize,
            "depth {depth} produced {} polylines",
            polylines.len()
         );
      }
   }

   #[test]
   fn bolt_is_jagged_not_a_straight_line() {
      let from = Pos2::new(0.5, 0.0);
      let to = Pos2::new(0.5, 1.0);
      let mut rng = ParticleRng::new(11);
      let bolt = generate_bolt(&mut rng, from, to, BOLT_LEVELS, DEFAULT_JITTER);

      let max_deviation = bolt
         .iter()
         .map(|point| distance_to_line(*point, from, to))
         .fold(0.0_f32, f32::max);
      assert!(
         max_deviation > 0.01,
         "the bolt deviated only {max_deviation} from the straight line"
      );
   }

   #[test]
   fn strike_with_degenerate_area_is_a_noop() {
      __run_test_ctx(|ctx| {
         let area = Rect::from_min_size(Pos2::ZERO, vec2(0.0, 64.0));
         Lightning::new("test_lightning_degenerate").area(area).strike(ctx);
         assert!(
            ctx.data(|d| d.get_temp::<LightningState>(Id::new("test_lightning_degenerate")))
               .is_none(),
            "a degenerate area must not create state"
         );
      });
   }

   #[test]
   fn strike_queues_a_bolt() {
      __run_test_ctx(|ctx| {
         let area = Rect::from_min_size(Pos2::ZERO, vec2(64.0, 64.0));
         Lightning::new("test_lightning_queue").area(area).strike(ctx);

         let state = ctx
            .data(|d| d.get_temp::<LightningState>(Id::new("test_lightning_queue")))
            .expect("LightningState should exist after strike");
         assert_eq!(state.strikes.len(), 1);
         assert!(!state.strikes[0].polylines.is_empty());
      });
   }

   #[test]
   fn envelope_is_zero_at_both_ends() {
      let pulses = sample_sub_flashes();
      assert_eq!(
         flash_envelope(0.0, 1.0, &pulses),
         0.0,
         "the envelope must start at zero"
      );
      assert_eq!(
         flash_envelope(1.0, 1.0, &pulses),
         0.0,
         "the envelope must end at zero"
      );
      assert_eq!(
         flash_envelope(1.0, 0.0, &pulses),
         0.0,
         "a zero-length strike must have a zero envelope"
      );
   }

   #[test]
   fn envelope_stays_within_bounds() {
      let pulses = sample_sub_flashes();
      for step in 0..=1000 {
         let age = step as f32 / 1000.0;
         let value = flash_envelope(age, 1.0, &pulses);
         assert!(
            (0.0..=1.0).contains(&value),
            "envelope {value} left 0.0..=1.0 at age {age}"
         );
      }
   }

   #[test]
   fn envelope_has_a_sub_flash_after_its_first_decay() {
      let pulses = sample_sub_flashes();
      let samples = 1000;
      let values: Vec<f32> = (0..=samples)
         .map(|i| flash_envelope(i as f32 / samples as f32, 1.0, &pulses))
         .collect();

      let first_max = (1..samples)
         .find(|&i| values[i] > values[i - 1] && values[i] > values[i + 1])
         .expect("the envelope must have an interior local maximum");

      let mut decayed = false;
      let mut later_max = false;
      for i in (first_max + 1)..samples {
         if values[i] < values[i - 1] {
            decayed = true;
         }
         if decayed && values[i] > values[i - 1] && values[i] > values[i + 1] {
            later_max = true;
            break;
         }
      }

      assert!(
         later_max,
         "the envelope has no sub-flash after its first decay"
      );
   }

   #[test]
   fn strike_ends_after_its_duration() {
      __run_test_ctx(|ctx| {
         let area = Rect::from_min_size(Pos2::ZERO, vec2(64.0, 64.0));
         let id = Id::new("test_lightning_ends");
         Lightning::new("test_lightning_ends").duration_ms(200.0).area(area).strike(ctx);

         assert_eq!(
            ctx.data(|d| d.get_temp::<LightningState>(id))
               .expect("state after strike")
               .strikes
               .len(),
            1
         );

         for frame in 0..8 {
            ctx.input_mut(|i| {
               i.stable_dt = 0.05;
               i.time = f64::from(frame) * 0.05;
            });
            Lightning::new("test_lightning_ends").duration_ms(200.0).area(area).show(ctx);
         }

         let state = ctx
            .data(|d| d.get_temp::<LightningState>(id))
            .expect("LightningState should exist after show");
         assert!(
            state.strikes.is_empty(),
            "a strike outlived its duration: {} still active",
            state.strikes.len()
         );
      });
   }

   #[test]
   fn idle_show_requests_no_repaint() {
      let ctx = Context::default();
      let area = Rect::from_min_size(Pos2::ZERO, vec2(64.0, 64.0));
      let raw_input = egui::RawInput {
         screen_rect: Some(area),
         ..Default::default()
      };

      // Two warm-up passes: the first creates the state and sends egui's
      // first-frame `SetTheme` viewport command, whose repaint request would
      // otherwise land on the pass under test.
      for _ in 0..2 {
         ctx.run_ui(raw_input.clone(), |ui| {
            Lightning::new("test_lightning_idle").area(area).show(ui.ctx());
         })
         .drop_without_applying_deltas();
      }

      let output = ctx.run_ui(raw_input, |ui| {
         Lightning::new("test_lightning_idle").area(area).show(ui.ctx());
      });
      let delay = output.viewport_output[&ViewportId::ROOT].repaint_delay;
      assert_eq!(
         delay,
         Duration::MAX,
         "an idle lightning effect requested a repaint after {delay:?}"
      );
   }

   #[test]
   fn from_settings_maps_fields_and_layer() {
      let settings = LightningSettings {
         color: Color32::from_rgb(1, 2, 3),
         bolt_count: 4,
         branch_depth: 5,
         thickness: 3.5,
         jitter: 0.4,
         flash_intensity: 0.6,
         duration_ms: 800.0,
         ambient_interval_secs: 3.0,
         layer: ParticleLayer::Top,
         ..Default::default()
      };

      let lightning = Lightning::from_settings("lightning_map", &settings);
      assert_eq!(lightning.color, Color32::from_rgb(1, 2, 3));
      assert_eq!(lightning.bolt_count, 4);
      assert_eq!(lightning.branch_depth, 5);
      assert_eq!(lightning.thickness, 3.5);
      assert_eq!(lightning.jitter, 0.4);
      assert_eq!(lightning.flash_intensity, 0.6);
      assert_eq!(lightning.duration, 0.8);
      assert_eq!(lightning.ambient_interval, 3.0);
      assert_eq!(lightning.layer_order, Order::Debug);

      let background = Lightning::from_settings(
         "lightning_map_bg",
         &LightningSettings {
            layer: ParticleLayer::Background,
            ..Default::default()
         },
      );
      assert_eq!(background.layer_order, Order::Background);
   }

   #[test]
   fn new_matches_default_settings() {
      let from_new = Lightning::new("lightning_defaults");
      let from_settings = Lightning::from_settings(
         "lightning_defaults",
         &LightningSettings::default(),
      );
      assert_eq!(from_new.color, from_settings.color);
      assert_eq!(from_new.bolt_count, from_settings.bolt_count);
      assert_eq!(from_new.branch_depth, from_settings.branch_depth);
      assert_eq!(from_new.thickness, from_settings.thickness);
      assert_eq!(from_new.jitter, from_settings.jitter);
      assert_eq!(
         from_new.flash_intensity,
         from_settings.flash_intensity
      );
      assert_eq!(from_new.duration, from_settings.duration);
      assert_eq!(
         from_new.ambient_interval,
         from_settings.ambient_interval
      );
      assert_eq!(from_new.layer_order, from_settings.layer_order);
   }
}
