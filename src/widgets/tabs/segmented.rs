//! The segmented chrome: a pill track, the hover washes, and the indicator that slides and lifts.

use egui::{CornerRadius, Rect, Shadow, Stroke, StrokeKind, Ui};

use super::{HOVER_WASH, PRESS_WASH, SETTLED, Strip, shade};
use crate::motion::Lerp;

/// Segment inset as a share of the strip height; the pill rides inside this ring.
///
/// The reference pads a 48-point track with 6 points all round, i.e. 0.125 of the track. Held at 0.09
/// because the crate's strip is shorter than the reference's, and the ring has to survive the
/// `fill_width` row: [Tabs::height](super::Tabs::height) scales the whole geometry.
const INSET_RATIO: f32 = 0.09;
/// Floor of an indicator radius derived from the strip, in points.
const MIN_RADIUS: f32 = 2.0;

/// Paint the track, the hover washes and the indicator; returns the band the labels sit in.
pub(super) fn paint(ui: &Ui, strip: &Strip<'_>) -> Rect {
   let Strip {
      rect,
      segments,
      faces,
      visuals,
      shadow,
      selected,
      t,
      corner_radius,
      enabled,
      ..
   } = *strip;

   let painter = ui.painter();
   let inset = (rect.height() * INSET_RATIO).round().max(1.0);
   let indicator_radius =
      CornerRadius::same((f32::from(corner_radius.nw) - inset).max(MIN_RADIUS).round() as u8);

   painter.rect_filled(rect, corner_radius, shade(visuals.track, enabled));
   if !visuals.track_border.is_empty() {
      let border = Stroke::new(
         visuals.track_border.width,
         shade(visuals.track_border.color, enabled),
      );
      painter.rect_stroke(rect, corner_radius, border, StrokeKind::Inside);
   }

   // ---- hover wash ----------------------------------------------------
   // The selected segment keeps its wash until the indicator has landed on it: a click moves the
   // index on the release frame, and the pill still has a whole leg to travel, so skipping the wash
   // the moment the segment is selected would drop it back to the bare track mid-motion.
   let settled = (t - selected as f32).abs() < SETTLED;
   for (index, face) in faces.iter().enumerate() {
      if index == selected && settled {
         continue;
      }
      let wash = face.blend(0.0, HOVER_WASH, PRESS_WASH);
      if wash > 0.001 {
         painter.rect_filled(
            segments[index].shrink(inset),
            indicator_radius,
            shade(visuals.hover, enabled).gamma_multiply(wash),
         );
      }
   }

   // ---- indicator -----------------------------------------------------
   // Between two neighbours only: the pill travels from one segment to the other and stretches
   // between them mid-flight, which reads as a segment sliding rather than a cell lighting up.
   let from = (t.floor() as usize).min(segments.len() - 1);
   let to = (from + 1).min(segments.len() - 1);
   let indicator_rect =
      segments[from].shrink(inset).lerp(segments[to].shrink(inset), t - from as f32);

   let lift = shadow.to_shadow(indicator_rect.height(), visuals.indicator, 1.0);
   if lift != Shadow::NONE {
      painter.add(lift.as_shape(indicator_rect, indicator_radius));
   }
   painter.rect_filled(
      indicator_rect,
      indicator_radius,
      shade(visuals.indicator, enabled),
   );

   rect
}
