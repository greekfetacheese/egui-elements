//! The underline chrome: a baseline under fitted tabs, and one accent bar travelling between them.

use egui::{CornerRadius, Rect, Ui, pos2};

use super::{HOVER_WASH, PRESS_WASH, SETTLED, Strip, shade};
use crate::motion::Lerp;

/// Underline thickness as a share of the label size.
const UNDERLINE_RATIO: f32 = 0.14;
/// Floor of the underline thickness, in points.
const MIN_UNDERLINE: f32 = 2.0;
/// Space between the labels and the underline, as a share of the label size.
const GAP_RATIO: f32 = 0.35;
/// Hover wash corner radius, as a share of the label band's height.
const WASH_RADIUS_RATIO: f32 = 0.18;

/// Thickness of the accent bar under the selected tab.
fn thickness(font_size: f32) -> f32 {
   (font_size * UNDERLINE_RATIO).max(MIN_UNDERLINE)
}

/// The room this style keeps under the labels for the baseline, the gap and the bar.
pub(super) fn reserve(font_size: f32) -> f32 {
   thickness(font_size) + font_size * GAP_RATIO
}

/// Paint the baseline, the hover washes and the underline bar; returns the band the labels sit in.
pub(super) fn paint(ui: &Ui, strip: &Strip<'_>) -> Rect {
   let Strip {
      rect,
      segments,
      faces,
      visuals,
      selected,
      t,
      font_size,
      enabled,
      ..
   } = *strip;

   let bottom = rect.bottom();
   let bar_height = thickness(font_size);
   let band = Rect::from_min_max(
      rect.min,
      pos2(rect.right(), bottom - reserve(font_size)),
   );
   let painter = ui.painter();

   // ---- baseline ------------------------------------------------------
   // The reference draws a one-point rule along the row and puts the two-point bar on top of it, so
   // the bar reads as the selected tab rather than as a floating stripe.
   if !visuals.track_border.is_empty() {
      let height = visuals.track_border.width.max(1.0);
      painter.rect_filled(
         Rect::from_min_max(
            pos2(rect.left(), bottom - height),
            pos2(rect.right(), bottom),
         ),
         CornerRadius::ZERO,
         shade(visuals.track_border.color, enabled),
      );
   }

   // ---- hover wash ----------------------------------------------------
   // Same rule as the segmented strip: the selected tab keeps its wash until the bar has landed on
   // it, so a click never falls back to the bare surface while the motion is still running.
   let settled = (t - selected as f32).abs() < SETTLED;
   let wash_radius = CornerRadius::same((band.height() * WASH_RADIUS_RATIO).round() as u8);
   for (index, face) in faces.iter().enumerate() {
      if index == selected && settled {
         continue;
      }
      let wash = face.blend(0.0, HOVER_WASH, PRESS_WASH);
      if wash > 0.001 {
         painter.rect_filled(
            Rect::from_min_max(
               pos2(segments[index].left(), band.top()),
               pos2(segments[index].right(), band.bottom()),
            ),
            wash_radius,
            shade(visuals.hover, enabled).gamma_multiply(wash),
         );
      }
   }

   // ---- underline -----------------------------------------------------
   // One bar, travelling and stretching between neighbours exactly as the pill does, seated on the
   // baseline so the two share their bottom edge.
   let from = (t.floor() as usize).min(segments.len() - 1);
   let to = (from + 1).min(segments.len() - 1);
   let travel = segments[from].lerp(segments[to], t - from as f32);
   painter.rect_filled(
      Rect::from_min_max(
         pos2(travel.left(), bottom - bar_height),
         pos2(travel.right(), bottom),
      ),
      CornerRadius::ZERO,
      shade(visuals.indicator, enabled),
   );

   band
}
