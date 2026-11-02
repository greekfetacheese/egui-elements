//! Corner badges — small notification circles painted over a themed widget.
//!
//! A [`Badge`] is a circle with optional text, anchored to one of the four
//! corners of a widget. Attach it to a [`Button`](crate::widgets::Button) to
//! signal that the control opens something with unseen content (balance or
//! approval changes, unread messages, …):
//!
//! ```
//! # use egui::__run_test_ui;
//! # use egui_elements::widgets::{Badge, BadgeCorner, Button};
//! # __run_test_ui(|ui| {
//! let changes = 3;
//! ui.add(
//!    Button::new("Balance & approvals")
//!       .badge(Badge::new(changes.to_string()).corner(BadgeCorner::TopRight)),
//! );
//! # });
//! ```
//!
//! The badge is an *overlay*: it is painted on top of the widget and never
//! changes its size or layout. The fill defaults to
//! [`ThemeColors::error`](crate::theme::ThemeColors::error) and the text to
//! [`ThemeColors::bg`](crate::theme::ThemeColors::bg), matching the theme rule
//! that chromatic fills carry `bg`-colored glyphs.
//!
//! A badge may straddle its host's corner with `.straddle()`, putting the
//! circle's centre on the corner point. Each effect it paints (the pulse, the
//! hop, a later task) spills outside the host rect without changing layout.
//!
//! A badge can also be added on its own with `ui.add(Badge::new("3"))`, which
//! allocates its own square; the halo and the hop still paint outside it.

use std::sync::Arc;

use egui::{
   Align2, Color32, Galley, Id, Pos2, Rect, Stroke, TextStyle, TextWrapMode, Ui, Vec2, WidgetText,
};

use crate::theme::Theme;

mod bounce;
mod pulse;

pub use bounce::{BadgeBounce, BadgeBounceTrigger};
pub use pulse::BadgePulse;

/// Default size (in points) of a badge with an empty text — a plain dot.
const DEFAULT_MIN_DIAMETER: f32 = 16.0;
/// Default distance (in points) the circle is inset from the widget corner.
const DEFAULT_INSET: f32 = 2.0;
/// Default gap (in points) between the badge text and the circle edge.
const DEFAULT_PADDING: f32 = 3.0;

/// Which corner of a widget a [`Badge`] is anchored to.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BadgeCorner {
   TopLeft,
   /// Where notification badges usually live; the default.
   #[default]
   TopRight,
   BottomLeft,
   BottomRight,
}

impl BadgeCorner {
   /// Anchor used to place the circle inside the widget rect.
   pub const fn align2(self) -> Align2 {
      match self {
         Self::TopLeft => Align2::LEFT_TOP,
         Self::TopRight => Align2::RIGHT_TOP,
         Self::BottomLeft => Align2::LEFT_BOTTOM,
         Self::BottomRight => Align2::RIGHT_BOTTOM,
      }
   }

   pub const fn to_str(self) -> &'static str {
      match self {
         Self::TopLeft => "Top left",
         Self::TopRight => "Top right",
         Self::BottomLeft => "Bottom left",
         Self::BottomRight => "Bottom right",
      }
   }

   pub const fn to_vec() -> &'static [Self] {
      &[
         Self::TopLeft,
         Self::TopRight,
         Self::BottomLeft,
         Self::BottomRight,
      ]
   }

   /// Outward direction of this corner; each component is `-1.0` or `1.0`.
   pub const fn sign(self) -> Vec2 {
      match self {
         Self::TopLeft => Vec2::new(-1.0, -1.0),
         Self::TopRight => Vec2::new(1.0, -1.0),
         Self::BottomLeft => Vec2::new(-1.0, 1.0),
         Self::BottomRight => Vec2::new(1.0, 1.0),
      }
   }

   /// The corner point itself, in `rect`'s coordinates.
   pub fn point(self, rect: Rect) -> Pos2 {
      let sign = self.sign();
      Pos2::new(
         if sign.x > 0.0 {
            rect.right()
         } else {
            rect.left()
         },
         if sign.y > 0.0 {
            rect.bottom()
         } else {
            rect.top()
         },
      )
   }
}

