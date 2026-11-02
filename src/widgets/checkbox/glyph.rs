//! The tick: a 3-point polyline with round caps and a round elbow.
//!
//! The points are the reference checkbox's measured ratios inside its box (`74..89 × 173..188` of a 16 px
//! box), so one [`Checkbox::size`](super::Checkbox::size) scales the glyph. epaint's `PathStroke` has no caps
//! and no round joins, so each tip and the elbow get a filled circle.

use egui::{Color32, Pos2, Rect, Shape, Stroke, vec2};

/// Left tip, as a fraction of the box rect.
pub(super) const P1: (f32, f32) = (0.28, 0.47);
/// Elbow, as a fraction of the box rect.
pub(super) const P2: (f32, f32) = (0.47, 0.66);
/// Right tip, as a fraction of the box rect.
pub(super) const P3: (f32, f32) = (0.72, 0.34);
/// Tick stroke width as a fraction of the box size.
pub(super) const STROKE_RATIO: f32 = 0.13;

/// Tick stroke width for a box of `size`.
pub(super) fn stroke_width(size: f32) -> f32 {
   size * STROKE_RATIO
}

/// The three points of the tick inside `rect`, in paint order (left tip, elbow, right tip).
pub(super) fn tick_points(rect: Rect) -> [Pos2; 3] {
   let p = |(x, y): (f32, f32)| rect.min + vec2(x * rect.width(), y * rect.height());
   [p(P1), p(P2), p(P3)]
}

/// Two segments plus three cap circles, all in `color`.
///
/// The elbow circle hides the miter notch the two segments leave at `P2`, which is what makes the tick read
/// as one round-capped stroke.
pub(super) fn tick_shapes(rect: Rect, color: Color32, width: f32) -> Vec<Shape> {
   let [a, b, c] = tick_points(rect);
   let stroke = Stroke::new(width, color);
   let cap = width * 0.5;
   vec![
      Shape::line_segment([a, b], stroke),
      Shape::line_segment([b, c], stroke),
      Shape::circle_filled(a, cap, color),
      Shape::circle_filled(b, cap, color),
      Shape::circle_filled(c, cap, color),
   ]
}
