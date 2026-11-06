//! A themed slider, ported from the Web Awesome [`wa-slider`].
//!
//! The control is the reference's: a fully rounded rail, a circular thumb with a thin surface ring,
//! an optional label above, optional reference captions below and an optional formatted value beside
//! the rail. Every measurement is a ratio of [`Slider::size`] (the thumb's diameter), so one knob
//! scales the whole widget: the rail is 0.444 x the thumb, the ring 0.111 x, a step marker 0.222 x.
//!
//! `no_run
//! # use egui::__run_test_ui;
//! # use egui_elements::widgets::Slider;
//! # __run_test_ui(|ui| {
//! let mut users = 3.0;
//! ui.add(
//!    Slider::new(&mut users, 0.0..=6.0)
//!        .text("Number of users")
//!        .hint("Limit six per team")
//!        .references(["Less", "More"])
//!        .step_by(1.0)
//!        .markers(true),
//! );
//! # });
//! `
//!
//! The value beside the rail is an editable [`egui::DragValue`], like the stock slider's, and its
//! cell is placed by the crate's [`Flex`] row so it stays centred on the rail. The whole control is
//! laid out from the style's spacing: the label, rail row, references and hint are one vertical
//! stack, and a vertical slider swaps the rail for a column.
//!
//! [`wa-slider`]: https://webawesome.com/docs/components/slider/

use core::ops::RangeInclusive;
use std::cell::Cell;
use std::sync::Arc;

use egui::text::Galley;
use egui::{
   Align, Color32, CornerRadius, CursorIcon, DragValue, EventFilter, Key, Layout, Pos2, Rect,
   Response, Sense, SliderOrientation, Stroke, StrokeKind, TextStyle, TextWrapMode, Ui, UiBuilder,
   Widget, WidgetInfo, WidgetText, emath, pos2, vec2,
};

use crate::layout::{CrossAlign, Flex, Item};
use crate::motion::{InteractT, Motion};
use crate::theme::{Theme, ThemeColors};

/// Default thumb diameter in points; the reference is 18 at 1x.
const DEFAULT_SIZE: f32 = 18.0;
/// Surface ring around the thumb, as a fraction of its diameter (2 pt at 18).
const THUMB_RING_RATIO: f32 = 0.111;
/// Rail thickness, as a fraction of the thumb diameter (8 pt at 18).
const TRACK_RATIO: f32 = 0.444;
/// Step marker diameter, as a fraction of the thumb diameter (4 pt at 18).
const MARKER_RATIO: f32 = 0.222;
/// Height of the rail row, as a fraction of the thumb diameter; leaves room for the grown thumb.
const ROW_RATIO: f32 = 1.55;
/// Extra thumb radius while hovered, as a fraction of the thumb diameter.
const HOVER_GROW: f32 = 0.04;
/// Extra thumb radius while pressed.
const PRESS_GROW: f32 = 0.08;
/// Alpha multiplier applied to a disabled slider.
const DISABLED_ALPHA: f32 = 0.5;
/// Most step markers painted; a slider with more steps than this would read as a dotted line.
const MAX_MARKERS: usize = 64;

/// The value written beside the rail, and returned to the formatter.
type Formatter<'a> = Arc<dyn Fn(f64, RangeInclusive<usize>) -> String + 'a>;

/// Per-widget colours for a [`Slider`].
///
/// Every default derives from the installed palette: the rail is the widget background pulled
/// toward the border, the fill and the thumb are [`ThemeColors::accent`], and the ring and the
/// markers are the page background, so they read as holes punched in the rail.
///
/// `no_run
/// # use egui::{__run_test_ui, Color32};
/// # use egui_elements::theme::{Theme, ThemeKind};
/// # use egui_elements::widgets::{Slider, SliderVisuals};
/// # let colors = Theme::new(ThemeKind::TokyoNight).colors;
/// # __run_test_ui(|ui| {
/// # let mut value = 0.5;
/// let visuals = SliderVisuals {
///    thumb: Color32::WHITE,
///    ..SliderVisuals::from_theme(colors)
/// };
/// ui.add(Slider::new(&mut value, 0.0..=1.0).visuals(visuals));
/// # });
/// `
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SliderVisuals {
   /// The unfilled rail.
   pub rail: Color32,
   /// The filled part of the rail.
   pub fill: Color32,
   /// The thumb's disc.
   pub thumb: Color32,
   /// The thin surface ring between the thumb and the rail.
   pub thumb_ring: Color32,
   /// The step markers.
   pub marker: Color32,
   /// The label above the rail.
   pub text: Color32,
   /// The references and the hint below the rail.
   pub text_muted: Color32,
}

impl SliderVisuals {
   /// The colours a slider derives from an installed palette.
   pub fn from_theme(colors: ThemeColors) -> Self {
      Self {
         rail: colors.widget_bg.lerp_to_gamma(colors.border, 0.55),
         fill: colors.accent,
         thumb: colors.accent,
         thumb_ring: colors.bg,
         marker: colors.bg,
         text: colors.text,
         text_muted: colors.text_muted,
      }
   }
}

