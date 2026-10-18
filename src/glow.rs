//! Shared radial-glow mesh primitive used by the particle effects and widgets.

use egui::epaint::Mesh;
use egui::{Color32, Pos2, TextureId, vec2};

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
/// Softness that reproduces the original ring profile.
const DEFAULT_SOFTNESS: f32 = 1.0;
/// Lower clamp of the softness knob, so a rogue value cannot collapse the fan.
const MIN_SOFTNESS: f32 = 0.05;
/// Exponent of the size-attenuation curve (peak alpha against mote radius).
const SIZE_ATTENUATION_POWER: f32 = 0.5;
/// Lower clamp of the size-attenuation curve.
const MIN_SIZE_ATTENUATION: f32 = 0.4;

/// Ring radii and alphas of the shared radial-glow fan.
///
/// Radii are fractions of the glow radius; alphas are fractions of the glow
/// colour. [`GlowProfile::DEFAULT`] is the profile every caller painted before
/// the profile became a parameter, and [`GlowProfile::softness`] reshapes it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GlowProfile {
   /// Hot-core ring radius as a fraction of the glow radius.
   core_ring: f32,
   /// Inner-ring radius as a fraction of the glow radius.
   inner_ring: f32,
   /// Outer-ring (rim) radius as a fraction of the glow radius.
   rim_ring: f32,
   /// Alpha of the hot-core ring, as a fraction of the glow colour.
   core_alpha: f32,
   /// Alpha of the inner ring, as a fraction of the glow colour.
   inner_alpha: f32,
}

impl GlowProfile {
   /// The shared fan's original profile: the hot core at [`CORE_RING`], the
   /// inner ring at [`INNER_RING`], the transparent rim at [`RIM_RING`], and
   /// the two ring alphas at [`CORE_ALPHA`] / [`INNER_ALPHA`].
   pub(crate) const DEFAULT: Self = Self {
      core_ring: CORE_RING,
      inner_ring: INNER_RING,
      rim_ring: RIM_RING,
      core_alpha: CORE_ALPHA,
      inner_alpha: INNER_ALPHA,
   };

   /// The default profile reshaped by `softness`.
   ///
   /// Each interior ring fraction `r` becomes `r.powf(1.0 / softness)`; the rim
   /// and both alphas are untouched. `softness > 1.0` pushes the hot core and
   /// inner ring outward, so the colour drops to the transparent rim over a
   /// longer run (a gentler falloff); `softness < 1.0` pulls them in for a
   /// tighter, harder core. Exactly [`DEFAULT_SOFTNESS`] returns
   /// [`Self::DEFAULT`], so `1.0` is vertex-for-vertex identical.
   pub(crate) fn softness(softness: f32) -> Self {
      if softness == DEFAULT_SOFTNESS {
         return Self::DEFAULT;
      }
      let exponent = softness.max(MIN_SOFTNESS).recip();
      Self {
         core_ring: CORE_RING.powf(exponent),
         inner_ring: INNER_RING.powf(exponent),
         ..Self::DEFAULT
      }
   }
}

/// Builds one radial glow as a triangle fan with a hot core and a long tail.
///
/// Shorthand for [`glow_mesh_with`] with [`GlowProfile::DEFAULT`]: the centre is
/// painted at full `color`, then three rings close the fan: a hot core at
/// `CORE_RING * radius` carrying `CORE_ALPHA` of the colour, an inner ring at
/// `INNER_RING * radius` carrying `INNER_ALPHA`, and a transparent rim at
/// `RIM_RING * radius`. The steep centre-to-core drop gives the point its hot
/// core and the long, faint core-to-rim tail gives it reach, which is what
/// reads as a light. A single linear two-ring fan instead fades at one constant
/// rate from centre to rim, and the eye reads that flat profile as a milky disc
/// rather than a glowing source.
///
/// The mesh uses the font texture (white at `WHITE_UV`), so the fan fades out
/// instead of tinting; the rim must stay premultiplied zero for that reason.
pub(crate) fn glow_mesh(center: Pos2, radius: f32, color: Color32) -> Mesh {
   glow_mesh_with(center, radius, color, GlowProfile::DEFAULT)
}

/// Builds one radial glow with a custom [`GlowProfile`].
///
/// [`glow_mesh`] is this function with [`GlowProfile::DEFAULT`]; see it for the
/// fan's shape and the texture contract.
pub(crate) fn glow_mesh_with(
   center: Pos2,
   radius: f32,
   color: Color32,
   profile: GlowProfile,
) -> Mesh {
   let segments = ((radius / 4.0).clamp(MIN_SEGMENTS as f32, MAX_SEGMENTS as f32)) as usize;
   let mut mesh = Mesh::default();
   mesh.texture_id = TextureId::default();

   mesh.colored_vertex(center, color);

   // Ring radii and alphas, innermost first; the rim stays fully transparent.
   let rings = [
      (
         profile.core_ring * radius,
         color.gamma_multiply(profile.core_alpha),
      ),
      (
         profile.inner_ring * radius,
         color.gamma_multiply(profile.inner_alpha),
      ),
      (profile.rim_ring * radius, Color32::TRANSPARENT),
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
pub(crate) fn size_attenuation(radius: f32, min_radius: f32, max_radius: f32) -> f32 {
   // Treat the smaller endpoint as the floor and never divide by zero, so an
   // inverted or zero-width range cannot produce a NaN alpha.
   let floor = min_radius.min(max_radius).max(f32::MIN_POSITIVE);
   (floor / radius.max(floor))
      .powf(SIZE_ATTENUATION_POWER)
      .clamp(MIN_SIZE_ATTENUATION, 1.0)
}
