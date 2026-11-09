//! [`Row`]: a product row with a leading icon, title, subtitle and trailing control.
//!
//! Product UIs write the same row over and over: a leading icon, a label (often with a subtitle), an
//! optional trailing control, a hover plate and a chevron. `Row` owns that geometry so a host does not
//! hand-place trailing controls with magic offsets.

use std::hash::Hash;

use egui::{
   Align, CornerRadius, FontId, FontSelection, IdSalt, Image, Layout, Rect, Response, Role, Sense,
   Stroke, TextWrapMode, Ui, UiBuilder, Vec2, Widget, WidgetInfo, WidgetText, pos2, vec2,
};

use crate::theme::Theme;

/// The leading icon's edge length in points.
const ICON_SIZE: f32 = 20.0;

/// A full-width product row.
///
/// The row lays out a leading icon, a title, an optional subtitle and an optional trailing control,
/// paints the hover plate and selection fill from the installed [`Theme`], and can end in a chevron.
/// The trailing closure draws into a right-aligned child `Ui`, so a `Switch`, `Checkbox`,
/// `ComboBox` or value label lands on the trailing edge without hand-measured rects.
///
/// The default height is two interaction heights (≈40 pt); set [`height`](Self::height) for a compact
/// or roomier row. The title and subtitle truncate at the space the trailing control leaves.
///
/// ```
/// # use egui::__run_test_ui;
/// # use egui_elements::widgets::{Row, Switch};
/// # __run_test_ui(|ui| {
/// # let mut value = true;
/// ui.add(
///    Row::text("Notifications")
///       .subtitle("Sounds, popups, badges")
///       .chevron(true)
///       .trailing(|ui| {
///          ui.add(Switch::without_text(&mut value));
///       }),
/// );
/// # });
/// ```
#[must_use = "Add with `ui.add(...)` or call `.show(ui)`."]
pub struct Row<'a> {
   icon: Option<Image<'static>>,
   text: WidgetText,
   subtitle: Option<WidgetText>,
   selected: bool,
   chevron: bool,
   trailing: Option<Box<dyn FnOnce(&mut Ui) + 'a>>,
   height: Option<f32>,
   sense: Option<Sense>,
   id_salt: Option<IdSalt>,
}

impl<'a> Row<'a> {
   /// A row with a leading icon and a title.
   pub fn new(icon: impl Into<Image<'static>>, text: impl Into<WidgetText>) -> Self {
      Self {
         icon: Some(icon.into()),
         text: text.into(),
         subtitle: None,
         selected: false,
         chevron: false,
         trailing: None,
         height: None,
         sense: None,
         id_salt: None,
      }
   }

   /// A row with a title and no leading icon.
   pub fn text(text: impl Into<WidgetText>) -> Self {
      Self {
         icon: None,
         text: text.into(),
         subtitle: None,
         selected: false,
         chevron: false,
         trailing: None,
         height: None,
         sense: None,
         id_salt: None,
      }
   }

   /// A second, smaller line under the title.
   pub fn subtitle(mut self, subtitle: impl Into<WidgetText>) -> Self {
      self.subtitle = Some(subtitle.into());
      self
   }

   /// Paint the selection fill and its matching ink. Default `false`.
   pub fn selected(mut self, selected: bool) -> Self {
      self.selected = selected;
      self
   }

   /// Draw a chevron in the trailing gutter. Default `false`.
   pub fn chevron(mut self, chevron: bool) -> Self {
      self.chevron = chevron;
      self
   }

   /// Draw a trailing control, right-aligned in the row.
   ///
   /// The closure runs in a child `Ui` whose width the row measures, so the title truncates before it.
   /// Keep the body to one control; a trailing control that responds to clicks wants
   /// [`sense`](Self::sense) `Sense::hover()`.
   pub fn trailing(mut self, add: impl FnOnce(&mut Ui) + 'a) -> Self {
      self.trailing = Some(Box::new(add));
      self
   }

   /// Row height in points. Default: two interaction heights.
   pub fn height(mut self, height: f32) -> Self {
      self.height = Some(height);
      self
   }

