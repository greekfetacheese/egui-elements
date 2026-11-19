use super::label::Label;
use crate::motion::{InteractT, Motion};
use egui::{
   Color32, Pos2, Rect, Response, Role, Sense, Stroke, StrokeKind, Ui, Vec2, Widget, WidgetInfo,
   epaint::RectShape,
};

/// A widget that displays multiple [`Label`]s in a single horizontal line,
/// with consistent vertical alignment (centered) and optional spacing between them.
#[must_use = "You should put this widget in a ui with `ui.add(widget);`"]
#[derive(Clone)]
pub struct MultiLabel {
   labels: Vec<Label>,
   inter_spacing: f32,
   sense: Option<Sense>,
}

impl MultiLabel {
   /// Create a new `MultiLabel` from a vec of [`Label`].
   /// The default spacing between labels is 6.0.
   pub fn new(labels: Vec<Label>) -> Self {
      Self {
         labels,
         inter_spacing: 6.0,
         sense: None,
      }
   }

   /// Set the space between each pair of labels.
   pub fn inter_spacing(mut self, spacing: f32) -> Self {
      self.inter_spacing = spacing;
      self
   }

   /// Make the entire multi-label respond to clicks and/or drags.
   /// This will use the same visuals for all inner content.
   pub fn sense(mut self, sense: Sense) -> Self {
      self.sense = Some(sense);
      self
   }
}

impl Widget for MultiLabel {
   fn ui(self, ui: &mut Ui) -> Response {
      if self.labels.is_empty() {
         return ui.allocate_response(Vec2::ZERO, Sense::empty());
      }

      // Calculate Intrinsic Sizes (no wrapping)
      let mut total_width = 0.0;
      let mut max_height: f32 = 0.0;
      let mut label_sizes: Vec<Vec2> = Vec::with_capacity(self.labels.len());

      let mut text = String::new();
      for label in &self.labels {
         let (galley, size) = label.galley_and_size(ui, f32::INFINITY);
         label_sizes.push(size);
         total_width += size.x;
         max_height = max_height.max(size.y);

         // The parts are painted one by one, so their text reaches AT only as this one
         // name. A space keeps the last word of one part off the first of the next.
         if !text.is_empty() {
            text.push(' ');
         }
         text.push_str(galley.text());
      }

      let num_gaps = (self.labels.len() as f32).max(0.0) - 1.0;
      total_width += self.inter_spacing * num_gaps;
      let desired_size = Vec2::new(total_width, max_height);

      // Allocate Space
      let sense = self.sense.unwrap_or(Sense::empty());
      let (rect, response) = ui.allocate_exact_size(desired_size, sense);

      // Paint
      if ui.is_rect_visible(rect) {
         let visuals = if self.sense.is_some() {
            ui.style().interact(&response).clone()
         } else {
            ui.style().noninteractive().clone()
         };

         // Paint Background
         let is_interactive = self.sense.is_some();
         let across = if is_interactive {
            InteractT::of(
               ui,
               response.id,
               response.hovered() || response.highlighted(),
               response.is_pointer_button_down_on() || response.clicked() || response.has_focus(),
               Motion::from_ui(ui),
            )
         } else {
            InteractT::default()
         };
         if is_interactive && (across.hover > 0.0 || across.press > 0.0) {
            let widgets = &ui.style().visuals.widgets;
            let background_rect = rect.expand(visuals.expansion);
            ui.painter().add(RectShape::new(
               background_rect,
               visuals.corner_radius,
               across.blend(
                  Color32::TRANSPARENT,
                  widgets.hovered.weak_bg_fill,
                  widgets.active.weak_bg_fill,
               ),
               across.blend(
                  Stroke::NONE,
                  widgets.hovered.bg_stroke,
                  widgets.active.bg_stroke,
               ),
               StrokeKind::Inside,
            ));
         }

         // Layout and Paint Each Label
         let mut x = rect.left();
         let mut i = 0;

         for label in self.labels {
            if i > 0 {
               x += self.inter_spacing;
            }

            let label_size = label_sizes[i];
            let y_offset = (max_height - label_size.y) * 0.5;
            let label_rect = Rect::from_min_size(Pos2::new(x, rect.top() + y_offset), label_size);

            label.paint_content_within_rect(ui, label_rect, &visuals);

            x += label_size.x;
            i += 1;
         }
      }

      let clickable = self.sense.is_some_and(|sense| sense.senses_click());
      response.widget_info(|| {
         if clickable {
            WidgetInfo::labeled(Role::Button, ui.is_enabled(), &text)
         } else {
            WidgetInfo::labeled(Role::Label, ui.is_enabled(), &text)
         }
      });

      response
   }
}

#[cfg(test)]
mod tests {
   use super::*;
   use egui::accesskit::{Node, Role};
   use egui::{Context, RawInput};

   fn accesskit_nodes(ctx: &Context, add: impl FnMut(&mut Ui)) -> Vec<Node> {
      ctx.enable_accesskit();

      let mut output = ctx.run_ui(RawInput::default(), add);
      let update = output
         .platform_output
         .accesskit_update
         .take()
         .expect("an AccessKit-enabled frame must produce a tree update");
      let nodes = update.nodes.into_iter().map(|(_, node)| node).collect();
      output.drop_without_applying_deltas();

      nodes
   }

   fn node_with_role(nodes: &[Node], role: Role) -> &Node {
      nodes
         .iter()
         .find(|node| node.role() == role)
         .expect("no node with the expected role")
   }

   #[test]
   fn multilabel_speaks_its_parts_as_one_name() {
      let ctx = Context::default();
      let nodes = accesskit_nodes(&ctx, |ui| {
         ui.add(MultiLabel::new(vec![
            Label::new("Delete", None),
            Label::new("3 selected", None),
         ]));
      });

      let node = node_with_role(&nodes, Role::Label);

      assert_eq!(node.value(), Some("Delete 3 selected"));
   }

   #[test]
   fn a_clickable_multilabel_is_a_button() {
      let ctx = Context::default();
      let nodes = accesskit_nodes(&ctx, |ui| {
         ui.add(MultiLabel::new(vec![Label::new("Open", None)]).sense(Sense::click()));
      });

      let node = node_with_role(&nodes, Role::Button);

      assert_eq!(node.label(), Some("Open"));
   }
}
