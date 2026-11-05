//! A square, icon-only button.

use egui::{Image, Response, Role, Ui, Vec2, Widget, WidgetInfo, WidgetText};

use crate::theme::Theme;
use crate::visuals::ButtonVisuals;
use crate::widgets::Button;

/// A square button that shows one image and no label.
///
/// The chrome is [`Button`]'s: palette, hover and press fills, selected fill, border, shadow and
/// cursor all come from the installed theme, so an icon button sits beside a text button without a
/// second visual language. [`IconButton::tooltip`] supplies the label and the accessibility name.
///
/// An SVG image (egui-lucide, an icon file) needs `egui_extras::install_image_loaders` with the
/// `svg` feature installed on the context before the first frame.
#[must_use = "You should put this widget in a ui with `ui.add(widget);`"]
pub struct IconButton<'a> {
   image: Image<'a>,
   size: Option<f32>,
   icon_size: Option<f32>,
   tooltip: Option<WidgetText>,
   selected: bool,
   enabled: bool,
   visuals: Option<ButtonVisuals>,
}

impl<'a> IconButton<'a> {
   /// An icon-only button. Default side: the icon plus two `sm` spacing steps.
   pub fn new(image: impl Into<Image<'a>>) -> Self {
      Self {
         image: image.into(),
         size: None,
         icon_size: None,
         tooltip: None,
         selected: false,
         enabled: true,
         visuals: None,
      }
   }

   /// Square side in points, including the padding around the icon.
   pub fn size(mut self, size: f32) -> Self {
      self.size = Some(size);
      self
   }

   /// Icon side in points. Default: body text plus 4.
   pub fn icon_size(mut self, icon_size: f32) -> Self {
      self.icon_size = Some(icon_size);
      self
   }

   /// Hover text, also used as the accessibility label.
   pub fn tooltip(mut self, tooltip: impl Into<WidgetText>) -> Self {
      self.tooltip = Some(tooltip.into());
      self
   }

   /// Paint the selected fill. Default `false`.
   pub fn selected(mut self, selected: bool) -> Self {
      self.selected = selected;
      self
   }

   /// Whether the button accepts input. Default `true`.
   pub fn enabled(mut self, enabled: bool) -> Self {
      self.enabled = enabled;
      self
   }

   /// Override the chrome that would otherwise come from the installed theme.
   pub fn visuals(mut self, visuals: ButtonVisuals) -> Self {
      self.visuals = Some(visuals);
      self
   }
}

impl Widget for IconButton<'_> {
   fn ui(self, ui: &mut Ui) -> Response {
      let theme = Theme::current(ui.ctx());
      let icon_size = self.icon_size.unwrap_or(theme.typography.normal + 4.0);
      let side = self.size.unwrap_or(icon_size + 2.0 * theme.spacing.sm);

      let mut button = Button::image(self.image.fit_to_exact_size(Vec2::splat(icon_size)))
         .min_size(Vec2::splat(side))
         .selected(self.selected);

      if let Some(visuals) = self.visuals {
         button = button.visuals(visuals);
      }

      let mut response = ui.add_enabled(self.enabled, button);

      if let Some(tooltip) = self.tooltip {
         let label = tooltip.text().to_owned();
         response = response.on_hover_text(tooltip);
         response.widget_info(|| WidgetInfo::labeled(Role::Button, self.enabled, label.clone()));
      }

      response
   }
}

#[cfg(test)]
mod tests {
   use super::*;
   use crate::theme::{Theme, ThemeKind};
   use egui::{
      Color32, Context, Event, FullOutput, Modifiers, PointerButton, Pos2, RawInput, Rect, pos2,
      vec2,
   };

   fn themed_ctx() -> Context {
      let ctx = Context::default();
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&ctx);
      ctx
   }

   fn frame(
      ctx: &Context,
      time: f64,
      events: Vec<Event>,
      mut body: impl FnMut(&mut Ui),
   ) -> FullOutput {
      let input = RawInput {
         events,
         time: Some(time),
         screen_rect: Some(Rect::from_min_size(
            pos2(0.0, 0.0),
            vec2(400.0, 300.0),
         )),
         ..Default::default()
      };
      let mut out = ctx.run_ui(input, |ui| body(ui));
      out.textures_delta.clear();
      out
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

   fn texture(ctx: &Context) -> Image<'static> {
      let handle: &'static egui::TextureHandle = Box::leak(Box::new(ctx.load_texture(
         "icon",
         egui::ColorImage::filled([8, 8], Color32::WHITE),
         egui::TextureOptions::LINEAR,
      )));
      Image::from_texture(handle)
   }

   #[test]
   fn the_button_is_square() {
      let ctx = themed_ctx();
      let mut rect = Rect::ZERO;

      frame(&ctx, 0.0, vec![], |ui| {
         rect = ui.add(IconButton::new(texture(ui.ctx())).size(40.0)).rect;
      });

      assert_eq!(rect.width(), 40.0);
      assert_eq!(rect.height(), 40.0);
   }

   #[test]
   fn a_click_reports() {
      let ctx = themed_ctx();
      let mut rect = Rect::ZERO;

      frame(&ctx, 0.0, vec![], |ui| {
         rect = ui.add(IconButton::new(texture(ui.ctx())).tooltip("Settings")).rect;
      });

      let at = rect.center();
      frame(&ctx, 0.1, vec![press(at)], |ui| {
         ui.add(IconButton::new(texture(ui.ctx())));
      });

      let mut clicked = false;
      frame(&ctx, 0.2, vec![release(at)], |ui| {
         clicked = ui.add(IconButton::new(texture(ui.ctx()))).clicked();
      });

      assert!(clicked, "the release frame must report the click");
   }

   #[test]
   fn a_disabled_button_ignores_clicks() {
      let ctx = themed_ctx();
      let mut rect = Rect::ZERO;

      frame(&ctx, 0.0, vec![], |ui| {
         rect = ui.add(IconButton::new(texture(ui.ctx())).enabled(false)).rect;
      });

      frame(&ctx, 0.1, vec![press(rect.center())], |ui| {
         ui.add(IconButton::new(texture(ui.ctx())).enabled(false));
      });

      let mut clicked = false;
      frame(&ctx, 0.2, vec![release(rect.center())], |ui| {
         clicked = ui.add(IconButton::new(texture(ui.ctx())).enabled(false)).clicked();
      });

      assert!(
         !clicked,
         "a disabled icon button must not report clicks"
      );
   }
}
