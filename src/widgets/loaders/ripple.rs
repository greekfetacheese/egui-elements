//! A ripple of coloured cells — a 3 x 3 grid that lights up in reading order.
//!
//! Ported from [alexruix / fluffy-starfish-52](https://uiverse.io/alexruix/fluffy-starfish-52)
//! (MIT). The CSS lights one cell every 100 ms, ramps it up over 30 % of its 1.5 s loop, fades it
//! back over the next 30 % and leaves it transparent for the rest, so a wave runs across the grid
//! and the nine cells are never bright together. The delay ladder covers cells 1..=5 at 0..400 ms
//! and every later cell repeats the last four, which is why the reference capture shows the first
//! three cells dark-to-bright across its top row while two later cells share a shade.
//!
//! Each cell keeps its own colour, so a lit run reads as a hue ramp instead of one brightness pulse.
//!
//! ```
//! # use egui::__run_test_ui;
//! # use egui_elements::widgets::Ripple;
//! # __run_test_ui(|ui| {
//! ui.add(Ripple::new());
//! ui.add(Ripple::new().size(38.0));
//! # });
//! ```
//!
//! The theme ramp is not one palette slot: the accent only seeds it ([Ripple::from_theme]), because
//! the ported look is nine fully saturated swatches walking *hue*, which a single colour cannot
//! supply.

use egui::{Color32, Rect, Response, Role, Sense, Ui, Vec2, Widget, WidgetInfo, vec2};

use crate::motion::Easing;
use crate::theme::{Theme, ThemeColors};
use crate::utils::Hsla;

/// Cells in the grid: a 3 x 3.
const CELLS: usize = 9;
/// A gutter as a fraction of the grid: the ported 1 pt gutter on a 160 pt grid. Used at the grid's
/// edge as well, which is what leaves the outer cells inset by one gutter.
const GAP_RATIO: f32 = 0.0066;
/// Side of one cell as a fraction of the grid: three cells and their four gutters fill it exactly,
/// so the block is symmetric inside the allocated square.
const CELL_RATIO: f32 = (1.0 - 4.0 * GAP_RATIO) / 3.0;
/// Corner radius of a cell as a fraction of the grid: the ported 4 pt on 160 pt.
const CORNER_RATIO: f32 = 0.025;
/// Smallest corner radius, in points, so a small loader keeps rounded corners.
const MIN_CORNER: f32 = 1.0;
/// Side of the square the widget allocates, as a fraction of the style's `interact_size.y`.
const DEFAULT_SIZE_RATIO: f32 = 2.6;
/// Seconds of one full ripple.
const DEFAULT_PERIOD: f32 = 1.5;
/// Shortest loop, so a caller cannot divide the phase by zero.
const MIN_PERIOD: f32 = 0.05;
/// Step between two neighbouring cells, in seconds.
const DEFAULT_DELAY: f32 = 0.1;
/// Steps in the delay ladder before it repeats — the CSS `:nth-child(1..5)` rule.
const DELAYS: usize = 5;
/// Share of the loop a cell spends ramping up from transparent.
const FADE_IN: f32 = 0.30;
/// Share of the loop a cell spends ramping back down.
const FADE_OUT: f32 = 0.30;
/// Hue span of the ported ramp, in degrees (`#00FF87` at 152° to `#60EFFF` at 187°).
const RAMP_HUE: f32 = 35.0;
/// Lightness of the ramp's first cell, in percent.
const RAMP_LIGHTNESS: f32 = 50.0;
/// Lightness of the ramp's last cell, in percent — the ported `#60EFFF`.
const RAMP_LIGHTNESS_END: f32 = 69.0;
/// Saturation of the ramp, in percent: every ported cell is fully saturated.
const RAMP_SATURATION: f32 = 100.0;

