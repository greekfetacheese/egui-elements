//! A segmented tab strip: a pill track, one indicator that slides between segments, and a lift
//! under it.
//!
//! The look is measured from the reference segmented control
//! (<https://uiverse.io/uiverse-astronaut/odd-bobcat-3>, MIT): a fully rounded track that holds the
//! segments, the selected one painted as a pill on a soft shadow, and every state change eased rather
//! than switched. The reference's 6-of-48 ring around the pill becomes the strip's inset, so one
//! [Tabs::height] scales the whole strip.
//!
//! The indicator does not fade in place: one pill travels from the old segment to the new one and
//! stretches between the two while it is in flight, which is what reads as a segmented control rather
//! than as a highlighted cell. It is seeded at the current selection, so a strip that first appears is
//! still, and the selection itself lands instantly — only the chrome eases, the crate's rule
//! everywhere.
//!
//! Every colour comes from the palette: the track is the canvas ([ThemeColors::bg]), the pill is
//! [ThemeColors::highlight] with [ThemeColors::ink_on] ink, and hover washes the segment with
//! [ThemeColors::hover] and lifts its ink toward [ThemeColors::text] — never `highlight`, which stays
//! the selection. The lift is the toggle family's [ToggleShadow], so a strip and a switch on one card
//! cast the same kind of shadow.

use std::hash::Hash;
use std::sync::Arc;

use egui::{
   Color32, CornerRadius, FontId, Galley, IdSalt, Rect, Response, Role, Sense, Shadow, Stroke,
   StrokeKind, Ui, WidgetInfo, pos2, vec2,
};

use crate::motion::{InteractT, Lerp, Motion, animate_from};
use crate::theme::{Theme, ThemeColors};
use crate::widgets::toggle::ToggleShadow;

/// Segment inset as a share of the strip height; the pill rides inside this ring.
///
/// The reference pads a 48-point track with 6 points all round, i.e. 0.125 of the track. Held at 0.09
/// because the crate's strip is shorter than the reference's, and the ring has to survive the
/// `fill_width` row: [`Tabs::height`] scales the whole geometry.
const INSET_RATIO: f32 = 0.09;
/// Hover wash alpha at full hover.
const HOVER_WASH: f32 = 0.55;
/// Hover wash alpha while the pointer is down on an unselected segment.
const PRESS_WASH: f32 = 0.85;
/// Alpha multiplier applied to a disabled strip.
const DISABLED_ALPHA: f32 = 0.5;
/// Floor of an indicator radius derived from the strip, in points.
const MIN_RADIUS: f32 = 2.0;

/// Per-widget colours for a [Tabs] strip.
///
/// [Default] is inert — a transparent track with no ink and [ToggleShadow::NONE] — so a struct literal
/// starts from [TabsVisuals::from_theme] and overrides the one slot it wants to change.
///
/// ```no_run
/// # use egui::{__run_test_ui, Color32};
/// # use egui_elements::theme::{Theme, ThemeKind};
/// # use egui_elements::widgets::{Tabs, TabsVisuals};
/// # let colors = Theme::new(ThemeKind::TokyoNight).colors;
/// # __run_test_ui(|ui| {
/// # let mut selected = 0;
/// let visuals = TabsVisuals {
///    indicator: colors.accent,
///    text_selected: colors.ink_on(colors.accent),
///    ..TabsVisuals::from_theme(colors)
/// };
/// Tabs::new("kind", &mut selected, ["All", "Media"]).visuals(visuals).show(ui);
/// # });
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TabsVisuals {
   /// Fill of the track the segments sit on.
   pub track: Color32,
   /// Outline drawn inside the track.
   pub track_border: Stroke,
   /// Wash behind a hovered, unselected segment.
   pub hover: Color32,
   /// Fill of the sliding indicator, i.e. the selected segment.
   pub indicator: Color32,
   /// Ink of an unselected segment.
   pub text: Color32,
   /// Ink of an unselected segment under the pointer.
   pub text_hover: Color32,
   /// Ink of the selected segment.
   pub text_selected: Color32,
   /// The lift under the indicator; [ToggleShadow::NONE] makes it flat.
   pub shadow: ToggleShadow,
}