/// A rail with a draggable thumb, a formatted value and optional captions.
///
/// [`Slider::new`] borrows the value like [`egui::Slider`], so `changed()` on the returned
/// [`Response`] is the only signal a host needs; the keyboard adjusts a focused slider and the
/// number beside the rail is editable by dragging or by clicking it.
#[must_use = "Sliders should be shown on a ui, e.g. ui.add(Slider::new(&mut value, 0.0..=1.0))"]
pub struct Slider<'a> {
   get_set: Box<dyn FnMut(Option<f64>) -> f64 + 'a>,
   range: RangeInclusive<f64>,
   step: Option<f64>,
   orientation: SliderOrientation,
   size: Option<f32>,
   width: Option<f32>,
   height: Option<f32>,
   text: Option<WidgetText>,
   hint: Option<WidgetText>,
   references: Vec<WidgetText>,
   show_value: bool,
   show_markers: bool,
   show_tooltip: bool,
   formatter: Option<Formatter<'a>>,
   min_decimals: usize,
   max_decimals: Option<usize>,
   decimals_explicit: bool,
   visuals: Option<SliderVisuals>,
   motion: Option<Motion>,
}

impl<'a> Slider<'a> {
   /// A horizontal slider over a borrowed number.
   ///
   /// Integral types default to a step of `1.0` and no decimals, exactly like [`egui::Slider`].
   pub fn new<Num: emath::Numeric>(
      value: &'a mut Num,
      range: impl Into<RangeInclusive<Num>>,
   ) -> Self {
      let range = range.into();
      let range_f64 = range.start().to_f64()..=range.end().to_f64();
      let mut slider = Self::from_get_set(range_f64, move |new_value: Option<f64>| {
         if let Some(new_value) = new_value {
            *value = Num::from_f64(new_value);
         }
         value.to_f64()
      });

      if Num::INTEGRAL {
         slider.step = Some(1.0);
         slider.min_decimals = 0;
         slider.max_decimals = Some(0);
      }

      slider
   }

   /// A slider over a value that cannot be borrowed directly.
   ///
   /// The closure reads the value when called with `None`, writes it when called with `Some`, and
   /// returns the value either way.
   pub fn from_get_set(
      range: RangeInclusive<f64>,
      get_set: impl 'a + FnMut(Option<f64>) -> f64,
   ) -> Self {
      Self {
         get_set: Box::new(get_set),
         range,
         step: None,
         orientation: SliderOrientation::Horizontal,
         size: None,
         width: None,
         height: None,
         text: None,
         hint: None,
         references: Vec::new(),
         show_value: true,
         show_markers: false,
         show_tooltip: false,
         formatter: None,
         min_decimals: 0,
         max_decimals: Some(2),
         decimals_explicit: false,
         visuals: None,
         motion: None,
      }
   }

   /// The label above the rail.
   pub fn text(mut self, text: impl Into<WidgetText>) -> Self {
      self.text = Some(text.into());
      self
   }

   /// A muted description below the rail.
   pub fn hint(mut self, hint: impl Into<WidgetText>) -> Self {
      self.hint = Some(hint.into());
      self
   }

   /// Contextual captions below the rail, spread with CSS `space-between`: the first sits on the
   /// rail's start, the last on its end, and the rest are evenly spaced between them.
   pub fn references<I, T>(mut self, references: I) -> Self
   where
      I: IntoIterator<Item = T>,
      T: Into<WidgetText>,
   {
      self.references = references.into_iter().map(Into::into).collect();
      self
   }

   /// Which way the rail runs. Default [`SliderOrientation::Horizontal`].
   pub fn orientation(mut self, orientation: SliderOrientation) -> Self {
      self.orientation = orientation;
      self
   }

   /// A vertical slider: the value grows towards the top and the rail uses [`Slider::height`].
   pub fn vertical(mut self) -> Self {
      self.orientation = SliderOrientation::Vertical;
      self
   }

   /// The thumb's diameter in points. Default `18.0`; the rail, ring, markers and row are ratios of
   /// it, so one call scales the whole control.
   pub fn size(mut self, size: f32) -> Self {
      self.size = Some(size);
      self
   }

   /// The control's total width in points, the value included. Default: the available width.
   pub fn width(mut self, width: f32) -> Self {
      self.width = Some(width);
      self
   }

   /// The rail's length in points on a vertical slider. Defaults to the style's `slider_width`.
   pub fn height(mut self, height: f32) -> Self {
      self.height = Some(height);
      self
   }

   /// Show the value beside the rail. Default `true`; the cell is aligned to the rail by a
   /// [`Flex`] row.
   pub fn show_value(mut self, show_value: bool) -> Self {
      self.show_value = show_value;
      self
   }

   /// Paint a marker at every interior step. Only meaningful with a step: the markers are skipped
   /// when the range holds more than 64 of them.
   pub fn markers(mut self, markers: bool) -> Self {
      self.show_markers = markers;
      self
   }

   /// Show the current value in a tooltip over the thumb while hovering, dragging or focused.
   pub fn tooltip(mut self, tooltip: bool) -> Self {
      self.show_tooltip = tooltip;
      self
   }

   /// Quantise the value to `step`, including the keyboard and the number field.
   pub fn step_by(mut self, step: f64) -> Self {
      self.step = Some(step);
      self
   }

   /// Format the value: the formatter receives the value and the allowed decimal range, and its
   /// output is used by both the number beside the rail and the tooltip.
   pub fn custom_formatter(
      mut self,
      formatter: impl 'a + Fn(f64, RangeInclusive<usize>) -> String,
   ) -> Self {
      self.formatter = Some(Arc::new(formatter));
      self
   }

