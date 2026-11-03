use super::{
   Avatar, AvatarContent, AvatarVisuals, DEFAULT_GAP, DEFAULT_OVERLAP, DEFAULT_SIZE,
   DISABLED_ALPHA, INITIALS_MAX_WIDTH, INITIALS_RATIO, RING_RATIO,
};
use crate::motion::{Motion, animate_with};
use crate::theme::Theme;
use egui::{
   Align, Align2, Color32, CornerRadius, FontId, Id, Rect, Response, Role, Sense, Stroke, Ui, Vec2,
   Widget, WidgetInfo, pos2,
};

/// A horizontal stack of [`Avatar`]s that fans out while the pointer is over it.
///
/// The avatars overlap by [`overlap`](Self::overlap) of their diameter while collapsed and spread to
/// [`gap`](Self::gap) apart on hover; only the spacing animates, on the installed style's
/// `animation_time` unless [`motion`](Self::motion) overrides it.
///
/// The widget always reserves the **expanded** width and spreads the stack around a fixed centre, so a
/// neighbour never reflows and nothing paints outside the allocated rect. Hovering the empty margin
/// beside a collapsed stack does not open it; the pointer must be over the avatars themselves.
///
/// ```
/// # use egui::__run_test_ui;
/// # use egui_elements::widgets::{Avatar, AvatarGroup};
/// # __run_test_ui(|ui| {
/// let chains = [Avatar::initials("ETH"), Avatar::initials("ARB"), Avatar::initials("OP")];
/// ui.add(AvatarGroup::new(&chains).size(24.0));
/// # });
/// ```
#[must_use = "Avatar groups should be shown on a ui, e.g. `ui.add(AvatarGroup::new(&items))`"]
pub struct AvatarGroup<'a> {
   items: &'a [Avatar],
   size: Option<f32>,
   overlap: Option<f32>,
   gap: Option<f32>,
   align: Align,
   expand: bool,
   visuals: Option<AvatarVisuals>,
   motion: Option<Motion>,
   label: Option<String>,
}

impl<'a> AvatarGroup<'a> {
   /// A stack of `items`, left to right, with the rightmost avatar painted on top.
   pub fn new(items: &'a [Avatar]) -> Self {
      Self {
         items,
         size: None,
         overlap: None,
         gap: None,
         align: Align::Center,
         expand: true,
         visuals: None,
         motion: None,
         label: None,
      }
   }

   /// Diameter of one avatar in points. Default [`DEFAULT_SIZE`]; every other measurement is a ratio of it.
   pub fn size(mut self, size: f32) -> Self {
      self.size = Some(size);
      self
   }

   /// Share of the diameter hidden behind the next avatar while collapsed, `0.0..=0.95`.
   ///
   /// Default [`DEFAULT_OVERLAP`] (`0.31`); the expanded state is always a gap, never an overlap.
   pub fn overlap(mut self, overlap: f32) -> Self {
      self.overlap = Some(overlap);
      self
   }

   /// Space between two neighbours while expanded, as a share of the diameter. Default [`DEFAULT_GAP`].
   pub fn gap(mut self, gap: f32) -> Self {
      self.gap = Some(gap);
      self
   }

   /// Where the stack sits in the width it reserves.
   ///
   /// [`Align::Center`] (the default) spreads the stack around a fixed centre, like the reference;
   /// [`Align::Min`] pins the leftmost avatar and lets the fan grow rightwards, which is what a row with
   /// content after the group wants.
   pub fn align(mut self, align: Align) -> Self {
      self.align = align;
      self
   }

   /// Fan the stack out while the pointer is over it. Default `true`; `false` keeps it stacked.
   pub fn expand(mut self, expand: bool) -> Self {
      self.expand = expand;
      self
   }

   /// Replace the palette-derived colours.
   pub fn visuals(mut self, visuals: AvatarVisuals) -> Self {
      self.visuals = Some(visuals);
      self
   }

   /// How the stack fans out. Defaults to [`Motion::from_ui`]; [`Motion::instant`] makes it snap.
   pub fn motion(mut self, motion: Motion) -> Self {
      self.motion = Some(motion);
      self
   }

