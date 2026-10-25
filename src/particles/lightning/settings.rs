//! [`LightningSettings`] — public knobs for the lightning effect plus a themed settings window.

use egui::{Color32, Context, DragValue, Slider, Ui};

use super::super::shared::{ParticleLayer, begin_body, settings_window};
use super::Lightning;
use super::effect::{
   DEFAULT_AMBIENT_INTERVAL_SECS, DEFAULT_BOLT_COUNT, DEFAULT_BRANCH_DEPTH, DEFAULT_COLOR,
   DEFAULT_DURATION_MS, DEFAULT_FLASH_INTENSITY, DEFAULT_JITTER, DEFAULT_THICKNESS,
};

/// User-facing knobs for [`Lightning`](super::Lightning).
///
/// The defaults are a cold white-blue bolt with one trunk, a shallow branch,
/// moderate jitter, a moderate full-area flash and a ~420 ms strike. The ambient
/// scheduler is off, so the effect is idle until the host strikes it.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct LightningSettings {
   /// Settings-window visibility. Default `false` (the example opens it).
   pub open: bool,
   /// Whether the effect is drawn at all.
   pub active: bool,
   /// Bolt and flash color.
   pub color: Color32,
   /// Number of bolts drawn per strike.
   pub bolt_count: usize,
   /// Branch recursion depth (`0` draws the trunk only).
   pub branch_depth: u32,
   /// Bright core stroke thickness in pixels.
   pub thickness: f32,
   /// Midpoint-displacement jitter as a fraction of segment length.
   pub jitter: f32,
   /// Peak alpha of the full-area flash in `0.0..=1.0`.
   pub flash_intensity: f32,
   /// Strike duration in milliseconds.
   pub duration_ms: f32,
   /// Ambient interval in seconds (`0.0` disables the scheduler).
   pub ambient_interval_secs: f32,
   /// Layer the effect is painted on.
   pub layer: ParticleLayer,
}

impl Default for LightningSettings {
   fn default() -> Self {
      Self {
         open: false,
         active: true,
         color: DEFAULT_COLOR,
         bolt_count: DEFAULT_BOLT_COUNT,
         branch_depth: DEFAULT_BRANCH_DEPTH,
         thickness: DEFAULT_THICKNESS,
         jitter: DEFAULT_JITTER,
         flash_intensity: DEFAULT_FLASH_INTENSITY,
         duration_ms: DEFAULT_DURATION_MS,
         ambient_interval_secs: DEFAULT_AMBIENT_INTERVAL_SECS,
         layer: ParticleLayer::Foreground,
      }
   }
}

impl LightningSettings {
   /// `id_source` the settings window's "Strike now" button queues a strike on.
   ///
   /// Construct the effect with this id
   /// (`Lightning::from_settings(LightningSettings::STRIKE_ID, &settings)`) so
   /// the button drives it instead of a second, invisible state.
   pub const STRIKE_ID: &'static str = "lightning";

   /// Knobs back to defaults; `open` is preserved (a reset must not close the window).
   pub fn reset(&mut self) {
      let open = self.open;
      *self = Self::default();
      self.open = open;
   }

   /// Themed floating settings window; returns the window's outer `Response` when open.
   ///
   /// Window chrome plus a vertical scroll area around [`Self::body_ui`], so a
   /// viewport shorter than the controls scrolls instead of clipping them.
   pub fn show_window(&mut self, ctx: &Context) -> Option<egui::Response> {
      // Local copy — `settings_window` borrows `open` while the body borrows `self`.
      let mut open = self.open;
      let resp = settings_window(ctx, "Lightning Settings", &mut open, |ui| {
         self.body_ui(ui)
      });
      self.open = open;
      resp
   }

