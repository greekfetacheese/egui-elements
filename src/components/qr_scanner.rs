use egui::{
   Color32, Context, Frame, Key, LayerId, Order, Rect, RichText, Stroke, StrokeKind,
   ViewportBuilder, ViewportClass, ViewportCommand, ViewportId, pos2, vec2,
};

use enigo::Mouse;
use rqrr::PreparedImage;
use secure_types::{SecureString, Zeroize};
use std::sync::{
   Arc, Mutex,
   atomic::{AtomicBool, AtomicI32, Ordering},
};
use std::time::{Duration, Instant};
use xcap::{Monitor, image::DynamicImage};

type Error = Box<dyn std::error::Error>;

const VIEWPORT_ID: &str = "qr_scanner";
#[cfg(target_os = "windows")]
const HELP_HEIGHT_PT: f32 = 124.0;
#[cfg(target_os = "windows")]
const HELP_MIN_WIDTH_PT: f32 = 360.0;
const MIN_CAPTURE_SIZE: i32 = 40;

#[cfg(target_os = "windows")]
mod win32 {
   use std::ffi::c_void;
   use std::sync::Mutex;

   pub type Hwnd = *mut c_void;
   type Hrgn = *mut c_void;

   #[repr(C)]
   struct Rect {
      left: i32,
      top: i32,
      right: i32,
      bottom: i32,
   }

   const RGN_DIFF: i32 = 4;
   const WDA_EXCLUDEFROMCAPTURE: u32 = 0x0000_0011;

   #[link(name = "user32")]
   unsafe extern "system" {
      pub fn GetAsyncKeyState(v_key: i32) -> i16;
      fn FindWindowW(class: *const u16, title: *const u16) -> Hwnd;
      fn GetClientRect(hwnd: Hwnd, rect: *mut Rect) -> i32;
      fn SetWindowRgn(hwnd: Hwnd, hrgn: Hrgn, redraw: i32) -> i32;
      fn SetWindowDisplayAffinity(hwnd: Hwnd, affinity: u32) -> i32;
   }

   #[link(name = "gdi32")]
   unsafe extern "system" {
      fn CreateRectRgn(x1: i32, y1: i32, x2: i32, y2: i32) -> Hrgn;
      fn CombineRgn(dst: Hrgn, src1: Hrgn, src2: Hrgn, mode: i32) -> i32;
      fn DeleteObject(obj: Hrgn) -> i32;
   }

   struct Applied {
      hwnd: usize,
      width: i32,
      height: i32,
      help_h: i32,
      capture: i32,
      inset: i32,
      affinity: bool,
   }

   static APPLIED: Mutex<Option<Applied>> = Mutex::new(None);

   fn overlay_hwnd() -> Hwnd {
      let mut title: Vec<u16> = "QR Scan Overlay".encode_utf16().collect();
      title.push(0);
      unsafe { FindWindowW(std::ptr::null(), title.as_ptr()) }
   }

   const RGN_OR: i32 = 2;

