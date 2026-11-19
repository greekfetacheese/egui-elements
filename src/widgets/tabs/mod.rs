//! Tab strips in two styles: a segmented control, and an underline bar.
//!
//! [TabsStyle::Segmented] is the measured reference segmented control
//! (<https://uiverse.io/uiverse-astronaut/odd-bobcat-3>, MIT): a fully rounded track that holds the
//! segments, the selected one painted as a pill on a soft shadow, and every state change eased rather
//! than switched. The reference's 6-of-48 ring around the pill becomes the strip's inset, so one
//! [Tabs::height] scales the whole strip. The indicator does not fade in place: one pill travels from
//! the old segment to the new one and stretches between the two while it is in flight, which is what
//! reads as a segmented control rather than as a highlighted cell.
//!
//! [TabsStyle::Underline] is the page-tab shape measured from the BoardUI tabs reference
//! (<https://www.boardui.com/components/tabs>): fitted tabs on a one-point baseline that spans the
//! row, the selected tab's ink in the accent colour, and a two-point accent bar sitting on the
//! baseline under exactly that tab. The bar travels to its neighbour the same way the pill does.
//!
//! Both styles keep one rule for the click: the segment under the pointer keeps its hover (or press)
//! wash until the indicator has landed on it, so a click never falls back to the bare surface while
//! the motion is still running.
//!
//! Every colour comes from the palette. The segmented track is the canvas ([ThemeColors::bg]), its
//! pill is [ThemeColors::highlight] with [ThemeColors::ink_on] ink, and hover washes the segment with
//! [ThemeColors::hover] and lifts its ink toward [ThemeColors::text] — never `highlight`, which stays
//! the selection. The underline strip's baseline is [ThemeColors::border], its bar and selected ink
//! are [ThemeColors::accent]. The segmented lift is the toggle family's [ToggleShadow], so a strip and
//! a switch on one card cast the same kind of shadow.

use std::hash::Hash;
use std::sync::Arc;

use egui::{
   Color32, CornerRadius, EventFilter, FocusDirection, FontId, Galley, Id, IdSalt, Key, Rect,
   Response, Role, Sense, Stroke, Ui, UiBuilder, WidgetInfo, pos2, vec2,
};

use crate::motion::{InteractT, Lerp, Motion, animate_from};
use crate::theme::{Theme, ThemeColors};
use crate::widgets::toggle::ToggleShadow;

mod segmented;
mod underline;

/// Hover wash alpha at full hover.
const HOVER_WASH: f32 = 0.55;
/// Hover wash alpha while the pointer is down on a segment.
const PRESS_WASH: f32 = 0.85;
/// Alpha multiplier applied to a disabled strip.
const DISABLED_ALPHA: f32 = 0.5;
/// An indicator closer than this to the selected segment has landed.
const SETTLED: f32 = 0.001;

/// Mute a colour on a disabled strip.
pub(super) fn shade(color: Color32, enabled: bool) -> Color32 {
   if enabled {
      color
   } else {
      color.gamma_multiply(DISABLED_ALPHA)
   }
}

/// The tab a focused tab's arrow key steps to, if any.
///
/// Only the tab that holds the keyboard listens, and the lock filter keeps egui from spending the
/// arrow on its own focus navigation, the way a slider claims its axis. The strip has no wrapping
/// model — a click can only land on a segment that exists — so the ends clamp and a press there
/// does nothing.
fn arrow_step(ui: &Ui, segment: Id, index: usize, len: usize) -> Option<usize> {
   if !ui.memory(|memory| memory.has_focus(segment)) {
      return None;
   }

   ui.memory_mut(|memory| {
      memory.set_focus_lock_filter(
         segment,
         EventFilter {
            horizontal_arrows: true,
            ..Default::default()
         },
      );
   });

   let (back, forward) = ui.input(|input| {
      (
         input.key_pressed(Key::ArrowLeft),
         input.key_pressed(Key::ArrowRight),
      )
   });
   let target = match (back, forward) {
      (true, false) => Some(index.saturating_sub(1)),
      (false, true) => Some((index + 1).min(len.saturating_sub(1))),
      _ => None,
   };

   if target.is_some() {
      // Drop the focus move egui queued for this frame before it can step a second tab, the way a
      // menu claims the arrows it reads itself.
      ui.memory_mut(|memory| memory.move_focus(FocusDirection::None));
   }

   target
}

