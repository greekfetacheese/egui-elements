//! Status indicator — a small light with three states and an optional pulse.
//!
//! [`IndicatorState::On`] paints a filled success dot inside a soft glow,
//! [`IndicatorState::Off`] an error ring with a bar through it, and
//! [`IndicatorState::Connecting`] a warning ring with no fill.
//!
//! Any state can pulse with the badge's loop
//! ([`BadgePulse`](crate::widgets::BadgePulse)): a copy of the light grows and
//! fades behind the core, phased from the widget's id so two lights on one row
//! do not beat in lockstep.
//!
//! ```
//! # use egui::__run_test_ui;
//! # use egui_elements::widgets::{BadgePulse, Indicator, IndicatorState};
//! # __run_test_ui(|ui| {
//! ui.horizontal(|ui| {
//!    ui.add(Indicator::new(IndicatorState::On));
//!    ui.add(Indicator::new(IndicatorState::On).size(14.0).pulse(BadgePulse::new()));
//!    ui.add(Indicator::new(IndicatorState::Connecting));
//!    ui.add(Indicator::new(IndicatorState::Off));
//! });
//! # });
//! ```
//!
//! Forked from <https://github.com/stephenberry/egui-elegance> (MIT OR
//! Apache-2.0, © 2026 Stephen Berry) and re-homed on this crate's
//! [`Theme`](crate::theme::Theme).
//!
//! Like the badge's halo, the pulse paints **outside** the allocated square;
//! leave a gap in a tight layout if that matters.

use egui::{
   Color32, CornerRadius, Id, Pos2, Rect, Response, Role, Sense, Stroke, Ui, Vec2, Widget,
   WidgetInfo,
};

use crate::theme::{Theme, ThemeColors};
use crate::widgets::BadgePulse;

/// Alpha (0..=255) of the glow behind an [`IndicatorState::On`] dot.
const GLOW_ALPHA: u8 = 70;
/// How far the [`IndicatorState::On`] glow reaches past the dot, in points.
const GLOW_SPREAD: f32 = 1.5;

/// The three visual states of an [`Indicator`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IndicatorState {
   /// Connected / healthy — a solid success dot with a subtle glow.
   On,
   /// Disconnected / error — an error ring with a horizontal bar inside.
   Off,
   /// Transient / in-progress — a warning ring only.
   Connecting,
}

impl IndicatorState {
   /// Palette colour this state reads from.
   fn color(self, colors: &ThemeColors) -> Color32 {
      match self {
         Self::On => colors.success,
         Self::Off => colors.error,
         Self::Connecting => colors.warning,
      }
   }

   /// AccessKit label, announced instead of the painted shape.
   fn label(self) -> &'static str {
      match self {
         Self::On => "status: on",
         Self::Off => "status: off",
         Self::Connecting => "status: connecting",
      }
   }
}

/// A small status light.
///
/// ```
/// # use egui::__run_test_ui;
/// # use egui_elements::widgets::{Indicator, IndicatorState};
/// # __run_test_ui(|ui| {
/// ui.add(Indicator::new(IndicatorState::On));
/// ui.add(Indicator::new(IndicatorState::Connecting).size(14.0));
/// # });
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use = "Add with `ui.add(...)`."]
pub struct Indicator {
   state: IndicatorState,
   size: f32,
   pulse: Option<BadgePulse>,
}

impl Indicator {
   /// Create an indicator in the given state. Default size: 10 points.
   pub fn new(state: IndicatorState) -> Self {
      Self {
         state,
         size: 10.0,
         pulse: None,
      }
   }

   /// Override the indicator diameter in points.
   pub fn size(mut self, size: f32) -> Self {
      self.size = size.max(1.0);
      self
   }

   /// Pulse a copy of the light behind the core.
   ///
   /// Same loop as [`Badge::pulse`](crate::widgets::Badge::pulse), and the same
   /// parameters: the halo holds its alpha, then fades as the copy reaches
   /// `scale`. The phase comes from the widget's id, so siblings do not beat in
   /// lockstep. Default: no pulse; a pulsing indicator repaints every frame.
   pub fn pulse(mut self, pulse: BadgePulse) -> Self {
      self.pulse = Some(pulse);
      self
   }

   /// Paint the halo, the glow and the state glyph into `center`.
   fn paint(&self, ui: &Ui, center: Pos2, radius: f32, fill: Color32, id: Id) {
      let painter = ui.painter();

      if let Some(pulse) = self.pulse {
         let now = ui.ctx().input(|i| i.time);
         let period = pulse.period.max(0.05) as f64;
         let p = ((now / period + BadgePulse::phase(id) as f64) % 1.0) as f32;
         let (scale, alpha) = pulse.frame(p);
         painter.circle_filled(
            center,
            radius * scale,
            pulse.color.unwrap_or(fill).gamma_multiply(alpha),
         );
         // A visible loop is never settled.
         ui.ctx().request_repaint();
      }

      match self.state {
         IndicatorState::On => {
            painter.circle_filled(
               center,
               radius + GLOW_SPREAD,
               with_alpha(fill, GLOW_ALPHA),
            );
            painter.circle_filled(center, radius, fill);
         }
         IndicatorState::Off => {
            painter.circle_stroke(
               center,
               (radius - 0.5).max(0.0),
               Stroke::new(1.0, fill),
            );
            let bar = Rect::from_center_size(center, Vec2::new(self.size * 0.7, 2.0));
            painter.rect_filled(bar, CornerRadius::same(1), fill);
         }
         IndicatorState::Connecting => {
            painter.circle_stroke(
               center,
               (radius - 0.5).max(0.0),
               Stroke::new(1.8, fill),
            );
         }
      }
   }
}

