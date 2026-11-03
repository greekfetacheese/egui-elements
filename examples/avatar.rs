//! Avatar-group proof: the reference stack, the live knobs, a size ladder and a multi-chain balance row.
//!
//! Run with: `cargo run --example avatar`

use eframe::egui::*;
use egui_elements::motion::Motion;
use egui_elements::widgets::{Avatar, AvatarGroup, AvatarVisuals, Frame as Card};
use egui_elements::{Theme, ThemeKind, utils};

fn main() -> eframe::Result {
   let options = eframe::NativeOptions {
      viewport: ViewportBuilder::default()
         .with_title("egui-elements avatar group")
         .with_inner_size([880.0, 720.0])
         .with_resizable(true),
      ..Default::default()
   };

   eframe::run_native(
      "egui-elements avatar group",
      options,
      Box::new(|cc| Ok(Box::new(AvatarApp::new(cc)))),
   )
}

struct AvatarApp {
   theme: Theme,
   size: f32,
   overlap: f32,
   gap: f32,
   duration: f32,
   reduce_motion: bool,
   photos: Vec<Avatar>,
   chains: Vec<Avatar>,
   /// Kept alive so egui does not free the sample pictures.
   _textures: Vec<TextureHandle>,
}

impl AvatarApp {
   fn new(cc: &eframe::CreationContext<'_>) -> Self {
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&cc.egui_ctx);
      // The widget reads the style's clock; the slider opens on the value it really uses.
      let duration = theme.style().animation_time;

      let mut textures = Vec::new();
      let mut photos = Vec::new();
      let palettes = [
         (
            Color32::from_rgb(248, 226, 186),
            Color32::from_rgb(58, 92, 146),
         ),
         (
            Color32::from_rgb(120, 148, 230),
            Color32::from_rgb(238, 240, 250),
         ),
         (
            Color32::from_rgb(34, 96, 118),
            Color32::from_rgb(198, 230, 224),
         ),
      ];
      for (index, (from, to)) in palettes.into_iter().enumerate() {
         let (image, texture) = sample_photo(&cc.egui_ctx, index, from, to);
         photos.push(Avatar::image(image));
         textures.push(texture);
      }
      photos.push(Avatar::initials("M"));

      // One letter each: initials are the fallback for a missing icon, and a three-letter ticker is
      // unreadable once the next avatar's ring overlaps it at 24–28 pt.
      let chains = [
         ("E", Color32::from_rgb(88, 104, 168)),
         ("A", Color32::from_rgb(56, 124, 188)),
         ("O", Color32::from_rgb(214, 72, 84)),
         ("P", Color32::from_rgb(128, 84, 196)),
      ]
      .into_iter()
      .map(|(ticker, plate)| Avatar::initials(ticker).plate(plate).ink(Color32::WHITE))
      .collect();