/// Where the circle hangs relative to the host's corner.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BadgeAnchor {
   /// The circle sits inside the host rect; `inset` points inward. The default.
   #[default]
   Inside,
   /// The circle's centre sits on the host's corner point; `inset` then points outward.
   Corner,
}

/// A notification circle painted in a corner of a widget.
///
/// Build one with [`Badge::new`] and attach it with
/// [`Button::badge`](crate::widgets::Button::badge). The colour and corner are
/// the two knobs you will reach for; everything else has a sane default.
///
/// Empty text ([`Badge::dot`]) renders a plain coloured dot.
#[derive(Clone, Debug)]
#[must_use = "Badges should be shown on a widget, e.g. `Button::new(\"…\").badge(badge)`"]
pub struct Badge {
   text: WidgetText,
   color: Option<Color32>,
   corner: BadgeCorner,
   anchor: BadgeAnchor,
   text_color: Option<Color32>,
   min_diameter: f32,
   inset: Option<f32>,
   padding: f32,
   outline: Option<Stroke>,
   pulse: Option<BadgePulse>,
   bounce: Option<BadgeBounce>,
   id: Option<Id>,
}

impl Badge {
   /// Create a badge showing `text` (e.g. a change count) in the top-right
   /// corner, filled with [`ThemeColors::error`](crate::theme::ThemeColors::error).
   pub fn new(text: impl Into<WidgetText>) -> Self {
      Self {
         text: text.into(),
         color: None,
         corner: BadgeCorner::default(),
         anchor: BadgeAnchor::default(),
         text_color: None,
         min_diameter: DEFAULT_MIN_DIAMETER,
         inset: None,
         padding: DEFAULT_PADDING,
         outline: None,
         pulse: None,
         bounce: None,
         id: None,
      }
   }

   /// Create a textless badge: a plain coloured dot.
   pub fn dot() -> Self {
      Self::new("")
   }

   /// Circle fill. Defaults to the theme error colour.
   #[inline]
   pub fn color(mut self, color: Color32) -> Self {
      self.color = Some(color);
      self
   }

   /// Which corner of the widget the badge sits in. Default: top-right.
   #[inline]
   pub fn corner(mut self, corner: BadgeCorner) -> Self {
      self.corner = corner;
      self
   }

   /// Where the circle hangs relative to the host's corner. Default: [`BadgeAnchor::Inside`].
   #[inline]
   pub fn anchor(mut self, anchor: BadgeAnchor) -> Self {
      self.anchor = anchor;
      self
   }

   /// Put the circle's **centre** on the host's corner, so it straddles the edge.
   ///
   /// Equivalent to `.anchor(BadgeAnchor::Corner)` with no explicit inset; `.inset(..)` afterwards
   /// pushes it further out.
   #[inline]
   pub fn straddle(self) -> Self {
      self.anchor(BadgeAnchor::Corner)
   }

   /// Colour of the badge text. Defaults to
   /// [`ThemeColors::bg`](crate::theme::ThemeColors::bg) so glyphs stay
   /// readable on chromatic fills.
   #[inline]
   pub fn text_color(mut self, text_color: Color32) -> Self {
      self.text_color = Some(text_color);
      self
   }

   /// Smallest diameter the circle will take, in points.
   ///
   /// The circle grows beyond this when the text needs the room. Default:
   /// `16.0`. Lower it for [`Button::small`](crate::widgets::Button::small).
   #[inline]
   pub fn min_diameter(mut self, min_diameter: f32) -> Self {
      self.min_diameter = min_diameter;
      self
   }

   /// Distance between the circle and the widget corner, in points.
   ///
   /// Negative values let the badge straddle the edge. Default: `2.0`.
   /// With [`BadgeAnchor::Corner`] the offset points outward instead.
   #[inline]
   pub fn inset(mut self, inset: f32) -> Self {
      self.inset = Some(inset);
      self
   }

   /// Gap between the text and the circle edge, in points. Default: `3.0`.
   #[inline]
   pub fn padding(mut self, padding: f32) -> Self {
      self.padding = padding;
      self
   }