   /// At least this many decimals in the value. Default `0`.
   pub fn min_decimals(mut self, min_decimals: usize) -> Self {
      self.min_decimals = min_decimals;
      self.decimals_explicit = true;
      self
   }

   /// At most this many decimals in the value, or `None` to let the widget pick.
   pub fn max_decimals(mut self, max_decimals: usize) -> Self {
      self.max_decimals = Some(max_decimals);
      self.decimals_explicit = true;
      self
   }

   /// Exactly this many decimals.
   pub fn fixed_decimals(mut self, decimals: usize) -> Self {
      self.min_decimals = decimals;
      self.max_decimals = Some(decimals);
      self.decimals_explicit = true;
      self
   }

   /// Replace the palette-derived colours.
   pub fn visuals(mut self, visuals: SliderVisuals) -> Self {
      self.visuals = Some(visuals);
      self
   }

   /// How the thumb's hover and press grow animate. Defaults to [`Motion::from_ui`];
   /// [`Motion::instant`] snaps.
   pub fn motion(mut self, motion: Motion) -> Self {
      self.motion = Some(motion);
      self
   }
}

impl Widget for Slider<'_> {
   fn ui(self, ui: &mut Ui) -> Response {
      let Self {
         mut get_set,
         range,
         step,
         orientation,
         size,
         width,
         height,
         text,
         hint,
         references,
         show_value,
         show_markers,
         show_tooltip,
         formatter,
         min_decimals,
         max_decimals,
         decimals_explicit,
         visuals,
         motion,
      } = self;

      let colors = Theme::colors_from_ctx(ui.ctx());
      let visuals = visuals.unwrap_or_else(|| SliderVisuals::from_theme(colors));
      let enabled = ui.is_enabled();
      let motion = motion.unwrap_or_else(|| Motion::from_ui(ui));

      let thumb_size = size.unwrap_or(DEFAULT_SIZE).max(1.0);
      let thumb_outer = thumb_size * (1.0 + 2.0 * THUMB_RING_RATIO);
      let thickness = thumb_size * TRACK_RATIO;
      let marker = thumb_size * MARKER_RATIO;
      let row_height = (thumb_size * ROW_RATIO).max(ui.spacing().interact_size.y);

      let (lo, hi) = bounds(&range);
      let step_decimals = step.map(decimals_of).unwrap_or(0);
      // A step decides the written precision unless the caller pinned the decimals.
      let (min_decimals, max_decimals) = if decimals_explicit {
         (min_decimals, max_decimals)
      } else if let Some(step) = step {
         let decimals = decimals_of(step);
         (decimals, Some(decimals))
      } else {
         (min_decimals, max_decimals)
      };
      let old_value = get_set(None);
      let value = Cell::new(old_value);

      let shade = |color: Color32| {
         if enabled {
            color
         } else {
            color.gamma_multiply(DISABLED_ALPHA)
         }
      };

      let label_text = text.as_ref().map(|text| text.text().to_owned()).unwrap_or_default();
      let available = ui.available_width();
      let available = if available.is_finite() && available > 1.0 {
         available
      } else {
         ui.spacing().slider_width + thumb_outer
      };
      let spacing = ui.spacing().item_spacing;

      let value_width = if show_value {
         value_cell_width(
            ui,
            formatter.as_ref(),
            lo,
            hi,
            old_value,
            min_decimals,
            max_decimals,
         )
      } else {
         0.0
      };
      let value_gap = if show_value { spacing.x } else { 0.0 };

      let (cell_width, cell_height, column_width) = match orientation {
         SliderOrientation::Horizontal => {
            let wanted = width.unwrap_or(available).min(available);
            let rail = (wanted - value_width - value_gap).max(thumb_outer);
            (rail, row_height, rail)
         }
         SliderOrientation::Vertical => {
            let column = width
               .unwrap_or_else(|| available.min(ui.spacing().slider_width.max(thumb_outer)))
               .min(available)
               .max(thumb_outer);
            let rail = height.unwrap_or(ui.spacing().slider_width);
            (thumb_outer, rail, column)
         }
      };
      let text_width = if orientation == SliderOrientation::Horizontal {
         cell_width
      } else {
         column_width
      };

      let label = text.map(|text| {
         themed_galley(
            ui,
            text,
            TextWrapMode::Wrap,
            text_width,
            TextStyle::Body,
            shade(visuals.text),
         )
      });
      let hint = hint.map(|text| {
         themed_galley(
            ui,
            text,
            TextWrapMode::Wrap,
            text_width,
            TextStyle::Small,
            shade(visuals.text_muted),
         )
      });
      let refs: Vec<Arc<Galley>> = references
         .into_iter()
         .map(|text| {
            themed_galley(
               ui,
               text,
               TextWrapMode::Extend,
               f32::INFINITY,
               TextStyle::Small,
               shade(visuals.text_muted),
            )
         })
         .collect();

      let label_height = label.as_ref().map_or(0.0, |galley| galley.size().y);
      let refs_height = refs.iter().map(|galley| galley.size().y).fold(0.0, f32::max);
      let hint_height = hint.as_ref().map_or(0.0, |galley| galley.size().y);
      let small_gap = (spacing.y * 0.5).max(2.0);
      let gap_label = if label_height > 0.0 { spacing.y } else { 0.0 };
      let gap_refs = if refs_height > 0.0 { small_gap } else { 0.0 };
      let gap_hint = if hint_height > 0.0 { small_gap } else { 0.0 };

      let row_width = cell_width + value_gap + value_width;
      let total_width = if orientation == SliderOrientation::Horizontal {
         row_width
      } else {
         column_width.max(row_width)
      };
      let total_height =
         label_height + gap_label + cell_height + gap_refs + refs_height + gap_hint + hint_height;
      let (rect, hover) = ui.allocate_exact_size(vec2(total_width, total_height), Sense::hover());

      let left = rect.left();
      let mut top = rect.top();
      if let Some(galley) = &label {
         ui.painter().galley(
            pos2(left, top),
            galley.clone(),
            shade(visuals.text),
         );
         top += galley.size().y + gap_label;
      }

      let row_rect = Rect::from_min_size(pos2(left, top), vec2(row_width, cell_height));
      top += cell_height + gap_refs;

      let value_formatter = formatter.clone();
      let points_to_value =
         ((range.end() - range.start()).abs() / cell_width.max(1.0) as f64).max(f64::MIN_POSITIVE);

      let mut row_ui = ui
         .new_child(UiBuilder::new().max_rect(row_rect).layout(Layout::left_to_right(Align::Min)));

      let track = Flex::row(hover.id.with("row"))
         .gap(value_gap)
         .align_items(CrossAlign::Center)
         .fill_main(false)
         .show(&mut row_ui, |flex| {
            let track = flex
               .item(Item::fixed(cell_width), |ui| {
                  ui.allocate_exact_size(
                     vec2(cell_width, cell_height),
                     Sense::click_and_drag(),
                  )
               })
               .inner
               .1;

            if show_value {
               let _ = flex.item(Item::fixed(value_width), |ui| {
                  let mut local = value.get();
                  let mut drag = DragValue::new(&mut local)
                     .range(lo..=hi)
                     .speed(points_to_value)
                     .min_decimals(min_decimals)
                     .max_decimals_opt(max_decimals);

                  if let Some(formatter) = value_formatter.clone() {
                     drag =
                        drag.custom_formatter(move |value, decimals| formatter(value, decimals));
                  }

                  let response = ui.add(drag);
                  let snapped = snap_to_step(local, lo, step, step_decimals);
                  if snapped != value.get() {
                     value.set(snapped);
                  }
                  response
               });
            }

            track
         })
         .inner;

      let track_cell = track.rect;
      let rail = match orientation {
         SliderOrientation::Horizontal => Rect::from_center_size(
            track_cell.center(),
            vec2(track_cell.width(), thickness),
         ),
         SliderOrientation::Vertical => Rect::from_center_size(
            track_cell.center(),
            vec2(thickness, track_cell.height()),
         ),
      };

      let mut response = track;
      if enabled {
         if response.clicked() {
            response.request_focus();
         }

         if let Some(pointer) = response.interact_pointer_pos() {
            let raw = value_at(orientation, pointer, rail, thumb_outer, &range);
            let snapped = snap_to_step(raw, lo, step, step_decimals);
            if snapped != value.get() {
               value.set(snapped);
            }
         }

         if let Some(change) = keyboard_change(ui, &response, orientation) {
            let span = (hi - lo).max(f64::MIN_POSITIVE);
            let key_step = step.unwrap_or(span / 100.0);
            let current = value.get();
            let next = match change {
               KeyChange::Decrement => snap_to_step(current - key_step, lo, step, step_decimals),
               KeyChange::Increment => snap_to_step(current + key_step, lo, step, step_decimals),
               KeyChange::Start => lo,
               KeyChange::End => hi,
            };
            if next != current {
               value.set(next);
            }
         }
      }

      let current = value.get();
      let centre = thumb_centre(orientation, rail, thumb_outer, &range, current);

      if ui.is_rect_visible(rect) {
         let across = InteractT::of(
            ui,
            response.id.with("across"),
            response.hovered() || response.dragged(),
            response.is_pointer_button_down_on(),
            motion,
         );
         let grow = 1.0 + HOVER_GROW * across.hover + PRESS_GROW * across.press;
         let painter = ui.painter();
         let radius =
            CornerRadius::same((thickness * 0.5).round().clamp(0.0, u8::MAX as f32) as u8);

         painter.rect_filled(rail, radius, shade(visuals.rail));
         let fill = fill_rect(orientation, rail, centre);
         if fill.width() > 0.5 && fill.height() > 0.5 {
            painter.rect_filled(fill, radius, shade(visuals.fill));
         }

         if show_markers && let Some(step) = step.filter(|step| *step > 0.0) {
            let markers = ((hi - lo) / step).round() as usize;
            if markers > 1 && markers <= MAX_MARKERS {
               for index in 1..markers {
                  let marker_value = lo + step * index as f64;
                  let at = thumb_centre(
                     orientation,
                     rail,
                     thumb_outer,
                     &range,
                     marker_value,
                  );
                  painter.circle_filled(at, marker * 0.5, shade(visuals.marker));
               }
            }
         }

         if response.has_focus() {
            painter.rect_stroke(
               rail.expand(3.0),
               CornerRadius::same(radius.nw.saturating_add(3)),
               Stroke::new(1.0, shade(visuals.fill).gamma_multiply(0.7)),
               StrokeKind::Outside,
            );
         }

         painter.circle_filled(
            centre,
            thumb_outer * 0.5 * grow,
            shade(visuals.thumb_ring),
         );
         painter.circle_filled(
            centre,
            thumb_size * 0.5 * grow,
            shade(visuals.thumb),
         );
      }

      paint_references(
         ui,
         &refs,
         RefRow {
            left,
            top,
            width: text_width,
         },
         shade(visuals.text_muted),
      );
      top += refs_height + gap_hint;
      if let Some(galley) = &hint {
         ui.painter().galley(
            pos2(left, top),
            galley.clone(),
            shade(visuals.text_muted),
         );
      }

      if show_tooltip && (response.hovered() || response.dragged() || response.has_focus()) {
         let text = format_value(
            ui,
            formatter.as_ref(),
            current,
            min_decimals,
            max_decimals,
         );
         let font = TextStyle::Small.resolve(ui.style());
         let galley = ui.painter().layout_no_wrap(text, font, shade(visuals.text));
         let padding = vec2(7.0, 3.0);
         let size = galley.size() + padding * 2.0;
         let at = match orientation {
            SliderOrientation::Horizontal => pos2(centre.x, rail.top() - 6.0 - size.y * 0.5),
            SliderOrientation::Vertical => pos2(rail.right() + 6.0 + size.x * 0.5, centre.y),
         };
         let bubble = Rect::from_center_size(at, size);
         let radius = CornerRadius::same((size.y * 0.3).round() as u8);
         ui.painter().rect_filled(bubble, radius, shade(colors.bg));
         ui.painter().rect_stroke(
            bubble,
            radius,
            Stroke::new(1.0, shade(colors.border)),
            StrokeKind::Inside,
         );
         ui.painter().galley(bubble.min + padding, galley, shade(visuals.text));
      }

      if current != old_value {
         get_set(Some(current));
         response.mark_changed();
      }

      let info_label = label_text.clone();
      response.widget_info(|| WidgetInfo::slider(enabled, current, &info_label));
      response = response.on_hover_cursor(if enabled {
         match orientation {
            SliderOrientation::Horizontal => CursorIcon::ResizeHorizontal,
            SliderOrientation::Vertical => CursorIcon::ResizeVertical,
         }
      } else {
         CursorIcon::Default
      });

      response.union(hover)
   }
}