/// A 3 x 3 grid of cells that light up in a wave.
///
/// The colours default to the theme ramp ([Ripple::from_theme]); pass
/// [`.color(...)`](Self::color) for one flat colour or [`.cells(...)`](Self::cells) for your own nine.
///
/// ```
/// # use egui::{__run_test_ui, Color32};
/// # use egui_elements::widgets::Ripple;
/// # __run_test_ui(|ui| {
/// ui.add(Ripple::new());                        // theme ramp
/// ui.add(Ripple::new().size(64.0).period(1.0)); // faster, larger
/// ui.add(Ripple::new().color(Color32::WHITE));  // one flat colour
/// # });
/// ```
#[derive(Clone, Debug, PartialEq)]
#[must_use = "Ripples should be shown on a ui, e.g. ui.add(Ripple::new())"]
pub struct Ripple {
   size: Option<f32>,
   cells: Option<[Color32; CELLS]>,
   period: f32,
   delay: f32,
   easing: Easing,
}

impl Ripple {
   /// A ripple in the theme ramp, sized to 2.6 x the style's `interact_size.y`.
   pub fn new() -> Self {
      Self {
         size: None,
         cells: None,
         period: DEFAULT_PERIOD,
         delay: DEFAULT_DELAY,
         easing: Easing::EaseOut,
      }
   }

   /// Side of the square the grid occupies. Defaults to 2.6 x the style's `interact_size.y`.
   #[inline]
   pub fn size(mut self, size: f32) -> Self {
      self.size = Some(size);
      self
   }

   /// Every cell the same colour. The grid still ripples; only the hue ramp is gone.
   #[inline]
   pub fn color(mut self, color: impl Into<Color32>) -> Self {
      self.cells = Some([color.into(); CELLS]);
      self
   }

   /// The nine cell colours, in reading order: row by row, three per row.
   ///
   /// Replaces the ramp whole; [Ripple::color] is the one-colour shorthand.
   #[inline]
   pub fn cells(mut self, cells: [Color32; CELLS]) -> Self {
      self.cells = Some(cells);
      self
   }

   /// Seconds of one full ripple. Default: `1.5`.
   #[inline]
   pub fn period(mut self, period: f32) -> Self {
      self.period = period;
      self
   }

   /// Step between two neighbouring cells, in seconds. `0.0` lights the whole grid at once.
   /// Default: `0.1`.
   #[inline]
   pub fn delay(mut self, delay: f32) -> Self {
      self.delay = delay;
      self
   }

   /// Curve of the fade in and out. Default: [`Easing::EaseOut`], the CSS `ease` measured against
   /// the reference capture.
   #[inline]
   pub fn easing(mut self, easing: Easing) -> Self {
      self.easing = easing;
      self
   }

   /// This loader with the palette's default ramp.
   ///
   /// The ramp is nine fully saturated swatches that walk [`ThemeColors::accent`]'s hue by 35° and
   /// lift the lightness from 50 % to 69 %, which is the measured shape of the ported palette
   /// (`#00FF87` to `#60EFFF`). The accent's own saturation and lightness are not used: a muted or
   /// pale accent would make the wave read as a brightness flicker.
   pub fn from_theme(colors: ThemeColors) -> Self {
      Self {
         cells: Some(Self::ramp(colors)),
         ..Self::new()
      }
   }

   /// The nine colours [Ripple::from_theme] derives from `colors`.
   fn ramp(colors: ThemeColors) -> [Color32; CELLS] {
      let mut seed = Hsla::from_color32(colors.accent);
      seed.s = RAMP_SATURATION;
      seed.a = 1.0;

      let mut cells = [Color32::TRANSPARENT; CELLS];
      for (index, cell) in cells.iter_mut().enumerate() {
         let t = index as f32 / (CELLS - 1) as f32;
         *cell = Hsla {
            h: seed.h + RAMP_HUE * t,
            l: RAMP_LIGHTNESS + (RAMP_LIGHTNESS_END - RAMP_LIGHTNESS) * t,
            ..seed
         }
         .to_color32();
      }

      cells
   }

   /// Rect of cell `index` inside `grid`, both ends of the row and of the column inset by one gutter.
   fn cell_rect(grid: Rect, index: usize) -> Rect {
      let size = grid.width();
      let cell = size * CELL_RATIO;
      let step = cell + size * GAP_RATIO;
      let inset = size * GAP_RATIO;
      let column = (index % 3) as f32;
      let row = (index / 3) as f32;
      let min = grid.min + vec2(inset + column * step, inset + row * step);

      Rect::from_min_size(min, Vec2::splat(cell))
   }