   /// Ring drawn around the circle, e.g. `Stroke::new(1.5, theme.colors.bg)`
   /// to lift the badge off a busy button fill.
   #[inline]
   pub fn outline(mut self, outline: impl Into<Stroke>) -> Self {
      self.outline = Some(outline.into());
      self
   }

   /// Loop a radar-like halo behind the circle. Off by default.
   #[inline]
   pub fn pulse(mut self, pulse: BadgePulse) -> Self {
      self.pulse = Some(pulse);
      self
   }

   /// Hop when the badge's text changes, when the host asks, or on a loop — see
   /// [`BadgeBounceTrigger`]. Off by default.
   #[inline]
   pub fn bounce(mut self, bounce: BadgeBounce) -> Self {
      self.bounce = Some(bounce);
      self
   }

   /// Start a hop on the badge carrying `id`, for [`BadgeBounceTrigger::Manual`].
   ///
   /// Call it before the badge paints this frame. `id` is the value passed to [`Badge::id`].
   pub fn bounce_now(ctx: &egui::Context, id: impl std::hash::Hash + std::fmt::Debug) {
      bounce::start_now(ctx, Id::unique(id));
   }

   /// Stable id for this badge's animation state.
   ///
   /// Defaults to the host's id — the `Response::id` of the widget the badge is painted on. Set it when
   /// a host wants [`Badge::bounce_now`] to reach this badge by name.
   #[inline]
   pub fn id(mut self, id: impl std::hash::Hash + std::fmt::Debug) -> Self {
      self.id = Some(Id::unique(id));
      self
   }

   /// Rect the circle occupies for a laid-out text of `text_size`, inside
   /// `widget_rect`. Pure geometry — shared by painting and tests.
   fn circle_rect(&self, text_size: Vec2, widget_rect: Rect) -> Rect {
      let diameter =
         (text_size + Vec2::splat(2.0 * self.padding)).max_elem().max(self.min_diameter);
      let size = Vec2::splat(diameter);
      let inset = self.inset.unwrap_or(match self.anchor {
         BadgeAnchor::Inside => DEFAULT_INSET,
         BadgeAnchor::Corner => 0.0,
      });
      match self.anchor {
         BadgeAnchor::Inside => {
            self.corner.align2().align_size_within_rect(size, widget_rect.shrink(inset))
         }
         BadgeAnchor::Corner => Rect::from_center_size(
            self.corner.point(widget_rect) + self.corner.sign() * inset,
            size,
         ),
      }
   }

   /// Paint the badge on top of `widget_rect` (usually a widget's response rect).
   ///
   /// Public so any widget can host a badge without re-implementing the placement and text metrics.
   pub fn paint_at(&self, ui: &Ui, widget_rect: Rect) {
      self.paint_at_id(ui, widget_rect, ui.scope_id().with("badge"));
   }

   /// [`Badge::paint_at`] with the host's id, which keys the hop state.
   ///
   /// Two badges hosted by two widgets in the same `Ui` share `ui.scope_id()`, so a host with more than one
   /// badge passes its own `Response::id` — otherwise one changing count hops every badge in the row.
   pub fn paint_at_id(&self, ui: &Ui, widget_rect: Rect, host_id: Id) {
      let galley = self.galley(ui);
      let circle = self.circle_rect(galley.size(), widget_rect);
      self.paint_circle(ui, circle, &galley, self.id.unwrap_or(host_id));
   }

   fn galley(&self, ui: &Ui) -> Arc<Galley> {
      self.text.clone().into_galley(
         ui,
         Some(TextWrapMode::Extend),
         f32::INFINITY,
         TextStyle::Small,
      )
   }

