use egui::{
   Atom, AtomExt as _, AtomKind, AtomLayout, AtomLayoutResponse, Color32, Frame, Id, Image,
   IntoAtoms, NumExt as _, Pos2, Rect, Response, Sense, Shadow, Shape, Stroke, TextStyle,
   TextWrapMode, Ui, Vec2, Widget, WidgetInfo, WidgetText, WidgetType, response::Flags,
};

use crate::glow::{glow_mesh, size_attenuation};
use crate::theme::Theme;
use crate::visuals::ButtonVisuals;
use crate::widgets::badge::Badge;
use crate::widgets::particles::Lightning;

/// Longest `stable_dt` the hold accumulation accepts, in seconds (spike guard).
const HOLD_MAX_DT: f32 = 0.1;
/// Width of the sweep's leading-edge line, in pixels.
const HOLD_EDGE_WIDTH: f32 = 1.5;
/// Duration of a completion burst, in milliseconds.
const HOLD_BURST_MS: f32 = 300.0;
/// Duration of a completion burst, in seconds.
const HOLD_BURST_SECS: f32 = HOLD_BURST_MS / 1000.0;
/// Number of bolts in a [`HoldBurst::Lightning`] completion burst.
const HOLD_BURST_BOLTS: usize = 1;
/// Branch depth of a [`HoldBurst::Lightning`] completion burst.
const HOLD_BURST_BRANCH_DEPTH: u32 = 1;
/// Peak flash alpha of a [`HoldBurst::Lightning`] completion burst.
const HOLD_BURST_FLASH: f32 = 0.22;
/// [`HoldBurst::Glow`] radius at the start of the burst, as a multiple of the
/// button's shorter side.
const HOLD_GLOW_START: f32 = 1.0;
/// [`HoldBurst::Glow`] radius at the end of the burst, as a multiple of the
/// button's shorter side.
const HOLD_GLOW_END: f32 = 2.2;

/// Completion burst drawn when a hold-to-confirm button reaches 100 %.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HoldBurst {
   /// An expanding radial glow at the button's centre, fading out.
   #[default]
   Glow,
   /// A single lightning bolt and flash struck across the button.
   Lightning,
}

/// Per-button hold-to-confirm state, keyed by the button's response [`Id`].
#[derive(Clone, Copy, Default)]
struct HoldState {
   /// Fraction of the hold completed, `0.0..=1.0`.
   progress: f32,
   /// Whether the current hold has already fired (so it fires once).
   fired: bool,
   /// Seconds since the burst started, while one is animating.
   burst_age: Option<f32>,
}

/// A [`HoldBurst::Lightning`] strike, addressed by a derived id so it never
/// collides with a host's own [`Lightning`].
fn burst_lightning(id: Id, area: Rect) -> Lightning {
   Lightning::new(("hold_burst", id))
      .bolt_count(HOLD_BURST_BOLTS)
      .branch_depth(HOLD_BURST_BRANCH_DEPTH)
      .duration_ms(HOLD_BURST_MS)
      .flash_intensity(HOLD_BURST_FLASH)
      .area(area)
}

/// Paints one frame of a completion burst on `ui`'s painter.
fn paint_burst(
   ui: &Ui,
   burst: HoldBurst,
   id: Id,
   rect: Rect,
   expansion: f32,
   age: f32,
   color: Color32,
) {
   let t = (age / HOLD_BURST_SECS).clamp(0.0, 1.0);
   match burst {
      HoldBurst::Glow => {
         // Grow and fade the shared glow fan; `size_attenuation` keeps the peak
         // alpha in family with the Light effect instead of a second falloff.
         let short = rect.width().min(rect.height()).max(1.0);
         let radius = short * (HOLD_GLOW_START + (HOLD_GLOW_END - HOLD_GLOW_START) * t);
         let attenuation = size_attenuation(radius, short, short * HOLD_GLOW_END);
         let tint = color.gamma_multiply((1.0 - t) * attenuation);
         ui.painter().add(Shape::mesh(glow_mesh(
            rect.center(),
            radius,
            tint,
         )));
      }
      HoldBurst::Lightning => {
         burst_lightning(id, rect.expand(expansion)).show(ui.ctx());
      }
   }
}