/// What a keyboard press asks the slider to do.
#[derive(Clone, Copy)]
enum KeyChange {
   Decrement,
   Increment,
   Start,
   End,
}

/// The arrow keys of a focused slider, matching its orientation.
fn keyboard_change(
   ui: &Ui,
   response: &Response,
   orientation: SliderOrientation,
) -> Option<KeyChange> {
   if !response.has_focus() {
      return None;
   }

   ui.memory_mut(|memory| {
      memory.set_focus_lock_filter(
         response.id,
         EventFilter {
            horizontal_arrows: orientation == SliderOrientation::Horizontal,
            vertical_arrows: orientation == SliderOrientation::Vertical,
            ..Default::default()
         },
      );
   });

   ui.input(|input| {
      if input.key_pressed(Key::Home) {
         return Some(KeyChange::Start);
      }
      if input.key_pressed(Key::End) {
         return Some(KeyChange::End);
      }

      match orientation {
         SliderOrientation::Horizontal => {
            if input.key_pressed(Key::ArrowRight) {
               Some(KeyChange::Increment)
            } else if input.key_pressed(Key::ArrowLeft) {
               Some(KeyChange::Decrement)
            } else {
               None
            }
         }
         SliderOrientation::Vertical => {
            if input.key_pressed(Key::ArrowUp) {
               Some(KeyChange::Increment)
            } else if input.key_pressed(Key::ArrowDown) {
               Some(KeyChange::Decrement)
            } else {
               None
            }
         }
      }
   })
}