   /// Alpha of cell `index` at `time`, in `0.0..=1.0`.
   ///
   /// The cell's own clock is `time` minus its delay; the fade up, the fade back down and the dark
   /// tail take 30 %, 30 % and 40 % of the loop.
   fn alpha_at(&self, time: f64, index: usize) -> f32 {
      let period = self.period.max(MIN_PERIOD) as f64;
      let delay = self.delay.max(0.0) as f64 * (index % DELAYS) as f64;
      let phase = ((time - delay) / period).rem_euclid(1.0) as f32;

      if phase < FADE_IN {
         self.easing.apply(phase / FADE_IN)
      } else if phase < FADE_IN + FADE_OUT {
         1.0 - self.easing.apply((phase - FADE_IN) / FADE_OUT)
      } else {
         0.0
      }
   }

   /// Paints the grid in `rect`, centred and inset to the shorter side.
   ///
   /// Public so a widget can host one without allocating a second `Ui`.
   pub fn paint_at(&self, ui: &Ui, rect: Rect) {
      if !ui.is_rect_visible(rect) {
         return;
      }

      // because it is animated
      ui.ctx().request_repaint();

      let size = rect.width().min(rect.height());
      if size <= 0.0 {
         return;
      }

      let cells = self.cells.unwrap_or_else(|| Self::ramp(Theme::colors_from_ctx(ui.ctx())));
      let corner = (size * CORNER_RATIO).max(MIN_CORNER);
      let grid = Rect::from_center_size(rect.center(), Vec2::splat(size));
      let time = ui.ctx().input(|input| input.time);
      let painter = ui.painter();

      for (index, cell) in cells.iter().enumerate() {
         let alpha = self.alpha_at(time, index);
         if alpha <= 0.0 {
            continue;
         }

         painter.rect_filled(
            Self::cell_rect(grid, index),
            corner,
            cell.gamma_multiply(alpha),
         );
      }
   }
}

impl Default for Ripple {
   fn default() -> Self {
      Self::new()
   }
}

impl Widget for Ripple {
   fn ui(self, ui: &mut Ui) -> Response {
      let size = self.size.unwrap_or_else(|| ui.spacing().interact_size.y * DEFAULT_SIZE_RATIO);
      let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
      response.widget_info(|| WidgetInfo::new(Role::ProgressIndicator));
      self.paint_at(ui, rect);

      response
   }
}

#[cfg(test)]
mod tests {
   use super::*;
   use egui::{FullOutput, Pos2, Shape, pos2};

   /// Side of the grid every test paints into.
   const GRID: f32 = 160.0;

   fn shapes(out: &FullOutput) -> Vec<Shape> {
      fn walk(shape: &Shape, into: &mut Vec<Shape>) {
         into.push(shape.clone());
         if let Shape::Vec(shapes) = shape {
            for shape in shapes {
               walk(shape, into);
            }
         }
      }

      let mut flat = Vec::new();
      for clipped in &out.shapes {
         walk(&clipped.shape, &mut flat);
      }
      flat
   }

   /// The grid rect every test paints into.
   fn grid() -> Rect {
      Rect::from_min_size(pos2(20.0, 20.0), Vec2::splat(GRID))
   }

   /// One frame at an explicit `time`, with the loader painted into a fixed 160 pt square.
   fn frame(ripple: &Ripple, time: f64) -> FullOutput {
      let ctx = egui::Context::default();
      let input = egui::RawInput {
         screen_rect: Some(Rect::from_min_size(
            Pos2::ZERO,
            vec2(400.0, 400.0),
         )),
         time: Some(time),
         ..Default::default()
      };
      let mut out = ctx.run_ui(input, |ui| ripple.paint_at(ui, grid()));
      out.textures_delta.clear();

      out
   }

   /// The cells a frame paints, in paint order, as `(rect, alpha)`.
   fn cells(out: &FullOutput) -> Vec<(Rect, f32)> {
      shapes(out)
         .into_iter()
         .filter_map(|shape| match shape {
            Shape::Rect(rect) if rect.fill.a() > 0 => {
               Some((rect.rect, rect.fill.a() as f32 / 255.0))
            }
            _ => None,
         })
         .collect()
   }

