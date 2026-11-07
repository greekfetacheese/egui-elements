//! Frame-rate metric: FPS plus the CPU time the host reports per frame.
//!
//! One call per frame feeds both readings. [`FpsMeter::frame`] takes the
//! host's own `Instant::elapsed()` and times the interval between calls
//! itself, so the two numbers stay independent.
//!
//! Paint it as a themed floating window with [`FpsMeter::show_window`], or
//! drop the chrome-free [`FpsMeter::body_ui`] into any container.
//!
//! ```
//! # use std::time::Instant;
//! # use egui::__run_test_ui;
//! # use egui_elements::widgets::FpsMeter;
//! # __run_test_ui(|ui| {
//! let mut meter = FpsMeter::new();
//! let start = Instant::now();
//! // … the host's frame work …
//! meter.frame(start.elapsed());
//! meter.body_ui(ui);
//! # });
//! ```

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use egui::{Align2, Context, Response, RichText, Stroke, Ui, Vec2};

use crate::theme::Theme;
use crate::widgets::{Divider, Window};

/// Samples kept at 60 fps — about two seconds of history.
const DEFAULT_CAPACITY: usize = 120;
/// Default width of the floating window, in points.
const WINDOW_WIDTH: f32 = 210.0;
/// Minimum width of the body, in points, so the readouts do not jitter.
const BODY_WIDTH: f32 = 150.0;

/// One frame: the interval since the previous call and the host's CPU time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FrameSample {
   dt: Duration,
   cpu: Duration,
}

/// Rolling minimum, mean and maximum of a millisecond series.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Spread {
   /// Smallest sample, in milliseconds.
   pub min: f32,
   /// Mean sample, in milliseconds.
   pub avg: f32,
   /// Largest sample, in milliseconds.
   pub max: f32,
}

impl Spread {
   /// Fold `value` in; `count` is the number of samples *including* this one.
   fn fold(&mut self, value: f32, count: usize) {
      if count == 1 {
         self.min = value;
         self.max = value;
         self.avg = value;
      } else {
         self.min = self.min.min(value);
         self.max = self.max.max(value);
         self.avg += (value - self.avg) / count as f32;
      }
   }
}

/// Aggregated readings over the meter's sample window.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FpsStats {
   /// Mean frames per second over the measured intervals; `0.0` until two
   /// frames have been recorded.
   pub fps: f32,
   /// Number of measured frame intervals in the window.
   pub frames: usize,
   /// Frame interval, in milliseconds.
   pub frame_ms: Spread,
   /// Host-reported CPU time, in milliseconds.
   pub cpu_ms: Spread,
}

/// Rolling FPS and CPU-time metric fed once per frame.
///
/// The meter keeps the last 120 frame intervals and CPU times — roughly two
/// seconds at 60 fps — and reports the mean FPS plus the
/// min / mean / max of both series. Call [`Self::frame`] once per frame, then
/// paint [`Self::body_ui`] wherever the metric belongs.
#[derive(Clone, Debug, PartialEq)]
pub struct FpsMeter {
   history: VecDeque<FrameSample>,
   capacity: usize,
   title: String,
   anchor: Option<(Align2, Vec2)>,
   last_frame: Option<Instant>,
}

impl FpsMeter {
   /// A meter with the default 120-frame window and the title `"FPS"`.
   pub fn new() -> Self {
      Self {
         history: VecDeque::new(),
         capacity: DEFAULT_CAPACITY,
         title: "FPS".to_owned(),
         anchor: None,
         last_frame: None,
      }
   }

   /// A meter that keeps at most `capacity` frames; `0` is clamped to `1`.
   pub fn with_capacity(capacity: usize) -> Self {
      let mut meter = Self::new();
      meter.capacity = capacity.max(1);
      meter
   }

   /// Window title, also used as the window's id. Default `"FPS"`.
   pub fn title(mut self, title: impl Into<String>) -> Self {
      self.title = title.into();
      self
   }

   /// Pin the window to a screen corner instead of letting it float.
   ///
   /// Wraps [`egui::Window::anchor`]: an anchored window cannot be dragged.
   /// Default: a free window at egui's own placement.
   pub fn anchor(mut self, align: Align2, offset: impl Into<Vec2>) -> Self {
      self.anchor = Some((align, offset.into()));
      self
   }

   /// Record one frame: `cpu` is the host's own `Instant::elapsed()`.
   ///
   /// Call once per frame. The meter times the interval since the previous
   /// call to derive FPS; the first call only seeds the clock, so it reports
   /// CPU time but no frame interval yet.
   pub fn frame(&mut self, cpu: Duration) {
      let now = Instant::now();
      let dt = self.last_frame.map_or(Duration::ZERO, |last| {
         now.saturating_duration_since(last)
      });
      self.last_frame = Some(now);
      self.record(dt, cpu);
   }