/// The horizontal band the reference captions occupy.
struct RefRow {
   left: f32,
   top: f32,
   width: f32,
}

/// Paint the captions spread with CSS `space-between`: first at the start, last at the end.
fn paint_references(ui: &Ui, refs: &[Arc<Galley>], row: RefRow, color: Color32) {
   let RefRow { left, top, width } = row;
   if refs.is_empty() {
      return;
   }

   let natural: f32 = refs.iter().map(|galley| galley.size().x).sum();
   let spacers = refs.len().saturating_sub(1).max(1);
   let spacing = ((width - natural) / spacers as f32).max(0.0);
   let height = refs.iter().map(|galley| galley.size().y).fold(0.0, f32::max);
   let mut x = left;
   for galley in refs {
      ui.painter().galley(
         pos2(x, top + (height - galley.size().y) * 0.5),
         galley.clone(),
         color,
      );
      x += galley.size().x + spacing;
   }
}

/// Ascending bounds of a range that may run high-to-low.
fn bounds(range: &RangeInclusive<f64>) -> (f64, f64) {
   let (start, end) = (*range.start(), *range.end());
   if start <= end {
      (start, end)
   } else {
      (end, start)
   }
}

/// Where `value` sits on the rail, `0.0` at the range's start and `1.0` at its end.
fn fraction(value: f64, range: &RangeInclusive<f64>) -> f32 {
   let (start, end) = (*range.start(), *range.end());
   if start == end {
      return 0.0;
   }
   (((value - start) / (end - start)) as f32).clamp(0.0, 1.0)
}