   /// Paint the halo and the circle into `circle`, on `badge_id`'s clock.
   fn paint_circle(&self, ui: &Ui, circle: Rect, galley: &Arc<Galley>, badge_id: Id) {
      let colors = Theme::colors_from_ctx(ui.ctx());
      let fill = self.color.unwrap_or(colors.error);
      let text_color = self.text_color.unwrap_or(colors.bg);
      let radius = circle.width() * 0.5;
      let now = ui.ctx().input(|i| i.time);

      let dy = self.bounce.map_or(0.0, |bounce| {
         bounce.offset_at(
            ui.ctx(),
            badge_id,
            self.text.text(),
            circle.width(),
            now,
         )
      });
      let circle = circle.translate(Vec2::new(0.0, dy));

      let pulse_reach = self.pulse.map_or(0.0, |p| (p.scale.max(1.0) - 1.0) * radius);
      let bounce_reach = self.bounce.map_or(0.0, |b| {
         b.amplitude.clamp(0.0, 1.5) * circle.width()
      });
      if !ui.is_rect_visible(circle.expand(pulse_reach.max(bounce_reach) + 1.0)) {
         return;
      }

      let painter = ui.painter();

      if let Some(pulse) = self.pulse {
         let period = pulse.period.max(0.05) as f64;
         let p = ((now / period + BadgePulse::phase(badge_id) as f64) % 1.0) as f32;
         let (scale, alpha) = pulse.frame(p);
         painter.circle_filled(
            circle.center(),
            radius * scale,
            pulse.color.unwrap_or(fill).gamma_multiply(alpha),
         );
         // A visible loop is never settled.
         ui.ctx().request_repaint();
      }

      painter.circle(
         circle.center(),
         radius,
         fill,
         self.outline.unwrap_or(Stroke::NONE),
      );
      if !galley.is_empty() {
         painter.galley(
            circle.center() - galley.size() * 0.5,
            galley.clone(),
            text_color,
         );
      }
   }

   /// Paint this badge on its own, allocating its own square in the layout.
   ///
   /// The halo and the hop paint **outside** the allocated rect, exactly like the corner placement
   /// paints outside its host — leave a gap in the surrounding layout if that matters.
   pub fn show(self, ui: &mut Ui) -> Rect {
      self.show_response(ui).0
   }

   fn show_response(self, ui: &mut Ui) -> (Rect, egui::Response) {
      let label = self.text.text().to_owned();
      let galley = self.galley(ui);
      let diameter = (galley.size() + Vec2::splat(2.0 * self.padding))
         .max_elem()
         .max(self.min_diameter);
      let (rect, response) = ui.allocate_exact_size(Vec2::splat(diameter), egui::Sense::hover());
      let circle = Rect::from_center_size(rect.center(), Vec2::splat(diameter));
      self.paint_circle(
         ui,
         circle,
         &galley,
         self.id.unwrap_or_else(|| ui.scope_id().with("badge-widget")),
      );
      response
         .widget_info(|| egui::WidgetInfo::labeled(egui::Role::Label, ui.is_enabled(), &label));
      (rect, response)
   }

   /// Center of the circle for a laid-out text of `text_size`. Test helper.
   #[cfg(test)]
   fn center_for(&self, text_size: Vec2, widget_rect: Rect) -> Pos2 {
      self.circle_rect(text_size, widget_rect).center()
   }
}

impl egui::Widget for Badge {
   fn ui(self, ui: &mut Ui) -> egui::Response {
      self.show_response(ui).1
   }
}

#[cfg(test)]
mod tests {
   use super::*;
   use egui::vec2;
   use egui::{FullOutput, Shape};

   fn rect() -> Rect {
      Rect::from_min_size(Pos2::new(100.0, 100.0), vec2(200.0, 40.0))
   }

   fn flatten(shape: &Shape, out: &mut Vec<Shape>) {
      out.push(shape.clone());
      if let Shape::Vec(shapes) = shape {
         for shape in shapes {
            flatten(shape, out);
         }
      }
   }

   fn shapes(out: &FullOutput) -> Vec<Shape> {
      let mut flat = Vec::new();
      for clipped in &out.shapes {
         flatten(&clipped.shape, &mut flat);
      }
      flat
   }