   /// Keep the help strip + a 3px frame around the scan box; everything else
   /// is a hole. Extra viewports cannot be alpha-transparent on Windows
   /// (egui #3632). Help width is independent of capture size.
   pub fn apply_overlay_hole(help_h_px: i32, border_px: i32, capture_px: i32) {
      let hwnd = overlay_hwnd();
      if hwnd.is_null() {
         return;
      }
      let hwnd_bits = hwnd as usize;

      let mut client = Rect {
         left: 0,
         top: 0,
         right: 0,
         bottom: 0,
      };
      if unsafe { GetClientRect(hwnd, &mut client) } == 0 {
         return;
      }
      let width = client.right - client.left;
      let height = client.bottom - client.top;
      if width <= 8 || height <= 8 {
         return;
      }

      let inset = border_px.max(2);
      let help_h = help_h_px.clamp(inset, height);
      let capture = capture_px.clamp(inset * 2 + 1, width.max(height));

      let mut guard = APPLIED.lock().unwrap_or_else(|e| e.into_inner());
      let same = guard.as_ref().is_some_and(|s| {
         s.hwnd == hwnd_bits
            && s.width == width
            && s.height == height
            && s.help_h == help_h
            && s.capture == capture
            && s.inset == inset
      });
      if !same {
         let cap_x = ((width - capture) / 2).max(0);
         let cap_y = help_h;
         let cap_r = (cap_x + capture).min(width);
         let cap_b = (cap_y + capture).min(height);

         let help = unsafe { CreateRectRgn(0, 0, width, help_h) };
         let frame = unsafe { CreateRectRgn(cap_x, cap_y, cap_r, cap_b) };
         let inner = unsafe {
            CreateRectRgn(
               cap_x + inset,
               cap_y + inset,
               (cap_r - inset).max(cap_x + inset + 1),
               (cap_b - inset).max(cap_y + inset + 1),
            )
         };
         if !help.is_null() && !frame.is_null() && !inner.is_null() {
            unsafe {
               CombineRgn(frame, frame, inner, RGN_DIFF);
               CombineRgn(help, help, frame, RGN_OR);
               SetWindowRgn(hwnd, help, 1);
               DeleteObject(frame);
               DeleteObject(inner);
            }
         } else {
            if !help.is_null() {
               unsafe { DeleteObject(help) };
            }
            if !frame.is_null() {
               unsafe { DeleteObject(frame) };
            }
            if !inner.is_null() {
               unsafe { DeleteObject(inner) };
            }
         }
         *guard = Some(Applied {
            hwnd: hwnd_bits,
            width,
            height,
            help_h,
            capture,
            inset,
            affinity: false,
         });
      }

      if let Some(state) = guard.as_mut()
         && !state.affinity
      {
         if unsafe { SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE) } != 0 {
            state.affinity = true;
         }
      }
   }
}

/// A QR scanner that can be used to scan QR codes within the current monitor.
///
/// Suitable for QR payloads that may contain sensitive information.
///
/// # Capture backends
///
/// | Platform | How `xcap` captures | Temp file? |
/// |----------|---------------------|------------|
/// | Linux X11 | XCB / RandR into RAM | No |
/// | Linux Wayland | GNOME `ScreenshotArea` or the desktop portal writes a PNG under the temp dir, then deletes it | Yes, briefly |
/// | Windows | GDI `BitBlt` + `GetDIBits` into RAM (`xcap` without `wgc`) | No |
///
/// Prefer X11 on Linux. Windows does **not** keep a screenshot temp file the
/// way Wayland does. The pixels still live in process memory until wiped.
///
/// # Usage:
///
/// ```no_run
/// # use egui::Context;
/// # use egui_elements::components::QRScanner;
/// let mut scanner = QRScanner::new();
/// scanner.open(Context::default());
/// let res = scanner.get_result();
/// if let Some(res) = res {
///    // reset the state
///    scanner.reset();
///    // use the decoded string
/// }
/// ```
#[derive(Clone)]
pub struct QRScanner {
   open: Arc<AtomicBool>,
   thread_running: Arc<AtomicBool>,
   viewport_shown: Arc<AtomicBool>,
   capture_size: Arc<AtomicI32>,
   capture_is_valid: Arc<AtomicBool>,
   result: Arc<Mutex<Option<SecureString>>>,
   last_error: Arc<Mutex<Option<String>>>,
   #[cfg(not(target_os = "windows"))]
   current_monitor: Option<xcap::Monitor>,
}

impl QRScanner {
   /// Create a closed scanner with no result.
   pub fn new() -> Self {
      Self {
         open: Arc::new(AtomicBool::new(false)),
         thread_running: Arc::new(AtomicBool::new(false)),
         viewport_shown: Arc::new(AtomicBool::new(false)),
         capture_size: Arc::new(AtomicI32::new(250)),
         capture_is_valid: Arc::new(AtomicBool::new(false)),
         result: Arc::new(Mutex::new(None)),
         last_error: Arc::new(Mutex::new(None)),
         #[cfg(not(target_os = "windows"))]
         current_monitor: None,
      }
   }