      Self {
         theme,
         size: 32.0,
         overlap: 0.31,
         gap: 0.11,
         duration,
         reduce_motion: false,
         photos,
         chains,
         _textures: textures,
      }
   }

   fn motion(&self) -> Motion {
      if self.reduce_motion {
         Motion::instant()
      } else {
         Motion::new(self.duration)
      }
   }

   /// The reference's card: title, description and the stack on the card's own surface.
   fn reference_card(&mut self, ui: &mut Ui) {
      let space = self.theme.spacing;
      let colors = self.theme.colors;
      let visuals = AvatarVisuals {
         ring: colors.widget_bg,
         ..AvatarVisuals::from_theme(colors)
      };

      Card::new()
         .fill(colors.widget_bg)
         .corner_radius(CornerRadius::same(12))
         .inner_margin(space.margin(space.xl))
         .show(ui, |ui| {
            ui.vertical_centered(|ui| {
               ui.add_space(space.lg);
               ui.add(
                  AvatarGroup::new(&self.photos).size(40.0).visuals(visuals).motion(self.motion()),
               );
               ui.add_space(space.xl);
            });
            ui.spacing_mut().item_spacing.y = space.xs;
            ui.label(
               RichText::new("Avatar")
                  .size(self.theme.typography.large)
                  .color(colors.text)
                  .strong(),
            );
            ui.label(
               RichText::new("User photo with initials fallback.")
                  .size(self.theme.typography.normal)
                  .color(colors.text_muted),
            );
         });
   }

   /// The live knobs: every measurement is a ratio of `size`, and the duration is the widget's clock.
   fn knobs_card(&mut self, ui: &mut Ui) {
      let space = self.theme.spacing;
      let colors = self.theme.colors;
      let normal = self.theme.typography.normal;
      let visuals = AvatarVisuals {
         ring: colors.widget_bg,
         ..AvatarVisuals::from_theme(colors)
      };
      let items = self.photos.len() as f32;
      let collapsed = self.size * (1.0 - self.overlap);
      let expanded = self.size * (1.0 + self.gap);
      let reserved = self.size + (items - 1.0) * expanded;

      Card::new()
         .fill(colors.widget_bg)
         .corner_radius(CornerRadius::same(12))
         .inner_margin(space.margin(space.lg))
         .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = space.sm;
            ui.spacing_mut().slider_width = 260.0;

            ui.add(Slider::new(&mut self.size, 16.0..=56.0).text("size (diameter)"));
            ui.add(Slider::new(&mut self.overlap, 0.0..=0.7).text("overlap (share of d)"));
            ui.add(Slider::new(&mut self.gap, 0.0..=0.4).text("gap when expanded"));
            ui.add(Slider::new(&mut self.duration, 0.05..=1.0).text("duration (s)"));
            ui.checkbox(
               &mut self.reduce_motion,
               RichText::new("reduce motion").size(normal).color(colors.text),
            );

            ui.add_space(space.sm);
            ui.add(
               AvatarGroup::new(&self.photos)
                  .size(self.size)
                  .overlap(self.overlap)
                  .gap(self.gap)
                  .visuals(visuals)
                  .motion(self.motion()),
            );
            ui.add_space(space.xs);
            ui.label(
               RichText::new(format!(
                  "step {collapsed:.1} pt collapsed → {expanded:.1} pt expanded · \
                   {reserved:.1} pt reserved, so nothing reflows"
               ))
               .size(normal)
               .color(colors.text_muted),
            );
         });
   }

   /// Zeus' row: one asset, every chain that holds a balance, the icons fanned out on hover.
   fn balance_card(&mut self, ui: &mut Ui) {
      let space = self.theme.spacing;
      let colors = self.theme.colors;
      let visuals = AvatarVisuals {
         ring: colors.widget_bg,
         ..AvatarVisuals::from_theme(colors)
      };

      Card::new()
         .fill(colors.widget_bg)
         .corner_radius(CornerRadius::same(12))
         .inner_margin(space.margin(space.lg))
         .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = space.xs;
            ui.horizontal(|ui| {
               ui.label(
                  RichText::new("2.4103 ETH")
                     .size(self.theme.typography.large)
                     .color(colors.text)
                     .strong(),
               );
               ui.add(
                  AvatarGroup::new(&self.chains)
                     .size(28.0)
                     .align(Align::Min)
                     .visuals(visuals)
                     .motion(self.motion()),
               );
            });
            ui.label(
               RichText::new("Balances on four chains — hover the icons to see them all.")
                  .size(self.theme.typography.normal)
                  .color(colors.text_muted),
            );
         });
   }

   /// One size, four ratios: the ladder is the proof that nothing is a point value.
   fn ladder_card(&mut self, ui: &mut Ui) {
      let space = self.theme.spacing;
      let colors = self.theme.colors;

      Card::new()
         .fill(colors.widget_bg)
         .corner_radius(CornerRadius::same(12))
         .inner_margin(space.margin(space.lg))
         .show(ui, |ui| {
            ui.label(
               RichText::new("Size ladder")
                  .size(self.theme.typography.normal)
                  .color(colors.text)
                  .strong(),
            );
            ui.horizontal_wrapped(|ui| {
               ui.spacing_mut().item_spacing.x = space.lg;
               for size in [20.0, 28.0, 36.0, 44.0] {
                  ui.add(
                     AvatarGroup::new(&self.chains)
                        .size(size)
                        .visuals(AvatarVisuals {
                           ring: colors.widget_bg,
                           ..AvatarVisuals::from_theme(colors)
                        })
                        .motion(self.motion()),
                  );
               }
            });
         });
   }

   fn header(&mut self, ui: &mut Ui) {
      ui.horizontal(|ui| {
         ui.spacing_mut().button_padding = self.theme.button_padding;

         ui.label(
            RichText::new("egui-elements avatar group")
               .size(self.theme.typography.very_large)
               .color(self.theme.colors.accent)
               .strong(),
         );

         ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if let Some(new_theme) = utils::theme_switcher(&self.theme, ui) {
               self.theme = new_theme;
            }
         });
      });
   }
}

impl eframe::App for AvatarApp {
   fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
      let space = self.theme.spacing;
      let bg = self.theme.colors.bg;

      Panel::top("header")
         .frame(
            Frame::new()
               .fill(bg)
               .inner_margin(space.margin_xy(space.lg, space.md))
               .stroke(Stroke::new(1.0, self.theme.colors.border)),
         )
         .show(ui, |ui| self.header(ui));

      CentralPanel::default()
         .frame(Frame::new().fill(bg).inner_margin(space.margin(space.lg)))
         .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = space.md;
            ScrollArea::vertical().show(ui, |ui| {
               ui.spacing_mut().item_spacing.y = space.lg;
               self.reference_card(ui);
               self.knobs_card(ui);
               self.balance_card(ui);
               self.ladder_card(ui);
            });
         });
   }
}

/// A stand-in for a profile photo: a diagonal two-colour ramp with a bright disc and a thin stripe,
/// so the circular crop and the ring are unambiguous in a live capture.
fn sample_photo(
   ctx: &Context,
   index: usize,
   from: Color32,
   to: Color32,
) -> (Image<'static>, TextureHandle) {
   let side = 96_usize;
   let sidef = side as f32 - 1.0;
   let mut pixels = Vec::with_capacity(side * side);
   for y in 0..side {
      for x in 0..side {
         let ramp = (x + y) as f32 / (2.0 * sidef);
         let mut color = from.lerp_to_gamma(to, ramp);

         let dx = x as f32 - side as f32 * 0.63;
         let dy = y as f32 - side as f32 * 0.40;
         if dx * dx + dy * dy < (side as f32 * 0.15).powi(2) {
            color = color.gamma_multiply(1.7);
         }
         if (x as i32 - y as i32).rem_euclid(23) < 3 {
            color = color.gamma_multiply(0.72);
         }

         pixels.push(color);
      }
   }

   let texture = ctx.load_texture(
      format!("avatar-photo-{index}"),
      ColorImage::new([side, side], pixels),
      TextureOptions::LINEAR,
   );
   let image = Image::from_texture((texture.id(), vec2(side as f32, side as f32)));
   (image, texture)
}