   #[test]
   fn paints_when_only_the_badge_is_visible() {
      let ctx = egui::Context::default();
      let screen = Rect::from_min_size(Pos2::ZERO, vec2(200.0, 200.0));
      // The host sits fully below the 200 pt screen; its top-right badge straddles y = 205 and reaches
      // up to y = 197, so the badge is on screen while the host rect is not.
      let host = Rect::from_min_size(Pos2::new(140.0, 205.0), vec2(60.0, 40.0));
      let input = egui::RawInput {
         screen_rect: Some(screen),
         ..Default::default()
      };
      let mut out = ctx.run_ui(input, |ui| {
         Badge::dot().straddle().color(Color32::RED).paint_at(ui, host);
      });
      out.textures_delta.clear();

      let painted = shapes(&out).into_iter().find_map(|shape| match shape {
         Shape::Circle(circle) if circle.fill == Color32::RED => Some(circle),
         _ => None,
      });
      let circle = painted.expect("the badge's circle was not painted even though it is on screen");
      assert!(
         (circle.center.x - 200.0).abs() < 0.01,
         "centre x: {}",
         circle.center.x
      );
      assert!(
         (circle.center.y - 205.0).abs() < 0.01,
         "centre y: {}",
         circle.center.y
      );
      assert!(
         (circle.radius - 8.0).abs() < 0.01,
         "radius: {}",
         circle.radius
      );
   }

   const TEXT: Vec2 = Vec2::new(8.0, 12.0);

   #[test]
   fn corners_anchor_to_the_matching_side() {
      let r = rect();
      let inset = DEFAULT_INSET;
      let pad = DEFAULT_PADDING;
      let min = DEFAULT_MIN_DIAMETER;

      let expected_diameter = (TEXT.x + 2.0 * pad).max(TEXT.y + 2.0 * pad).max(min);
      let radius = expected_diameter * 0.5;

      let cases = [
         (
            BadgeCorner::TopLeft,
            r.left() + inset + radius,
            r.top() + inset + radius,
         ),
         (
            BadgeCorner::TopRight,
            r.right() - inset - radius,
            r.top() + inset + radius,
         ),
         (
            BadgeCorner::BottomLeft,
            r.left() + inset + radius,
            r.bottom() - inset - radius,
         ),
         (
            BadgeCorner::BottomRight,
            r.right() - inset - radius,
            r.bottom() - inset - radius,
         ),
      ];

      for (corner, x, y) in cases {
         let center = Badge::dot().corner(corner).center_for(TEXT, r);
         assert!(
            (center.x - x).abs() < 0.01 && (center.y - y).abs() < 0.01,
            "{corner:?}: expected ({x}, {y}), got {center:?}"
         );
      }
   }

   #[test]
   fn circle_covers_text_and_min_diameter() {
      let badge = Badge::new("12");
      let small = badge.clone().circle_rect(Vec2::ZERO, rect());
      assert_eq!(small.width(), DEFAULT_MIN_DIAMETER);

      let wide = badge.circle_rect(vec2(40.0, 12.0), rect());
      assert_eq!(wide.width(), 40.0 + 2.0 * DEFAULT_PADDING);
   }

   #[test]
   fn negative_inset_lets_the_badge_straddle_the_corner() {
      let inside = Badge::dot().center_for(TEXT, rect());
      let outside = Badge::dot().inset(-6.0).center_for(TEXT, rect());
      assert!(outside.x > inside.x);
   }

   #[test]
   fn corner_anchor_centres_on_the_corner_point() {
      let r = rect();
      for (corner, (x, y)) in [
         (BadgeCorner::TopLeft, (r.left(), r.top())),
         (BadgeCorner::TopRight, (r.right(), r.top())),
         (BadgeCorner::BottomLeft, (r.left(), r.bottom())),
         (BadgeCorner::BottomRight, (r.right(), r.bottom())),
      ] {
         let centre = Badge::dot().corner(corner).straddle().center_for(TEXT, r);
         assert!(
            (centre.x - x).abs() < 0.01 && (centre.y - y).abs() < 0.01,
            "{corner:?}: expected ({x}, {y}), got {centre:?}"
         );
      }
   }

   #[test]
   fn corner_inset_pushes_outward() {
      let r = rect();
      let resting = Badge::dot().straddle().center_for(TEXT, r);
      let pushed = Badge::dot().straddle().inset(4.0).center_for(TEXT, r);
      assert!((pushed.x - (resting.x + 4.0)).abs() < 0.01);
      assert!((pushed.y - (resting.y - 4.0)).abs() < 0.01);
   }

   #[test]
   fn defaults_are_top_right_error_and_bg_text() {
      let badge = Badge::dot();
      assert_eq!(badge.corner, BadgeCorner::TopRight);
      assert_eq!(badge.color, None);
      assert_eq!(badge.text_color, None);
   }