   /// Open the overlay viewport. Starts a repaint/capture thread the first time.
   pub fn open(&self, ctx: Context) {
      *self.last_error.lock().unwrap() = None;
      *self.result.lock().unwrap() = None;
      self.capture_is_valid.store(false, Ordering::Relaxed);
      self.open.store(true, Ordering::Relaxed);
      ctx.send_viewport_cmd_to(
         ViewportId::from_hash_of(VIEWPORT_ID),
         ViewportCommand::CancelClose,
      );
      if !self.thread_running.swap(true, Ordering::Relaxed) {
         let open = self.open.clone();
         let running = self.thread_running.clone();
         let capture_size = self.capture_size.clone();
         let capture_is_valid = self.capture_is_valid.clone();
         let last_error = self.last_error.clone();
         let result = self.result.clone();
         std::thread::spawn(move || {
            overlay_worker(
               ctx,
               open,
               capture_size,
               capture_is_valid,
               last_error,
               result,
            );
            running.store(false, Ordering::Relaxed);
         });
      }
   }

   /// Hide the overlay. Does not clear a captured result.
   pub fn close(&self) {
      self.open.store(false, Ordering::Relaxed);
   }

   /// Hide the overlay and drop any result / error.
   ///
   /// Keeps the shared flags so a running capture thread still sees `close()`.
   pub fn reset(&mut self) {
      self.close();
      *self.result.lock().unwrap() = None;
      *self.last_error.lock().unwrap() = None;
      self.capture_is_valid.store(false, Ordering::Relaxed);
      self.capture_size.store(250, Ordering::Relaxed);
      #[cfg(not(target_os = "windows"))]
      {
         self.current_monitor = None;
      }
   }

   /// Whether the overlay viewport is currently requested open.
   pub fn is_open(&self) -> bool {
      self.open.load(Ordering::Relaxed)
   }

   /// Drive the overlay. Call every frame while you want the scanner available.
   pub fn show(&mut self, ctx: &egui::Context) {
      if !self.is_open() {
         // Close once on the open→closed edge. Sending Close every idle frame
         // leaves `close_requested` set, so the next overlay dies immediately.
         if self.viewport_shown.swap(false, Ordering::Relaxed) {
            ctx.send_viewport_cmd_to(
               ViewportId::from_hash_of(VIEWPORT_ID),
               ViewportCommand::Close,
            );
         }
         return;
      }
      self.viewport_shown.store(true, Ordering::Relaxed);

      // Keys on the parent viewport. On Windows the overlay must not steal
      // focus (winit Focus injects Alt via SendInput every frame and freezes
      // the cursor), so Esc / +/- have to land here.
      handle_overlay_keys(ctx, &self.open, &self.capture_size);

      #[cfg(target_os = "windows")]
      self.show_follow_cursor(ctx);
      #[cfg(not(target_os = "windows"))]
      self.show_fullscreen_monitor(ctx);
   }

