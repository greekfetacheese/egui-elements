//! Proof for the `motion` module: five sections, each a value or a container a
//! host used to hand-roll with a height cache, a clip rect and a sleep loop.
//!
//! Sections A–E read the same `Motion` built once per frame, so the timing
//! controls at the bottom retime every animation above them.
//!
//! Run with: `cargo run --example motion`

use eframe::egui::*;
use egui_elements::layout::{Flex, Item};
use egui_elements::motion::{Collapse, Easing, Fade, FadeMode, Lerp, Motion, animate_with};
use egui_elements::theme::{Theme, ThemeKind};
use egui_elements::widgets::{Button, Label};

/// One line: the collapse band is this body's measured height.
const SHORT_BODY: &str =
   "A one-line body: the band is exactly its measured height, and no height cache was kept.";

/// Four wrapped lines.
const MEDIUM_BODY: &str = "A four-line body. The collapse measures it once, on the first open, \
with an invisible probe run of this closure; that frame allocates nothing and the body runs once, \
not twice. Every later frame lays the body out at its natural height, paints it clipped to the \
animated band, and advances the cursor by the band alone. The paragraph after the accordion rides \
that band, which is why its label moves while the section is still opening.";

/// Ten wrapped lines.
const LONG_BODY: &str = "A ten-line body. Nothing here knows that a collapse is above it, and \
nothing here reports its own height: the container measured this text the first time the section \
opened and has followed that number since. A body that changes size animates one frame behind, \
because the height it follows is the last measured one. That trade is what removes the host-side \
height cache, the slack constant and the manual clip rect the account panel carries today. The \
clip is explicit here rather than a cell rect: a layout cell places content in a rect but never \
clips it, so a button that paints past its band still paints. A collapse intersects the band with \
the parent clip, so the body can only be seen inside the animated height. Closed, it allocates \
nothing at all, so the rows below close up with no leftover gap and no spacer to tune.";

fn main() -> eframe::Result {
   let options = eframe::NativeOptions {
      viewport: ViewportBuilder::default()
         .with_title("egui-elements motion")
         .with_inner_size([900.0, 760.0])
         .with_resizable(true),
      ..Default::default()
   };

   eframe::run_native(
      "egui-elements motion",
      options,
      Box::new(|cc| Ok(Box::new(MotionApp::new(cc)))),
   )
}

struct MotionApp {
   theme: Theme,
   open: bool,
   target: f32,
   duration: f32,
   easing: Easing,
   section_open: [bool; 3],
}

impl MotionApp {
   fn new(cc: &eframe::CreationContext<'_>) -> Self {
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&cc.egui_ctx);

      Self {
         theme,
         open: true,
         target: 50.0,
         duration: 0.3,
         easing: Easing::EaseOut,
         section_open: [true, false, false],
      }
   }
}

impl eframe::App for MotionApp {
   fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
      let mut open = self.open;
      let mut target = self.target;
      let mut duration = self.duration;
      let mut easing = self.easing;
      let mut section_open = self.section_open;
      let theme = &self.theme;
      let bg = theme.colors.bg;

      let motion = Motion::new(self.duration).easing(self.easing);

      CentralPanel::default()
         .frame(Frame::new().fill(bg).inner_margin(theme.spacing.margin(theme.spacing.lg)))
         .show(ui, |ui| {
            ScrollArea::vertical()
               .id_salt("motion_scroll")
               .auto_shrink([false, false])
               .show(ui, |ui| {
                  ui.spacing_mut().item_spacing.y = theme.spacing.md;

                  section_value(ui, theme, motion, &mut target);
                  section_interpolation(ui, theme);
                  section_fade(ui, theme, motion, &mut open);
                  section_collapse(ui, theme, motion, &mut section_open);
                  section_timing(ui, theme, &mut duration, &mut easing);
               });
         });

      self.open = open;
      self.target = target;
      self.duration = duration;
      self.easing = easing;
      self.section_open = section_open;
   }
}