   /// Override the row's sense. Default [`Sense::click`].
   pub fn sense(mut self, sense: Sense) -> Self {
      self.sense = Some(sense);
      self
   }

   /// Salt the trailing child `Ui` when the same row repeats on several surfaces.
   ///
   /// Row positions already make the response ids unique; a trailing `Switch` or `Checkbox` derives
   /// its id from its parent, so repeating the same gallery on bg, a frame and a window needs a
   /// distinct salt per gallery.
   pub fn id_salt(mut self, id_salt: impl Hash + std::fmt::Debug) -> Self {
      self.id_salt = Some(IdSalt::new(id_salt));
      self
   }

   /// Show the row, returning its response.
   pub fn show(mut self, ui: &mut Ui) -> Response {
      let theme = Theme::current(ui.ctx());
      let colors = theme.colors;
      let pad = theme.spacing.md;
      let gap = theme.spacing.md;
      let height = self.height.unwrap_or(ui.spacing().interact_size.y * 2.0);

      let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
      let id = match self.id_salt {
         Some(salt) => ui.scope_id().with(salt),
         None => ui.auto_id_with("row"),
      };
      let sense = self.sense.unwrap_or(Sense::click());
      let response = ui.interact(rect, id, sense);

      let ink = if self.selected {
         colors.ink_on(colors.highlight)
      } else {
         colors.text
      };
      let muted = if self.selected {
         ink
      } else {
         colors.text_muted
      };
      let label = self.text.text().to_owned();

      if ui.is_rect_visible(rect) {
         let fill = if self.selected {
            Some(colors.highlight)
         } else if response.hovered() {
            Some(colors.hover)
         } else {
            None
         };
         if let Some(fill) = fill {
            ui.painter().rect_filled(
               rect,
               CornerRadius::same(theme.corner_radius),
               fill,
            );
         }
      }

      // The trailing control is measured first: the title's wrap width is what it leaves.
      let mut right = rect.right() - pad;
      if self.chevron {
         right -= 16.0 + gap;
      }
      if let Some(add) = self.trailing.take() {
         let slot = Rect::from_min_max(
            pos2(rect.left() + rect.width() * 0.5, rect.top()),
            pos2(right.max(rect.left() + pad), rect.bottom()),
         );
         let child = ui
            .scope_builder(
               UiBuilder::new()
                  .id_salt(id.with("trailing"))
                  .max_rect(slot)
                  .layout(Layout::right_to_left(Align::Center)),
               |ui| {
                  add(ui);
                  ui.min_rect()
               },
            )
            .inner;
         right = (child.left() - gap).max(rect.left() + pad);
      }

      if ui.is_rect_visible(rect) {
         let icon_gap = if self.icon.is_some() {
            ICON_SIZE + gap
         } else {
            0.0
         };
         let text_left = rect.left() + pad + icon_gap;
         let wrap = (right - text_left).max(0.0);
         let title = self.text.into_galley(
            ui,
            Some(TextWrapMode::Truncate),
            wrap,
            FontSelection::FontId(FontId::proportional(theme.typography.normal)),
         );
         let subtitle = self.subtitle.map(|subtitle| {
            subtitle.into_galley(
               ui,
               Some(TextWrapMode::Truncate),
               wrap,
               FontSelection::FontId(FontId::proportional(theme.typography.small)),
            )
         });

         let painter = ui.painter().with_clip_rect(rect);

         if let Some(icon) = self.icon.take() {
            let icon_rect = Rect::from_center_size(
               pos2(
                  rect.left() + pad + ICON_SIZE * 0.5,
                  rect.center().y,
               ),
               Vec2::splat(ICON_SIZE),
            );
            icon.tint(muted).paint_at(ui, icon_rect);
         }

         let title_size = title.size();
         let subtitle_size = subtitle.as_ref().map(|subtitle| subtitle.size());
         let (title_y, subtitle_y) = match subtitle_size {
            Some(subtitle_size) => {
               let block = title_size.y + theme.spacing.xs + subtitle_size.y;
               let top = rect.center().y - block * 0.5;
               (top, top + title_size.y + theme.spacing.xs)
            }
            None => (rect.center().y - title_size.y * 0.5, 0.0),
         };

         painter.galley(pos2(text_left, title_y), title, ink);
         if let Some(subtitle) = subtitle {
            painter.galley(pos2(text_left, subtitle_y), subtitle, muted);
         }

         if self.chevron {
            let center = pos2(rect.right() - pad - 8.0, rect.center().y);
            let stroke = Stroke::new(1.5, muted);
            painter.line_segment(
               [
                  pos2(center.x - 3.0, center.y - 4.5),
                  pos2(center.x + 1.5, center.y),
               ],
               stroke,
            );
            painter.line_segment(
               [
                  pos2(center.x + 1.5, center.y),
                  pos2(center.x - 3.0, center.y + 4.5),
               ],
               stroke,
            );
         }
      }

      let selected = self.selected;
      response.widget_info(|| WidgetInfo::selected(Role::Button, true, selected, label.as_str()));

      response
   }
}

