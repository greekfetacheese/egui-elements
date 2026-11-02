//! A themed loading spinner: an orbiting arc, its glow and a faint track.
//!
//! Same geometry as [`egui::Spinner`] — a rotating arc whose sweep is modulated
//! by `sin(time)`, 240° at its longest — drawn with the three colour effects the
//! reference spinner uses:
//!
//! - the arc's alpha ramps from a dim tail to a full-strength head
//!   ([`Spinner::comet`]), which reads as a light orbiting the ring;
//! - a glow ([`SpinnerShadow`]) follows the arc, so it is strongest where the
//!   arc is brightest;
//! - a faint track ([`Spinner::track_strength`]) completes the ring under the
//!   gap.
//!
//! Everything derives from one colour: the arc defaults to
//! [`ThemeColors::accent`](crate::theme::ThemeColors::accent) read from the
//! installed [`Theme`], and the track and the glow are
//! fractions of whatever arc colour is in effect — so
//! `Spinner::new().color(Color32::WHITE)` on a filled button yields a white arc,
//! a white track and a white glow without a second knob.
//!
//! ```
//! # use egui::__run_test_ui;
//! # use egui_elements::widgets::Spinner;
//! # __run_test_ui(|ui| {
//! ui.add(Spinner::new());
//! ui.add(Spinner::new().size(48.0).comet(0.0));
//! # });
//! ```
//!
//! The glow spills past the allocated square by `blur_ratio · size`, like any
//! other shadow; a host that needs it contained paints a smaller
//! [`Spinner::paint_at`] rect inside a larger allocation.

mod mesh;

use egui::{Color32, Rect, Response, Role, Sense, Shape, Stroke, Ui, Widget, WidgetInfo, vec2};

use crate::theme::Theme;
use mesh::{Arc, arc_mesh};

/// Stroke width as a fraction of the spinner's size.
const THICKNESS_RATIO: f32 = 0.10;
/// Narrowest stroke, in points.
const MIN_THICKNESS: f32 = 2.0;
/// Widest stroke, in points.
const MAX_THICKNESS: f32 = 6.0;
/// Default alpha of the track ring, as a fraction of the arc colour.
pub(crate) const TRACK_STRENGTH: f32 = 0.16;
/// Default alpha at the arc's tail (`Spinner::comet` = `1.0`).
const DEFAULT_COMET: f32 = 1.0;
/// Default glow reach, as a fraction of the spinner's size.
const BLUR_RATIO: f32 = 0.22;
/// Default peak alpha of the glow.
const SHADOW_STRENGTH: f32 = 0.30;

/// The glow that follows a spinner's arc: a blurred copy of it, so it is
/// strongest where the arc is brightest.
///
/// `blur_ratio` is the fringe's reach as a fraction of the spinner's size, so the
/// glow scales with the widget; `strength` is its peak alpha. Build one from the
/// arc colour with [`SpinnerShadow::new`] to keep the two in family, or take
/// [`SpinnerShadow::NONE`] to turn the glow off.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SpinnerShadow {
   /// Colour of the fringe.
   pub color: Color32,
   /// Reach of the fringe past the arc's edge, as a fraction of the size.
   pub blur_ratio: f32,
   /// Peak alpha of the fringe, `0.0..=1.0`.
   pub strength: f32,
}

impl SpinnerShadow {
   /// No glow. The arc still feathers its own edges.
   pub const NONE: Self = Self {
      color: Color32::TRANSPARENT,
      blur_ratio: 0.0,
      strength: 0.0,
   };

   /// A glow in `color` at the default reach and strength.
   pub fn new(color: Color32) -> Self {
      Self {
         color,
         blur_ratio: BLUR_RATIO,
         strength: SHADOW_STRENGTH,
      }
   }

   /// Colour of the fringe.
   #[inline]
   pub fn color(mut self, color: Color32) -> Self {
      self.color = color;
      self
   }

   /// Reach of the fringe past the arc's edge, as a fraction of the size.
   #[inline]
   pub fn blur_ratio(mut self, blur_ratio: f32) -> Self {
      self.blur_ratio = blur_ratio;
      self
   }

   /// Peak alpha of the fringe, `0.0..=1.0`.
   #[inline]
   pub fn strength(mut self, strength: f32) -> Self {
      self.strength = strength;
      self
   }
}