/// Clickable themed button, built on egui [`Atom`]s.
///
/// Reads [`ButtonVisuals`] from the installed [`Theme`] unless you pass
/// [`.visuals(...)`](Self::visuals). Use [`Button::bg_color`] for a one-off
/// idle fill (semantic actions). Chromatic fills should use `theme.colors.bg`
/// as the text color so glyphs stay readable.
///
/// ```
/// # use egui::__run_test_ui;
/// # use egui_elements::widgets::Button;
/// # __run_test_ui(|ui| {
/// if ui.add(Button::new("Save")).clicked() {
///     // …
/// }
/// # });
/// ```
#[must_use = "You should put this widget in a ui with `ui.add(widget);`"]
pub struct Button<'a> {
   layout: AtomLayout<'a>,
   visuals: Option<ButtonVisuals>,
   bg_color: Option<Color32>,
   badge: Option<Badge>,
   small: bool,
   frame_when_inactive: bool,
   min_size: Vec2,
   selected: bool,
   image_tint_follows_text_color: bool,
   limit_image_size: bool,
   hold_seconds: Option<f32>,
   hold_fill: Option<Color32>,
   hold_burst: HoldBurst,
}

impl<'a> Button<'a> {
   /// Create a button from atoms (text, images, or a mix).
   pub fn new(atoms: impl IntoAtoms<'a>) -> Self {
      Self {
         layout: AtomLayout::new(atoms.into_atoms())
            .sense(Sense::click())
            .fallback_font(TextStyle::Button),
         visuals: None,
         bg_color: None,
         badge: None,
         small: false,
         frame_when_inactive: true,
         min_size: Vec2::ZERO,
         selected: false,
         image_tint_follows_text_color: false,
         limit_image_size: false,
         hold_seconds: None,
         hold_fill: None,
         hold_burst: HoldBurst::default(),
      }
   }

   /// Show a selectable button.
   ///
   /// Equivalent to:
   /// ```rust
   /// # use egui::{Button, IntoAtoms, __run_test_ui};
   /// # __run_test_ui(|ui| {
   /// let selected = true;
   /// ui.add(Button::new("toggle me").selected(selected).frame_when_inactive(!selected).frame(true));
   /// # });
   /// ```
   ///
   /// See also:
   ///   - [`Ui::selectable_value`]
   ///   - [`Ui::selectable_label`]
   pub fn selectable(selected: bool, atoms: impl IntoAtoms<'a>) -> Self {
      Self::new(atoms).selected(selected).frame_when_inactive(selected)
   }

   /// Creates a button with an image. The size of the image as displayed is defined by the provided size.
   ///
   /// Note: In contrast to [`Button::new`], this limits the image size to the default font height
   /// (using [`egui::AtomExt::atom_max_height_font_size`]).
   pub fn image(image: impl Into<Image<'a>>) -> Self {
      Self::opt_image_and_text(Some(image.into()), None)
   }

   /// Creates a button with an image to the left of the text.
   ///
   /// Note: In contrast to [`Button::new`], this limits the image size to the default font height
   /// (using [`egui::AtomExt::atom_max_height_font_size`]).
   pub fn image_and_text(image: impl Into<Image<'a>>, text: impl Into<WidgetText>) -> Self {
      Self::opt_image_and_text(Some(image.into()), Some(text.into()))
   }

   /// Create a button with an optional image and optional text.
   ///
   /// Note: In contrast to [`Button::new`], this limits the image size to the default font height
   /// (using [`egui::AtomExt::atom_max_height_font_size`]).
   pub fn opt_image_and_text(image: Option<Image<'a>>, text: Option<WidgetText>) -> Self {
      let mut button = Self::new(());
      if let Some(image) = image {
         button.layout.push_right(image);
      }
      if let Some(text) = text {
         button.layout.push_right(text);
      }
      button.limit_image_size = true;
      button
   }

   /// Set the wrap mode for the text.
   ///
   /// By default, [`egui::Ui::wrap_mode`] will be used, which can be overridden with [`egui::Style::wrap_mode`].
   ///
   /// Note that any `\n` in the text will always produce a new line.
   #[inline]
   pub fn wrap_mode(mut self, wrap_mode: TextWrapMode) -> Self {
      self.layout = self.layout.wrap_mode(wrap_mode);
      self
   }

   /// Set [`Self::wrap_mode`] to [`TextWrapMode::Wrap`].
   #[inline]
   pub fn wrap(self) -> Self {
      self.wrap_mode(TextWrapMode::Wrap)
   }

   /// Set [`Self::wrap_mode`] to [`TextWrapMode::Truncate`].
   #[inline]
   pub fn truncate(self) -> Self {
      self.wrap_mode(TextWrapMode::Truncate)
   }

   /// Make this a small button, suitable for embedding into text.
   #[inline]
   pub fn small(mut self) -> Self {
      self.small = true;
      self
   }