   #[test]
   fn paints_without_panicking() {
      egui::__run_test_ui(|ui| {
         let response = ui.add(
            crate::widgets::Button::new("Balance & approvals").badge(
               Badge::new("3")
                  .corner(BadgeCorner::TopRight)
                  .outline(Stroke::new(1.5, Color32::BLACK)),
            ),
         );
         assert!(response.rect.width() > 0.0);
      });
   }

   #[test]
   fn a_pulsing_badge_paints_a_halo_behind_the_core() {
      let ctx = egui::Context::default();
      let host = Rect::from_min_size(Pos2::new(60.0, 60.0), vec2(60.0, 40.0));
      let input = egui::RawInput {
         screen_rect: Some(Rect::from_min_size(
            Pos2::ZERO,
            vec2(200.0, 200.0),
         )),
         ..Default::default()
      };
      let mut repaint = false;
      let mut out = ctx.run_ui(input, |ui| {
         Badge::dot().color(Color32::RED).pulse(BadgePulse::new()).paint_at(ui, host);
         repaint = ui.ctx().has_requested_repaint();
      });
      out.textures_delta.clear();

      let circles: Vec<_> = shapes(&out)
         .into_iter()
         .filter_map(|shape| match shape {
            Shape::Circle(circle) => Some(circle),
            _ => None,
         })
         .collect();
      assert_eq!(circles.len(), 2, "the halo and the core");
      assert!(
         circles[0].radius > circles[1].radius,
         "the halo is painted first and is larger"
      );
      assert_eq!(
         circles[1].radius, 8.0,
         "the core keeps its measured size"
      );
      assert!(
         circles[0].fill.a() < 255,
         "the halo is translucent"
      );
      assert!((circles[0].center - circles[1].center).length() < 0.01);
      assert!(repaint, "a visible pulse must keep repainting");
   }

   #[test]
   fn a_badge_hops_when_its_text_changes() {
      let ctx = egui::Context::default();
      let host = Rect::from_min_size(Pos2::new(60.0, 60.0), vec2(60.0, 40.0));
      let screen = Rect::from_min_size(Pos2::ZERO, vec2(200.0, 200.0));

      let frame = |text: &'static str, time: f64| {
         let input = egui::RawInput {
            screen_rect: Some(screen),
            time: Some(time),
            ..Default::default()
         };
         let mut out = ctx.run_ui(input, |ui| {
            Badge::new(text)
               .color(Color32::RED)
               .id("inbox")
               .bounce(BadgeBounce::new())
               .paint_at_id(ui, host, Id::unique("host"));
         });
         out.textures_delta.clear();
         shapes(&out)
            .into_iter()
            .find_map(|shape| match shape {
               Shape::Circle(circle) if circle.fill == Color32::RED => Some(circle),
               _ => None,
            })
            .expect("the core circle")
      };

      let first = frame("2", 0.0);
      let same = frame("2", 0.5);
      let changed = frame("3", 1.0);
      let mid_hop = frame("3", 1.15);

      // Compare frames, not literal points: the diameter depends on the real font metrics.
      assert_eq!(
         first.center.y, same.center.y,
         "the same text never hops"
      );
      assert_eq!(
         same.center.y, changed.center.y,
         "the trigger frame is still at rest"
      );
      assert!(
         mid_hop.center.y < first.center.y - 1.0,
         "the badge is up mid-hop: rest {} then {}",
         first.center.y,
         mid_hop.center.y
      );
      assert_eq!(
         mid_hop.center.x, first.center.x,
         "the hop is vertical only"
      );
   }

   #[test]
   fn a_standalone_badge_allocates_a_square() {
      egui::__run_test_ui(|ui| {
         let rect = Badge::new("12").show(ui);
         assert!(
            (rect.width() - rect.height()).abs() < 0.001,
            "square: {rect:?}"
         );
         assert!(
            rect.width() >= DEFAULT_MIN_DIAMETER,
            "at least the minimum: {rect:?}"
         );

         let response = ui.add(Badge::dot());
         assert!((response.rect.width() - response.rect.height()).abs() < 0.001);
      });
   }
}
