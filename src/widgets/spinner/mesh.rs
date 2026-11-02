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

   fn arc() -> Arc {
      Arc {
         center: Pos2::new(100.0, 100.0),
         radius: 20.0,
         thickness: 4.0,
         start: 0.0,
         end: std::f64::consts::PI,
      }
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
