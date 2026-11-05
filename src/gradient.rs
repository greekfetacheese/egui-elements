//! Two-stop linear gradients, and the [`Fill`] a themed rect is painted with.
//!
//! egui paints a rect with one colour: a `Shape::Rect` carries a single [`Color32`], so a gradient
//! has to go through a [`Mesh`]. The vertices carry the colours and the GPU interpolates between
//! them, which is exact for a linear ramp across a convex shape — and a rounded rect is convex.
//!
//! [`Fill`] is the fill slot of the themed visuals, so a widget never has to know whether it paints
//! a solid or a ramp. [`Gradient::bevel`] is the one derivation that turns a solid token into a
//! ramp, and it clamps both ends so the label ink keeps [`CONTRAST_AA_TEXT`].

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use egui::{
   Color32, CornerRadius, Mesh, Painter, Pos2, Rect, Response, Sense, Shape, Ui, Vec2, pos2,
};

use crate::utils::{CONTRAST_AA_TEXT, contrast_ratio, mix};

/// Anti-aliased rim drawn inside a gradient mesh, in points.
///
/// epaint feathers its own rects, so a mesh without a rim reads as a hard edge beside them.
const FEATHER: f32 = 1.0;

/// Lowest contrast [`Gradient::bevel`] leaves on either end of the ramp.
///
/// A ramp end is rounded to `u8` and then sampled by [`Gradient::sample`], so the clamp keeps a
/// little headroom instead of aiming at [`CONTRAST_AA_TEXT`] exactly.
const CLAMP_CONTRAST: f32 = CONTRAST_AA_TEXT + 0.05;

/// Samples [`Gradient::min_contrast`] measures along a ramp.
const CONTRAST_SAMPLES: usize = 16;

/// Bisection steps [`Gradient::bevel`] spends clamping one end of the ramp.
const CLAMP_STEPS: usize = 12;

/// Axis a [`Gradient`] ramps along, in screen space.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GradientDir {
   /// `from` at the top edge, `to` at the bottom.
   #[default]
   TopDown,
   /// `from` at the bottom edge, `to` at the top.
   BottomUp,
   /// `from` at the left edge, `to` at the right.
   LeftToRight,
   /// `from` at the right edge, `to` at the left.
   RightToLeft,
}

/// Two-stop linear gradient, painted as a vertex-coloured mesh.
///
/// Cardinal directions sample every vertex in the same space, so barycentric interpolation across
/// the rounded outline is an exact linear ramp — no banding, and the two stops are the only places a
/// colour is chosen.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Gradient {
   /// Colour at the start edge named by [`Self::dir`].
   pub from: Color32,
   /// Colour at the end edge named by [`Self::dir`].
   pub to: Color32,
   /// Axis the ramp runs along.
   pub dir: GradientDir,
}

impl Gradient {
   /// `from` at the top edge, `to` at the bottom.
   pub const fn new(from: Color32, to: Color32) -> Self {
      Self {
         from,
         to,
         dir: GradientDir::TopDown,
      }
   }

   /// Same colour everywhere: the ramp a solid fill becomes when it has to blend with a gradient.
   pub const fn flat(color: Color32) -> Self {
      Self {
         from: color,
         to: color,
         dir: GradientDir::TopDown,
      }
   }

   /// `from` at the top edge, `to` at the bottom.
   pub const fn vertical(from: Color32, to: Color32) -> Self {
      Self {
         from,
         to,
         dir: GradientDir::TopDown,
      }
   }

   /// `from` at the left edge, `to` at the right.
   pub const fn horizontal(from: Color32, to: Color32) -> Self {
      Self {
         from,
         to,
         dir: GradientDir::LeftToRight,
      }
   }

   pub const fn with_dir(self, dir: GradientDir) -> Self {
      Self {
         from: self.from,
         to: self.to,
         dir,
      }
   }