   /// Accessibility label for the whole group. Defaults to `"{n} avatars"`.
   pub fn label(mut self, label: impl Into<String>) -> Self {
      self.label = Some(label.into());
      self
   }
}

/// Everything the paint needs besides the items and the clock.
struct Stack {
   size: f32,
   overlap: f32,
   gap: f32,
   align: Align,
   visuals: AvatarVisuals,
}

impl Widget for AvatarGroup<'_> {
   fn ui(self, ui: &mut Ui) -> Response {
      let AvatarGroup {
         items,
         size,
         overlap,
         gap,
         align,
         expand,
         visuals,
         motion,
         label,
      } = self;

      let enabled = ui.is_enabled();
      let size = size.unwrap_or(DEFAULT_SIZE).max(1.0);
      let overlap = overlap.unwrap_or(DEFAULT_OVERLAP).clamp(0.0, 0.95);
      let gap = gap.unwrap_or(DEFAULT_GAP).max(0.0);
      let count = items.len();
      if count == 0 {
         return ui.allocate_response(Vec2::ZERO, Sense::hover());
      }

      let motion = motion.unwrap_or_else(|| Motion::from_ui(ui));
      let reserved = stack_width(size, count, step_at(size, overlap, gap, 1.0));
      let (rect, response) = ui.allocate_exact_size(Vec2::new(reserved, size), Sense::hover());

      // Hover is tested against the stack as it was painted on the previous frame: the reserved margin
      // beside a collapsed stack must not open it, and the gaps between expanded avatars must not close it.
      let key = Id::unique(("avatar-group", response.id));
      let shown = ui.ctx().data_mut(|data| *data.get_temp_mut_or_insert_with(key, || 0.0));
      let band = stack_width(size, count, step_at(size, overlap, gap, shown));
      let left = stack_left(rect, band, align);
      let over_stack = ui
         .ctx()
         .pointer_latest_pos()
         .is_some_and(|pointer| (left..=left + band).contains(&pointer.x));
      let fan = expand && enabled && response.hovered() && over_stack;
      let t = animate_with(
         ui,
         response.id,
         if fan { 1.0 } else { 0.0 },
         motion,
      );
      ui.ctx().data_mut(|data| data.insert_temp(key, t));

      if ui.is_rect_visible(rect) {
         let stack = Stack {
            size,
            overlap,
            gap,
            align,
            visuals: visuals
               .unwrap_or_else(|| AvatarVisuals::from_theme(Theme::colors_from_ctx(ui.ctx()))),
         };
         if enabled {
            paint(ui, rect, items, t, &stack);
         } else {
            ui.scope(|ui| {
               ui.multiply_opacity(DISABLED_ALPHA);
               paint(ui, rect, items, t, &stack);
            });
         }
      }

      let label = label.unwrap_or_else(|| format!("{count} avatars"));
      response.widget_info(|| WidgetInfo::labeled(Role::Image, enabled, &label));
      response
   }
}

/// Centre-to-centre distance between two neighbours at fan-out progress `t`.
fn step_at(size: f32, overlap: f32, gap: f32, t: f32) -> f32 {
   let collapsed = size * (1.0 - overlap);
   let expanded = size * (1.0 + gap);
   collapsed + (expanded - collapsed) * t
}

/// Total width of `count` avatars laid out `step` apart.
fn stack_width(size: f32, count: usize, step: f32) -> f32 {
   size + count.saturating_sub(1) as f32 * step
}

/// Left edge of a `width`-wide stack inside `rect` for `align`.
fn stack_left(rect: Rect, width: f32, align: Align) -> f32 {
   rect.left()
      + match align {
         Align::Min => 0.0,
         Align::Center => (rect.width() - width) * 0.5,
         Align::Max => rect.width() - width,
      }
}