   /// If `false`, the button will not have a frame when inactive.
   ///
   /// Default: `true`.
   ///
   /// Note: When `ui.visuals().button_frame` is `false`, this setting
   /// has no effect.
   #[inline]
   pub fn frame_when_inactive(mut self, frame_when_inactive: bool) -> Self {
      self.frame_when_inactive = frame_when_inactive;
      self
   }

   /// By default, buttons senses clicks.
   /// Change this to a drag-button with `Sense::drag()`.
   #[inline]
   pub fn sense(mut self, sense: Sense) -> Self {
      self.layout = self.layout.sense(sense);
      self
   }

   /// Set the minimum size of the button.
   #[inline]
   pub fn min_size(mut self, min_size: Vec2) -> Self {
      self.min_size = min_size;
      self
   }

   /// If true, the tint of the image is multiplied by the widget text color.
   ///
   /// This makes sense for images that are white, that should have the same color as the text color.
   /// This will also make the icon color depend on hover state.
   ///
   /// Default: `false`.
   #[inline]
   pub fn image_tint_follows_text_color(mut self, image_tint_follows_text_color: bool) -> Self {
      self.image_tint_follows_text_color = image_tint_follows_text_color;
      self
   }

   /// Show some text on the right side of the button, in weak color.
   ///
   /// Designed for menu buttons, for setting a keyboard shortcut text (e.g. `Ctrl+S`).
   ///
   /// The text can be created with [`egui::Context::format_shortcut`].
   ///
   /// See also [`Self::right_text`].
   #[inline]
   pub fn shortcut_text(mut self, shortcut_text: impl Into<Atom<'a>>) -> Self {
      let mut atom = shortcut_text.into();
      atom.kind = match atom.kind {
         AtomKind::Text(text) => AtomKind::Text(text.weak()),
         other => other,
      };
      self.layout.push_right(Atom::grow());
      self.layout.push_right(atom);
      self
   }

   /// Show some text on the right side of the button.
   #[inline]
   pub fn right_text(mut self, right_text: impl Into<Atom<'a>>) -> Self {
      self.layout.push_right(Atom::grow());
      self.layout.push_right(right_text.into());
      self
   }

   /// If `true`, mark this button as "selected".
   #[inline]
   pub fn selected(mut self, selected: bool) -> Self {
      self.selected = selected;
      self
   }

   /// Set the visuals of the button
   #[inline]
   pub fn visuals(mut self, visuals: ButtonVisuals) -> Self {
      self.visuals = Some(visuals);
      self
   }

   /// Set the background color of the button
   #[inline]
   pub fn bg_color(mut self, color: Color32) -> Self {
      self.bg_color = Some(color);
      self
   }

   /// Attach a notification [`Badge`] to a corner of this button.
   ///
   /// The badge is painted on top of the button and does not change its size or
   /// layout. Use it to signal unseen content behind the button — e.g. a red
   /// circle with the number of notifications.
   ///
   /// ```
   /// # use egui::__run_test_ui;
   /// # use egui::Color32;
   /// # use egui_elements::widgets::{Badge, BadgeCorner, Button};
   /// # __run_test_ui(|ui| {
   /// ui.add(
   ///    Button::new("Notifications").badge(
   ///       Badge::new("3")
   ///          .color(Color32::RED)
   ///          .corner(BadgeCorner::TopRight),
   ///    ),
   /// );
   /// # });
   /// ```
   #[inline]
   pub fn badge(mut self, badge: Badge) -> Self {
      self.badge = Some(badge);
      self
   }

   /// Arm the button as hold-to-confirm.
   ///
   /// [`clicked`](Response::clicked) then reports `true` only once the pointer
   /// has been held down on the button for `seconds` — never for a plain click.
   /// A non-positive `seconds` leaves the button as an ordinary click button.
   ///
   /// While the hold progresses the frame's fill is replaced by a left-to-right
   /// sweep that grows as the hold completes; when it fills, the button fires
   /// once and plays its [`HoldBurst`]. Pressing Enter or Space on a focused
   /// button confirms immediately, like egui's keyboard activation for buttons.
   pub fn hold(mut self, seconds: f32) -> Self {
      self.hold_seconds = Some(seconds);
      self
   }

   /// Colour of the hold sweep, and the default tint of a
   /// [`HoldBurst::Glow`] burst.
   ///
   /// Defaults to the installed theme's
   /// [`colors.accent`](crate::theme::ThemeColors::accent).
   pub fn hold_fill(mut self, color: Color32) -> Self {
      self.hold_fill = Some(color);
      self
   }