   /// Swap the two stops, keeping the direction.
   pub const fn reverse(self) -> Self {
      Self {
         from: self.to,
         to: self.from,
         dir: self.dir,
      }
   }

   /// `true` when both stops are the same colour, so the ramp paints a flat fill.
   pub fn is_flat(self) -> bool {
      self.from == self.to
   }

   /// Colour at `t` in `0..=1` along the gradient.
   ///
   /// Gamma-space interpolation, which is what epaint does with a mesh's vertex colours; sampling
   /// in linear light would shift every mid-tone.
   pub fn sample(self, t: f32) -> Color32 {
      self.from.lerp_to_gamma(self.to, t.clamp(0.0, 1.0))
   }

   /// `0` at the `from` edge, `1` at the `to` edge, for `pos` in `rect`.
   pub fn t_along(self, rect: Rect, pos: Pos2) -> f32 {
      let (span, offset) = match self.dir {
         GradientDir::TopDown => (rect.height(), pos.y - rect.top()),
         GradientDir::BottomUp => (rect.height(), rect.bottom() - pos.y),
         GradientDir::LeftToRight => (rect.width(), pos.x - rect.left()),
         GradientDir::RightToLeft => (rect.width(), rect.right() - pos.x),
      };

      if span <= 0.0 {
         return 0.0;
      }

      (offset / span).clamp(0.0, 1.0)
   }

   /// [`Self::sample`] at `pos` in `rect`.
   pub fn sample_at(self, rect: Rect, pos: Pos2) -> Color32 {
      self.sample(self.t_along(rect, pos))
   }

   /// A two-stop bevel of `base`, for painting a solid token as a gradient.
   ///
   /// One end is lifted toward white and the other dropped toward black, each by at most
   /// `strength`, and each *clamped* by contrast: a stop stops moving as soon as `ink` would drop
   /// under [`CONTRAST_AA_TEXT`]. A fill with no headroom therefore stays flat instead of turning
   /// its label unreadable, which is what makes the themed gradient variant safe on every palette.
   ///
   /// `strength` is the fraction of the way to the extreme (`0.1` is a soft sheen, `1.0` a full
   /// white-to-black ramp); a transparent `base` stays transparent.
   pub fn bevel(base: Color32, ink: Color32, strength: f32) -> Self {
      let strength = strength.clamp(0.0, 1.0);

      if base.a() == 0 || strength <= 0.0 {
         return Self::flat(base);
      }

      let light = clamp_reach(base, ink, Color32::WHITE, strength);
      let dark = clamp_reach(base, ink, Color32::BLACK, strength);

      Self {
         from: mix(base, Color32::WHITE, light),
         to: mix(base, Color32::BLACK, dark),
         dir: GradientDir::TopDown,
      }
   }

   /// Worst contrast the ramp reaches against `ink`, sampled along it.
   ///
   /// Stops are expected opaque: [`contrast_ratio`] ignores alpha, so composite a translucent fill
   /// over its surface first.
   pub fn min_contrast(self, ink: Color32) -> f32 {
      (0..=CONTRAST_SAMPLES)
         .map(|step| {
            let t = step as f32 / CONTRAST_SAMPLES as f32;
            contrast_ratio(ink, self.sample(t))
         })
         .fold(f32::INFINITY, f32::min)
   }

   /// Sharp rectangle (no corner radius).
   pub fn mesh(self, rect: Rect) -> Mesh {
      self.mesh_rounded(rect, CornerRadius::ZERO)
   }