/// Paint the stack, back to front, so the rightmost avatar and its ring sit on top.
fn paint(ui: &Ui, rect: Rect, items: &[Avatar], t: f32, stack: &Stack) {
   let Stack {
      size,
      overlap,
      gap,
      align,
      visuals,
   } = *stack;
   let step = step_at(size, overlap, gap, t);
   let left = stack_left(rect, stack_width(size, items.len(), step), align);
   let radius = size / 2.0;
   let ring = Stroke::new(RING_RATIO * size, visuals.ring);

   for (index, avatar) in items.iter().enumerate() {
      let center = pos2(
         left + radius + index as f32 * step,
         rect.center().y,
      );
      match &avatar.content {
         AvatarContent::Image(image) => {
            let avatar_rect = Rect::from_center_size(center, Vec2::splat(size));
            image
               .clone()
               .corner_radius(CornerRadius::same(u8::MAX))
               .paint_at(ui, avatar_rect);
         }
         AvatarContent::Initials(text) => {
            ui.painter().circle_filled(
               center,
               radius,
               avatar.plate.unwrap_or(visuals.fallback_fill),
            );
            if !text.is_empty() {
               ui.painter().text(
                  center,
                  Align2::CENTER_CENTER,
                  text,
                  FontId::proportional(initials_size(ui, text, size)),
                  avatar.ink.unwrap_or(visuals.fallback_text),
               );
            }
         }
      }
      // Inside the edge, so the avatar's own diameter is the widget's: the ring eats into the picture
      // exactly as the reference's does, and nothing paints outside the allocated rect.
      ui.painter().circle_stroke(center, (radius - ring.width * 0.5).max(0.0), ring);
   }
}

/// Initials font size, shrunk until the text fits the plate.
fn initials_size(ui: &Ui, text: &str, size: f32) -> f32 {
   let mut font = INITIALS_RATIO * size;
   let measured = ui.fonts_mut(|fonts| {
      fonts
         .layout_no_wrap(
            text.to_owned(),
            FontId::proportional(font),
            Color32::WHITE,
         )
         .size()
         .x
   });
   let widest = INITIALS_MAX_WIDTH * size;
   if measured > widest && measured > 0.0 {
      font *= widest / measured;
   }
   font
}

#[cfg(test)]
mod tests {
   use super::*;
   use crate::theme::{Theme, ThemeKind};
   use egui::{Context, Event, FullOutput, Pos2, RawInput, epaint::CircleShape, pos2, vec2};

   /// A ring and plate far from any palette, so the paint can be found in the shape list.
   const RING: Color32 = Color32::from_rgb(9, 9, 9);
   const PLATE: Color32 = Color32::from_rgb(200, 40, 40);