   /// Completion burst played when the hold fires.
   ///
   /// Default: [`HoldBurst::Glow`].
   pub fn hold_burst(mut self, burst: HoldBurst) -> Self {
      self.hold_burst = burst;
      self
   }

   /// Show the button and return a [`AtomLayoutResponse`] for painting custom contents.
   pub fn atom_ui(self, ui: &mut Ui) -> AtomLayoutResponse {
      let Button {
         mut layout,
         bg_color,
         badge,
         small,
         visuals,
         frame_when_inactive,
         mut min_size,
         selected,
         image_tint_follows_text_color,
         limit_image_size,
         hold_seconds,
         hold_fill,
         hold_burst,
      } = self;

      let hold_seconds = hold_seconds.filter(|seconds| *seconds > 0.0);

      let visuals = visuals.or_else(|| Theme::button_visuals_from_ctx(ui.ctx()));

      if !small {
         min_size.y = min_size.y.at_least(ui.spacing().interact_size.y);
      }
      if limit_image_size {
         layout.map_atoms(|atom| {
            if matches!(&atom.kind, AtomKind::Image(_)) {
               atom.atom_max_height_font_size(ui)
            } else {
               atom
            }
         });
      }

      let text = layout.text().map(String::from);

      let has_frame_margin = visuals.is_some() || ui.visuals().button_frame;
      let mut button_padding = if has_frame_margin {
         ui.spacing().button_padding
      } else {
         Vec2::ZERO
      };

      if small {
         button_padding.y = 0.0;
      }

      let frame = Frame::new().inner_margin(button_padding);
      let mut prepared = layout.frame(frame).min_size(min_size).allocate(ui);

      let response = if ui.is_rect_visible(prepared.response.rect) {
         let interact_visuals = ui.style().interact_selectable(&prepared.response, selected);

         let is_active = prepared.response.is_pointer_button_down_on(); // Clicked
         let is_hovered = prepared.response.hovered();

         // Select custom visuals based on state, or fall back to defaults
         let (fill, stroke, corner_radius) = if let Some(v) = &visuals {
            if is_active {
               (v.bg_click, v.border_click, v.corner_radius)
            } else if is_hovered {
               (v.bg_hover, v.border_hover, v.corner_radius)
            } else {
               let bg = bg_color.unwrap_or(v.bg);
               (bg, v.border, v.corner_radius)
            }
         } else {
            (
               interact_visuals.weak_bg_fill,
               interact_visuals.bg_stroke,
               interact_visuals.corner_radius,
            )
         };

         let fill = match selected {
            false => fill,
            true => {
               visuals.as_ref().map(|v| v.bg_selected).unwrap_or(interact_visuals.weak_bg_fill)
            }
         };

         let text_color = visuals.as_ref().map(|v| v.text).unwrap_or(interact_visuals.text_color());

         let visible_frame = if frame_when_inactive {
            has_frame_margin
         } else {
            has_frame_margin && (is_hovered || is_active || prepared.response.has_focus())
         };

         if image_tint_follows_text_color {
            prepared.map_images(|image| image.tint(text_color));
         }
         prepared.fallback_text_color = text_color;

         if visible_frame {
            let shadow = visuals.as_ref().map(|v| v.shadow).unwrap_or(Shadow::NONE);
            prepared.frame = prepared
               .frame
               .inner_margin(
                  button_padding + Vec2::splat(interact_visuals.expansion)
                     - Vec2::splat(stroke.width),
               )
               .outer_margin(-Vec2::splat(interact_visuals.expansion))
               .fill(fill)
               .stroke(stroke)
               .corner_radius(corner_radius)
               .shadow(shadow);
         }

         // ---- hold-to-confirm gesture ------------------------------------
         let mut hold = HoldState::default();
         let mut fired_now = false;
         if let Some(seconds) = hold_seconds {
            let id = prepared.response.id;
            let dt = ui.ctx().input(|i| i.stable_dt).min(HOLD_MAX_DT);
            hold = ui.ctx().data_mut(|data| {
               let state = data.get_temp_mut_or_insert_with(id, HoldState::default);
               if prepared.response.is_pointer_button_down_on() {
                  state.progress = (state.progress + dt / seconds).clamp(0.0, 1.0);
               } else {
                  // Release and drag-off both land here.
                  state.progress = 0.0;
                  state.fired = false;
               }
               if state.progress >= 1.0 && !state.fired {
                  state.fired = true;
                  state.burst_age = Some(0.0);
                  fired_now = true;
               }
               let snapshot = *state;
               // Advance the burst clock for the next frame; the snapshot above
               // is this frame's age, so the burst paints before it advances.
               if let Some(age) = state.burst_age.as_mut() {
                  *age += dt;
                  if *age >= HOLD_BURST_SECS {
                     state.burst_age = None;
                  }
               }
               snapshot
            });

            // A press+release must never report a plain click: only a completed
            // hold confirms.
            prepared.response.flags.remove(Flags::CLICKED);

            // A completed hold reports `clicked()` for exactly this frame.
            if fired_now {
               prepared.response.flags.insert(Flags::FAKE_PRIMARY_CLICKED);
            }

            // Queue the lightning once, at completion.
            if fired_now && matches!(hold_burst, HoldBurst::Lightning) {
               burst_lightning(
                  id,
                  prepared.response.rect.expand(interact_visuals.expansion),
               )
               .strike(ui.ctx());
            }
         }

         // ---- sweep + burst painting -------------------------------------
         let sweep_active = hold.progress > 0.0;
         let burst_active = hold.burst_age.is_some();
         let hold_color = (sweep_active || burst_active)
            .then(|| hold_fill.unwrap_or_else(|| Theme::current(ui.ctx()).colors.accent));

         if let Some(hold_color) = hold_color {
            if visible_frame {
               // Give up the frame's own fill so the base fill, sweep and edge
               // can be painted under it; stroke and shadow still land on top.
               prepared.frame = prepared.frame.fill(Color32::TRANSPARENT);

               let rect = prepared.response.rect;
               let expanded = rect.expand(interact_visuals.expansion);
               ui.painter().add(Shape::rect_filled(expanded, corner_radius, fill));

               if sweep_active {
                  // Clip the same full rounded rect to the swept span rather than
                  // build a partial one, so the button keeps its shape and only
                  // the leading edge is straight. Intersect the painter's own
                  // clip so a scroll area's clip is never widened.
                  let sweep_clip = Rect::from_min_max(
                     Pos2::new(rect.left(), expanded.top()),
                     Pos2::new(
                        rect.left() + hold.progress * rect.width(),
                        expanded.bottom(),
                     ),
                  );
                  let clip = ui.painter().clip_rect().intersect(sweep_clip);
                  ui.painter().with_clip_rect(clip).add(Shape::rect_filled(
                     expanded,
                     corner_radius,
                     hold_color,
                  ));

                  let edge_x = rect.left() + hold.progress * rect.width();
                  let edge_clip = ui.painter().clip_rect().intersect(expanded);
                  ui.painter().with_clip_rect(edge_clip).add(Shape::line_segment(
                     [
                        Pos2::new(edge_x, expanded.top()),
                        Pos2::new(edge_x, expanded.bottom()),
                     ],
                     Stroke::new(HOLD_EDGE_WIDTH, hold_color),
                  ));
               }
            }

            if let Some(age) = hold.burst_age {
               paint_burst(
                  ui,
                  hold_burst,
                  prepared.response.id,
                  prepared.response.rect,
                  interact_visuals.expansion,
                  age,
                  hold_color,
               );
            }
         }

         // Repaint only while the hold progresses or a burst animates; a full,
         // held sweep is static, so it does not pin the frame loop.
         if (sweep_active && hold.progress < 1.0) || burst_active {
            ui.ctx().request_repaint();
         }

         let painted = prepared.paint(ui);
         if let Some(badge) = &badge {
            badge.paint_at(ui, painted.response.rect);
         }
         painted
      } else {
         AtomLayoutResponse::empty(prepared.response)
      };

      response.response.widget_info(|| {
         if let Some(text) = &text {
            WidgetInfo::labeled(WidgetType::Button, ui.is_enabled(), text)
         } else {
            WidgetInfo::new(WidgetType::Button)
         }
      });

      response
   }
}