   /// Convex rounded rectangle, vertex-coloured by [`Self::sample_at`].
   ///
   /// Two rings: an inner outline sampled in the gradient and an outer outline left transparent,
   /// whose triangles are the anti-aliased rim. One full fan inside, so a flat gradient costs the
   /// same as any other.
   pub fn mesh_rounded(self, rect: Rect, corner_radius: impl Into<CornerRadius>) -> Mesh {
      let mut mesh = Mesh::default();

      if rect.width() <= 0.0 || rect.height() <= 0.0 {
         return mesh;
      }

      let corner_radius = corner_radius.into();
      let outer = outline(rect, corner_radius);
      let feather = FEATHER.min(rect.width() * 0.5).min(rect.height() * 0.5);
      let inner_rect = rect.shrink(feather);

      if feather <= 0.0 || inner_rect.width() <= 0.0 || inner_rect.height() <= 0.0 {
         for point in &outer {
            mesh.colored_vertex(*point, self.sample_at(rect, *point));
         }

         add_fan(&mut mesh, 0, outer.len());

         return mesh;
      }

      // The inset ring shares the outer ring's corner steps, so vertex `i` on one lines up with
      // vertex `i` on the other and the rim triangles never cross.
      let inner = outline(inner_rect, inset_corners(corner_radius, feather));

      for point in &inner {
         mesh.colored_vertex(*point, self.sample_at(rect, *point));
      }

      let inner_count = inner.len() as u32;

      for point in &outer {
         mesh.colored_vertex(*point, Color32::TRANSPARENT);
      }

      add_fan(&mut mesh, 0, inner.len());

      for index in 0..inner_count {
         let next = (index + 1) % inner_count;
         mesh.add_triangle(index, next, inner_count + index);
         mesh.add_triangle(inner_count + index, next, inner_count + next);
      }

      mesh
   }

   pub fn shape_rounded(self, rect: Rect, corner_radius: impl Into<CornerRadius>) -> Shape {
      Shape::mesh(self.mesh_rounded(rect, corner_radius))
   }

   pub fn paint_rounded(
      self,
      painter: &Painter,
      rect: Rect,
      corner_radius: impl Into<CornerRadius>,
   ) {
      if rect.width() <= 0.0 || rect.height() <= 0.0 {
         return;
      }

      painter.add(self.shape_rounded(rect, corner_radius));
   }
}

/// Solid colour or a two-stop gradient.
///
/// This is what the themed visuals store for a fill, so a `.visuals(...)` override can hand a widget
/// either one. `Fill::from(Color32)` keeps the solid spelling short.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(
   feature = "serde",
   derive(serde::Serialize, serde::Deserialize),
   serde(untagged)
)]
pub enum Fill {
   /// One flat colour.
   Solid(Color32),
   /// A two-stop linear ramp.
   Gradient(Gradient),
}

impl Fill {
   pub const fn solid(color: Color32) -> Self {
      Self::Solid(color)
   }

   /// Two-stop vertical ramp (`from` at the top).
   pub const fn vertical(from: Color32, to: Color32) -> Self {
      Self::Gradient(Gradient::vertical(from, to))
   }

   /// Two-stop horizontal ramp (`from` at the left).
   pub const fn horizontal(from: Color32, to: Color32) -> Self {
      Self::Gradient(Gradient::horizontal(from, to))
   }

   /// A bevel of `base` — see [`Gradient::bevel`].
   ///
   /// Collapses to [`Fill::Solid`] when the ramp has no room to move, so a tight fill is painted as
   /// a plain rect instead of a mesh that would look identical.
   pub fn bevel(base: Color32, ink: Color32, strength: f32) -> Self {
      let gradient = Gradient::bevel(base, ink, strength);

      if gradient.is_flat() {
         Self::Solid(base)
      } else {
         Self::Gradient(gradient)
      }
   }

   /// The ramp this fill paints; a solid becomes a flat one.
   pub fn gradient(self) -> Gradient {
      match self {
         Self::Solid(color) => Gradient::flat(color),
         Self::Gradient(gradient) => gradient,
      }
   }

   /// `true` for a solid fill that paints nothing.
   pub fn is_transparent(self) -> bool {
      matches!(self, Self::Solid(color) if color.a() == 0)
   }