/// A section caption: what the reader should look at.
fn caption(ui: &mut Ui, theme: &Theme, text: &str) {
   ui.add(Label::new(
      RichText::new(text).size(theme.typography.small).color(theme.colors.text_muted),
      None,
   ));
}

/// A section heading.
fn heading(ui: &mut Ui, theme: &Theme, text: &str) {
   ui.add(Label::new(
      RichText::new(text)
         .size(theme.typography.normal)
         .strong()
         .color(theme.colors.text),
      None,
   ));
}

/// A. The `animate` proof: five targets, one per-`Id` value, and a bar that
/// follows it. The number is the value on screen, not the target.
fn section_value(ui: &mut Ui, theme: &Theme, motion: Motion, target: &mut f32) {
   heading(ui, theme, "A. Value animation");
   caption(
      ui,
      theme,
      "animate(ui, \"value_demo\", target) eases 0..=100 over the selected duration; a new target retargets from the bar's current value.",
   );

   Flex::row("value_targets").gap(theme.spacing.sm).show(ui, |f| {
      for value in [0.0, 25.0, 50.0, 75.0, 100.0] {
         f.item(Item::new(), |ui| {
            let text = format!("{value:.0}");
            if ui.add(Button::new(text).selected(*target == value)).clicked() {
               *target = value;
            }
         });
      }
   });

   let v = animate_with(ui, "value_demo", *target, motion);
   let (bar, _) = ui.allocate_exact_size(vec2(ui.available_width(), 18.0), Sense::hover());
   let filled = Rect::from_min_size(
      bar.min,
      vec2(bar.width() * v / 100.0, bar.height()),
   );
   let fill = theme.colors.widget_bg.lerp(theme.colors.accent, v / 100.0);
   ui.painter().rect_filled(
      filled,
      CornerRadius::same(theme.corner_radius),
      fill,
   );

   caption(
      ui,
      theme,
      &format!("{v:.1} — settles without repainting: the number stops changing on its own."),
   );
}

/// B. The `Lerp` proof, with no animation: nine swatches at `t = i / 8`, one
/// blend per step. Colours interpolate in gamma space, not linear light.
fn section_interpolation(ui: &mut Ui, theme: &Theme) {
   heading(ui, theme, "B. Interpolation");
   caption(
      ui,
      theme,
      "Lerp is the curve a colour or rect needs; nine steps expose it without a tooltip.",
   );

   Flex::row("swatches").gap(theme.spacing.sm).show(ui, |f| {
      for i in 0..9 {
         let t = i as f32 / 8.0;
         f.item(Item::new(), |ui| {
            let (rect, _) = ui.allocate_exact_size(vec2(28.0, 28.0), Sense::hover());
            ui.painter().rect_filled(
               rect,
               CornerRadius::same(theme.corner_radius),
               theme.colors.bg.lerp(theme.colors.error, t),
            );
         });
      }
   });

   caption(
      ui,
      theme,
      "colors.bg.lerp(colors.error, t) with t = i / 8 for i in 0..=8 — the two roles and the blend, left to right.",
   );
}

/// C. The `Fade` proof: the left panel closes instantly, so it never stacks
/// with the right one; the right one fades out while its body keeps running.
fn section_fade(ui: &mut Ui, theme: &Theme, motion: Motion, open: &mut bool) {
   heading(ui, theme, "C. Fade");
   caption(
      ui,
      theme,
      "One toggle, two panels: the same opacity animation with the two close modes.",
   );

   Flex::row("fade_panels").gap(theme.spacing.md).show(ui, |f| {
      f.item(Item::grow(1.0), |ui| {
         Fade::new("fade_in", *open).motion(motion).show(ui, |ui| {
            theme.frame2.show(ui, |ui| {
               ui.set_width(ui.available_width());
               ui.add(Label::new("FadeMode::In", None));
               ui.add(Label::new("Fades in from nothing.", None));
               ui.add(Label::new(
                  "Closes the instant it is toggled.",
                  None,
               ));
            });
         });
      });
      f.item(Item::grow(1.0), |ui| {
         Fade::new("fade_both", *open)
            .mode(FadeMode::Both)
            .motion(motion)
            .show(ui, |ui| {
               theme.frame2.show(ui, |ui| {
                  ui.set_width(ui.available_width());
                  ui.add(Label::new("FadeMode::Both", None));
                  ui.add(Label::new("Fades in and out.", None));
                  ui.add(Label::new("The body runs while it fades.", None));
               });
            });
      });
   });

   if ui.add(Button::new("Open / close both")).clicked() {
      *open = !*open;
   }

   caption(
      ui,
      theme,
      "Left closes instantly (FadeMode::In) so nothing stacks; right fades out (FadeMode::Both).",
   );
}