/// Round `value` to `step` above `base`, then to `decimals` so a fractional step leaves no dust.
fn snap_to_step(value: f64, base: f64, step: Option<f64>, decimals: usize) -> f64 {
   match step.filter(|step| *step > 0.0) {
      Some(step) => emath::round_to_decimals(
         ((value - base) / step).round() * step + base,
         decimals,
      ),
      None => value,
   }
}

/// How many decimals `step` needs to be written exactly: `0.25` -> 2.
fn decimals_of(step: f64) -> usize {
   format!("{step}").split_once('.').map_or(0, |(_, fraction)| {
      fraction.trim_end_matches('0').len()
   })
}

/// The value a pointer position asks for.
fn value_at(
   orientation: SliderOrientation,
   pointer: Pos2,
   rail: Rect,
   thumb_outer: f32,
   range: &RangeInclusive<f64>,
) -> f64 {
   let inset = thumb_outer * 0.5;
   let t = match orientation {
      SliderOrientation::Horizontal => {
         let start = rail.left() + inset;
         let end = rail.right() - inset;
         let span = end - start;
         if span.abs() < 1.0 {
            0.0
         } else {
            ((pointer.x - start) / span).clamp(0.0, 1.0)
         }
      }
      SliderOrientation::Vertical => {
         let start = rail.bottom() - inset;
         let end = rail.top() + inset;
         let span = end - start;
         if span.abs() < 1.0 {
            0.0
         } else {
            ((pointer.y - start) / span).clamp(0.0, 1.0)
         }
      }
   };
   let start = *range.start();
   let end = *range.end();
   start + (end - start) * t as f64
}

/// Where the thumb's centre belongs for `value`.
fn thumb_centre(
   orientation: SliderOrientation,
   rail: Rect,
   thumb_outer: f32,
   range: &RangeInclusive<f64>,
   value: f64,
) -> Pos2 {
   let t = fraction(value, range);
   let inset = thumb_outer * 0.5;
   match orientation {
      SliderOrientation::Horizontal => pos2(
         emath::lerp((rail.left() + inset)..=(rail.right() - inset), t),
         rail.center().y,
      ),
      SliderOrientation::Vertical => pos2(
         rail.center().x,
         emath::lerp((rail.bottom() - inset)..=(rail.top() + inset), t),
      ),
   }
}

/// The filled part of the rail, from its start to the thumb.
fn fill_rect(orientation: SliderOrientation, rail: Rect, centre: Pos2) -> Rect {
   match orientation {
      SliderOrientation::Horizontal => Rect::from_min_max(
         pos2(rail.left(), rail.top()),
         pos2(centre.x.max(rail.left()), rail.bottom()),
      ),
      SliderOrientation::Vertical => Rect::from_min_max(
         pos2(rail.left(), centre.y.min(rail.bottom())),
         pos2(rail.right(), rail.bottom()),
      ),
   }
}

/// A galley laid out in `color`: plain text bakes the style's ink, so the override is what lets
/// [`SliderVisuals::text`] repaint a label on a filled surface.
fn themed_galley(
   ui: &mut Ui,
   text: WidgetText,
   wrap: TextWrapMode,
   width: f32,
   style: TextStyle,
   color: Color32,
) -> Arc<Galley> {
   let previous = ui.visuals().override_text_color;
   ui.visuals_mut().override_text_color = Some(color);
   let galley = text.into_galley(ui, Some(wrap), width, style);
   ui.visuals_mut().override_text_color = previous;
   galley
}

/// The value as the number field and the tooltip write it.
fn format_value(
   ui: &Ui,
   formatter: Option<&Formatter<'_>>,
   value: f64,
   min_decimals: usize,
   max_decimals: Option<usize>,
) -> String {
   let decimals = min_decimals..=max_decimals.unwrap_or(min_decimals.max(2));
   match formatter {
      Some(formatter) => formatter(value, decimals),
      None => ui.style().number_formatter.format(value, decimals),
   }
}

/// The width the value's cell reserves: the widest of the range's ends and the current value.
fn value_cell_width(
   ui: &Ui,
   formatter: Option<&Formatter<'_>>,
   lo: f64,
   hi: f64,
   current: f64,
   min_decimals: usize,
   max_decimals: Option<usize>,
) -> f32 {
   let font = ui.style().drag_value_text_style.resolve(ui.style());
   let mut width: f32 = 0.0;
   for sample in [lo, hi, current] {
      let text = format_value(ui, formatter, sample, min_decimals, max_decimals);
      let galley = ui.painter().layout_no_wrap(text, font.clone(), Color32::WHITE);
      width = width.max(galley.size().x);
   }
   (width + 2.0 * ui.spacing().button_padding.x + 4.0).max(ui.spacing().interact_size.x)
}

#[cfg(test)]
mod tests {
   use super::*;
   use crate::theme::{Theme, ThemeKind};
   use egui::{Context, Event, FullOutput, Key, Modifiers, PointerButton, RawInput, Shape, pos2};