   /// Windows: a small always-on-top window that follows the cursor.
   ///
   /// A monitor-sized overlay is treated like borderless fullscreen and
   /// traps the pointer (Esc does nothing; only Ctrl+F4 kills the app).
   #[cfg(target_os = "windows")]
   fn show_follow_cursor(&mut self, ctx: &egui::Context) {
      let Ok(enigo) = enigo::Enigo::new(&enigo::Settings::default()) else {
         return;
      };
      let Ok((mouse_x_px, mouse_y_px)) = enigo.location() else {
         return;
      };

      let ppp = ctx.pixels_per_point().max(0.1);
      let capture_size_px = self.capture_size.load(Ordering::Relaxed).max(MIN_CAPTURE_SIZE);
      let half_px = capture_size_px / 2;
      let capture_pt = capture_size_px as f32 / ppp;
      // Help keeps a readable min width; the scan box can be smaller.
      let window_w = capture_pt.max(HELP_MIN_WIDTH_PT);
      let inner = vec2(window_w, capture_pt + HELP_HEIGHT_PT);
      let pos = pos2(
         (mouse_x_px as f32 / ppp) - window_w * 0.5,
         (mouse_y_px - half_px) as f32 / ppp - HELP_HEIGHT_PT,
      );

      let open = self.open.clone();
      let capture_is_valid = self.capture_is_valid.clone();
      let last_error = self.last_error.clone();
      let capture_size = self.capture_size.clone();

      ctx.show_viewport_deferred(
         ViewportId::from_hash_of(VIEWPORT_ID),
         ViewportBuilder::default()
            .with_title("QR Scan Overlay")
            .with_transparent(true)
            .with_decorations(false)
            .with_always_on_top()
            .with_mouse_passthrough(true)
            .with_taskbar(false)
            .with_active(false)
            .with_resizable(false)
            .with_position(pos)
            .with_inner_size(inner),
         move |ui, class| {
            if class == ViewportClass::EmbeddedWindow {
               ui.label("This viewport is embedded.");
               return;
            }

            ui.ctx().request_repaint();

            if ui.ctx().input(|i| i.viewport().close_requested()) {
               open.store(false, Ordering::Relaxed);
            }

            let screen = ui.ctx().content_rect();
            let ppp = ui.ctx().pixels_per_point().max(0.1);
            let capture_pt =
               capture_size.load(Ordering::Relaxed).max(MIN_CAPTURE_SIZE) as f32 / ppp;
            let cap_x = screen.min.x + (screen.width() - capture_pt) * 0.5;
            let cap_y = screen.min.y + HELP_HEIGHT_PT;
            let rect = Rect::from_min_size(pos2(cap_x, cap_y), vec2(capture_pt, capture_pt));

            let painter = ui.ctx().layer_painter(LayerId::new(
               Order::Foreground,
               egui::Id::new("qr_border_layer"),
            ));
            let color = if capture_is_valid.load(Ordering::Relaxed) {
               Color32::GREEN
            } else {
               Color32::RED
            };
            painter.rect_stroke(
               rect,
               0.0,
               Stroke::new(3.0, color),
               StrokeKind::Inside,
            );

            let err = last_error.lock().ok().and_then(|g| g.clone());
            egui::Area::new("qr_help".into())
               .fixed_pos(pos2(screen.min.x + 6.0, screen.min.y + 4.0))
               .show(ui.ctx(), |ui| {
                  ui.set_min_width(HELP_MIN_WIDTH_PT - 20.0);
                  ui.set_max_width(HELP_MIN_WIDTH_PT - 20.0);
                  paint_help(ui, err.as_deref());
               });

            // Extra viewports stay opaque on Windows; clip the HWND so the
            // scan box is a hole (see-through + not captured). Help strip
            // stays a fixed min width when the box shrinks.
            win32::apply_overlay_hole(
               (HELP_HEIGHT_PT * ppp) as i32,
               (3.0 * ppp) as i32,
               (capture_pt * ppp) as i32,
            );
         },
      );
   }

