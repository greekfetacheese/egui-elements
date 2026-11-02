//! Arc geometry for [`Spinner`](super::Spinner): one mesh that carries the arc
//! band, its tail-to-head alpha ramp and the glow that follows it.
//!
//! egui cannot stroke a path with per-vertex colour and it has no blur, so the
//! arc is not a stroked path plus a shadow: it is one triangle strip built from
//! six concentric rings around the arc's centreline — glow rim (transparent),
//! glow mid, the two band edges, glow mid, glow rim. Interpolating alpha across
//! those rings produces the soft fringe; interpolating it along the strip
//! produces the comet ramp. The mesh uses the font texture (white at
//! `WHITE_UV`), so it fades out instead of tinting, and the rim therefore stays
//! premultiplied zero.

use egui::epaint::Mesh;
use egui::{Color32, Pos2, TextureId, vec2};

/// Alpha at the arc's tail when the comet ramp is fully on.
pub(crate) const TAIL_ALPHA: f32 = 0.25;
/// Narrowest fringe, in points: the band's edges are always feathered, so the
/// arc stays anti-aliased with the glow turned off.
pub(crate) const FEATHER: f32 = 1.0;
/// Alpha of a mid glow ring, as a fraction of the glow's strength.
pub(crate) const GLOW_MID_ALPHA: f32 = 0.35;
/// Where a mid glow ring sits between the band edge and the fringe rim.
pub(crate) const GLOW_MID_RING: f32 = 0.45;
/// Rings one strip is built from: rim, mid, band edge, band edge, mid, rim.
const RINGS: usize = 6;
/// Ring indices that carry the arc colour instead of the glow colour.
const BAND: std::ops::RangeInclusive<usize> = 2..=3;
/// Bounds of the sample count along the arc.
const MIN_SAMPLES: usize = 8;
const MAX_SAMPLES: usize = 160;

/// One arc to paint, in screen points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Arc {
   /// Centre of the ring.
   pub center: Pos2,
   /// Centreline radius.
   pub radius: f32,
   /// Stroke width, centred on `radius`.
   pub thickness: f32,
   /// Start angle in radians; the arc may run backwards.
   pub start: f64,
   /// End angle in radians.
   pub end: f64,
}

impl Arc {
   /// Samples along the arc, proportional to its pixel length; one more than the
   /// segment count, so both ends are emitted.
   fn samples(&self) -> usize {
      let span = (self.end - self.start).abs() as f32;
      ((self.radius * span).round() as usize).clamp(MIN_SAMPLES, MAX_SAMPLES) + 1
   }
}

/// Ring offsets from the centreline (negative is inward), with their alphas.
fn rings(arc: &Arc, glow_width: f32) -> [(f32, f32); RINGS] {
   let half = (arc.thickness * 0.5).max(0.0);
   let fringe = glow_width.max(FEATHER);

   [
      (-(half + fringe), 0.0),
      (-(half + fringe * GLOW_MID_RING), GLOW_MID_ALPHA),
      (-half, 1.0),
      (half, 1.0),
      (half + fringe * GLOW_MID_RING, GLOW_MID_ALPHA),
      (half + fringe, 0.0),
   ]
}

/// Alpha multiplier at `t` (`0.0` = tail, `1.0` = head) of the comet ramp.
///
/// `comet = 0.0` is flat (every sample at `1.0`); `comet = 1.0` runs from
/// [`TAIL_ALPHA`] at the tail to `1.0` at the head.
pub(crate) fn comet_alpha(t: f32, comet: f32) -> f32 {
   let comet = comet.clamp(0.0, 1.0);
   (1.0 - comet) + comet * (TAIL_ALPHA + (1.0 - TAIL_ALPHA) * t.clamp(0.0, 1.0))
}

/// Builds the arc band, its comet ramp and its glow as one mesh.
///
/// `color` paints the two band rings and `glow` the four fringe rings, so a white
/// arc can carry a coloured halo. `glow_width` is the fringe's reach in points and
/// `glow_strength` its peak alpha; a strength of `0.0` leaves only the
/// [`FEATHER`] ring that keeps the band's edges anti-aliased.
pub(crate) fn arc_mesh(
   arc: &Arc,
   color: Color32,
   glow: Color32,
   glow_width: f32,
   glow_strength: f32,
   comet: f32,
) -> Mesh {
   let samples = arc.samples();
   let rings = rings(arc, glow_width);
   let glow_strength = glow_strength.clamp(0.0, 1.0);

   let mut mesh = Mesh::default();
   mesh.texture_id = TextureId::default();

   for i in 0..samples {
      let t = i as f64 / (samples - 1) as f64;
      let angle = arc.start + (arc.end - arc.start) * t;
      let (sin, cos) = angle.sin_cos();
      let dir = vec2(cos as f32, sin as f32);
      let ramp = comet_alpha(t as f32, comet);

      for (ring, (offset, ring_alpha)) in rings.iter().enumerate() {
         let band = BAND.contains(&ring);
         let ring_color = if band { color } else { glow };
         let scale = if band { 1.0 } else { glow_strength };
         let distance = (arc.radius + offset).max(0.0);

         mesh.colored_vertex(
            arc.center + dir * distance,
            ring_color.gamma_multiply(ring_alpha * scale * ramp),
         );
      }
   }

   let step = RINGS as u32;
   for segment in 0..(samples as u32 - 1) {
      for ring in 0..(step - 1) {
         let inner = segment * step + ring;
         let lower = inner + step;
         mesh.add_triangle(inner, lower, lower + 1);
         mesh.add_triangle(inner, lower + 1, inner + 1);
      }
   }

   mesh
}