   #[test]
   fn the_wave_lights_the_cells_in_reading_order() {
      let ripple = Ripple::new().color(Color32::WHITE);
      let alphas: Vec<f32> = (0..CELLS).map(|i| ripple.alpha_at(0.3, i)).collect();

      // 0.3 s into the loop the first row is mid-fade, brightest first.
      assert!(
         alphas[0] > alphas[1] && alphas[1] > alphas[2] && alphas[2] > 0.0,
         "the leading cell is the brightest: {alphas:?}"
      );
      // The ported ladder is five steps long, so cells 6 and 7 repeat cells 1 and 2.
      assert_eq!(alphas[5], alphas[0], "cell 6 repeats cell 1");
      assert_eq!(alphas[6], alphas[1], "cell 7 repeats cell 2");

      let painted = cells(&frame(&ripple, 0.3));
      assert_eq!(painted.len(), 6, "six cells are lit: {painted:?}");
      let row = painted[0].0.top();
      let xs: Vec<f32> = painted
         .iter()
         .filter_map(|(rect, _)| (rect.top() == row).then_some(rect.left()))
         .collect();
      assert!(
         xs[0] < xs[1] && xs[1] < xs[2],
         "the lit cells run left to right: {xs:?}"
      );
   }

   #[test]
   fn a_zero_delay_lights_every_cell_together() {
      let painted = cells(&frame(
         &Ripple::new().color(Color32::WHITE).delay(0.0),
         0.2,
      ));

      assert_eq!(painted.len(), CELLS, "the whole grid is lit");
      let first = painted[0].1;
      assert!(first > 0.0);
      assert!(
         painted.iter().all(|(_, alpha)| (alpha - first).abs() < 0.001),
         "one alpha for the grid: {:?}",
         painted.iter().map(|(_, a)| *a).collect::<Vec<_>>()
      );
   }

   #[test]
   fn a_cell_is_a_square_inset_by_one_gutter() {
      let painted = cells(&frame(
         &Ripple::new().color(Color32::WHITE).delay(0.0),
         0.2,
      ));
      assert_eq!(painted.len(), CELLS);

      let (first, _) = painted[0];
      let (last, _) = painted[CELLS - 1];
      let expected = GRID * CELL_RATIO;
      let gutter = GRID * GAP_RATIO;

      for (rect, _) in &painted {
         assert!(
            (rect.width() - expected).abs() < 0.01 && (rect.height() - expected).abs() < 0.01,
            "cell is {expected} pt square, got {} x {}",
            rect.width(),
            rect.height()
         );
      }
      assert!(
         (first.left() - (20.0 + gutter)).abs() < 0.01,
         "the first cell is inset by one gutter: {}",
         first.left()
      );
      assert!(
         (last.right() + gutter - (20.0 + GRID)).abs() < 0.01,
         "the last cell stops one gutter short: {}",
         last.right()
      );
   }

   #[test]
   fn the_theme_ramp_walks_the_accent_hue_once() {
      let colors = crate::theme::Theme::new(crate::theme::ThemeKind::TokyoNight).colors;
      let ramp = Ripple::ramp(colors);
      let accent = Hsla::from_color32(colors.accent);

      for (index, cell) in ramp.iter().enumerate() {
         let hsla = Hsla::from_color32(*cell);
         let expected = accent.h + RAMP_HUE * index as f32 / (CELLS - 1) as f32;
         assert!(
            (hsla.h - expected).abs() < 1.0,
            "cell {index} sits at its step of the ramp: {} vs {}",
            hsla.h,
            expected
         );
         assert!(
            (hsla.s - RAMP_SATURATION).abs() < 1.0,
            "cell {index} is saturated"
         );
      }
   }

   #[test]
   fn a_visible_loader_requests_a_repaint() {
      let ctx = egui::Context::default();
      let input = egui::RawInput {
         screen_rect: Some(Rect::from_min_size(
            Pos2::ZERO,
            vec2(400.0, 400.0),
         )),
         time: Some(0.2),
         ..Default::default()
      };
      let mut repaint = false;
      let mut out = ctx.run_ui(input, |ui| {
         Ripple::new().paint_at(ui, grid());
         repaint = ui.ctx().has_requested_repaint();
      });
      out.textures_delta.clear();

      assert!(repaint, "a running loop is never settled");
   }
}