impl TabsVisuals {
   /// The palette-derived defaults, also used when a widget sets no visuals.
   pub fn from_theme(colors: ThemeColors) -> Self {
      Self {
         track: colors.bg,
         track_border: Stroke::new(1.0, colors.border),
         hover: colors.hover,
         indicator: colors.highlight,
         text: colors.text_muted,
         text_hover: colors.text,
         text_selected: colors.ink_on(colors.highlight),
         shadow: ToggleShadow::derived(),
      }
   }
}

impl Default for TabsVisuals {
   fn default() -> Self {
      Self {
         track: Color32::TRANSPARENT,
         track_border: Stroke::NONE,
         hover: Color32::TRANSPARENT,
         indicator: Color32::TRANSPARENT,
         text: Color32::TRANSPARENT,
         text_hover: Color32::TRANSPARENT,
         text_selected: Color32::TRANSPARENT,
         shadow: ToggleShadow::NONE,
      }
   }
}

/// A horizontal segmented control: one selected segment, index written back.
///
/// The strip eases the indicator between segments and lifts it on a shadow; the duration comes from the
/// installed style's `animation_time`, so [Tabs::motion] is the per-widget override and
/// [Motion::instant] the reduce-motion answer.
///
/// ```rust
/// # use egui::__run_test_ui;
/// # use egui_elements::widgets::Tabs;
/// # __run_test_ui(|ui| {
/// let mut selected = 0;
/// Tabs::new("kind", &mut selected, ["All", "Media", "Files"]).show(ui);
/// # });
/// ```
#[must_use = "You should call .show()"]
pub struct Tabs<'a> {
   id_salt: IdSalt,
   selected: &'a mut usize,
   labels: Vec<&'a str>,
   fill_width: bool,
   height: Option<f32>,
   gap: f32,
   visuals: Option<TabsVisuals>,
   shadow: Option<ToggleShadow>,
   motion: Option<Motion>,
   corner_radius: Option<CornerRadius>,
}

impl<'a> Tabs<'a> {
   /// Tabs over `labels`, writing the chosen index into `selected`.
   pub fn new(
      id_salt: impl Hash + std::fmt::Debug,
      selected: &'a mut usize,
      labels: impl IntoIterator<Item = &'a str>,
   ) -> Self {
      Self {
         id_salt: IdSalt::new(id_salt),
         selected,
         labels: labels.into_iter().collect(),
         fill_width: true,
         height: None,
         gap: 2.0,
         visuals: None,
         shadow: None,
         motion: None,
         corner_radius: None,
      }
   }

   /// Stretch the strip across the available width. Default `true`.
   pub fn fill_width(mut self, fill: bool) -> Self {
      self.fill_width = fill;
      self
   }

   /// Strip height in points. Default: body text plus the button padding.
   pub fn height(mut self, height: f32) -> Self {
      self.height = Some(height);
      self
   }

   /// Space between segments, in points. Default `2.0`.
   pub fn gap(mut self, gap: f32) -> Self {
      self.gap = gap;
      self
   }

   /// Replace the palette-derived colours.
   pub fn visuals(mut self, visuals: TabsVisuals) -> Self {
      self.visuals = Some(visuals);
      self
   }

   /// Replace the lift under the indicator; [ToggleShadow::NONE] turns it off.
   pub fn shadow(mut self, shadow: ToggleShadow) -> Self {
      self.shadow = Some(shadow);
      self
   }

   /// How the indicator and the hover wash animate. Defaults to [Motion::from_ui] (the style's
   /// `animation_time`); [Motion::instant] makes every state land on its frame.
   pub fn motion(mut self, motion: Motion) -> Self {
      self.motion = Some(motion);
      self
   }

   /// Corner radius of the track. Default `u8::MAX`, which clamps to a pill.
   ///
   /// The indicator's radius is this radius minus the inset, so a pill track always holds a pill.
   pub fn corner_radius(mut self, corner_radius: CornerRadius) -> Self {
      self.corner_radius = Some(corner_radius);
      self
   }