   /// Draw the settings controls into `ui` — no window chrome.
   ///
   /// Render them in your own container (`egui::Window`, `SidePanel`, `ScrollArea`, …)
   /// instead of using [`Self::show_window`]. If that container can be shorter than the
   /// controls, wrap it in an `egui::ScrollArea`.
   pub fn body_ui(&mut self, ui: &mut Ui) {
      let theme = begin_body(ui);

      ui.heading("General");
      ui.checkbox(&mut self.active, "Effect Active");
      ui.horizontal(|ui| {
         ui.label("Color:");
         ui.color_edit_button_srgba(&mut self.color);
      });

      ui.separator();
      ui.heading("Bolt");

      ui.horizontal(|ui| {
         ui.label("Bolts:");
         ui.add(Slider::new(&mut self.bolt_count, 1..=8));
      });

      ui.horizontal(|ui| {
         ui.label("Branch Depth:");
         ui.add(Slider::new(&mut self.branch_depth, 0..=8))
            .on_hover_text("0 draws the trunk only; each step forks a shorter side branch");
      });

      ui.horizontal(|ui| {
         ui.label("Thickness (px):");
         ui.add(Slider::new(&mut self.thickness, 0.5..=12.0));
      });

      ui.horizontal(|ui| {
         ui.label("Jitter:");
         ui.add(Slider::new(&mut self.jitter, 0.0..=0.6))
            .on_hover_text("Midpoint displacement as a fraction of segment length");
      });

      ui.separator();
      ui.heading("Flash");

      ui.horizontal(|ui| {
         ui.label("Intensity:");
         ui.add(Slider::new(&mut self.flash_intensity, 0.0..=1.0));
      });

      ui.horizontal(|ui| {
         ui.label("Duration (ms):");
         ui.add(Slider::new(&mut self.duration_ms, 80.0..=2000.0));
      });

      if ui.button("Strike now").clicked() {
         Lightning::from_settings(Self::STRIKE_ID, self).strike(ui.ctx());
      }

      ui.separator();
      ui.heading("Ambient");

      ui.horizontal(|ui| {
         ui.label("Interval (s):");
         ui.add(
            DragValue::new(&mut self.ambient_interval_secs)
               .speed(0.1)
               .range(0.0..=120.0)
               .suffix(" s"),
         )
         .on_hover_text("0 disables the ambient scheduler");
      });

      ui.separator();
      ui.heading("Rendering");
      ui.horizontal(|ui| {
         ui.label("Layer:");
         ui.selectable_value(
            &mut self.layer,
            ParticleLayer::Background,
            ParticleLayer::Background.to_str(),
         )
         .on_hover_text("Behind everything (Order::Background)");
         ui.selectable_value(
            &mut self.layer,
            ParticleLayer::Foreground,
            ParticleLayer::Foreground.to_str(),
         )
         .on_hover_text("In front of windows (Order::Foreground)");
         ui.selectable_value(
            &mut self.layer,
            ParticleLayer::Top,
            ParticleLayer::Top.to_str(),
         )
         .on_hover_text("On top of everything including debug/tooltips (Order::Debug)");
      });

      ui.add_space(theme.spacing.sm);
      if ui.button("Reset Defaults").clicked() {
         self.reset();
      }
   }
}

#[cfg(test)]
mod tests {
   use super::*;
   use egui::__run_test_ctx;
   use egui::Vec2;

   #[test]
   fn defaults_are_sane() {
      let settings = LightningSettings::default();
      assert!(!settings.open);
      assert!(settings.active);
      assert_eq!(settings.bolt_count, 1);
      assert!(settings.branch_depth >= 1);
      assert!(settings.thickness > 0.0);
      assert!((0.0..=1.0).contains(&settings.jitter));
      assert!((0.0..=1.0).contains(&settings.flash_intensity));
      assert!(settings.duration_ms > 100.0);
      assert_eq!(settings.ambient_interval_secs, 0.0);
      assert_eq!(settings.layer, ParticleLayer::Foreground);
   }

   #[test]
   fn reset_keeps_open() {
      let mut settings = LightningSettings {
         open: true,
         active: false,
         color: Color32::from_rgb(9, 9, 9),
         bolt_count: 7,
         branch_depth: 5,
         thickness: 9.0,
         jitter: 0.6,
         flash_intensity: 0.1,
         duration_ms: 900.0,
         ambient_interval_secs: 12.0,
         layer: ParticleLayer::Top,
      };

      settings.reset();

      assert!(settings.open, "reset must not close the window");
      assert_eq!(
         settings,
         LightningSettings {
            open: true,
            ..Default::default()
         }
      );
   }

   /// Lay the open settings window out on `screen` twice and return the second
   /// pass's outer window height (window rects settle on the second pass).
   ///
   /// Unlike [`egui::__run_test_ctx`] this keeps the real fonts: empty fonts
   /// make every galley 0 tall and the measured height meaningless.
   fn window_height_on(screen: Vec2) -> f32 {
      let ctx = Context::default();
      // Mirror the demo: install the theme once so spacing/`item_spacing`
      // match the real app instead of egui's tighter default style.
      let mut theme = crate::theme::Theme::new(crate::theme::ThemeKind::TokyoNight);
      theme.install(&ctx);
      let mut settings = LightningSettings {
         open: true,
         ..Default::default()
      };
      let mut size = Vec2::ZERO;
      for _ in 0..2 {
         let output = ctx.run_ui(
            egui::RawInput {
               screen_rect: Some(egui::Rect::from_min_size(
                  egui::Pos2::ZERO,
                  screen,
               )),
               ..Default::default()
            },
            |ui| {
               if let Some(resp) = settings.show_window(ui.ctx()) {
                  size = resp.rect.size();
               }
            },
         );
         output.drop_without_applying_deltas();
      }
      size.y
   }

   #[test]
   fn settings_body_fits_the_demo_root() {
      let height = window_height_on(Vec2::new(900.0, 2000.0));
      assert!(
         height <= 650.0,
         "settings window is {height} tall, root is 650"
      );
   }

   #[test]
   fn body_ui_renders_without_a_window() {
      let mut settings = LightningSettings::default();
      assert!(!settings.open);
      egui::__run_test_ui(|ui| {
         settings.body_ui(ui);
      });
      assert!(
         !settings.open,
         "body_ui must not touch the window flag"
      );
   }

   #[test]
   fn show_window_returns_response_only_when_open() {
      __run_test_ctx(|ctx| {
         let mut closed = LightningSettings::default();
         assert!(closed.show_window(ctx).is_none());
      });

      __run_test_ctx(|ctx| {
         let mut open = LightningSettings {
            open: true,
            ..Default::default()
         };
         assert!(open.show_window(ctx).is_some());
      });
   }
}