   fn themed_ctx() -> Context {
      let ctx = Context::default();
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&ctx);
      ctx
   }

   /// One frame at an explicit `input.time` (no sleeps, no wall clock).
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

   fn shapes(out: &FullOutput) -> Vec<Shape> {
      let mut shapes = Vec::new();
      for clipped in &out.shapes {
         flatten(&clipped.shape, &mut shapes);
      }
      shapes
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

   fn texts(shapes: &[Shape]) -> Vec<egui::epaint::TextShape> {
      shapes
         .iter()
         .filter_map(|shape| match shape {
            Shape::Text(text) => Some(text.clone()),
            _ => None,
         })
         .collect()
   }

   /// The rail: the widest rectangle in the paint; the fill and the rail share its band.
   fn rail(shapes: &[Shape]) -> Rect {
      shapes
         .iter()
         .filter_map(|shape| match shape {
            Shape::Rect(rect) => Some(rect.rect),
            _ => None,
         })
         .filter(|rect| rect.height() < 40.0 && rect.width() > 20.0)
         .max_by(|a, b| a.width().total_cmp(&b.width()))
         .expect("the slider paints a rail")
   }

   /// Paint one slider with no interaction and return its shapes and rect.
   fn paint(ctx: &Context, mut build: impl FnMut(&mut Ui) -> Response) -> (Vec<Shape>, Rect) {
      let mut rect = Rect::ZERO;
      let out = frame(ctx, 0.0, vec![], |ui| {
         rect = build(ui).rect;
      });
      (shapes(&out), rect)
   }

   #[test]
   fn a_click_on_the_rail_moves_the_value_and_reports_changed() {
      let ctx = themed_ctx();
      let mut value = 0.0_f32;
      let (_, rect) = paint(&ctx, |ui| {
         ui.add(Slider::new(&mut value, 0.0..=1.0).width(200.0).show_value(false))
      });

      let at = pos2(rect.left() + rect.width() * 0.75, rect.center().y);
      let mut changed = false;
      frame(&ctx, 0.1, vec![press(at)], |ui| {
         changed = ui
            .add(Slider::new(&mut value, 0.0..=1.0).width(200.0).show_value(false))
            .changed();
      });

      assert!(
         (value - 0.79).abs() < 0.05,
         "the rail is inset by half a thumb; got {value}"
      );
      assert!(
         changed,
         "the frame that moves the value reports changed"
      );

      let before = value;
      frame(&ctx, 0.2, vec![release(at)], |ui| {
         ui.add(Slider::new(&mut value, 0.0..=1.0).width(200.0).show_value(false));
      });
      assert_eq!(
         value, before,
         "releasing does not move the value again"
      );
   }

   #[test]
   fn a_vertical_slider_grows_upwards() {
      let ctx = themed_ctx();
      let mut value = 0.0_f32;
      let (_, rect) = paint(&ctx, |ui| {
         ui.add(
            Slider::new(&mut value, 0.0..=100.0)
               .vertical()
               .height(150.0)
               .width(60.0)
               .show_value(false),
         )
      });

      let track_left = rect.left() + 18.0 * TRACK_RATIO;
      let at = pos2(track_left, rect.top() + 8.0);
      frame(&ctx, 0.1, vec![press(at)], |ui| {
         ui.add(
            Slider::new(&mut value, 0.0..=100.0)
               .vertical()
               .height(150.0)
               .width(60.0)
               .show_value(false),
         );
      });
      assert!(
         value > 90.0,
         "the top of a vertical rail is its high end; got {value}"
      );
   }

   #[test]
   fn markers_are_painted_at_the_interior_steps() {
      let ctx = themed_ctx();
      let mut value = 0_i32;
      let (shapes, _) = paint(&ctx, |ui| {
         ui.add(
            Slider::new(&mut value, 0..=4)
               .size(18.0)
               .width(240.0)
               .show_value(false)
               .markers(true),
         )
      });

      let expected = 18.0 * MARKER_RATIO * 0.5;
      let markers = circles(&shapes)
         .into_iter()
         .filter(|circle| (circle.radius - expected).abs() < 0.01)
         .count();
      assert_eq!(
         markers, 3,
         "interior steps of 0..=4 are 1, 2 and 3"
      );
   }

   #[test]
   fn size_scales_the_rail_and_the_thumb() {
      let ctx = themed_ctx();

      let mut small = 0.5_f32;
      let (small_shapes, _) = paint(&ctx, |ui| {
         ui.add(Slider::new(&mut small, 0.0..=1.0).size(18.0).show_value(false))
      });
      let mut large = 0.5_f32;
      let (large_shapes, _) = paint(&ctx, |ui| {
         ui.add(Slider::new(&mut large, 0.0..=1.0).size(36.0).show_value(false))
      });

      let small_rail = rail(&small_shapes).height();
      let large_rail = rail(&large_shapes).height();
      assert!(
         (small_rail - 18.0 * TRACK_RATIO).abs() < 0.6,
         "rail {small_rail}"
      );
      assert!(
         (large_rail - 36.0 * TRACK_RATIO).abs() < 0.6,
         "rail {large_rail}"
      );

      let small_thumb = circles(&small_shapes)
         .iter()
         .map(|circle| circle.radius)
         .fold(0.0_f32, f32::max);
      let large_thumb = circles(&large_shapes)
         .iter()
         .map(|circle| circle.radius)
         .fold(0.0_f32, f32::max);
      assert!(
         (small_thumb - 11.0).abs() < 0.3,
         "ring {small_thumb}"
      );
      assert!(
         (large_thumb - 22.0).abs() < 0.3,
         "ring {large_thumb}"
      );
   }

   #[test]
   fn references_span_the_rail_from_end_to_end() {
      let ctx = themed_ctx();
      let mut value = 1.0_f32;
      let (shapes, rect) = paint(&ctx, |ui| {
         ui.add(
            Slider::new(&mut value, 1.0..=5.0)
               .size(18.0)
               .width(300.0)
               .show_value(false)
               .references(["Slow", "Medium", "Fast"]),
         )
      });

      let refs: Vec<_> = texts(&shapes)
         .into_iter()
         .filter(|text| ["Slow", "Medium", "Fast"].contains(&text.galley.text()))
         .collect();
      assert_eq!(refs.len(), 3, "three references are painted");

      let first = refs.iter().find(|text| text.galley.text() == "Slow").unwrap();
      let last = refs.iter().find(|text| text.galley.text() == "Fast").unwrap();
      assert!(
         (first.pos.x - rect.left()).abs() < 1.0,
         "the first caption starts on the rail"
      );
      assert!(
         (last.pos.x + last.galley.size().x - rect.right()).abs() < 1.0,
         "the last caption ends on the rail"
      );
   }

   #[test]
   fn the_value_is_painted_after_the_rail() {
      let ctx = themed_ctx();
      let mut value = 40.0_f64;
      let (shapes, _) = paint(&ctx, |ui| {
         ui.add(
            Slider::new(&mut value, 0.0..=100.0)
               .width(300.0)
               .custom_formatter(|value, _| format!("{value:.0} units")),
         )
      });

      let rail = rail(&shapes);
      let painted = texts(&shapes)
         .into_iter()
         .find(|text| text.galley.text() == "40 units")
         .expect("the formatted value is painted");
      assert!(
         painted.pos.x > rail.right(),
         "the value sits after the rail: {} vs {}",
         painted.pos.x,
         rail.right()
      );
   }

   #[test]
   fn a_disabled_slider_does_not_move() {
      let ctx = themed_ctx();
      let mut value = 0.25_f32;
      let (_, rect) = paint(&ctx, |ui| {
         ui.add_enabled(
            false,
            Slider::new(&mut value, 0.0..=1.0).width(200.0).show_value(false),
         )
      });

      let at = pos2(rect.left() + rect.width() * 0.8, rect.center().y);
      frame(&ctx, 0.1, vec![press(at), release(at)], |ui| {
         ui.add_enabled(
            false,
            Slider::new(&mut value, 0.0..=1.0).width(200.0).show_value(false),
         );
      });
      assert_eq!(
         value, 0.25,
         "a disabled slider ignores the pointer"
      );
   }

   #[test]
   fn the_keyboard_steps_a_focused_slider() {
      let ctx = themed_ctx();
      let mut value = 0.0_f32;
      let (_, rect) = paint(&ctx, |ui| {
         ui.add(Slider::new(&mut value, 0.0..=1.0).width(200.0).step_by(0.1).show_value(false))
      });

      let at = pos2(rect.left() + rect.width() * 0.5, rect.center().y);
      frame(&ctx, 0.1, vec![press(at)], |ui| {
         ui.add(Slider::new(&mut value, 0.0..=1.0).width(200.0).step_by(0.1).show_value(false));
      });
      frame(&ctx, 0.2, vec![release(at)], |ui| {
         ui.add(Slider::new(&mut value, 0.0..=1.0).width(200.0).step_by(0.1).show_value(false));
      });
      let focused = value;
      frame(&ctx, 0.3, vec![key(Key::ArrowRight)], |ui| {
         ui.add(Slider::new(&mut value, 0.0..=1.0).width(200.0).step_by(0.1).show_value(false));
      });

      assert!(
         (value - (focused + 0.1)).abs() < 1e-4,
         "arrow-right adds one step: {focused} -> {value}"
      );
   }

   #[test]
   fn the_step_quantises_the_dragged_value() {
      let ctx = themed_ctx();
      let mut value = 0.0_f32;
      let (_, rect) = paint(&ctx, |ui| {
         ui.add(Slider::new(&mut value, 0.0..=1.0).width(200.0).step_by(0.25).show_value(false))
      });

      let at = pos2(rect.left() + rect.width() * 0.62, rect.center().y);
      frame(&ctx, 0.1, vec![press(at)], |ui| {
         ui.add(Slider::new(&mut value, 0.0..=1.0).width(200.0).step_by(0.25).show_value(false));
      });
      assert!(
         (value * 4.0).fract().abs() < 1e-4,
         "{value} is not a multiple of 0.25"
      );
   }

   #[test]
   fn the_thumb_and_the_pointer_agree_on_a_value() {
      let rail = Rect::from_min_size(pos2(10.0, 10.0), vec2(200.0, 20.0));
      let range = 0.0..=100.0;
      for value in [0.0, 25.0, 50.0, 100.0] {
         let centre = thumb_centre(
            SliderOrientation::Horizontal,
            rail,
            22.0,
            &range,
            value,
         );
         let back = value_at(
            SliderOrientation::Horizontal,
            centre,
            rail,
            22.0,
            &range,
         );
         assert!(
            (back - value).abs() < 0.5,
            "{value} came back as {back}"
         );
      }
   }

   #[test]
   fn an_unlabelled_slider_paints_no_text() {
      let ctx = themed_ctx();
      let mut value = 0.0_f32;
      let (shapes, _) = paint(&ctx, |ui| {
         ui.add(Slider::new(&mut value, 0.0..=1.0).width(200.0).show_value(false))
      });
      assert!(
         texts(&shapes).is_empty(),
         "no label, references or hint"
      );
   }
}