   /// Draw the strip. The returned [Response] reports `changed` when the index moved.
   pub fn show(self, ui: &mut Ui) -> Response {
      let Tabs {
         id_salt,
         selected,
         labels,
         fill_width,
         height,
         gap,
         visuals,
         shadow,
         motion,
         corner_radius,
      } = self;

      let theme = Theme::current(ui.ctx());
      let colors = theme.colors;
      let visuals = visuals.unwrap_or_else(|| TabsVisuals::from_theme(colors));
      let shadow = shadow.unwrap_or(visuals.shadow);
      let motion = motion.unwrap_or_else(|| Motion::from_ui(ui));
      let enabled = ui.is_enabled();
      let pad_x = ui.spacing().button_padding.x.max(10.0);
      let pad_y = ui.spacing().button_padding.y.max(6.0);
      let font = FontId::proportional(theme.typography.normal);
      let height = height.unwrap_or(theme.typography.normal + 2.0 * pad_y);
      let radius = corner_radius.unwrap_or(CornerRadius::same(u8::MAX));

      let painter = ui.painter().clone();
      // Measured once with a placeholder ink and painted per state: a galley's size does not depend
      // on its colour, and `Painter::galley` replaces a placeholder with the colour it is given.
      let galleys: Vec<Arc<Galley>> = labels
         .iter()
         .map(|label| {
            painter.layout_no_wrap(
               (*label).to_owned(),
               font.clone(),
               Color32::PLACEHOLDER,
            )
         })
         .collect();

      let widths: Vec<f32> = if galleys.is_empty() {
         Vec::new()
      } else if fill_width {
         let gap_total = gap * (galleys.len() - 1) as f32;
         let each = ((ui.available_width() - gap_total) / galleys.len() as f32).max(1.0);
         vec![each; galleys.len()]
      } else {
         galleys.iter().map(|galley| galley.size().x + 2.0 * pad_x).collect()
      };

      let strip_width: f32 =
         widths.iter().sum::<f32>() + gap * (galleys.len().saturating_sub(1) as f32);
      let (rect, mut response) = ui.allocate_exact_size(vec2(strip_width, height), Sense::hover());

      if !ui.is_rect_visible(rect) {
         return response;
      }

      let shade = |color: Color32| {
         if enabled {
            color
         } else {
            color.gamma_multiply(DISABLED_ALPHA)
         }
      };
      let inset = (rect.height() * INSET_RATIO).round().max(1.0);
      let indicator_radius =
         CornerRadius::same((f32::from(radius.nw) - inset).max(MIN_RADIUS).round() as u8);

      painter.rect_filled(rect, radius, shade(visuals.track));
      if !visuals.track_border.is_empty() {
         let border = Stroke::new(
            visuals.track_border.width,
            shade(visuals.track_border.color),
         );
         painter.rect_stroke(rect, radius, border, StrokeKind::Inside);
      }

      if galleys.is_empty() {
         return response;
      }

      // ---- segments ------------------------------------------------------
      let mut segments = Vec::with_capacity(widths.len());
      let mut x = rect.left();
      for width in &widths {
         segments.push(Rect::from_min_size(
            pos2(x, rect.top()),
            vec2(*width, rect.height()),
         ));
         x += *width + gap;
      }

      let strip_id = ui.scope_id().with_salt(id_salt);

      // ---- interaction ---------------------------------------------------
      let mut faces = Vec::with_capacity(segments.len());
      let mut changed = false;

      for (index, label) in labels.iter().enumerate() {
         let segment = ui.interact(
            segments[index],
            strip_id.with(index),
            Sense::click(),
         );
         if segment.clicked() {
            *selected = index;
            changed = true;
         }
         segment
            .widget_info(|| WidgetInfo::selected(Role::Tab, enabled, index == *selected, *label));
         faces.push(InteractT::of(
            ui,
            segment.id.with("face"),
            segment.hovered(),
            segment.is_pointer_button_down_on(),
            motion,
         ));
      }

      // The clicks above already moved the index, so this is the selection of the frame being painted.
      let selected = (*selected).min(labels.len() - 1);

      // ---- hover wash ----------------------------------------------------
      for (index, face) in faces.iter().enumerate() {
         if index == selected {
            continue;
         }
         let wash = face.blend(0.0, HOVER_WASH, PRESS_WASH);
         if wash > 0.001 {
            painter.rect_filled(
               segments[index].shrink(inset),
               indicator_radius,
               shade(visuals.hover).gamma_multiply(wash),
            );
         }
      }

      // ---- indicator -----------------------------------------------------
      // Seeded at the selection: a strip that first appears is still, a moved one slides.
      let t = animate_from(
         ui,
         strip_id.with("indicator"),
         selected as f32,
         selected as f32,
         motion,
      )
      .clamp(0.0, (labels.len() - 1) as f32);
      // Between two neighbours only: the pill travels from one segment to the other and stretches
      // between them mid-flight, which reads as a segment sliding rather than a cell lighting up.
      let from = (t.floor() as usize).min(labels.len() - 1);
      let to = (from + 1).min(labels.len() - 1);
      let indicator_rect =
         segments[from].shrink(inset).lerp(segments[to].shrink(inset), t - from as f32);

      let lift = shadow.to_shadow(indicator_rect.height(), visuals.indicator, 1.0);
      if lift != Shadow::NONE {
         painter.add(lift.as_shape(indicator_rect, indicator_radius));
      }
      painter.rect_filled(
         indicator_rect,
         indicator_radius,
         shade(visuals.indicator),
      );

      // ---- labels --------------------------------------------------------
      for (index, galley) in galleys.iter().enumerate() {
         let ink = if index == selected {
            visuals.text_selected
         } else {
            visuals.text.lerp(visuals.text_hover, faces[index].hover)
         };
         let pos = segments[index].center() - galley.size() / 2.0;
         painter.galley(pos, galley.clone(), shade(ink));
      }

      if changed {
         response.mark_changed();
      }
      response
   }
}