/// An orbiting-arc loading indicator.
///
/// Reads [`ThemeColors`](crate::theme::ThemeColors) from the installed
/// [`Theme`] unless you pass [`.color(...)`](Self::color).
///
/// ```
/// # use egui::__run_test_ui;
/// # use egui_elements::widgets::{Spinner, SpinnerShadow};
/// # __run_test_ui(|ui| {
/// if ui.add(Spinner::new().size(32.0)).hovered() {
///     // …
/// }
/// let plain = Spinner::new().shadow(SpinnerShadow::NONE).comet(0.0);
/// ui.add(plain);
/// # });
/// ```
#[derive(Clone, Debug)]
#[must_use = "Spinners should be shown on a ui, e.g. `ui.add(Spinner::new())`"]
pub struct Spinner {
   size: Option<f32>,
   color: Option<Color32>,
   thickness: Option<f32>,
   track_strength: f32,
   comet: f32,
   shadow: Option<SpinnerShadow>,
}

impl Spinner {
   /// A spinner in the theme accent, sized to the style's `interact_size`.
   pub fn new() -> Self {
      Self {
         size: None,
         color: None,
         thickness: None,
         track_strength: TRACK_STRENGTH,
         comet: DEFAULT_COMET,
         shadow: None,
      }
   }

   /// Side of the square the spinner occupies. Defaults to the style's
   /// `interact_size.y`.
   #[inline]
   pub fn size(mut self, size: f32) -> Self {
      self.size = Some(size);
      self
   }

   /// Arc colour. Defaults to [`ThemeColors::accent`](crate::theme::ThemeColors::accent);
   /// the track and the glow follow it.
   #[inline]
   pub fn color(mut self, color: impl Into<Color32>) -> Self {
      self.color = Some(color.into());
      self
   }

   /// Stroke width in points. Defaults to 10 % of the size, clamped to `2..=6`.
   #[inline]
   pub fn thickness(mut self, thickness: f32) -> Self {
      self.thickness = Some(thickness);
      self
   }

   /// Alpha of the track ring, as a fraction of the arc colour. `0.0` hides it.
   /// Default: `0.16`.
   #[inline]
   pub fn track_strength(mut self, track_strength: f32) -> Self {
      self.track_strength = track_strength;
      self
   }

   /// Tail-to-head alpha ramp along the arc: `1.0` fades the tail to a quarter of
   /// the arc colour, `0.0` paints a flat arc. Default: `1.0`.
   #[inline]
   pub fn comet(mut self, comet: f32) -> Self {
      self.comet = comet;
      self
   }

   /// The glow that follows the arc. Defaults to [`SpinnerShadow::new`] of the
   /// arc colour; [`SpinnerShadow::NONE`] turns it off.
   #[inline]
   pub fn shadow(mut self, shadow: SpinnerShadow) -> Self {
      self.shadow = Some(shadow);
      self
   }

   /// Stroke width for a spinner of `size`, honouring [`Spinner::thickness`].
   fn stroke_width(&self, size: f32) -> f32 {
      self
         .thickness
         .unwrap_or_else(|| (size * THICKNESS_RATIO).clamp(MIN_THICKNESS, MAX_THICKNESS))
   }

   /// Paints the spinner in `rect`, centred and inset to the shorter side.
   ///
   /// Public so a widget can host one without allocating a second `Ui`.
   pub fn paint_at(&self, ui: &Ui, rect: Rect) {
      if !ui.is_rect_visible(rect) {
         return;
      }

      // because it is animated
      ui.ctx().request_repaint();

      let colors = Theme::colors_from_ctx(ui.ctx());
      let size = rect.width().min(rect.height());
      let color = self.color.unwrap_or(colors.accent);
      let thickness = self.stroke_width(size);
      let radius = (size * 0.5 - thickness * 0.5).max(0.0);

      let time = ui.input(|input| input.time);
      let start = time * std::f64::consts::TAU;
      let end = start + 240f64.to_radians() * time.sin();

      let painter = ui.painter();

      if self.track_strength > 0.0 {
         painter.circle_stroke(
            rect.center(),
            radius,
            Stroke::new(
               thickness,
               color.gamma_multiply(self.track_strength),
            ),
         );
      }

      let shadow = self.shadow.unwrap_or_else(|| SpinnerShadow::new(color));
      let arc = Arc {
         center: rect.center(),
         radius,
         thickness,
         start,
         end,
      };

      painter.add(Shape::mesh(arc_mesh(
         &arc,
         color,
         shadow.color,
         shadow.blur_ratio * size,
         shadow.strength,
         self.comet,
      )));
   }
}

