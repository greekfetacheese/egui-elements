//! Painting primitives the menu family shares.
//!
//! These replace upstream helpers that lived in other modules of
//! `egui-elegance`; the signature is kept where the call site did not need to
//! change.

use std::sync::Arc;

use egui::{
   Color32, FontId, FontSelection, Frame, Galley, Rect, Response, RichText, Sense, TextWrapMode,
   Ui, Vec2, WidgetText,
};

use crate::theme::Theme;

/// Popup chrome for a menu panel: the theme's overlay frame with the tight
/// margin a list of rows wants instead of a window's roomier one.
pub(super) fn menu_frame(theme: &Theme) -> Frame {
   theme.window_frame.inner_margin(theme.spacing.margin(theme.spacing.xs))
}

/// Lay `text` out at `size` without wrapping, in the crate's proportional font.
pub(super) fn galley(ui: &Ui, text: &str, size: f32, strong: bool) -> Arc<Galley> {
   let mut rich = RichText::new(text).size(size).color(Color32::PLACEHOLDER);

   if strong {
      rich = rich.strong();
   }

   let font = FontSelection::FontId(FontId::proportional(size));

   WidgetText::from(rich).into_galley(
      ui,
      Some(TextWrapMode::Extend),
      f32::INFINITY,
      font,
   )
}

/// `color` at `alpha`, straight alpha so a tint does not brighten the fill.
pub(super) fn with_alpha(color: Color32, alpha: u8) -> Color32 {
   Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}

/// [`Ui::allocate_at_least`] whose [`Response`] egui also sees as disabled when
/// `enabled` is false.
///
/// Painting happens afterwards, in the restored `Ui`, so the widget's own
/// disabled palette is not faded again by egui's disabled opacity.
pub(super) fn allocate_at_least(
   ui: &mut Ui,
   desired_size: Vec2,
   sense: Sense,
   enabled: bool,
) -> (Rect, Response) {
   let response = ui.add_enabled(enabled, |ui: &mut Ui| {
      ui.allocate_response(desired_size, sense)
   });

   (response.rect, response)
}