#[cfg(test)]
mod tests {
   use super::*;
   use crate::theme::{Theme, ThemeKind};
   use egui::{
      Context, Event, FullOutput, Modifiers, PointerButton, Pos2, RawInput, Rect, Shape, pos2, vec2,
   };

   fn themed_ctx() -> Context {
      let ctx = Context::default();
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&ctx);
      ctx
   }

   fn frame(
      ctx: &Context,
      time: f64,
      events: Vec<Event>,
      mut body: impl FnMut(&mut Ui),
   ) -> FullOutput {
      let input = RawInput {
         events,
         time: Some(time),
         screen_rect: Some(Rect::from_min_size(
            pos2(0.0, 0.0),
            vec2(400.0, 300.0),
         )),
         ..Default::default()
      };
      let mut out = ctx.run_ui(input, |ui| body(ui));
      out.textures_delta.clear();
      out
   }

   fn press(pos: Pos2) -> Event {
      Event::PointerButton {
         pos,
         button: PointerButton::Primary,
         pressed: true,
         modifiers: Modifiers::NONE,
      }
   }

   fn release(pos: Pos2) -> Event {
      Event::PointerButton {
         pos,
         button: PointerButton::Primary,
         pressed: false,
         modifiers: Modifiers::NONE,
      }
   }

   /// Every painted rounded rect, recursing into shape vectors: a strip is a track, a wash and an
   /// indicator in one frame.
   fn rects(out: &FullOutput) -> Vec<(Rect, Color32)> {
      fn walk(shape: &Shape, into: &mut Vec<(Rect, Color32)>) {
         match shape {
            Shape::Rect(rect) => into.push((rect.rect, rect.fill)),
            Shape::Vec(shapes) => {
               for shape in shapes {
                  walk(shape, into);
               }
            }
            _ => {}
         }
      }

      let mut into = Vec::new();
      for clipped in &out.shapes {
         walk(&clipped.shape, &mut into);
      }
      into
   }

   /// The one indicator fill in `out`, or none.
   fn indicator(out: &FullOutput, fill: Color32) -> Option<Rect> {
      let mut found = rects(out)
         .into_iter()
         .filter(|(_, painted)| *painted == fill)
         .map(|(rect, _)| rect);
      let first = found.next();
      assert!(found.next().is_none(), "one indicator per frame");
      first
   }

   /// Blurred shadows in `out`: the lift is a `RectShape` with a blur, never a mesh.
   fn lifted(out: &FullOutput) -> usize {
      fn walk(shape: &Shape, count: &mut usize) {
         match shape {
            Shape::Rect(rect) if rect.blur_width > 0.0 => *count += 1,
            Shape::Vec(shapes) => {
               for shape in shapes {
                  walk(shape, count);
               }
            }
            _ => {}
         }
      }

      let mut count = 0;
      for clipped in &out.shapes {
         walk(&clipped.shape, &mut count);
      }
      count
   }

   #[test]
   fn clicking_a_segment_selects_it() {
      let ctx = themed_ctx();
      let mut selected = 0usize;
      let mut rect = Rect::ZERO;

      frame(&ctx, 0.0, vec![], |ui| {
         rect = Tabs::new("tabs", &mut selected, ["All", "Media"]).show(ui).rect;
      });

      let at = pos2(rect.left() + rect.width() * 0.75, rect.center().y);
      frame(&ctx, 0.1, vec![press(at)], |ui| {
         Tabs::new("tabs", &mut selected, ["All", "Media"]).show(ui);
      });
      frame(&ctx, 0.2, vec![release(at)], |ui| {
         Tabs::new("tabs", &mut selected, ["All", "Media"]).show(ui);
      });

      assert_eq!(
         selected, 1,
         "the second segment must take the selection"
      );
   }

   #[test]
   fn fill_width_stretches_to_the_row() {
      let ctx = themed_ctx();
      let mut wide = 0usize;
      let mut narrow = 0usize;
      let mut wide_rect = Rect::ZERO;
      let mut narrow_rect = Rect::ZERO;

      frame(&ctx, 0.0, vec![], |ui| {
         wide_rect = Tabs::new("wide", &mut wide, ["All", "Media"]).fill_width(true).show(ui).rect;
         narrow_rect = Tabs::new("narrow", &mut narrow, ["All", "Media"])
            .fill_width(false)
            .show(ui)
            .rect;
      });

      assert!(
         wide_rect.width() > 380.0,
         "fill_width must use the row"
      );
      assert!(narrow_rect.width() < wide_rect.width());
   }

   #[test]
   fn an_empty_strip_does_not_panic() {
      let ctx = themed_ctx();
      let mut selected = 0usize;
      frame(&ctx, 0.0, vec![], |ui| {
         Tabs::new("empty", &mut selected, [] as [&str; 0]).show(ui);
      });
   }

   /// First sight is a state, not an entrance: the indicator is seeded on the selected segment and the
   /// strip asks for no repaint.
   #[test]
   fn a_first_sight_strip_is_still() {
      let ctx = themed_ctx();
      let highlight = Theme::new(ThemeKind::TokyoNight).colors.highlight;
      let mut selected = 2usize;
      let mut strip = Rect::ZERO;
      let mut idle = false;

      // Two warm-up frames absorb egui's own first-frame repaint request.
      frame(&ctx, 9.8, vec![], |_ui| {});
      frame(&ctx, 9.9, vec![], |_ui| {});
      let out = frame(&ctx, 10.0, vec![], |ui| {
         strip = Tabs::new("still", &mut selected, ["A", "B", "C"])
            .fill_width(false)
            .show(ui)
            .rect;
         idle = !ui.ctx().has_requested_repaint();
      });

      let painted = indicator(&out, highlight).expect("the indicator is painted");
      assert!(
         painted.center().x > strip.center().x,
         "the seeded indicator must sit on the third segment: {painted:?} in {strip:?}"
      );
      assert!(
         idle,
         "a first-sight strip must not request a repaint"
      );
   }

   /// A moved selection slides the indicator: it leaves the old segment after the click frame and lands
   /// on the new one, it does not jump.
   #[test]
   fn the_indicator_slides_to_the_new_segment() {
      let ctx = themed_ctx();
      let highlight = Theme::new(ThemeKind::TokyoNight).colors.highlight;
      let mut selected = 0usize;
      let mut strip = Rect::ZERO;

      frame(&ctx, 10.0, vec![], |ui| {
         strip = Tabs::new("slide", &mut selected, ["A", "B", "C"])
            .fill_width(false)
            .show(ui)
            .rect;
      });

      let at = pos2(strip.right() - 10.0, strip.center().y);
      let start = indicator(
         &frame(&ctx, 10.1, vec![], |ui| {
            Tabs::new("slide", &mut selected, ["A", "B", "C"]).fill_width(false).show(ui);
         }),
         highlight,
      )
      .expect("indicator");

      frame(&ctx, 10.2, vec![press(at)], |ui| {
         Tabs::new("slide", &mut selected, ["A", "B", "C"]).fill_width(false).show(ui);
      });
      let on_click = indicator(
         &frame(&ctx, 10.3, vec![release(at)], |ui| {
            Tabs::new("slide", &mut selected, ["A", "B", "C"]).fill_width(false).show(ui);
         }),
         highlight,
      )
      .expect("indicator");
      assert_eq!(selected, 2, "the click selects the last segment");
      assert!(
         (on_click.left() - start.left()).abs() < 0.5,
         "the click frame must still paint the old position: {on_click:?} vs {start:?}"
      );

      // Half a 0.3 s leg in, the pill has travelled away from where it started.
      let mid = indicator(
         &frame(&ctx, 10.45, vec![], |ui| {
            Tabs::new("slide", &mut selected, ["A", "B", "C"]).fill_width(false).show(ui);
         }),
         highlight,
      )
      .expect("indicator");
      assert!(
         mid.left() > on_click.left() + 4.0,
         "the indicator must travel toward the new segment: {mid:?} from {on_click:?}"
      );

      let settled = indicator(
         &frame(&ctx, 10.9, vec![], |ui| {
            Tabs::new("slide", &mut selected, ["A", "B", "C"]).fill_width(false).show(ui);
         }),
         highlight,
      )
      .expect("indicator");
      assert!(
         settled.left() > mid.left(),
         "and it must arrive by travelling: {settled:?} from {mid:?}"
      );
      assert!(
         settled.left() > strip.center().x,
         "the settled indicator must sit on the right-hand segment: {settled:?}"
      );
   }

   /// Hover washes the segment under the pointer with the palette's hover colour, not the selection.
   #[test]
   fn hover_washes_the_segment_under_the_pointer() {
      let ctx = themed_ctx();
      let colors = Theme::new(ThemeKind::TokyoNight).colors;
      let mut selected = 0usize;
      let mut strip = Rect::ZERO;

      frame(&ctx, 10.0, vec![], |ui| {
         strip = Tabs::new("hover", &mut selected, ["A", "B"])
            .fill_width(false)
            .motion(Motion::instant())
            .show(ui)
            .rect;
      });

      let at = pos2(strip.right() - 10.0, strip.center().y);
      let out = frame(&ctx, 10.1, vec![Event::PointerMoved(at)], |ui| {
         Tabs::new("hover", &mut selected, ["A", "B"])
            .fill_width(false)
            .motion(Motion::instant())
            .show(ui);
      });

      let wash = colors.hover.gamma_multiply(HOVER_WASH);
      let fills = rects(&out);
      assert!(
         fills.iter().any(|(_, fill)| *fill == wash),
         "a hovered unselected segment must carry the hover wash"
      );
      assert_ne!(
         wash, colors.highlight,
         "the wash must never read as the selection fill"
      );
      assert_eq!(
         fills.iter().filter(|(_, fill)| *fill == colors.highlight).count(),
         1,
         "only the indicator carries the selection fill"
      );
   }

   /// The lift is a blurred rect under the indicator; turning it off keeps the pill.
   #[test]
   fn a_flat_strip_drops_the_lift() {
      let ctx = themed_ctx();
      let highlight = Theme::new(ThemeKind::TokyoNight).colors.highlight;
      let mut selected = 0usize;

      let lifted_out = frame(&ctx, 10.0, vec![], |ui| {
         Tabs::new("lifted", &mut selected, ["A", "B"]).show(ui);
      });
      assert!(
         lifted(&lifted_out) >= 1,
         "a themed strip lifts the indicator"
      );

      let flat_out = frame(&ctx, 10.1, vec![], |ui| {
         Tabs::new("flat", &mut selected, ["A", "B"]).shadow(ToggleShadow::NONE).show(ui);
      });
      assert_eq!(
         lifted(&flat_out),
         0,
         "ToggleShadow::NONE removes the lift"
      );
      assert!(
         indicator(&flat_out, highlight).is_some(),
         "and the indicator is still painted"
      );
   }

   /// Every palette reads: the selected ink is the crate's `ink_on` rule, never the surface ink, and
   /// muted ink differs from hover ink.
   #[test]
   fn the_default_visuals_follow_every_palette() {
      for kind in ThemeKind::to_vec() {
         let colors = Theme::new(kind).colors;
         let visuals = TabsVisuals::from_theme(colors);
         assert_eq!(visuals.indicator, colors.highlight);
         assert_eq!(
            visuals.text_selected,
            colors.ink_on(colors.highlight)
         );
         assert_ne!(
            visuals.text, visuals.text_hover,
            "muted ink and hover ink must differ"
         );
         assert_ne!(
            visuals.track, visuals.indicator,
            "the track and the selected segment must not collapse"
         );
      }
   }
}