impl Widget for Button<'_> {
   fn ui(self, ui: &mut Ui) -> Response {
      self.atom_ui(ui).response
   }
}

#[cfg(test)]
mod tests {
   use super::*;
   use egui::{
      Context, Event, FullOutput, Key, Modifiers, PointerButton, RawInput, ViewportId, vec2,
   };
   use std::time::Duration;

   /// Screen the test buttons live in; small enough to keep clicks deterministic.
   fn screen() -> Rect {
      Rect::from_min_size(Pos2::ZERO, vec2(320.0, 200.0))
   }

   /// Time step used by every pass; a binary fraction so progress sums exactly.
   const STEP: f32 = 0.125;

   /// The sweep colour used by the shape-inspection tests.
   const SWEEP: Color32 = Color32::from_rgb(1, 2, 3);

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

   fn enter() -> Event {
      Event::Key {
         key: Key::Enter,
         physical_key: None,
         pressed: true,
         repeat: false,
         modifiers: Modifiers::NONE,
      }
   }

   fn hold_button() -> Button<'static> {
      Button::new("Hold").hold(0.5)
   }

   /// Runs one pass, adding exactly one button and returning the frame's output
   /// and the button's [`Response`].
   fn run_pass(
      ctx: &Context,
      events: Vec<Event>,
      time: f64,
      build: impl FnOnce() -> Button<'static>,
   ) -> (FullOutput, Response) {
      let mut build = Some(build);
      let mut response = None;
      let mut output = ctx.run_ui(
         RawInput {
            events,
            screen_rect: Some(screen()),
            time: Some(time),
            predicted_dt: STEP,
            ..Default::default()
         },
         |ui| {
            let build = build.take().expect("the builder runs once");
            response = Some(ui.add(build()));
         },
      );
      // The tests inspect `shapes`; the font-atlas delta is never applied.
      output.textures_delta.clear();
      (
         output,
         response.expect("the button must be added"),
      )
   }

   fn hold_state(ctx: &Context, id: Id) -> Option<HoldState> {
      ctx.data(|d| d.get_temp::<HoldState>(id))
   }

   /// The clipped fill of the sweep shape, as `(clip_width, clip_left)`.
   fn sweep_shape(output: &FullOutput, color: Color32) -> Option<(f32, f32)> {
      output.shapes.iter().find_map(|clipped| match &clipped.shape {
         Shape::Rect(rect) if rect.fill == color => Some((
            clipped.clip_rect.width(),
            clipped.clip_rect.left(),
         )),
         _ => None,
      })
   }

   fn has_mesh(output: &FullOutput) -> bool {
      output.shapes.iter().any(|clipped| matches!(clipped.shape, Shape::Mesh(_)))
   }

   #[test]
   fn click_fires_only_at_the_end_of_the_hold() {
      let ctx = Context::default();
      let (_, warm) = run_pass(&ctx, vec![], 0.0, hold_button);
      let id = warm.id;
      let center = warm.rect.center();

      let mut time = f64::from(STEP);
      let (_, pressed) = run_pass(&ctx, vec![press(center)], time, hold_button);
      assert!(
         !pressed.clicked(),
         "a fresh press must not report a click"
      );
      assert_eq!(
         hold_state(&ctx, id).expect("hold state").progress,
         0.2,
         "the press frame must accumulate one capped step"
      );

      // Advance 0.4 s in steps: still below the 0.5 s duration.
      for _ in 0..3 {
         time += f64::from(STEP);
         let (_, frame) = run_pass(&ctx, vec![], time, hold_button);
         assert!(
            !frame.clicked(),
            "the hold reported a click before its duration elapsed"
         );
         assert!(hold_state(&ctx, id).expect("hold state").progress < 1.0);
      }
      assert_eq!(
         hold_state(&ctx, id).expect("hold state").progress,
         0.8
      );

      // The next step crosses 0.5 s: fire exactly here.
      time += f64::from(STEP);
      let (_, completed) = run_pass(&ctx, vec![], time, hold_button);
      assert!(
         completed.clicked(),
         "the completed hold must report a click"
      );
      assert_eq!(
         hold_state(&ctx, id).expect("hold state").progress,
         1.0
      );

      // Held but already fired: the click must not repeat.
      time += f64::from(STEP);
      let (_, after) = run_pass(&ctx, vec![], time, hold_button);
      assert!(
         !after.clicked(),
         "the click must not repeat after completion"
      );
   }

   #[test]
   fn releasing_midway_resets_and_never_fires() {
      let ctx = Context::default();
      let (_, warm) = run_pass(&ctx, vec![], 0.0, hold_button);
      let id = warm.id;
      let center = warm.rect.center();

      let mut time = f64::from(STEP);
      let (_, pressed) = run_pass(&ctx, vec![press(center)], time, hold_button);
      assert!(!pressed.clicked());
      assert!(hold_state(&ctx, id).expect("hold state").progress > 0.0);

      time += f64::from(STEP);
      let (_, released) = run_pass(&ctx, vec![release(center)], time, hold_button);
      assert!(
         !released.clicked(),
         "a release under the duration must not click"
      );
      assert_eq!(
         hold_state(&ctx, id).expect("hold state").progress,
         0.0,
         "a release must reset the hold"
      );

      for _ in 0..3 {
         time += f64::from(STEP);
         let (_, frame) = run_pass(&ctx, vec![], time, hold_button);
         assert!(
            !frame.clicked(),
            "a released hold must never fire"
         );
      }
      assert_eq!(
         hold_state(&ctx, id).expect("hold state").progress,
         0.0
      );
   }

   #[test]
   fn dragging_off_resets_the_hold() {
      let ctx = Context::default();
      let (_, warm) = run_pass(&ctx, vec![], 0.0, hold_button);
      let id = warm.id;
      let center = warm.rect.center();

      let mut time = f64::from(STEP);
      run_pass(&ctx, vec![press(center)], time, hold_button);
      assert!(hold_state(&ctx, id).expect("hold state").progress > 0.0);

      time += f64::from(STEP);
      let (_, dragged) = run_pass(
         &ctx,
         vec![Event::PointerMoved(Pos2::new(310.0, 190.0))],
         time,
         hold_button,
      );
      assert!(
         !dragged.is_pointer_button_down_on(),
         "dragging off the button must end the press"
      );
      assert_eq!(
         hold_state(&ctx, id).expect("hold state").progress,
         0.0,
         "dragging off must reset the hold"
      );

      time += f64::from(STEP);
      run_pass(&ctx, vec![], time, hold_button);
      assert_eq!(
         hold_state(&ctx, id).expect("hold state").progress,
         0.0
      );
   }

   #[test]
   fn a_plain_click_never_fires_a_hold() {
      let ctx = Context::default();
      let (_, warm) = run_pass(&ctx, vec![], 0.0, hold_button);
      let center = warm.rect.center();

      let mut time = f64::from(STEP);
      let (_, pressed) = run_pass(&ctx, vec![press(center)], time, hold_button);
      assert!(!pressed.clicked());

      time += f64::from(STEP);
      let (_, released) = run_pass(&ctx, vec![release(center)], time, hold_button);
      assert!(
         !released.clicked(),
         "a plain click must not confirm a hold-to-confirm button"
      );
   }

   #[test]
   fn keyboard_activation_confirms_immediately() {
      let ctx = Context::default();
      let (_, warm) = run_pass(&ctx, vec![], 0.0, hold_button);
      let id = warm.id;

      // Focus the button, then press Enter.
      ctx.memory_mut(|mem| mem.request_focus(id));
      let (_, activated) = run_pass(&ctx, vec![enter()], f64::from(STEP), hold_button);
      assert!(
         activated.clicked(),
         "Enter on a focused hold button must confirm"
      );
      assert!(
         hold_state(&ctx, id).expect("hold state").progress < 1.0,
         "keyboard confirmation must not wait for the hold duration"
      );
   }

   #[test]
   fn sweep_width_tracks_progress() {
      let build = || Button::new("Hold").hold(0.5).hold_fill(SWEEP);
      let ctx = Context::default();
      let (_, warm) = run_pass(&ctx, vec![], 0.0, build);
      let center = warm.rect.center();

      let (output, pressed) = run_pass(&ctx, vec![press(center)], f64::from(STEP), build);
      let progress = hold_state(&ctx, pressed.id).expect("hold state").progress;
      assert!(
         (progress - 0.2).abs() < 1e-6,
         "unexpected progress {progress}"
      );

      let (clip_width, clip_left) = sweep_shape(&output, SWEEP).expect("a sweep shape");
      assert!(
         (clip_width - progress * pressed.rect.width()).abs() < 0.01,
         "sweep width {clip_width} does not track progress {progress} of {}",
         pressed.rect.width()
      );
      assert!(
         (clip_left - pressed.rect.left()).abs() < 0.01,
         "the sweep must start at the button's left edge"
      );
   }

   #[test]
   fn two_hold_buttons_keep_separate_progress() {
      let ctx = Context::default();
      let mut first = None;
      let mut second = None;
      let warm = ctx.run_ui(
         RawInput {
            screen_rect: Some(screen()),
            time: Some(0.0),
            predicted_dt: STEP,
            ..Default::default()
         },
         |ui| {
            ui.horizontal(|ui| {
               first = Some(ui.add(Button::new("First").hold(0.5)));
               second = Some(ui.add(Button::new("Second").hold(0.5)));
            });
         },
      );
      warm.drop_without_applying_deltas();
      let first = first.expect("first button");
      let second = second.expect("second button");
      let center = first.rect.center();

      let (_, pressed_first, pressed_second) = {
         let mut first = None;
         let mut second = None;
         let mut output = ctx.run_ui(
            RawInput {
               events: vec![press(center)],
               screen_rect: Some(screen()),
               time: Some(f64::from(STEP)),
               predicted_dt: STEP,
               ..Default::default()
            },
            |ui| {
               ui.horizontal(|ui| {
                  first = Some(ui.add(Button::new("First").hold(0.5)));
                  second = Some(ui.add(Button::new("Second").hold(0.5)));
               });
            },
         );
         output.textures_delta.clear();
         (
            output,
            first.expect("first button"),
            second.expect("second button"),
         )
      };
      assert_eq!(
         pressed_first.id, first.id,
         "the first button's id must be stable"
      );
      assert_eq!(
         pressed_second.id, second.id,
         "the second button's id must be stable"
      );

      let first_progress = hold_state(&ctx, pressed_first.id).expect("first hold state").progress;
      let second_progress =
         hold_state(&ctx, pressed_second.id).expect("second hold state").progress;
      assert!(
         first_progress > 0.0,
         "the pressed button must progress"
      );
      assert_eq!(
         second_progress, 0.0,
         "the other button must not progress"
      );
   }

   #[test]
   fn idle_hold_button_requests_no_repaint() {
      let ctx = Context::default();
      // Two warm-up passes absorb egui's first-frame `SetTheme` command.
      for _ in 0..2 {
         run_pass(&ctx, vec![], 0.0, hold_button).0.drop_without_applying_deltas();
      }

      let (output, _) = run_pass(&ctx, vec![], 0.0, hold_button);
      let delay = output.viewport_output[&ViewportId::ROOT].repaint_delay;
      assert_eq!(
         delay,
         Duration::MAX,
         "an idle hold button requested a repaint after {delay:?}"
      );
   }

   #[test]
   fn hold_unset_changes_nothing() {
      let build = || Button::new("Hold").hold_fill(SWEEP);
      let ctx = Context::default();
      for _ in 0..2 {
         run_pass(&ctx, vec![], 0.0, build).0.drop_without_applying_deltas();
      }

      let (output, response) = run_pass(&ctx, vec![], 0.0, build);
      let delay = output.viewport_output[&ViewportId::ROOT].repaint_delay;
      assert_eq!(
         delay,
         Duration::MAX,
         "a button without `hold` requested a repaint after {delay:?}"
      );
      assert!(
         sweep_shape(&output, SWEEP).is_none(),
         "a button without `hold` must not paint a sweep"
      );
      assert!(
         hold_state(&ctx, response.id).is_none(),
         "a button without `hold` must not create hold state"
      );
   }

   #[test]
   fn the_burst_ends() {
      let build = || Button::new(()).hold(0.5);
      let ctx = Context::default();
      let (_, warm) = run_pass(&ctx, vec![], 0.0, build);
      let id = warm.id;
      let center = warm.rect.center();

      let mut time = f64::from(STEP);
      run_pass(&ctx, vec![press(center)], time, build);

      // Hold until it fires.
      let mut completed = None;
      let mut completion_time = time;
      for _ in 0..6 {
         time += f64::from(STEP);
         let (output, frame) = run_pass(&ctx, vec![], time, build);
         if frame.clicked() {
            completed = Some(output);
            completion_time = time;
            break;
         }
      }
      let completed = completed.expect("the hold must complete");
      assert!(
         hold_state(&ctx, id).expect("hold state").burst_age.is_some(),
         "a burst must start when the hold completes"
      );
      assert!(
         has_mesh(&completed),
         "the glow burst must paint a mesh"
      );

      // Release; the burst keeps animating on its own.
      time = completion_time + f64::from(STEP);
      let (_, released) = run_pass(&ctx, vec![release(center)], time, build);
      assert!(!released.clicked());

      // Let it run its course, then it must be gone and cost no frames.
      for _ in 0..4 {
         time += f64::from(STEP);
         run_pass(&ctx, vec![], time, build);
      }
      assert!(
         hold_state(&ctx, id).expect("hold state").burst_age.is_none(),
         "the burst must end"
      );

      time += f64::from(STEP);
      let (idle, _) = run_pass(&ctx, vec![], time, build);
      assert!(
         !has_mesh(&idle),
         "the ended burst must paint no mesh"
      );
      let delay = idle.viewport_output[&ViewportId::ROOT].repaint_delay;
      assert_eq!(
         delay,
         Duration::MAX,
         "the ended burst requested a repaint after {delay:?}"
      );
   }
}