   /// The solid colour this fill stands in for, for paths that cannot paint a mesh (an
   /// `egui::Frame` fill, a popup surface).
   ///
   /// A gradient collapses to the midpoint of its ramp, which is the colour a reader perceives.
   pub fn to_solid(self) -> Color32 {
      match self {
         Self::Solid(color) => color,
         Self::Gradient(gradient) => mix(gradient.from, gradient.to, 0.5),
      }
   }

   /// Worst contrast against `ink` — see [`Gradient::min_contrast`].
   pub fn min_contrast(self, ink: Color32) -> f32 {
      match self {
         Self::Solid(color) => contrast_ratio(ink, color),
         Self::Gradient(gradient) => gradient.min_contrast(ink),
      }
   }

   pub fn gamma_multiply(self, factor: f32) -> Self {
      match self {
         Self::Solid(color) => Self::Solid(color.gamma_multiply(factor)),
         Self::Gradient(gradient) => Self::Gradient(Gradient {
            from: gradient.from.gamma_multiply(factor),
            to: gradient.to.gamma_multiply(factor),
            dir: gradient.dir,
         }),
      }
   }

   /// Replace `old` with `new` in whichever stop still carries it.
   ///
   /// A palette switch rewrites the visuals slots that still match the old palette; this is the fill
   /// half of that, so a derived ramp follows its slot.
   pub fn remap_color(&mut self, old: Color32, new: Color32) {
      match self {
         Self::Solid(color) => {
            if *color == old {
               *color = new;
            }
         }
         Self::Gradient(gradient) => {
            if gradient.from == old {
               gradient.from = new;
            }

            if gradient.to == old {
               gradient.to = new;
            }
         }
      }
   }

   pub fn shape_rounded(self, rect: Rect, corner_radius: impl Into<CornerRadius>) -> Shape {
      match self {
         Self::Solid(color) => Shape::rect_filled(rect, corner_radius, color),
         Self::Gradient(gradient) => gradient.shape_rounded(rect, corner_radius),
      }
   }

   pub fn paint_rounded(
      self,
      painter: &Painter,
      rect: Rect,
      corner_radius: impl Into<CornerRadius>,
   ) {
      if rect.width() <= 0.0 || rect.height() <= 0.0 {
         return;
      }

      match self {
         Self::Solid(color) => {
            painter.rect_filled(rect, corner_radius, color);
         }
         Self::Gradient(gradient) => gradient.paint_rounded(painter, rect, corner_radius),
      }
   }

   /// Allocate `size` and paint it. Hover-sense only, for swatches.
   pub fn show(self, ui: &mut Ui, size: Vec2) -> Response {
      self.show_rounded(ui, size, CornerRadius::ZERO)
   }

   /// Allocate `size` and paint it with `corner_radius`. Hover-sense only.
   pub fn show_rounded(
      self,
      ui: &mut Ui,
      size: Vec2,
      corner_radius: impl Into<CornerRadius>,
   ) -> Response {
      let corner_radius = corner_radius.into();
      let (rect, response) = ui.allocate_exact_size(size, Sense::hover());

      if ui.is_rect_visible(rect) {
         self.paint_rounded(ui.painter(), rect, corner_radius);
      }

      response
   }
}

impl Default for Fill {
   fn default() -> Self {
      Self::Solid(Color32::TRANSPARENT)
   }
}

impl From<Color32> for Fill {
   fn from(color: Color32) -> Self {
      Self::Solid(color)
   }
}

impl From<Gradient> for Fill {
   fn from(gradient: Gradient) -> Self {
      Self::Gradient(gradient)
   }
}