/// The measured, unpainted strip handed to a style module.
#[derive(Clone, Copy)]
pub(super) struct Strip<'a> {
   /// The whole strip: the track (segmented) or the baseline row (underline).
   pub rect: Rect,
   /// One hit rect per label, left to right.
   pub segments: &'a [Rect],
   /// Each segment's eased hover and press weights.
   pub faces: &'a [InteractT],
   pub visuals: TabsVisuals,
   pub shadow: ToggleShadow,
   /// The segment the index points at, already clamped to the labels.
   pub selected: usize,
   /// The eased indicator position, in segment units.
   pub t: f32,
   pub corner_radius: CornerRadius,
   /// The label size the strip is built on.
   pub font_size: f32,
   pub enabled: bool,
}

/// How a [Tabs] strip paints.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TabsStyle {
   /// A pill track with one indicator that slides between segments. The default, and the strip
   /// [TabsStyle::Segmented] has always painted.
   #[default]
   Segmented,
   /// Fitted tabs on a full-width baseline, the selected one marked by an accent underline.
   Underline,
}

impl TabsStyle {
   /// The palette-derived visuals this style starts from.
   ///
   /// A strip that sets no [Tabs::visuals] uses this; [TabsVisuals::from_theme] is the segmented
   /// answer and [TabsVisuals::underline_from_theme] the underline one.
   pub fn visuals(self, colors: ThemeColors) -> TabsVisuals {
      match self {
         TabsStyle::Segmented => TabsVisuals::from_theme(colors),
         TabsStyle::Underline => TabsVisuals::underline_from_theme(colors),
      }
   }

   /// Space below the labels this style keeps for its own chrome.
   fn reserve(self, font_size: f32) -> f32 {
      match self {
         TabsStyle::Segmented => 0.0,
         TabsStyle::Underline => underline::reserve(font_size),
      }
   }
}

/// Per-widget colours for a [Tabs] strip.
///
/// [Default] is inert — a transparent track with no ink and [ToggleShadow::NONE] — so a struct literal
/// starts from [TabsVisuals::from_theme] (or [TabsVisuals::underline_from_theme]) and overrides the one
/// slot it wants to change.
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
   /// Fill of the track the segments sit on. Unused by [TabsStyle::Underline].
   pub track: Color32,
   /// Outline drawn inside the track. An underline strip reads only its colour and width, as the
   /// baseline the accent bar sits on.
   pub track_border: Stroke,
   /// Wash behind a hovered, unselected segment.
   pub hover: Color32,
   /// Fill of the sliding indicator: the segmented pill, or the underline bar.
   pub indicator: Color32,
   /// Ink of an unselected segment.
   pub text: Color32,
   /// Ink of an unselected segment under the pointer.
   pub text_hover: Color32,
   /// Ink of the selected segment.
   pub text_selected: Color32,
   /// The lift under the segmented indicator; [ToggleShadow::NONE] makes it flat. An underline strip
   /// ignores it.
   pub shadow: ToggleShadow,
}

impl TabsVisuals {
   /// The segmented style's palette-derived defaults, also used when a widget sets no visuals.
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