impl Widget for Indicator {
   fn ui(self, ui: &mut Ui) -> Response {
      let colors = Theme::colors_from_ctx(ui.ctx());
      let fill = self.state.color(&colors);
      let (rect, response) = ui.allocate_exact_size(Vec2::splat(self.size), Sense::hover());

      let radius = self.size * 0.5;
      let halo_reach = self.pulse.map_or(0.0, |pulse| (pulse.scale.max(1.0) - 1.0) * radius);
      // The glow and the halo both paint outside the allocated square.
      if ui.is_rect_visible(rect.expand(halo_reach.max(GLOW_SPREAD) + 1.0)) {
         self.paint(ui, rect.center(), radius, fill, response.id);
      }

      response
         .widget_info(|| WidgetInfo::labeled(Role::Label, ui.is_enabled(), self.state.label()));
      response
   }
}

/// `color` at `alpha` (0..=255), preserving its hue — a glow must fade without dimming.
fn with_alpha(color: Color32, alpha: u8) -> Color32 {
   Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}

#[cfg(test)]
mod tests {
   use egui::{Context, RawInput, Shape};

   use super::*;

   /// Render one indicator in a sized frame, then return its shapes and whether
   /// the frame asked for another one.
   fn paint(indicator: Indicator) -> (Vec<Shape>, bool) {
      let ctx = Context::default();
      let mut repaint_requested = false;

      let output = ctx.run_ui(
         RawInput {
            time: Some(0.0),
            screen_rect: Some(Rect::from_min_size(
               Pos2::ZERO,
               egui::vec2(200.0, 200.0),
            )),
            ..Default::default()
         },
         |ui| {
            ui.add(indicator);
            repaint_requested = ui.ctx().has_requested_repaint();
         },
      );

      let mut shapes = Vec::new();
      for clipped in &output.shapes {
         flatten(&clipped.shape, &mut shapes);
      }
      output.drop_without_applying_deltas();

      (shapes, repaint_requested)
   }

   fn flatten(shape: &Shape, out: &mut Vec<Shape>) {
      match shape {
         Shape::Vec(shapes) => {
            for shape in shapes {
               flatten(shape, out);
            }
         }
         other => out.push(other.clone()),
      }
   }

   fn circles(shapes: &[Shape]) -> Vec<egui::epaint::CircleShape> {
      shapes
         .iter()
         .filter_map(|shape| match shape {
            Shape::Circle(circle) => Some(*circle),
            _ => None,
         })
         .collect()
   }

   #[test]
   fn on_state_paints_a_dot_inside_its_glow() {
      let (shapes, _) = paint(Indicator::new(IndicatorState::On));
      let circles = circles(&shapes);

      assert_eq!(circles.len(), 2, "glow + dot");
      assert!(circles.iter().any(|c| (c.radius - 5.0).abs() < 1e-4));
      assert!(circles.iter().any(|c| (c.radius - (5.0 + GLOW_SPREAD)).abs() < 1e-4));
   }

   #[test]
   fn connecting_paints_one_unfilled_ring() {
      let (shapes, _) = paint(Indicator::new(IndicatorState::Connecting));
      let circles = circles(&shapes);

      assert_eq!(circles.len(), 1);
      assert_eq!(circles[0].fill, Color32::TRANSPARENT);
      assert!((circles[0].stroke.width - 1.8).abs() < 1e-4);
   }

   #[test]
   fn off_state_paints_a_ring_and_the_bar() {
      let (shapes, _) = paint(Indicator::new(IndicatorState::Off));

      let circles = circles(&shapes);
      assert_eq!(circles.len(), 1);
      assert!((circles[0].stroke.width - 1.0).abs() < 1e-4);

      let bar = shapes.iter().find_map(|shape| match shape {
         Shape::Rect(rect) => Some(rect.rect),
         _ => None,
      });
      let bar = bar.expect("the Off state paints its bar");
      assert!(
         (bar.width() - 7.0).abs() < 1e-4,
         "0.7 x the diameter"
      );
      assert!((bar.height() - 2.0).abs() < 1e-4);
   }

   #[test]
   fn pulse_adds_a_halo_behind_the_core_and_keeps_repainting() {
      let (shapes, repaint_requested) =
         paint(Indicator::new(IndicatorState::On).pulse(BadgePulse::new().scale(2.5).alpha(1.0)));

      assert!(
         repaint_requested,
         "a visible loop is never settled"
      );
      // Glow, dot, and the halo copy — the halo is painted even at scale 1.0.
      assert!(circles(&shapes).len() >= 3);
   }

   #[test]
   fn size_allocates_a_square_of_that_size() {
      let ctx = Context::default();
      let mut rect = Rect::NOTHING;

      let output = ctx.run_ui(RawInput::default(), |ui| {
         rect = ui.add(Indicator::new(IndicatorState::On).size(20.0)).rect;
      });
      output.drop_without_applying_deltas();

      assert_eq!(rect.size(), egui::vec2(20.0, 20.0));
   }
}