/// Largest share of the way from `base` to `extreme` that keeps `ink` at [`CONTRAST_AA_TEXT`].
///
/// [`mix`] moves every channel monotonically, so contrast against a fixed ink is monotonic in the
/// share once the ramp is on the ink's far side — which is exactly the regime the ceiling caps.
fn clamp_reach(base: Color32, ink: Color32, extreme: Color32, ceiling: f32) -> f32 {
   let at = |share: f32| contrast_ratio(ink, mix(base, extreme, share));

   if at(ceiling) >= CLAMP_CONTRAST {
      return ceiling;
   }

   let (mut low, mut high) = (0.0_f32, ceiling);

   for _ in 0..CLAMP_STEPS {
      let middle = (low + high) * 0.5;

      if at(middle) >= CLAMP_CONTRAST {
         low = middle;
      } else {
         high = middle;
      }
   }

   low
}

/// Clockwise outline, y-down: NW → NE → SE → SW.
///
/// Segment counts follow the radius, so a pill and a 3 pt corner both keep their silhouette; the
/// inset ring of [`Gradient::mesh_rounded`] shares them vertex for vertex.
fn outline(rect: Rect, corner_radius: CornerRadius) -> Vec<Pos2> {
   let limit = (rect.width() * 0.5).min(rect.height() * 0.5).max(0.0);
   let nw = (corner_radius.nw as f32).clamp(0.0, limit);
   let ne = (corner_radius.ne as f32).clamp(0.0, limit);
   let se = (corner_radius.se as f32).clamp(0.0, limit);
   let sw = (corner_radius.sw as f32).clamp(0.0, limit);

   let mut path = Vec::new();

   push_corner(
      &mut path,
      pos2(rect.left() + nw, rect.top() + nw),
      nw,
      PI,
      PI + FRAC_PI_2,
   );
   push_corner(
      &mut path,
      pos2(rect.right() - ne, rect.top() + ne),
      ne,
      PI + FRAC_PI_2,
      TAU,
   );
   push_corner(
      &mut path,
      pos2(rect.right() - se, rect.bottom() - se),
      se,
      0.0,
      FRAC_PI_2,
   );
   push_corner(
      &mut path,
      pos2(rect.left() + sw, rect.bottom() - sw),
      sw,
      FRAC_PI_2,
      PI,
   );

   path
}

/// One corner arc, always `steps + 1` points so two rings with the same radii line up.
fn push_corner(path: &mut Vec<Pos2>, center: Pos2, radius: f32, start: f32, end: f32) {
   if radius <= 0.0 {
      path.push(center);
      return;
   }

   let steps = quadrant_steps(radius);

   for step in 0..=steps {
      let t = step as f32 / steps as f32;
      let angle = start + (end - start) * t;
      path.push(pos2(
         center.x + radius * angle.cos(),
         center.y + radius * angle.sin(),
      ));
   }
}

fn quadrant_steps(radius: f32) -> u32 {
   if radius <= 2.0 {
      2
   } else if radius <= 5.0 {
      4
   } else if radius < 18.0 {
      8
   } else if radius < 50.0 {
      16
   } else {
      32
   }
}

/// Corner radii pulled in by the rim width, so the inset ring sits inside the outer one.
fn inset_corners(corner_radius: CornerRadius, by: f32) -> CornerRadius {
   let by = by.round().clamp(0.0, u8::MAX as f32) as u8;

   CornerRadius {
      nw: corner_radius.nw.saturating_sub(by),
      ne: corner_radius.ne.saturating_sub(by),
      se: corner_radius.se.saturating_sub(by),
      sw: corner_radius.sw.saturating_sub(by),
   }
}

/// Fan from vertex `first` over `count` consecutive vertices; valid for any convex ring.
fn add_fan(mesh: &mut Mesh, first: u32, count: usize) {
   for index in 2..count as u32 {
      mesh.add_triangle(first, first + index - 1, first + index);
   }
}

#[cfg(test)]
mod tests {
   use super::*;
   use egui::vec2;

   const DARK: Color32 = Color32::from_rgb(0x1a, 0x1b, 0x26);
   const INK: Color32 = Color32::from_rgb(0xc0, 0xca, 0xf5);