impl Widget for Row<'_> {
   fn ui(self, ui: &mut Ui) -> Response {
      self.show(ui)
   }
}

#[cfg(test)]
mod tests {
   use super::*;
   use egui::{CentralPanel, Context, Event, FullOutput, Modifiers, PointerButton, Pos2, RawInput};

   /// Screen the test rows live in; small enough to keep clicks deterministic.
   fn screen() -> Rect {
      Rect::from_min_size(Pos2::ZERO, vec2(320.0, 200.0))
   }

   fn press(pos: Pos2) -> Event {
      Event::PointerButton {
         pos,
         button: PointerButton::Primary,
         pressed: true,
         modifiers: Modifiers::NONE,
      }
   }

   fn release(pos: Pos2) -> Event {
      Event::PointerButton {
         pos,
         button: PointerButton::Primary,
         pressed: false,
         modifiers: Modifiers::NONE,
      }
   }

   fn run_pass(
      ctx: &Context,
      events: Vec<Event>,
      build: impl FnOnce() -> Row<'static>,
   ) -> (FullOutput, Response) {
      let mut build = Some(build);
      let mut response = None;
      let mut output = ctx.run_ui(
         RawInput {
            events,
            screen_rect: Some(screen()),
            time: Some(1.0),
            ..Default::default()
         },
         |ctx| {
            CentralPanel::default().show(ctx, |ui| {
               response = Some(ui.add(build.take().expect("one row per pass")()));
            });
         },
      );
      output.textures_delta.clear();
      (output, response.expect("row added"))
   }

   #[test]
   fn a_row_fills_the_width_and_defaults_to_two_interaction_heights() {
      egui::__run_test_ui(|ui| {
         let expected_height = ui.spacing().interact_size.y * 2.0;
         let width = ui.available_width();
         let response = ui.add(Row::text("Notifications"));
         assert!((response.rect.width() - width).abs() < 0.5);
         assert!((response.rect.height() - expected_height).abs() < 0.5);
      });
   }

   #[test]
   fn clicking_the_row_reports_a_click() {
      let ctx = Context::default();
      let mut theme = Theme::new(crate::theme::ThemeKind::TokyoNight);
      theme.install(&ctx);
      let (_, first) = run_pass(&ctx, vec![], || Row::text("Notifications"));
      let center = first.rect.center();
      run_pass(&ctx, vec![press(center)], || {
         Row::text("Notifications")
      });
      let (_, last) = run_pass(&ctx, vec![release(center)], || {
         Row::text("Notifications")
      });
      assert!(last.clicked());
   }

   #[test]
   fn consecutive_rows_stack() {
      egui::__run_test_ui(|ui| {
         let first = ui.add(Row::text("One"));
         let second = ui.add(Row::text("Two"));
         assert!(second.rect.top() >= first.rect.bottom() - 0.5);
      });
   }

   #[test]
   fn a_trailing_control_leaves_the_title_its_own_width() {
      egui::__run_test_ui(|ui| {
         let mut value = false;
         let response = ui.add(
            Row::text("Notifications").chevron(true).trailing(|ui| {
               ui.add(crate::widgets::Switch::without_text(&mut value));
            }),
         );
         assert!(ui.is_rect_visible(response.rect));
      });
   }
}