impl Default for Spinner {
   fn default() -> Self {
      Self::new()
   }
}

impl Widget for Spinner {
   fn ui(self, ui: &mut Ui) -> Response {
      let size = self.size.unwrap_or_else(|| ui.spacing().interact_size.y);
      let (rect, response) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
      response.widget_info(|| WidgetInfo::new(Role::ProgressIndicator));
      self.paint_at(ui, rect);

      response
   }
}

#[cfg(test)]
mod tests {
   use super::*;
   use crate::theme::{Theme, ThemeKind};
   use egui::epaint::{CircleShape, Mesh};
   use egui::{Context, FullOutput, Pos2, RawInput, Vec2};

   fn themed_ctx() -> Context {
      let ctx = Context::default();
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&ctx);
      ctx
   }

   fn accent() -> Color32 {
      Theme::new(ThemeKind::TokyoNight).colors.accent
   }

   fn frame(ctx: &Context, body: impl FnMut(&mut Ui)) -> FullOutput {
      let input = RawInput {
         screen_rect: Some(Rect::from_min_size(
            Pos2::ZERO,
            Vec2::new(400.0, 300.0),
         )),
         ..Default::default()
      };
      let mut out = ctx.run_ui(input, body);
      out.textures_delta.clear();
      out
   }

   fn flatten(shape: &Shape, out: &mut Vec<Shape>) {
      out.push(shape.clone());
      if let Shape::Vec(shapes) = shape {
         for shape in shapes {
            flatten(shape, out);
         }
      }
   }

   /// Paints `spinner` on a themed frame; returns its rect and every emitted
   /// shape, flattened out of the `Shape::Vec` groups.
   fn paint(spinner: Spinner) -> (Rect, Vec<Shape>) {
      let ctx = themed_ctx();
      let mut rect = Rect::ZERO;
      let out = frame(&ctx, |ui| {
         rect = ui.add(spinner.clone()).rect;
      });

      let mut flat = Vec::new();
      for clipped in &out.shapes {
         flatten(&clipped.shape, &mut flat);
      }

      (rect, flat)
   }

   fn track(shapes: &[Shape]) -> Option<&CircleShape> {
      shapes.iter().find_map(|shape| match shape {
         Shape::Circle(circle) => Some(circle),
         _ => None,
      })
   }

   fn arc(shapes: &[Shape]) -> &Mesh {
      shapes
         .iter()
         .find_map(|shape| match shape {
            Shape::Mesh(mesh) => Some(mesh.as_ref()),
            _ => None,
         })
         .expect("the arc mesh")
   }

   #[test]
   fn default_size_is_the_style_interact_size() {
      let (rect, _) = paint(Spinner::new());
      assert_eq!(rect.size(), Vec2::new(20.0, 20.0));
   }

   #[test]
   fn size_builder_squares_the_widget() {
      let (rect, _) = paint(Spinner::new().size(40.0));
      assert_eq!(rect.size(), Vec2::new(40.0, 40.0));
   }

   #[test]
   fn paints_a_track_ring_and_an_arc_mesh() {
      let (_, shapes) = paint(Spinner::new().size(40.0));

      let ring = track(&shapes).expect("the track ring");
      assert_eq!(ring.radius, 18.0);
      assert_eq!(ring.stroke.width, 4.0);
      assert_eq!(
         ring.stroke.color,
         accent().gamma_multiply(TRACK_STRENGTH)
      );

      // the comet ramp reaches the full arc colour at the head
      assert!(arc(&shapes).vertices.iter().any(|v| v.color == accent()));
   }

   #[test]
   fn track_strength_zero_paints_no_track() {
      let (_, shapes) = paint(Spinner::new().size(40.0).track_strength(0.0));

      assert!(track(&shapes).is_none());
      assert!(!arc(&shapes).vertices.is_empty());
   }

   #[test]
   fn shadow_none_leaves_only_the_arc() {
      let (rect, shapes) = paint(Spinner::new().size(40.0).shadow(SpinnerShadow::NONE));
      let center = rect.center();
      let edge = 18.0 + 4.0 * 0.5;

      for vertex in &arc(&shapes).vertices {
         if vertex.pos.distance(center) > edge + 0.01 {
            assert_eq!(
               vertex.color.a(),
               0,
               "fringe vertex at {:?}",
               vertex.pos
            );
         }
      }

      assert!(arc(&shapes).vertices.iter().any(|v| v.color == accent()));
   }
}