   fn visuals() -> AvatarVisuals {
      AvatarVisuals::new(RING, PLATE, Color32::WHITE)
   }

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
         time: Some(time),
         events,
         screen_rect: Some(Rect::from_min_size(
            Pos2::ZERO,
            vec2(400.0, 300.0),
         )),
         ..Default::default()
      };
      let mut out = ctx.run_ui(input, |ui| body(ui));
      out.textures_delta.clear();
      out
   }

   fn plates(count: usize) -> Vec<Avatar> {
      (0..count).map(|index| Avatar::initials(index.to_string())).collect()
   }

   fn group(items: &[Avatar], size: f32, motion: Motion) -> AvatarGroup<'_> {
      AvatarGroup::new(items).size(size).motion(motion).visuals(visuals())
   }

   /// Every circle in the frame's paint, including the ones nested in shape vecs.
   fn circles(out: &FullOutput) -> Vec<CircleShape> {
      fn walk(shape: &egui::Shape, found: &mut Vec<CircleShape>) {
         match shape {
            egui::Shape::Vec(shapes) => shapes.iter().for_each(|shape| walk(shape, found)),
            egui::Shape::Circle(circle) => found.push(*circle),
            _ => {}
         }
      }
      let mut found = Vec::new();
      for clipped in &out.shapes {
         walk(&clipped.shape, &mut found);
      }
      found
   }

   /// The x centres of the fallback plates, left to right.
   fn plate_centers(out: &FullOutput) -> Vec<f32> {
      let mut centers: Vec<f32> = circles(out)
         .into_iter()
         .filter(|circle| circle.fill == PLATE)
         .map(|circle| circle.center.x)
         .collect();
      centers.sort_by(|a, b| a.partial_cmp(b).expect("finite centres"));
      centers
   }

   fn step_between(centers: &[f32]) -> f32 {
      centers[1] - centers[0]
   }

   #[test]
   fn the_group_reserves_the_expanded_width() {
      let ctx = themed_ctx();
      let items = plates(4);
      let size = 40.0;
      let mut rect = Rect::ZERO;
      frame(&ctx, 0.0, vec![], |ui| {
         rect = ui.add(group(&items, size, Motion::new(0.2))).rect;
      });
      let expected = size + 3.0 * size * (1.0 + DEFAULT_GAP);
      // egui rounds an allocated rect onto its pixel grid, so compare to within a fraction of a point.
      assert!(
         (rect.width() - expected).abs() < 0.05,
         "reserved width {:.3}, expected {expected:.3}",
         rect.width()
      );
      assert!(
         (rect.height() - size).abs() < 0.05,
         "height {rect:?}"
      );
   }

   #[test]
   fn hovering_fans_the_stack_out_and_settles_back() {
      let ctx = themed_ctx();
      let items = plates(4);
      let size = 40.0;
      let motion = Motion::new(0.2);
      let collapsed = size * (1.0 - DEFAULT_OVERLAP);
      let expanded = size * (1.0 + DEFAULT_GAP);

      let mut rect = Rect::ZERO;
      frame(&ctx, 10.0, vec![], |ui| {
         rect = ui.add(group(&items, size, motion)).rect;
      });
      let centers = plate_centers(&frame(&ctx, 10.0, vec![], |ui| {
         ui.add(group(&items, size, motion));
      }));
      assert_eq!(centers.len(), 4);
      assert!(
         (step_between(&centers) - collapsed).abs() < 0.01,
         "collapsed step: {centers:?}"
      );
      assert!(
         ((centers[0] + centers[3]) * 0.5 - rect.center().x).abs() < 0.01,
         "the collapsed stack is centred: {centers:?} in {rect:?}"
      );

      // The pointer lands: the retargeting frame still paints the collapsed stack…
      let pointer = rect.center();
      let centers = plate_centers(&frame(
         &ctx,
         10.1,
         vec![Event::PointerMoved(pointer)],
         |ui| {
            ui.add(group(&items, size, motion));
         },
      ));
      assert!(
         (step_between(&centers) - collapsed).abs() < 0.01,
         "retarget frame: {centers:?}"
      );

      // …half a leg later it is 87.5 % of the way out (cubic ease-out)…
      let centers = plate_centers(&frame(
         &ctx,
         10.2,
         vec![Event::PointerMoved(pointer)],
         |ui| {
            ui.add(group(&items, size, motion));
         },
      ));
      let half = collapsed + (expanded - collapsed) * 0.875;
      assert!(
         (step_between(&centers) - half).abs() < 0.05,
         "half a leg from {collapsed} to {expanded}: {centers:?}"
      );

      // …and settles on the expanded step around the same centre.
      let centers = plate_centers(&frame(
         &ctx,
         10.3,
         vec![Event::PointerMoved(pointer)],
         |ui| {
            ui.add(group(&items, size, motion));
         },
      ));
      assert!(
         (step_between(&centers) - expanded).abs() < 0.05,
         "expanded step: {centers:?}"
      );
      assert!(
         ((centers[0] + centers[3]) * 0.5 - rect.center().x).abs() < 0.05,
         "the centre does not move: {centers:?} in {rect:?}"
      );

      // Leaving collapses it again, from the value on screen. The pointer has to leave the reserved
      // box, not just the stack: while expanded, the whole reserved width is the hover target.
      let away = pos2(380.0, 280.0);
      let mut centers = Vec::new();
      for time in [10.4, 10.5, 10.6] {
         centers = plate_centers(&frame(
            &ctx,
            time,
            vec![Event::PointerMoved(away)],
            |ui| {
               ui.add(group(&items, size, motion));
            },
         ));
      }
      assert!(
         (step_between(&centers) - collapsed).abs() < 0.05,
         "settled back: {centers:?}"
      );
   }

   #[test]
   fn the_reserved_margin_does_not_fan_the_stack() {
      let ctx = themed_ctx();
      let items = plates(4);
      let size = 40.0;
      let motion = Motion::new(0.2);
      let mut rect = Rect::ZERO;
      frame(&ctx, 20.0, vec![], |ui| {
         rect = ui.add(group(&items, size, motion)).rect;
      });

      // Inside the reserved box, outside the collapsed stack.
      let margin = pos2(rect.right() - 2.0, rect.center().y);
      let mut centers = Vec::new();
      for time in [20.1, 20.2, 20.3] {
         centers = plate_centers(&frame(
            &ctx,
            time,
            vec![Event::PointerMoved(margin)],
            |ui| {
               ui.add(group(&items, size, motion));
            },
         ));
      }
      assert!(
         (step_between(&centers) - size * (1.0 - DEFAULT_OVERLAP)).abs() < 0.01,
         "the reserved margin opened the stack: {centers:?}"
      );
   }

   #[test]
   fn a_static_group_never_fans_out() {
      let ctx = themed_ctx();
      let items = plates(3);
      let size = 32.0;
      let motion = Motion::new(0.2);
      let mut rect = Rect::ZERO;
      frame(&ctx, 25.0, vec![], |ui| {
         rect = ui.add(group(&items, size, motion).expand(false)).rect;
      });
      let mut centers = Vec::new();
      for time in [25.1, 25.2, 25.3] {
         centers = plate_centers(&frame(
            &ctx,
            time,
            vec![Event::PointerMoved(rect.center())],
            |ui| {
               ui.add(group(&items, size, motion).expand(false));
            },
         ));
      }
      assert!(
         (step_between(&centers) - size * (1.0 - DEFAULT_OVERLAP)).abs() < 0.01,
         "expand(false) must keep the stack together: {centers:?}"
      );
   }

   #[test]
   fn a_settled_group_requests_no_repaint() {
      let ctx = themed_ctx();
      let items = plates(3);
      let motion = Motion::new(0.2);
      // Two warm-up frames absorb egui's own first-frame repaint request.
      for _ in 0..2 {
         frame(&ctx, 30.0, vec![], |ui| {
            ui.add(group(&items, 32.0, motion));
         });
      }
      let out = frame(&ctx, 30.0, vec![], |ui| {
         ui.add(group(&items, 32.0, motion));
      });
      let delay = out.viewport_output[&egui::ViewportId::ROOT].repaint_delay;
      assert_eq!(
         delay,
         std::time::Duration::MAX,
         "a settled group requested a repaint"
      );
   }

   #[test]
   fn an_override_paints_the_plate_and_the_ring() {
      let ctx = themed_ctx();
      let items = plates(2);
      let out = frame(&ctx, 40.0, vec![], |ui| {
         ui.add(group(&items, 32.0, Motion::new(0.2)));
      });
      let circles = circles(&out);
      let rings: Vec<&CircleShape> =
         circles.iter().filter(|circle| circle.stroke.color == RING).collect();
      let plates: Vec<&CircleShape> =
         circles.iter().filter(|circle| circle.fill == PLATE).collect();
      assert_eq!(rings.len(), 2, "one ring per avatar");
      assert_eq!(plates.len(), 2, "one plate per initials avatar");
      assert!(
         rings.iter().all(|ring| ring.radius < 16.0),
         "the ring is painted inside the plate's edge"
      );
   }

   #[test]
   fn extreme_ratios_are_clamped() {
      let ctx = themed_ctx();
      let items = plates(2);
      let out = frame(&ctx, 50.0, vec![], |ui| {
         ui.add(AvatarGroup::new(&items).size(40.0).overlap(4.0).gap(-3.0).visuals(visuals()));
      });
      let centers = plate_centers(&out);
      // Overlap clamps to 0.95, so two 40 pt avatars sit 2 pt apart; a negative gap clamps to zero.
      assert!(
         (step_between(&centers) - 2.0).abs() < 0.01,
         "clamped step: {centers:?}"
      );
   }
}