#[cfg(test)]
mod tests {
   use super::*;

   const ACCENT: Color32 = Color32::from_rgb(11, 140, 255);
   const GLOW: Color32 = Color32::from_rgb(255, 0, 0);

   fn arc() -> Arc {
      Arc {
         center: Pos2::new(100.0, 100.0),
         radius: 20.0,
         thickness: 4.0,
         start: 0.0,
         end: std::f64::consts::PI,
      }
   }

   fn at(mesh: &Mesh, sample: usize, ring: usize) -> (Pos2, Color32) {
      let vertex = &mesh.vertices[sample * RINGS + ring];
      (vertex.pos, vertex.color)
   }

   #[test]
   fn band_edges_sit_a_half_thickness_either_side_of_the_centreline() {
      let arc = arc();
      let mesh = arc_mesh(&arc, ACCENT, GLOW, 0.0, 0.0, 0.0);
      let samples = arc.samples();

      let (inner, inner_color) = at(&mesh, 0, 2);
      let (outer, outer_color) = at(&mesh, samples - 1, 3);

      assert!(
         (inner.distance(arc.center) - 18.0).abs() < 0.01,
         "{inner:?}"
      );
      assert!(
         (outer.distance(arc.center) - 22.0).abs() < 0.01,
         "{outer:?}"
      );
      assert_eq!(inner_color, ACCENT);
      assert_eq!(outer_color, ACCENT);
   }

   #[test]
   fn the_fringe_falls_off_to_a_transparent_rim() {
      let arc = arc();
      let strength = 0.3;
      let mesh = arc_mesh(&arc, ACCENT, GLOW, 6.0, strength, 0.0);
      let samples = arc.samples();
      let mid = (255.0 * GLOW_MID_ALPHA * strength + 0.5) as u8;

      assert_eq!(at(&mesh, 0, 0).1.a(), 0, "inner rim");
      assert_eq!(at(&mesh, 0, 5).1.a(), 0, "outer rim");
      assert_eq!(
         at(&mesh, samples - 1, 1).1.a(),
         mid,
         "inner mid ring"
      );
      assert_eq!(
         at(&mesh, samples - 1, 4).1.a(),
         mid,
         "outer mid ring"
      );
      assert_eq!(at(&mesh, samples - 1, 2).1.a(), 255, "band");
   }

   #[test]
   fn the_fringe_uses_the_glow_colour() {
      let arc = arc();
      let mesh = arc_mesh(&arc, ACCENT, GLOW, 6.0, 1.0, 0.0);
      let samples = arc.samples();

      assert_eq!(at(&mesh, 0, 2).1, ACCENT);
      assert_eq!(
         at(&mesh, samples - 1, 4).1,
         GLOW.gamma_multiply(GLOW_MID_ALPHA)
      );
   }

   #[test]
   fn the_comet_ramp_runs_from_the_tail_alpha_to_full_at_the_head() {
      let arc = arc();
      let mesh = arc_mesh(&arc, ACCENT, ACCENT, 0.0, 0.0, 1.0);
      let samples = arc.samples();

      let tail = at(&mesh, 0, 2).1.a();
      let middle = at(&mesh, samples / 2, 2).1.a();
      let head = at(&mesh, samples - 1, 2).1.a();

      assert_eq!(tail, (255.0 * TAIL_ALPHA + 0.5) as u8);
      assert_eq!(head, 255);
      assert!(
         tail < middle && middle < head,
         "tail {tail} middle {middle} head {head}"
      );
   }

   #[test]
   fn comet_zero_keeps_every_band_vertex_opaque() {
      let arc = arc();
      let mesh = arc_mesh(&arc, ACCENT, ACCENT, 0.0, 0.0, 0.0);

      for sample in 0..arc.samples() {
         assert_eq!(at(&mesh, sample, 2).1.a(), 255);
         assert_eq!(at(&mesh, sample, 3).1.a(), 255);
      }
   }

   #[test]
   fn a_glowless_arc_still_feathers_its_edges() {
      let arc = arc();
      let mesh = arc_mesh(&arc, ACCENT, ACCENT, 0.0, 0.0, 0.0);
      let (rim, rim_color) = at(&mesh, 0, 5);

      assert!(
         (rim.distance(arc.center) - 23.0).abs() < 0.01,
         "{rim:?}"
      );
      assert_eq!(rim_color.a(), 0);
   }

   #[test]
   fn degenerate_arcs_stay_finite() {
      let cases = [
         Arc {
            start: 1.0,
            end: 1.0,
            ..arc()
         },
         Arc {
            radius: 0.0,
            ..arc()
         },
         Arc {
            thickness: 0.0,
            ..arc()
         },
      ];

      for arc in cases {
         let mesh = arc_mesh(&arc, ACCENT, ACCENT, 6.0, 0.3, 1.0);
         assert!(!mesh.vertices.is_empty());
         assert!(
            mesh
               .vertices
               .iter()
               .all(|vertex| vertex.pos.x.is_finite() && vertex.pos.y.is_finite())
         );
      }
   }
}