   /// Push one sample. Split from [`Self::frame`] so tests can drive an explicit clock.
   fn record(&mut self, dt: Duration, cpu: Duration) {
      if self.history.len() == self.capacity {
         self.history.pop_front();
      }
      self.history.push_back(FrameSample { dt, cpu });
   }

   /// Forget every sample and restart the frame clock.
   pub fn clear(&mut self) {
      self.history.clear();
      self.last_frame = None;
   }

   /// Number of frames in the window.
   pub fn len(&self) -> usize {
      self.history.len()
   }

   /// Whether no frame has been recorded since construction or [`Self::clear`].
   pub fn is_empty(&self) -> bool {
      self.history.is_empty()
   }

   /// Sample-window capacity in frames.
   pub fn capacity(&self) -> usize {
      self.capacity
   }

   /// Mean FPS over the window; `0.0` until two frames have been recorded.
   pub fn fps(&self) -> f32 {
      self.stats().fps
   }

   /// CPU time the host reported for the latest frame, in milliseconds;
   /// `0.0` while the window is empty.
   pub fn cpu_ms(&self) -> f32 {
      self.history.back().map_or(0.0, |sample| millis(sample.cpu))
   }

   /// Min / mean / max of the frame interval and of the CPU time over the window.
   pub fn stats(&self) -> FpsStats {
      let mut frame_ms = Spread::default();
      let mut cpu_ms = Spread::default();
      let mut frames = 0usize;
      let mut elapsed = 0.0f64;
      let mut cpu_frames = 0usize;

      for sample in &self.history {
         cpu_ms.fold(millis(sample.cpu), cpu_frames + 1);
         cpu_frames += 1;

         if !sample.dt.is_zero() {
            frame_ms.fold(millis(sample.dt), frames + 1);
            frames += 1;
            elapsed += sample.dt.as_secs_f64();
         }
      }

      FpsStats {
         fps: if elapsed > 0.0 {
            (frames as f64 / elapsed) as f32
         } else {
            0.0
         },
         frames,
         frame_ms,
         cpu_ms,
      }
   }

   /// Draw the metric into `ui` with no window chrome.
   ///
   /// Host it in a panel, a card, or an existing window. The body requests a
   /// repaint ([`Context::request_repaint`]) because a metric that is on screen
   /// must keep the frame loop running to stay live.
   pub fn body_ui(&self, ui: &mut Ui) {
      let theme = Theme::current(ui.ctx());
      let colors = theme.colors;
      let spacing = theme.spacing;
      let typography = theme.typography;
      let stats = self.stats();

      ui.scope(|ui| {
         ui.set_min_width(BODY_WIDTH);
         ui.spacing_mut().item_spacing.y = spacing.xs;

         let fps = if stats.frames > 0 {
            format!("{:.1}", stats.fps)
         } else {
            "—".to_owned()
         };
         let cpu = if self.is_empty() {
            "—".to_owned()
         } else {
            format!("{:.2} ms", self.cpu_ms())
         };
         let spread = if self.is_empty() {
            "no samples yet".to_owned()
         } else {
            format!(
               "min {:.2} · avg {:.2} · max {:.2} ms",
               stats.cpu_ms.min, stats.cpu_ms.avg, stats.cpu_ms.max
            )
         };

         ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = spacing.sm;
            ui.label(RichText::new(fps).size(typography.heading).color(colors.text).strong());
            ui.label(RichText::new("FPS").size(typography.small).color(colors.text_muted));
         });

         ui.add_space(spacing.xs);
         ui.add(Divider::new());
         ui.add_space(spacing.xs);

         ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = spacing.sm;
            ui.label(RichText::new("CPU").size(typography.small).color(colors.text_muted));
            ui.label(RichText::new(cpu).size(typography.normal).color(colors.text));
         });

         ui.label(RichText::new(spread).size(typography.very_small).color(colors.text_muted));

         ui.ctx().request_repaint();
      });
   }

   /// Draw the themed floating window; returns its outer `Response` when open.
   ///
   /// The title is also the window's id, so give each meter a distinct
   /// [`Self::title`] when more than one is on screen.
   pub fn show_window(&self, ctx: &Context) -> Option<Response> {
      let theme = Theme::current(ctx);
      let frame = theme.window_frame;

      let window = Window::new(self.title.as_str())
         .default_width(WINDOW_WIDTH)
         .resizable([false, false])
         .frame(frame)
         .title_frame(frame.stroke(Stroke::NONE));

      let window = match self.anchor {
         Some((align, offset)) => window.anchor(align, offset),
         None => window,
      };

      window.show(ctx, |ui| self.body_ui(ui)).map(|inner| inner.response)
   }
}

impl Default for FpsMeter {
   fn default() -> Self {
      Self::new()
   }
}

/// `duration` in milliseconds.
fn millis(duration: Duration) -> f32 {
   (duration.as_secs_f64() * 1000.0) as f32
}

#[cfg(test)]
mod tests {
   use egui::{Context, Pos2, RawInput, Rect, vec2};

