//! The shared plate: row allocation, interaction weights and the paint pass.

use std::sync::Arc;

use egui::{
   Color32, CornerRadius, Galley, Rect, Response, Sense, Stroke, TextStyle, Ui, Vec2, WidgetText,
   pos2, vec2,
};

use super::glyph::{self, dot_shapes, tick_shapes};
use super::{ToggleShadow, ToggleVisuals};
use crate::gradient::Fill;
use crate::motion::{InteractT, Lerp, Motion, animate_from};
use crate::theme::Theme;
use crate::visuals::paint_shape;

/// Default plate size in points. The reference is 16 at 1×, up from egui's own `icon_width` of 14.
pub(crate) const DEFAULT_SIZE: f32 = 18.0;
/// Border width as a fraction of the plate size, floored at one point.
const BORDER_RATIO: f32 = 0.0625;
/// Track width as a fraction of its height (the reference switch measures 1.91).
pub(crate) const TRACK_RATIO: f32 = 1.9;
/// Knob diameter as a fraction of the track height.
const KNOB_RATIO: f32 = 0.82;
/// Gap between the knob and each end of the track, as a fraction of its height.
const KNOB_INSET_RATIO: f32 = 0.09;
/// Plate scale while the pointer is down.
const PRESS_SCALE: f32 = 0.93;
/// Glyph scale at `t = 0`; it pops to 1.0 and overshoots on the way.
const GLYPH_POP_FROM: f32 = 0.35;
/// How fast the glyph's alpha reaches full, relative to `t`.
const GLYPH_FADE: f32 = 2.2;
/// Alpha multiplier applied to a disabled toggle.
const DISABLED_ALPHA: f32 = 0.5;

/// The outline a plate paints.
#[derive(Clone, Copy)]
pub(crate) enum PlateShape {
   /// A box whose corner radius is `ratio` of the plate size.
   Rounded { ratio: f32 },
   /// A circle.
   Circle,
}

/// The glyph a plate pops in when it is checked.
#[derive(Clone, Copy)]
pub(crate) enum PlateGlyph {
   Tick,
   Dot,
   /// The switch's knob: painted in both states, its centre travelling on `t`.
   Knob,
}

/// The row a toggle allocates: plate, `icon_spacing`, label — one `Sense::click()` rect.
pub(crate) struct Row<'u> {
   ui: &'u mut Ui,
   rect: Rect,
   plate: Vec2,
   pub(crate) response: Response,
   galley: Option<Arc<Galley>>,
}

impl<'u> Row<'u> {
   /// Lay the label out to wrap inside what the plate leaves, then allocate the row.
   pub(crate) fn new(ui: &'u mut Ui, text: Option<WidgetText>, plate: Vec2) -> Self {
      let gap = ui.spacing().icon_spacing;
      let galley = text.map(|text| {
         let available = (ui.available_width() - plate.x - gap).max(plate.x);
         text.into_galley(
            ui,
            Some(egui::TextWrapMode::Wrap),
            available,
            TextStyle::Body,
         )
      });
      let text_size = galley.as_ref().map(|galley| galley.size()).unwrap_or(Vec2::ZERO);

      let row = vec2(
         plate.x
            + if galley.is_some() {
               gap + text_size.x
            } else {
               0.0
            },
         plate.y.max(text_size.y).max(ui.spacing().interact_size.y),
      );
      let (rect, response) = ui.allocate_exact_size(row, Sense::click());

      Self {
         ui,
         rect,
         plate,
         response,
         galley,
      }
   }