   fn unit_rect() -> Rect {
      Rect::from_min_size(pos2(0.0, 0.0), vec2(10.0, 10.0))
   }

   /// The ink a fill would carry: whichever of the dark/light pair reads better on it.
   fn ink_on(fill: Color32) -> Color32 {
      if contrast_ratio(fill, INK) >= contrast_ratio(fill, DARK) {
         INK
      } else {
         DARK
      }
   }

   #[test]
   fn sample_endpoints_are_the_stops() {
      let from = Color32::from_rgb(0, 0, 0);
      let to = Color32::from_rgb(255, 255, 255);
      let gradient = Gradient::vertical(from, to);

      assert_eq!(gradient.sample(0.0), from);
      assert_eq!(gradient.sample(1.0), to);
      assert_eq!(gradient.sample(-1.0), from);
      assert_eq!(gradient.sample(2.0), to);
   }

   #[test]
   fn sample_at_hits_the_corners_named_by_the_dir() {
      let from = Color32::RED;
      let to = Color32::BLUE;
      let rect = unit_rect();

      let horizontal = Gradient::horizontal(from, to);
      assert_eq!(horizontal.sample_at(rect, rect.left_top()), from);
      assert_eq!(horizontal.sample_at(rect, rect.right_top()), to);

      let bottom_up = Gradient::new(from, to).with_dir(GradientDir::BottomUp);
      assert_eq!(
         bottom_up.sample_at(rect, rect.left_bottom()),
         from
      );
      assert_eq!(bottom_up.sample_at(rect, rect.left_top()), to);

      let right_to_left = Gradient::new(from, to).with_dir(GradientDir::RightToLeft);
      assert_eq!(
         right_to_left.sample_at(rect, rect.right_top()),
         from
      );
      assert_eq!(right_to_left.sample_at(rect, rect.left_top()), to);
   }

   #[test]
   fn reverse_swaps_stops_and_keeps_dir() {
      let from = Color32::WHITE;
      let to = Color32::BLACK;
      let reversed = Gradient::horizontal(from, to).reverse();

      assert_eq!(reversed.from, to);
      assert_eq!(reversed.to, from);
      assert_eq!(reversed.dir, GradientDir::LeftToRight);
   }

   #[test]
   fn sharp_mesh_is_a_feathered_ring() {
      let gradient = Gradient::vertical(Color32::WHITE, Color32::BLACK);
      let mesh = gradient.mesh(unit_rect());

      // Four corners, two rings: the sampled inner outline and the transparent rim.
      assert_eq!(mesh.vertices.len(), 8);
      assert_eq!(mesh.indices.len() % 3, 0);
      assert!(mesh.is_valid());
      assert!(mesh.vertices[4..].iter().all(|vertex| vertex.color == Color32::TRANSPARENT));
   }

   #[test]
   fn rounded_mesh_stays_inside_the_rect_and_is_valid() {
      let gradient = Gradient::vertical(Color32::WHITE, Color32::BLACK);
      let rect = unit_rect();
      let mesh = gradient.mesh_rounded(rect, 4);

      assert!(mesh.vertices.len() > 8);
      assert!(mesh.is_valid());
      assert_eq!(mesh.indices.len() % 3, 0);

      for vertex in &mesh.vertices {
         assert!(
            rect.expand(0.01).contains(vertex.pos),
            "{:?}",
            vertex.pos
         );
      }
   }

   #[test]
   fn pill_mesh_clamps_every_radius() {
      let gradient = Gradient::vertical(Color32::WHITE, Color32::BLACK);
      let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(20.0, 8.0));
      let mesh = gradient.mesh_rounded(rect, 255);

      assert!(mesh.is_valid());