   /// Linux: fullscreen transparent overlay covering the monitor under the cursor.
   #[cfg(not(target_os = "windows"))]
   fn show_fullscreen_monitor(&mut self, ctx: &egui::Context) {
      // Get monitor under cursor if not set
      if self.current_monitor.is_none() {
         let Ok(enigo) = enigo::Enigo::new(&enigo::Settings::default()) else {
            return;
         };
         let Ok((mouse_x, mouse_y)) = enigo.location() else {
            return;
         };
         if let Ok(monitor) = xcap::Monitor::from_point(mouse_x, mouse_y) {
            self.current_monitor = Some(monitor);
         } else {
            self.open.store(false, Ordering::Relaxed);
            return;
         }
      }

      let monitor = self.current_monitor.as_ref().unwrap();

      let Ok(mon_x_px) = monitor.x().map(|v| v as f32) else {
         return;
      };
      let Ok(mon_y_px) = monitor.y().map(|v| v as f32) else {
         return;
      };
      let Ok(mon_width_px) = monitor.width().map(|v| v as f32) else {
         return;
      };
      let Ok(mon_height_px) = monitor.height().map(|v| v as f32) else {
         return;
      };

      let ppp = ctx.pixels_per_point(); // Use main ctx ppp

      let open = self.open.clone();
      let capture_size = self.capture_size.clone();
      let capture_is_valid = self.capture_is_valid.clone();
      let last_error = self.last_error.clone();

      ctx.show_viewport_deferred(
         ViewportId::from_hash_of(VIEWPORT_ID),
         ViewportBuilder::default()
            .with_title("QR Scan Overlay")
            .with_transparent(true)
            .with_decorations(false)
            .with_always_on_top()
            .with_mouse_passthrough(true)
            .with_position(pos2(mon_x_px / ppp, mon_y_px / ppp))
            .with_inner_size(vec2(mon_width_px / ppp, mon_height_px / ppp))
            .with_active(true),
         move |ui, class| {
            if class == ViewportClass::EmbeddedWindow {
               ui.label("This viewport is embedded.");
               return;
            }

            if !ui.ctx().input(|i| i.viewport().focused.unwrap_or(false)) {
               ui.ctx().send_viewport_cmd(ViewportCommand::Focus);
            }

            handle_overlay_keys(ui.ctx(), &open, &capture_size);

            // Look the monitor up inside the callback. `xcap::Monitor` is not
            // `Send`/`Sync` on Windows (`HMONITOR` is a raw pointer), and
            // `show_viewport_deferred` requires both.
            let Ok(enigo) = enigo::Enigo::new(&enigo::Settings::default()) else {
               return;
            };
            let Ok((mouse_x_px, mouse_y_px)) = enigo.location() else {
               return;
            };
            let Ok(monitor) = xcap::Monitor::from_point(mouse_x_px, mouse_y_px) else {
               return;
            };

            // Get ppp for this viewport (should match main)
            let ppp = ui.ctx().pixels_per_point();

            // Calculate capture region in pixels (for xcap)
            let capture_size_px = capture_size.load(Ordering::Relaxed);
            let half_px = capture_size_px / 2;
            let Ok(mon_x_px) = monitor.x() else {
               return;
            };
            let Ok(mon_y_px) = monitor.y() else {
               return;
            };
            let Ok(mon_width_px) = monitor.width().map(|v| v as i32) else {
               return;
            };
            let Ok(mon_height_px) = monitor.height().map(|v| v as i32) else {
               return;
            };

            let cap_x_px = (mouse_x_px - half_px)
               .max(mon_x_px)
               .min(mon_x_px + mon_width_px - capture_size_px);

            let cap_y_px = (mouse_y_px - half_px)
               .max(mon_y_px)
               .min(mon_y_px + mon_height_px - capture_size_px);

            let cap_width_px = capture_size_px.min(mon_x_px + mon_width_px - cap_x_px);
            let cap_height_px = capture_size_px.min(mon_y_px + mon_height_px - cap_y_px);

            // Relative coords in pixels (from monitor origin)
            let rel_x_px = (cap_x_px - mon_x_px) as f32;
            let rel_y_px = (cap_y_px - mon_y_px) as f32;

            // Convert to egui points (local to viewport)
            let rel_x_pt = rel_x_px / ppp;
            let rel_y_pt = rel_y_px / ppp;
            let cap_width_pt = cap_width_px as f32 / ppp;
            let cap_height_pt = cap_height_px as f32 / ppp;

            let rect = Rect::from_min_size(
               pos2(rel_x_pt, rel_y_pt),
               vec2(cap_width_pt, cap_height_pt),
            );

            // Draw border using layer painter
            let painter = ui.ctx().layer_painter(LayerId::new(
               Order::Foreground,
               egui::Id::new("qr_border_layer"),
            ));

            let color = if capture_is_valid.load(Ordering::Relaxed) {
               Color32::GREEN
            } else {
               Color32::RED
            };

            let stroke = Stroke::new(3.0, color);
            let stroke_kind = StrokeKind::Outside;
            painter.rect_stroke(rect, 0.0, stroke, stroke_kind);

            // Show help frame above the capture area
            let help_pos = pos2(rel_x_pt, rel_y_pt - 100.0); // Offset above
            let err = last_error.lock().ok().and_then(|g| g.clone());
            egui::Area::new("qr_help".into()).fixed_pos(help_pos).show(ui.ctx(), |ui| {
               paint_help(ui, err.as_deref());
            });

            if ui.ctx().input(|i| i.viewport().close_requested()) {
               open.store(false, Ordering::Relaxed);
            }
         },
      );
   }