   /// Run the toggle clock, blend the interaction weights and paint plate, glyph and label.
   /// `checked` is the state AFTER this frame's click.
   pub(crate) fn paint(
      self,
      checked: bool,
      shape: PlateShape,
      glyph: PlateGlyph,
      visuals: &ToggleVisuals,
      shadow: ToggleShadow,
      motion: Option<Motion>,
   ) -> Response {
      let Row {
         ui,
         rect,
         plate,
         response,
         galley,
      } = self;

      let enabled = ui.is_enabled();
      let gap = ui.spacing().icon_spacing;
      let colors = Theme::colors_from_ctx(ui.ctx());

      if ui.is_rect_visible(rect) {
         let motion = motion.unwrap_or_else(|| Motion::from_ui(ui));
         let target = if checked { 1.0 } else { 0.0 };
         // Seeded at the target: a toggle that starts checked must be checked, not animate itself on.
         let t = animate_from(
            ui,
            response.id.with("check"),
            target,
            target,
            motion,
         );
         let across = InteractT::of(
            ui,
            response.id.with("face"),
            response.hovered(),
            response.is_pointer_button_down_on(),
            Motion::from_ui(ui),
         );

         let scale = 1.0 - (1.0 - PRESS_SCALE) * across.press;
         let box_rect = Rect::from_center_size(
            pos2(rect.left() + plate.x * 0.5, rect.center().y),
            plate * scale,
         );

         let shade = |color: Color32| {
            if enabled {
               color
            } else {
               color.gamma_multiply(DISABLED_ALPHA)
            }
         };

         // Each state's idle shade is blended into the checked colour by `t`, so at `t == 1.0` every state
         // collapses onto the accent and the pressed feedback is the scale, not a colour.
         let fill = across.blend(
            shade(visuals.bg.lerp(visuals.bg_checked, t)),
            shade(visuals.bg_hover.lerp(visuals.bg_checked, t)),
            shade(visuals.bg_hover.lerp(visuals.bg_checked, t)),
         );
         let border = across.blend(
            shade(visuals.border.lerp(visuals.border_checked, t)),
            shade(visuals.border_hover.lerp(visuals.border_checked, t)),
            shade(visuals.border_hover.lerp(visuals.border_checked, t)),
         );

         let stroke = Stroke::new((plate.y * BORDER_RATIO).max(1.0), border);
         let radius = match shape {
            PlateShape::Rounded { ratio } => {
               CornerRadius::same((plate.min_elem() * ratio).round() as u8)
            }
            PlateShape::Circle => CornerRadius::same(u8::MAX),
         };
         let lift = if enabled { shadow } else { ToggleShadow::NONE };

         ui.painter().add(paint_shape(
            box_rect,
            Fill::solid(fill),
            stroke,
            radius,
            lift.to_shadow(plate.y * scale, shade(visuals.bg_checked), t),
         ));

         match glyph {
            PlateGlyph::Knob => {
               let diameter = box_rect.height() * KNOB_RATIO;
               let inset = box_rect.height() * KNOB_INSET_RATIO;
               let travel = box_rect.width() - 2.0 * (inset + diameter * 0.5);
               let centre = pos2(
                  box_rect.left() + inset + diameter * 0.5 + travel * t,
                  box_rect.center().y,
               );
               ui.painter().add(egui::Shape::circle_filled(
                  centre,
                  diameter * 0.5,
                  shade(visuals.border.lerp(visuals.mark, t)),
               ));
            }
            PlateGlyph::Tick | PlateGlyph::Dot => {
               if t > 0.0 {
                  let color = shade(visuals.mark).gamma_multiply((t * GLYPH_FADE).min(1.0));
                  let pop =
                     GLYPH_POP_FROM + (1.0 - GLYPH_POP_FROM) * egui::emath::easing::back_out(t);
                  let glyph_rect = Rect::from_center_size(box_rect.center(), box_rect.size() * pop);
                  let shapes = match glyph {
                     PlateGlyph::Tick => tick_shapes(
                        glyph_rect,
                        color,
                        glyph::stroke_width(glyph_rect.width()),
                     ),
                     PlateGlyph::Dot => dot_shapes(glyph_rect, color),
                     PlateGlyph::Knob => Vec::new(),
                  };
                  for shape in shapes {
                     ui.painter().add(shape);
                  }
               }
            }
         }

         if let Some(galley) = galley {
            let pos = pos2(
               box_rect.right() + gap,
               rect.center().y - galley.size().y * 0.5,
            );
            let color = shade(colors.text);
            ui.painter().galley(pos, galley, color);
         }
      }

      response
   }
}