      for vertex in &mesh.vertices {
         assert!(rect.expand(0.01).contains(vertex.pos));
      }
   }

   #[test]
   fn bevel_lifts_one_end_and_drops_the_other() {
      let gradient = Gradient::bevel(DARK, INK, 0.1);

      assert_eq!(gradient.dir, GradientDir::TopDown);
      assert_ne!(gradient.from, gradient.to);
      assert!(
         contrast_ratio(gradient.from, Color32::BLACK) > contrast_ratio(DARK, Color32::BLACK),
         "the top end must be the lighter one"
      );
      assert!(
         contrast_ratio(gradient.to, Color32::WHITE) > contrast_ratio(DARK, Color32::WHITE),
         "the bottom end must be the darker one"
      );
   }

   #[test]
   fn bevel_keeps_ink_contrast_on_every_end() {
      let fills = [
         DARK,
         Color32::from_rgb(0x7a, 0xa2, 0xf7),
         Color32::from_rgb(0xf7, 0x76, 0x8e),
         Color32::from_rgb(0xe0, 0xaf, 0x68),
         Color32::from_rgb(0xf5, 0xf5, 0xf5),
         Color32::from_rgb(0x9a, 0x9a, 0x9a),
      ];

      for fill in fills {
         let ink = ink_on(fill);
         let gradient = Gradient::bevel(fill, ink, 0.35);

         assert!(
            gradient.min_contrast(ink) >= CONTRAST_AA_TEXT,
            "{fill:?} on {ink:?} fell to {:.2}",
            gradient.min_contrast(ink)
         );
      }
   }

   #[test]
   fn bevel_flattens_a_fill_with_no_headroom() {
      // #767676 against white is 4.54:1, so lifting it would immediately break AA.
      let base = Color32::from_rgb(0x76, 0x76, 0x76);
      let gradient = Gradient::bevel(base, Color32::WHITE, 0.4);

      assert_eq!(gradient.from, base);
      assert!(gradient.min_contrast(Color32::WHITE) >= CONTRAST_AA_TEXT);
   }

   #[test]
   fn bevel_leaves_transparency_and_zero_strength_flat() {
      assert!(Gradient::bevel(Color32::TRANSPARENT, INK, 0.5).is_flat());
      assert!(Gradient::bevel(DARK, INK, 0.0).is_flat());
      assert!(Gradient::bevel(DARK, INK, 0.0).sample(0.5) == DARK);
   }

   #[test]
   fn fill_bevel_collapses_to_solid_when_it_cannot_ramp() {
      assert_eq!(
         Fill::bevel(Color32::TRANSPARENT, INK, 0.5),
         Fill::Solid(Color32::TRANSPARENT)
      );

      let tight = Color32::from_rgb(0x76, 0x76, 0x76);
      assert_eq!(
         Fill::bevel(tight, Color32::WHITE, 0.0),
         Fill::Solid(tight)
      );
   }

   #[test]
   fn fill_gamma_multiply_fades_both_stops() {
      let fill = Fill::vertical(Color32::WHITE, Color32::BLACK);

      match fill.gamma_multiply(0.5) {
         Fill::Gradient(gradient) => {
            assert_eq!(gradient.from.a(), gradient.to.a());
            assert!(gradient.from.a() < Color32::WHITE.a());
         }
         Fill::Solid(_) => panic!("a gradient must stay a gradient"),
      }
   }

   #[test]
   fn remap_replaces_a_stop() {
      let mut fill = Fill::vertical(Color32::RED, Color32::BLUE);
      fill.remap_color(Color32::BLUE, Color32::GREEN);

      assert_eq!(fill, Fill::vertical(Color32::RED, Color32::GREEN));
   }

   #[test]
   fn to_solid_is_the_midpoint_of_a_ramp() {
      assert_eq!(
         Fill::vertical(Color32::BLACK, Color32::WHITE).to_solid(),
         mix(Color32::BLACK, Color32::WHITE, 0.5)
      );
      assert_eq!(Fill::solid(Color32::RED).to_solid(), Color32::RED);
      assert!(Fill::solid(Color32::TRANSPARENT).is_transparent());
   }
}