   /// Take the last successful decode, if any.
   pub fn take_result(&self) -> Option<SecureString> {
      self.result.lock().unwrap().take()
   }

   /// Clone of the last successful decode, if any.
   pub fn get_result(&self) -> Option<SecureString> {
      self.result.lock().unwrap().clone()
   }
}

fn handle_overlay_keys(ctx: &Context, open: &Arc<AtomicBool>, capture_size: &Arc<AtomicI32>) {
   ctx.input_mut(|i| {
      if i.consume_key(egui::Modifiers::NONE, Key::Plus)
         || i.consume_key(egui::Modifiers::NONE, Key::Equals)
      {
         capture_size.fetch_add(5, Ordering::Relaxed);
      }

      if i.consume_key(egui::Modifiers::NONE, Key::Minus) {
         let new = (capture_size.load(Ordering::Relaxed) - 5).max(MIN_CAPTURE_SIZE);
         capture_size.store(new, Ordering::Relaxed);
      }

      if i.consume_key(egui::Modifiers::NONE, Key::Escape) {
         open.store(false, Ordering::Relaxed);
      }
   });
}

fn paint_help(ui: &mut egui::Ui, last_error: Option<&str>) {
   let frame = Frame::window(ui.style()).inner_margin(8.0);
   frame.show(ui, |ui| {
      ui.spacing_mut().item_spacing.y = 4.0;
      ui.label(RichText::new("Move the mouse to target the QR code.").size(13.0));
      ui.label(RichText::new("+ / −  resize the capture box").size(13.0));
      ui.label(RichText::new("Esc  cancel").size(13.0));
      if let Some(e) = last_error {
         ui.label(RichText::new(e).size(13.0).color(Color32::RED));
      }
   });
}

#[cfg(target_os = "windows")]
fn poll_win_hotkeys(open: &Arc<AtomicBool>, capture_size: &Arc<AtomicI32>, prev: &mut u8) {
   const VK_ESCAPE: i32 = 0x1B;
   const VK_OEM_PLUS: i32 = 0xBB;
   const VK_OEM_MINUS: i32 = 0xBD;
   const VK_ADD: i32 = 0x6B;
   const VK_SUBTRACT: i32 = 0x6D;
   const BIT_ESC: u8 = 1;
   const BIT_PLUS: u8 = 2;
   const BIT_MINUS: u8 = 4;

   fn down(vk: i32) -> bool {
      unsafe { win32::GetAsyncKeyState(vk) as u16 & 0x8000 != 0 }
   }

   let mut now: u8 = 0;
   if down(VK_ESCAPE) {
      now |= BIT_ESC;
   }
   if down(VK_OEM_PLUS) || down(VK_ADD) {
      now |= BIT_PLUS;
   }
   if down(VK_OEM_MINUS) || down(VK_SUBTRACT) {
      now |= BIT_MINUS;
   }

   let rising = now & !*prev;
   *prev = now;

   if rising & BIT_ESC != 0 {
      open.store(false, Ordering::Relaxed);
   }
   if rising & BIT_PLUS != 0 {
      capture_size.fetch_add(10, Ordering::Relaxed);
   }
   if rising & BIT_MINUS != 0 {
      let new = (capture_size.load(Ordering::Relaxed) - 10).max(MIN_CAPTURE_SIZE);
      capture_size.store(new, Ordering::Relaxed);
   }
}