/// D. The height proof: three bodies of clearly different heights, each in its
/// own `Collapse`. The label below adds up the three heights it was given, so
/// it moves with the animation without knowing about it.
fn section_collapse(ui: &mut Ui, theme: &Theme, motion: Motion, section_open: &mut [bool; 3]) {
   heading(ui, theme, "D. Collapse");
   caption(
      ui,
      theme,
      "Three accordion rows; each body is laid out inside its collapse, so the label below rides the animated height.",
   );

   let sections: [(&str, &str, &str); 3] = [
      (
         "Section 1 — short body",
         "Fade and nav both allocate",
         SHORT_BODY,
      ),
      (
         "Section 2 — medium body",
         "Measured once",
         MEDIUM_BODY,
      ),
      (
         "Section 3 — long body",
         "Clipped to the band",
         LONG_BODY,
      ),
   ];

   let mut total = 0.0;
   for (i, (title, caption_text, body)) in sections.iter().enumerate() {
      if ui.add(Button::new(*title)).clicked() {
         section_open[i] = !section_open[i];
      }

      let height = Collapse::new(("section", i), section_open[i]).motion(motion).show(ui, |ui| {
         theme.frame2.show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.add(Label::new(
               RichText::new(*caption_text)
                  .size(theme.typography.small)
                  .color(theme.colors.text_muted),
               None,
            ));
            ui.add(Label::new(*body, None).wrap());
         });
      });
      total += height;
   }

   ui.add(Label::new(
      format!("nav rides the height: {total:.1} pt"),
      None,
   ));

   caption(
      ui,
      theme,
      "Each body is measured once on its first open, then the band animates and the body is clipped to it.",
   );
}

/// E. The timing controls: the duration and the curve every container above
/// reads. `Instant` is `Motion::instant()`, duration `0.0`.
fn section_timing(ui: &mut Ui, theme: &Theme, duration: &mut f32, easing: &mut Easing) {
   heading(ui, theme, "E. Timing");
   caption(
      ui,
      theme,
      "The same Motion is built once per frame and handed to every section; change it here and the sections above retime.",
   );

   let durations: [(&str, f32); 4] = [
      ("Instant", 0.0),
      ("0.15 s", 0.15),
      ("0.3 s", 0.3),
      ("0.6 s", 0.6),
   ];
   Flex::row("durations").gap(theme.spacing.sm).show(ui, |f| {
      for (text, secs) in durations {
         f.item(Item::new(), |ui| {
            if ui.add(Button::new(text).selected(*duration == secs)).clicked() {
               *duration = secs;
            }
         });
      }
   });

   let curves: [(&str, Easing); 4] = [
      ("Linear", Easing::Linear),
      ("In", Easing::EaseIn),
      ("Out", Easing::EaseOut),
      ("In-out", Easing::EaseInOut),
   ];
   Flex::row("curves").gap(theme.spacing.sm).show(ui, |f| {
      for (text, curve) in curves {
         f.item(Item::new(), |ui| {
            if ui.add(Button::new(text).selected(*easing == curve)).clicked() {
               *easing = curve;
            }
         });
      }
   });

   caption(
      ui,
      theme,
      "Motion::instant() is duration 0 — every container above lands immediately and requests no repaint.",
   );
}