   use super::*;
   use crate::theme::ThemeKind;

   fn meter_from(samples: &[(u64, u64)]) -> FpsMeter {
      let mut meter = FpsMeter::with_capacity(samples.len().max(1));
      for (dt_ms, cpu_ms) in samples {
         meter.record(
            Duration::from_millis(*dt_ms),
            Duration::from_millis(*cpu_ms),
         );
      }
      meter
   }

   #[test]
   fn fps_is_the_mean_of_the_measured_intervals() {
      let meter = meter_from(&[(10, 1), (20, 2)]);
      let stats = meter.stats();

      assert_eq!(stats.frames, 2);
      assert!(
         (stats.fps - 200.0 / 3.0).abs() < 1e-3,
         "10 ms + 20 ms is {} fps",
         stats.fps
      );
      assert!((stats.frame_ms.min - 10.0).abs() < 1e-3);
      assert!((stats.frame_ms.avg - 15.0).abs() < 1e-3);
      assert!((stats.frame_ms.max - 20.0).abs() < 1e-3);
   }

   #[test]
   fn cpu_spread_folds_min_mean_max() {
      let meter = meter_from(&[(16, 1), (16, 2), (16, 6)]);
      let stats = meter.stats();

      assert!((stats.cpu_ms.min - 1.0).abs() < 1e-3);
      assert!((stats.cpu_ms.avg - 3.0).abs() < 1e-3);
      assert!((stats.cpu_ms.max - 6.0).abs() < 1e-3);
      assert!((meter.cpu_ms() - 6.0).abs() < 1e-3);
   }

   #[test]
   fn first_frame_reports_cpu_without_an_fps_reading() {
      let mut meter = FpsMeter::new();
      meter.frame(Duration::from_millis(3));

      let stats = meter.stats();
      assert_eq!(stats.frames, 0);
      assert_eq!(stats.fps, 0.0);
      assert!((meter.cpu_ms() - 3.0).abs() < 1e-3);
      assert!(!meter.is_empty());
   }

   #[test]
   fn zero_length_intervals_do_not_enter_the_fps_mean() {
      let mut meter = FpsMeter::with_capacity(3);
      meter.record(Duration::ZERO, Duration::from_millis(1));
      meter.record(
         Duration::from_millis(10),
         Duration::from_millis(1),
      );
      meter.record(
         Duration::from_millis(10),
         Duration::from_millis(1),
      );

      let stats = meter.stats();
      assert_eq!(stats.frames, 2);
      assert!((stats.fps - 100.0).abs() < 1e-3);
   }

   #[test]
   fn capacity_evicts_the_oldest_sample() {
      let mut meter = FpsMeter::with_capacity(2);
      meter.record(
         Duration::from_millis(10),
         Duration::from_millis(9),
      );
      meter.record(
         Duration::from_millis(10),
         Duration::from_millis(1),
      );
      meter.record(
         Duration::from_millis(10),
         Duration::from_millis(5),
      );

      assert_eq!(meter.len(), 2);
      assert_eq!(meter.capacity(), 2);

      let stats = meter.stats();
      assert!((stats.cpu_ms.min - 1.0).abs() < 1e-3);
      assert!((stats.cpu_ms.max - 5.0).abs() < 1e-3);
   }

   #[test]
   fn clear_forgets_the_window_and_the_clock() {
      let mut meter = meter_from(&[(16, 2), (16, 3)]);
      meter.clear();

      assert!(meter.is_empty());
      assert_eq!(meter.stats().frames, 0);
      assert_eq!(meter.fps(), 0.0);
      assert_eq!(meter.cpu_ms(), 0.0);
   }

   fn themed_ctx() -> Context {
      let ctx = Context::default();
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&ctx);
      ctx
   }

   #[test]
   fn body_ui_keeps_the_frame_loop_running() {
      let ctx = themed_ctx();
      let meter = meter_from(&[(16, 2), (16, 3)]);
      let mut repaint = false;

      let output = ctx.run_ui(RawInput::default(), |ui| {
         meter.body_ui(ui);
         repaint = ui.ctx().has_requested_repaint();
      });
      output.drop_without_applying_deltas();

      assert!(repaint, "a visible metric must keep measuring");
   }

   #[test]
   fn show_window_paints_the_body_in_a_window() {
      let ctx = themed_ctx();
      let meter = meter_from(&[(16, 2), (16, 3)]);
      let mut shown = false;

      for _ in 0..2 {
         let output = ctx.run_ui(
            RawInput {
               screen_rect: Some(Rect::from_min_size(
                  Pos2::ZERO,
                  vec2(600.0, 400.0),
               )),
               ..Default::default()
            },
            |ui| {
               shown = meter.show_window(ui.ctx()).is_some();
            },
         );
         output.drop_without_applying_deltas();
      }

      assert!(shown, "the window reports its response");
   }
}