fn overlay_worker(
   ctx: Context,
   open: Arc<AtomicBool>,
   capture_size: Arc<AtomicI32>,
   capture_is_valid: Arc<AtomicBool>,
   last_error: Arc<Mutex<Option<String>>>,
   result: Arc<Mutex<Option<SecureString>>>,
) {
   let mut last_capture = Instant::now();
   #[cfg(target_os = "windows")]
   let mut prev_keys: u8 = 0;

   loop {
      if !open.load(Ordering::Relaxed) {
         ctx.request_repaint();
         return;
      }

      ctx.request_repaint();

      #[cfg(target_os = "windows")]
      poll_win_hotkeys(&open, &capture_size, &mut prev_keys);

      // Decode off the UI thread. A blocking xcap capture inside the
      // viewport callback freezes the overlay (and the cursor) on Windows.
      if last_capture.elapsed() >= Duration::from_millis(80) {
         last_capture = Instant::now();
         if let Ok(enigo) = enigo::Enigo::new(&enigo::Settings::default()) {
            if let Ok((mouse_x, mouse_y)) = enigo.location() {
               if let Ok(monitor) = Monitor::from_point(mouse_x, mouse_y) {
                  match capture_and_decode(capture_size.load(Ordering::Relaxed), &monitor) {
                     Ok(res) => {
                        capture_is_valid.store(true, Ordering::Relaxed);
                        *last_error.lock().unwrap() = None;
                        *result.lock().unwrap() = Some(res);
                        open.store(false, Ordering::Relaxed);
                     }
                     Err(e) => {
                        capture_is_valid.store(false, Ordering::Relaxed);
                        *last_error.lock().unwrap() = Some(e.to_string());
                     }
                  }
               }
            }
         }
      }

      std::thread::sleep(Duration::from_millis(16));
   }
}

/// Capture a square around the cursor on `monitor` and decode the first QR.
///
/// The RGBA screenshot is wiped after the luma copy. The prepared detector
/// image is wiped on drop (`rqrr-zeroize`).
pub fn capture_and_decode(capture_size: i32, monitor: &Monitor) -> Result<SecureString, Error> {
   // Get global mouse position
   let enigo = enigo::Enigo::new(&enigo::Settings::default())?;
   let (mouse_x, mouse_y) = enigo.location()?;

   // Define capture region (centered; clamp to monitor bounds)
   let half = capture_size / 2;
   let mon_x = monitor.x()?;
   let mon_y = monitor.y()?;
   let mon_width = monitor.width()? as i32;
   let mon_height = monitor.height()? as i32;
   let cap_x = (mouse_x - half).max(mon_x).min(mon_x + mon_width - capture_size);
   let cap_y = (mouse_y - half).max(mon_y).min(mon_y + mon_height - capture_size);
   let cap_width = capture_size.min(mon_x + mon_width - cap_x);
   let cap_height = capture_size.min(mon_y + mon_height - cap_y);

   let cap_width: u32 = cap_width.try_into()?;
   let cap_height: u32 = cap_height.try_into()?;

   // Capture (coords relative to monitor origin)
   let rel_x: u32 = (cap_x - mon_x).try_into()?;
   let rel_y: u32 = (cap_y - mon_y).try_into()?;
   let image = monitor.capture_region(rel_x, rel_y, cap_width, cap_height)?;

   // Decode QR
   let mut img = DynamicImage::ImageRgba8(image);
   let luma = img.to_luma8();

   debug_assert!(img.as_rgba8().is_some());

   if let Some(img) = img.as_mut_rgba8() {
      img.zeroize();
   }

   let mut prepared = PreparedImage::prepare(luma);
   let grids = prepared.detect_grids();

   if grids.is_empty() {
      return Err(format!("No QR grids detected (try adjusting size/position)").into());
   }

   let (_, content) = grids[0].decode()?;
   let sec_string = SecureString::from(content);

   Ok(sec_string)
}
