//! Themed window chrome shared by the particle settings panels.

use egui::{Context, ScrollArea, Stroke, Ui};

use crate::theme::Theme;
use crate::widgets::Window;

/// Draws the themed settings window around `body`.
///
/// Crate [`Window`] with the current [`Theme::window_frame`], a vertical
/// `ScrollArea` so short viewports scroll instead of clipping, and a
/// chrome-free title frame. `open` receives the window-close toggle.
pub(crate) fn settings_window(
   ctx: &Context,
   title: &str,
   open: &mut bool,
   body: impl FnOnce(&mut Ui),
) -> Option<egui::Response> {
   let theme = Theme::current(ctx);
   let frame = theme.window_frame;
   let title_frame = frame.stroke(Stroke::NONE);
   let resp = Window::new(title)
      .open(open)
      .default_width(300.0)
      .resizable([true, true])
      .frame(frame)
      .title_frame(title_frame)
      .show(ctx, |ui| {
         ScrollArea::vertical().show(ui, |ui| {
            body(ui);
         });
      });

   resp.map(|r| r.response)
}

/// Prologue for a settings body: installs the compact row spacing and returns
/// the current theme so the caller can use its spacing tokens.
pub(crate) fn begin_body(ui: &mut Ui) -> Theme {
   let theme = Theme::current(ui.ctx());
   // Every row pays `item_spacing.y`; the installed style uses `md` (12).
   ui.spacing_mut().item_spacing.y = theme.spacing.sm;
   theme
}