   /// The underline style's palette-derived defaults: a [ThemeColors::border] baseline, an
   /// [ThemeColors::accent] bar and selected ink, no track and no lift.
   pub fn underline_from_theme(colors: ThemeColors) -> Self {
      Self {
         track: Color32::TRANSPARENT,
         track_border: Stroke::new(1.0, colors.border),
         hover: colors.hover,
         indicator: colors.accent,
         text: colors.text_muted,
         text_hover: colors.text,
         text_selected: colors.accent,
         shadow: ToggleShadow::NONE,
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

/// A horizontal tab strip: one selected segment, index written back.
///
/// The strip eases the indicator between segments and, in the segmented style, lifts it on a shadow;
/// the duration comes from the installed style's `animation_time`, so [Tabs::motion] is the per-widget
/// override and [Motion::instant] the reduce-motion answer.
///
/// A focused tab answers the horizontal arrows the way the ARIA tabs pattern asks: Left and Right
/// move the selection to the neighbouring tab and take the keyboard focus with them. The strip has
/// no wrapping model, so the first and the last tab stay put.
///
/// ```rust
/// # use egui::__run_test_ui;
/// # use egui_elements::widgets::{Tabs, TabsStyle};
/// # __run_test_ui(|ui| {
/// let mut selected = 0;
/// Tabs::new("kind", &mut selected, ["All", "Media", "Files"]).show(ui);
/// Tabs::new("page", &mut selected, ["Banners", "Posts", "Members"])
///    .style(TabsStyle::Underline)
///    .show(ui);
/// # });
/// ```
#[must_use = "You should call .show()"]
pub struct Tabs<'a> {
   id_salt: IdSalt,
   selected: &'a mut usize,
   labels: Vec<&'a str>,
   fill_width: bool,
   height: Option<f32>,
   gap: Option<f32>,
   visuals: Option<TabsVisuals>,
   shadow: Option<ToggleShadow>,
   motion: Option<Motion>,
   corner_radius: Option<CornerRadius>,
   text_size: Option<f32>,
   style: TabsStyle,
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
         gap: None,
         visuals: None,
         shadow: None,
         motion: None,
         corner_radius: None,
         text_size: None,
         style: TabsStyle::Segmented,
      }
   }

   /// Paint the segmented track (default) or an underline bar.
   ///
   /// The style picks the default [TabsVisuals] (an underline strip starts on [ThemeColors::accent]
   /// over [ThemeColors::border], with no lift), the default gap, and the strip height that leaves
   /// room under the labels for the underline.
   pub fn style(mut self, style: TabsStyle) -> Self {
      self.style = style;
      self
   }

   /// Stretch the strip across the available width. Default `true`.
   ///
   /// Segmented tabs split the row evenly; underline tabs are always fitted to their labels, and this
   /// only decides whether the baseline spans the row (as the reference does) or stops at the last tab.
   pub fn fill_width(mut self, fill: bool) -> Self {
      self.fill_width = fill;
      self
   }

   /// Label size in points. Default: [`Typography::normal`](crate::theme::Typography::normal).
   ///
   /// The strip's default height follows the label size, so a compact strip needs this and nothing
   /// else.
   pub fn text_size(mut self, size: f32) -> Self {
      self.text_size = Some(size.max(1.0));
      self
   }

   /// Strip height in points.
   ///
   /// Default: body text plus the button padding, plus the underline's own room in the underline
   /// style.
   pub fn height(mut self, height: f32) -> Self {
      self.height = Some(height);
      self
   }

   /// Space between segments, in points. Default `2.0` segmented, `ThemeSpacing::md` underlined.
   pub fn gap(mut self, gap: f32) -> Self {
      self.gap = Some(gap);
      self
   }

   /// Replace the palette-derived colours.
   pub fn visuals(mut self, visuals: TabsVisuals) -> Self {
      self.visuals = Some(visuals);
      self
   }

   /// Replace the lift under the segmented indicator; [ToggleShadow::NONE] turns it off.
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

