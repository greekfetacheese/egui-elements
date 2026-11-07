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
/// seconds at 60 fps — and reports the measured mean FPS, the `1000 / cpu_ms`
/// ceiling, and the min / mean / max of both series. Call [`Self::frame`] once
/// per frame, then paint [`Self::body_ui`] wherever the metric belongs.
#[derive(Clone, Debug, PartialEq)]
pub struct FpsMeter {
   history: VecDeque<FrameSample>,
   capacity: usize,
   title: String,
   open: bool,
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
         open: true,
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

   /// Start with the window open or closed. Default `true`.
   pub fn open(mut self, open: bool) -> Self {
      self.open = open;
      self
   }

   /// Whether [`Self::show_window`] currently paints; the title bar's close
   /// button clears it.
   pub fn is_open(&self) -> bool {
      self.open
   }

   /// Open or close the window from the host (a toolbar toggle, for example).
   pub fn set_open(&mut self, open: bool) {
      self.open = open;
   }

   /// Pin the window to a screen corner instead of letting it float.
   ///
   /// Wraps [`egui::Window::anchor`]: an anchored window cannot be dragged.
   /// Default: a free, movable window at egui's own placement.
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

   /// Theoretical FPS ceiling from the latest CPU time: `1000 / cpu_ms`.
   ///
   /// This is the headroom the frame work allows, so it stays meaningful on a
   /// vsync-capped display where [`Self::fps`] reads the refresh rate. `0.0`
   /// while the window is empty or the reported CPU time is zero.
   pub fn max_fps(&self) -> f32 {
      let cpu = self.cpu_ms();
      if cpu > 0.0 { 1000.0 / cpu } else { 0.0 }
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
   /// Host it in a panel, a card, or an existing window. Painting is
   /// side-effect free: the body never schedules a repaint, so the host decides
   /// when the reading is refreshed. Call `ctx.request_repaint()` when it must
   /// stay live while the app is otherwise idle.
   pub fn body_ui(&self, ui: &mut Ui) {
      let theme = Theme::current(ui.ctx());
      let colors = theme.colors;
      let spacing = theme.spacing;
      let typography = theme.typography;
      let stats = self.stats();

      ui.scope(|ui| {
         ui.set_min_width(BODY_WIDTH);
         ui.spacing_mut().item_spacing.y = spacing.xs;

         let ceiling = self.max_fps();
         let max_fps = if ceiling > 0.0 {
            format!("{:.1}", ceiling)
         } else {
            "—".to_owned()
         };
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
            ui.label(RichText::new(max_fps).size(typography.heading).color(colors.text).strong());
            ui.label(RichText::new("MAX FPS").size(typography.small).color(colors.text_muted));
         });

         ui.add_space(spacing.xs);
         ui.add(Divider::new());
         ui.add_space(spacing.xs);

         ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = spacing.xs;
            ui.label(RichText::new("FPS").size(typography.small).color(colors.text_muted));
            ui.label(RichText::new(fps).size(typography.normal).color(colors.text));
            ui.add_space(spacing.sm);
            ui.label(RichText::new("CPU").size(typography.small).color(colors.text_muted));
            ui.label(RichText::new(cpu).size(typography.normal).color(colors.text));
         });

         ui.label(RichText::new(spread).size(typography.very_small).color(colors.text_muted));
      });
   }

   /// Draw the themed floating window; returns its outer `Response` when open.
   ///
   /// The window is movable and carries a close button: the button clears
   /// [`Self::is_open`], so the host can restore it with [`Self::set_open`].
   /// Nothing is painted while the window is closed.
   ///
   /// The title is also the window's id, so give each meter a distinct
   /// [`Self::title`] when more than one is on screen.
   pub fn show_window(&mut self, ctx: &Context) -> Option<Response> {
      if !self.open {
         return None;
      }

      let theme = Theme::current(ctx);
      let frame = theme.window_frame;

      // Local copy: the window borrows `open` while the body borrows `self`.
      let mut open = self.open;
      let window = Window::new(self.title.as_str())
         .open(&mut open)
         .default_width(WINDOW_WIDTH)
         .resizable([false, false])
         .frame(frame)
         .title_frame(frame.stroke(Stroke::NONE));

      let window = match self.anchor {
         Some((align, offset)) => window.anchor(align, offset),
         None => window,
      };

      let shown = window.show(ctx, |ui| self.body_ui(ui)).map(|inner| inner.response);
      self.open = open;
      shown
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