   /// Corner radius of the segmented track. Default `u8::MAX`, which clamps to a pill.
   ///
   /// The indicator's radius is this radius minus the inset, so a pill track always holds a pill.
   /// The underline style ignores it.
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
         text_size,
         style,
      } = self;

      let theme = Theme::current(ui.ctx());
      let colors = theme.colors;
      let visuals = visuals.unwrap_or_else(|| style.visuals(colors));
      let shadow = shadow.unwrap_or(visuals.shadow);
      let motion = motion.unwrap_or_else(|| Motion::from_ui(ui));
      let enabled = ui.is_enabled();
      let pad_x = ui.spacing().button_padding.x.max(10.0);
      let pad_y = ui.spacing().button_padding.y.max(6.0);
      let font_size = text_size.unwrap_or(theme.typography.normal);
      let font = FontId::proportional(font_size);
      let height = height.unwrap_or(font_size + 2.0 * pad_y + style.reserve(font_size));
      let radius = corner_radius.unwrap_or(CornerRadius::same(u8::MAX));
      let gap = gap.unwrap_or(match style {
         TabsStyle::Segmented => 2.0,
         TabsStyle::Underline => theme.spacing.md,
      });

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
      } else if fill_width && style == TabsStyle::Segmented {
         let gap_total = gap * (galleys.len() - 1) as f32;
         let each = ((ui.available_width() - gap_total) / galleys.len() as f32).max(1.0);
         vec![each; galleys.len()]
      } else {
         galleys.iter().map(|galley| galley.size().x + 2.0 * pad_x).collect()
      };

      let tabs_width: f32 =
         widths.iter().sum::<f32>() + gap * (galleys.len().saturating_sub(1) as f32);
      // Underline tabs stay fitted whatever `fill_width` says; the flag only decides whether the
      // baseline runs on to the edge of the row.
      let strip_width = if galleys.is_empty() {
         0.0
      } else if style == TabsStyle::Underline && fill_width {
         ui.available_width().max(tabs_width)
      } else {
         tabs_width
      };
      let (rect, mut response) = ui.allocate_exact_size(vec2(strip_width, height), Sense::hover());

      if !ui.is_rect_visible(rect) || galleys.is_empty() {
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

      // The strip is one list, not a row of loose tabs: a child Ui over the strip's rect that holds
      // nothing but the segment interactions, so every tab node hangs under a single tab-list node
      // instead of under whatever happens to contain the strip. It claims no space (the strip rect
      // above is already allocated) and paints nothing of its own.
      let strip_ui = ui.new_child(UiBuilder::new().scope_id(strip_id.with("list")).max_rect(rect));
      strip_ui.ctx().accesskit_node_builder(strip_ui.unique_id(), |node| {
         node.set_role(Role::TabList)
      });

      // ---- interaction ---------------------------------------------------
      let mut faces = Vec::with_capacity(segments.len());
      let mut changed = false;
      let mut focused = None;

      for (index, label) in labels.iter().enumerate() {
         let segment = strip_ui.interact(
            segments[index],
            strip_id.with(index),
            Sense::click(),
         );
         if segment.clicked() {
            *selected = index;
            changed = true;
         }
         if segment.has_focus() {
            focused = Some(index);
         }
         let chosen = index == *selected;
         segment.widget_info(|| WidgetInfo::selected(Role::Tab, enabled, chosen, *label));
         // `WidgetInfo::selected` writes the toggled state; ARIA spells the same bit `selected`,
         // and a tab needs both, so the second is set from the same value rather than re-derived.
         ui.ctx().accesskit_node_builder(segment.id, |node| node.set_selected(chosen));
         faces.push(InteractT::of(
            ui,
            segment.id.with("face"),
            segment.hovered(),
            segment.is_pointer_button_down_on(),
            motion,
         ));
      }

      // A tab holding the keyboard steps the selection with the arrows and takes the focus with it.
      // Handled after the loop, so a focus request made here cannot be read back as focus gained by
      // a later segment of this same frame.
      if let Some(index) = focused
         && let Some(target) = arrow_step(ui, strip_id.with(index), index, labels.len())
         && target != index
      {
         *selected = target;
         changed = true;
         ui.memory_mut(|memory| memory.request_focus(strip_id.with(target)));
      }

      // The clicks above already moved the index, so this is the selection of the frame being painted.
      let selected = (*selected).min(labels.len() - 1);

      // Seeded at the selection: a strip that first appears is still, a moved one slides.
      let t = animate_from(
         ui,
         strip_id.with("indicator"),
         selected as f32,
         selected as f32,
         motion,
      )
      .clamp(0.0, (labels.len() - 1) as f32);

      let strip = Strip {
         rect,
         segments: &segments,
         faces: &faces,
         visuals,
         shadow,
         selected,
         t,
         corner_radius: radius,
         font_size,
         enabled,
      };
      let band = match style {
         TabsStyle::Segmented => segmented::paint(ui, &strip),
         TabsStyle::Underline => underline::paint(ui, &strip),
      };

      // ---- labels --------------------------------------------------------
      for (index, galley) in galleys.iter().enumerate() {
         let ink = if index == selected {
            visuals.text_selected
         } else {
            visuals.text.lerp(visuals.text_hover, faces[index].hover)
         };
         let pos = pos2(
            segments[index].center().x - galley.size().x / 2.0,
            band.center().y - galley.size().y / 2.0,
         );
         ui.painter().galley(pos, galley.clone(), shade(ink, enabled));
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
   use crate::theme::{Theme, ThemeColors, ThemeKind};
   use egui::accesskit::Toggled;
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

   fn key(key: Key) -> Event {
      Event::Key {
         key,
         physical_key: None,
         pressed: true,
         repeat: false,
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

   /// The hover wash a palette paints at full hover.
   fn wash(colors: ThemeColors) -> Color32 {
      colors.hover.gamma_multiply(HOVER_WASH)
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
         Tabs::new("empty_line", &mut selected, [] as [&str; 0])
            .style(TabsStyle::Underline)
            .show(ui);
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

      let wash = wash(colors);
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

   /// A click moves the index on the release frame, but the indicator still has a whole leg to travel:
   /// the clicked segment must keep its wash until the indicator lands on it, or the strip falls back
   /// to the bare track for that leg.
   #[test]
   fn a_clicked_segment_keeps_its_wash_until_the_indicator_lands() {
      let ctx = themed_ctx();
      let colors = Theme::new(ThemeKind::TokyoNight).colors;
      let mut selected = 0usize;
      let mut strip = Rect::ZERO;

      frame(&ctx, 10.0, vec![], |ui| {
         strip = Tabs::new("retain", &mut selected, ["A", "B", "C"])
            .fill_width(false)
            .show(ui)
            .rect;
      });

      let at = pos2(strip.right() - 8.0, strip.center().y);
      // The pointer settles on the last segment while it is still unselected.
      frame(&ctx, 10.1, vec![Event::PointerMoved(at)], |ui| {
         Tabs::new("retain", &mut selected, ["A", "B", "C"]).fill_width(false).show(ui);
      });
      let hovered = frame(&ctx, 10.5, vec![], |ui| {
         Tabs::new("retain", &mut selected, ["A", "B", "C"]).fill_width(false).show(ui);
      });
      assert!(
         rects(&hovered).iter().any(|(_, fill)| *fill == wash(colors)),
         "a hovered unselected segment carries the wash"
      );

      frame(&ctx, 10.6, vec![press(at)], |ui| {
         Tabs::new("retain", &mut selected, ["A", "B", "C"]).fill_width(false).show(ui);
      });
      let clicked = frame(&ctx, 10.7, vec![release(at)], |ui| {
         Tabs::new("retain", &mut selected, ["A", "B", "C"]).fill_width(false).show(ui);
      });
      assert_eq!(
         selected, 2,
         "the last segment takes the selection"
      );
      let fills = rects(&clicked);
      assert!(
         fills
            .iter()
            .any(|(rect, fill)| *fill == wash(colors) && rect.center().x > strip.center().x + 8.0),
         "the clicked segment keeps its wash while the indicator is on its way: {fills:?}"
      );

      // One animation leg later the indicator has landed, and it replaces the wash.
      let landed = frame(&ctx, 11.2, vec![], |ui| {
         Tabs::new("retain", &mut selected, ["A", "B", "C"]).fill_width(false).show(ui);
      });
      assert!(
         !rects(&landed).iter().any(|(_, fill)| *fill == wash(colors)),
         "the landed indicator replaces the wash"
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

   /// The underline style: a border rule along the bottom of the strip, and one accent bar sitting on
   /// it under the selected tab.
   #[test]
   fn the_underline_style_paints_a_baseline_and_a_bar() {
      let ctx = themed_ctx();
      let colors = Theme::new(ThemeKind::TokyoNight).colors;
      let mut selected = 1usize;
      let mut strip = Rect::ZERO;

      let out = frame(&ctx, 10.0, vec![], |ui| {
         strip = Tabs::new("line", &mut selected, ["All", "Media", "Files"])
            .style(TabsStyle::Underline)
            .fill_width(false)
            .show(ui)
            .rect;
      });

      let fills = rects(&out);
      let baseline = fills
         .iter()
         .find(|(rect, fill)| *fill == colors.border && rect.height() <= 1.5)
         .map(|(rect, _)| *rect)
         .expect("the baseline is painted");
      assert!(
         (baseline.bottom() - strip.bottom()).abs() < 0.5,
         "the rule is the last row"
      );
      assert!(
         (baseline.width() - strip.width()).abs() < 0.5,
         "a fitted baseline spans the whole strip: {baseline:?} in {strip:?}"
      );

      let bar = indicator(&out, colors.accent).expect("one accent bar");
      assert!(
         (bar.bottom() - strip.bottom()).abs() < 0.5,
         "the bar sits on the rule"
      );
      assert!(
         bar.height() >= 2.0 && bar.height() <= 3.0,
         "a two-point bar, scaled from the label size: {bar:?}"
      );
      assert!(
         bar.left() > strip.left() && bar.right() < strip.right(),
         "the bar is under the selected tab, not the row: {bar:?}"
      );
      assert_eq!(
         lifted(&out),
         0,
         "the underline style casts no lift"
      );
   }

   /// `fill_width` decides how far the baseline runs, never how wide a tab is: underline tabs stay
   /// fitted to their labels.
   #[test]
   fn an_underline_strip_runs_its_baseline_across_the_row() {
      let ctx = themed_ctx();
      let colors = Theme::new(ThemeKind::TokyoNight).colors;
      let mut selected = 0usize;
      let mut wide = Rect::ZERO;
      let mut narrow = Rect::ZERO;

      let wide_out = frame(&ctx, 10.0, vec![], |ui| {
         wide = Tabs::new("wide_line", &mut selected, ["All", "Media"])
            .style(TabsStyle::Underline)
            .show(ui)
            .rect;
      });
      let narrow_out = frame(&ctx, 10.1, vec![], |ui| {
         narrow = Tabs::new("narrow_line", &mut selected, ["All", "Media"])
            .style(TabsStyle::Underline)
            .fill_width(false)
            .show(ui)
            .rect;
      });

      assert!(
         wide.width() > 380.0,
         "the default baseline uses the row"
      );
      assert!(
         narrow.width() < wide.width() * 0.5,
         "a fitted one stops at the tabs"
      );
      let wide_bar = indicator(&wide_out, colors.accent).expect("bar");
      let narrow_bar = indicator(&narrow_out, colors.accent).expect("bar");
      assert!(
         (wide_bar.width() - narrow_bar.width()).abs() < 0.5,
         "a tab keeps its fitted width either way: {wide_bar:?} vs {narrow_bar:?}"
      );
   }

   /// The underline travels between segments like the pill does.
   #[test]
   fn the_underline_slides_to_the_new_segment() {
      let ctx = themed_ctx();
      let accent = Theme::new(ThemeKind::TokyoNight).colors.accent;
      let mut selected = 0usize;
      let mut strip = Rect::ZERO;

      frame(&ctx, 10.0, vec![], |ui| {
         strip = Tabs::new("line_slide", &mut selected, ["A", "B", "C"])
            .style(TabsStyle::Underline)
            .fill_width(false)
            .show(ui)
            .rect;
      });

      let at = pos2(strip.right() - 8.0, strip.center().y);
      let start = indicator(
         &frame(&ctx, 10.1, vec![], |ui| {
            Tabs::new("line_slide", &mut selected, ["A", "B", "C"])
               .style(TabsStyle::Underline)
               .fill_width(false)
               .show(ui);
         }),
         accent,
      )
      .expect("bar");

      frame(&ctx, 10.2, vec![press(at)], |ui| {
         Tabs::new("line_slide", &mut selected, ["A", "B", "C"])
            .style(TabsStyle::Underline)
            .fill_width(false)
            .show(ui);
      });
      let on_click = indicator(
         &frame(&ctx, 10.3, vec![release(at)], |ui| {
            Tabs::new("line_slide", &mut selected, ["A", "B", "C"])
               .style(TabsStyle::Underline)
               .fill_width(false)
               .show(ui);
         }),
         accent,
      )
      .expect("bar");
      assert_eq!(selected, 2, "the click selects the last tab");
      assert!(
         (on_click.left() - start.left()).abs() < 0.5,
         "the click frame still paints the old tab: {on_click:?} vs {start:?}"
      );

      let mid = indicator(
         &frame(&ctx, 10.45, vec![], |ui| {
            Tabs::new("line_slide", &mut selected, ["A", "B", "C"])
               .style(TabsStyle::Underline)
               .fill_width(false)
               .show(ui);
         }),
         accent,
      )
      .expect("bar");
      assert!(
         mid.left() > on_click.left() + 4.0,
         "the bar must travel: {mid:?} from {on_click:?}"
      );

      let settled = indicator(
         &frame(&ctx, 10.9, vec![], |ui| {
            Tabs::new("line_slide", &mut selected, ["A", "B", "C"])
               .style(TabsStyle::Underline)
               .fill_width(false)
               .show(ui);
         }),
         accent,
      )
      .expect("bar");
      assert!(
         settled.left() > strip.center().x,
         "the settled bar sits under the last tab: {settled:?}"
      );
   }

   /// The underline style reads the accent, the border rule and no lift from every palette.
   #[test]
   fn the_underline_visuals_follow_every_palette() {
      for kind in ThemeKind::to_vec() {
         let colors = Theme::new(kind).colors;
         let visuals = TabsVisuals::underline_from_theme(colors);
         assert_eq!(visuals.indicator, colors.accent);
         assert_eq!(visuals.text_selected, colors.accent);
         assert_eq!(
            visuals.track_border,
            Stroke::new(1.0, colors.border)
         );
         assert_eq!(visuals.shadow, ToggleShadow::NONE);
         assert_ne!(
            visuals.text, visuals.text_hover,
            "muted ink and hover ink must differ"
         );
         assert_ne!(
            visuals.indicator, visuals.text,
            "the accent bar must read against muted ink"
         );
      }
   }

   /// The strip is a tab list in the accessibility tree: one `TabList` node that owns the tab
   /// nodes, and a tab whose `selected` and `toggled` states agree.
   #[test]
   fn the_strip_reports_a_tab_list() {
      let ctx = themed_ctx();
      ctx.enable_accesskit();
      let mut selected = 1usize;

      let mut out = frame(&ctx, 10.0, vec![], |ui| {
         Tabs::new("list", &mut selected, ["All", "Media"]).show(ui);
      });
      let update = out
         .platform_output
         .accesskit_update
         .take()
         .expect("an AccessKit-enabled frame must produce a tree update");

      let (_, list) = update
         .nodes
         .iter()
         .find(|(_, node)| node.role() == Role::TabList)
         .expect("the strip must report a tab list");

      let mut tabs = 0;
      let mut selected_tabs = 0;
      for (id, node) in update.nodes.iter().filter(|(_, node)| node.role() == Role::Tab) {
         tabs += 1;
         assert!(
            list.children().contains(id),
            "every tab must hang under the tab list"
         );
         if node.is_selected() == Some(true) {
            selected_tabs += 1;
            assert_eq!(
               node.toggled(),
               Some(Toggled::True),
               "a tab's selected and toggled states must agree"
            );
         }
      }

      assert_eq!(tabs, 2, "one node per segment");
      assert_eq!(
         selected_tabs, 1,
         "exactly one tab carries the selection"
      );
      assert_eq!(
         selected, 1,
         "reporting the list must not move the index"
      );
   }

   /// A focused tab steps the selection with the horizontal arrows and takes the keyboard focus
   /// along, so the next press lands on the next tab. The strip does not wrap: the ends clamp.
   #[test]
   fn the_arrows_step_a_focused_tab() {
      let ctx = themed_ctx();
      let mut selected = 0usize;
      let mut changed = false;

      // Tab into the strip: the segments are the only focusable widgets on the row.
      frame(&ctx, 10.0, vec![key(Key::Tab)], |ui| {
         Tabs::new("arrows", &mut selected, ["All", "Media", "Files"]).show(ui);
      });
      // The lock filter is stored on the first frame the tab is focused and read on the next, as a
      // slider's is, so give it that frame before pressing an arrow.
      frame(&ctx, 10.1, vec![], |ui| {
         Tabs::new("arrows", &mut selected, ["All", "Media", "Files"]).show(ui);
      });

      frame(&ctx, 10.2, vec![key(Key::ArrowRight)], |ui| {
         changed = Tabs::new("arrows", &mut selected, ["All", "Media", "Files"]).show(ui).changed();
      });
      assert_eq!(
         selected, 1,
         "ArrowRight moves the selection one tab on"
      );
      assert!(changed, "a moved selection reports `changed`");

      frame(&ctx, 10.3, vec![key(Key::ArrowRight)], |ui| {
         changed = Tabs::new("arrows", &mut selected, ["All", "Media", "Files"]).show(ui).changed();
      });
      assert_eq!(
         selected, 2,
         "the focus must travel with the selection, or the next press could not land"
      );
      assert!(changed, "and the second press moves again");

      frame(&ctx, 10.4, vec![key(Key::ArrowRight)], |ui| {
         changed = Tabs::new("arrows", &mut selected, ["All", "Media", "Files"]).show(ui).changed();
      });
      assert_eq!(
         selected, 2,
         "the last tab clamps instead of wrapping"
      );
      assert!(
         !changed,
         "a press that cannot move reports nothing"
      );

      frame(&ctx, 10.5, vec![key(Key::ArrowLeft)], |ui| {
         changed = Tabs::new("arrows", &mut selected, ["All", "Media", "Files"]).show(ui).changed();
      });
      assert_eq!(selected, 1, "ArrowLeft steps back");
      assert!(changed, "stepping back reports `changed` too");
   }

   #[test]
   fn text_size_sets_the_strip_height() {
      let ctx = themed_ctx();
      let mut selected = 0;
      let mut normal = Rect::NOTHING;
      let mut compact = Rect::NOTHING;

      frame(&ctx, 0.0, vec![], |ui| {
         normal = Tabs::new("normal_size", &mut selected, ["All", "Media"]).show(ui).rect;
      });
      frame(&ctx, 0.1, vec![], |ui| {
         compact = Tabs::new("compact_size", &mut selected, ["All", "Media"])
            .text_size(9.0)
            .show(ui)
            .rect;
      });

      assert!(
         compact.height() < normal.height(),
         "a smaller label must lower the strip"
      );
   }
}
