use egui::{
   Align, Align2, Atom, AtomExt as _, AtomKind, Color32, Frame, Id, Image, IntoAtoms, NumExt as _,
   Pos2, Rect, Response, Role, Sense, Shadow, Shape, Stroke, TextStyle, TextWrapMode, Ui, Vec2,
   Widget, WidgetAtom, WidgetAtomResponse, WidgetInfo, WidgetText, response::Flags,
};

use crate::glow::{GlowProfile, glow_mesh_with, size_attenuation};
use crate::motion::{InteractT, Motion};
use crate::particles::Lightning;
use crate::theme::Theme;
use crate::visuals::ButtonVisuals;
use crate::widgets::badge::Badge;

mod hold;
mod push;

pub use hold::{HoldBurst, HoldSettings};
pub use push::PushStyle;

/// Longest `stable_dt` the hold accumulation accepts, in seconds (spike guard).
const HOLD_MAX_DT: f32 = 0.1;
/// Width of the sweep's leading-edge line, in pixels.
const HOLD_EDGE_WIDTH: f32 = 1.5;
/// Peak flash alpha of a [`HoldBurst::Lightning`] completion burst at full
/// intensity.
const HOLD_BURST_FLASH: f32 = 0.22;
/// [`HoldBurst::Glow`] radius at the start of the burst, as a multiple of the
/// button's shorter side; the default of [`HoldSettings::burst_glow_from`].
pub(crate) const HOLD_GLOW_START: f32 = 1.0;
/// [`HoldBurst::Glow`] radius at the end of the burst, as a multiple of the
/// button's shorter side; the default of [`HoldSettings::burst_glow_to`].
pub(crate) const HOLD_GLOW_END: f32 = 2.2;

/// Per-button hold-to-confirm state, keyed by the button's response [`Id`].
#[derive(Clone, Copy, Default)]
struct HoldState {
   /// Whether this widget owns the current press.
   ///
   /// Latched from egui's hit test at press time, while that test is still
   /// valid, and kept while the primary button is down so the gesture outlives
   /// egui's `max_click_duration`.
   pressing: bool,
   /// Fraction of the hold completed, `0.0..=1.0`.
   progress: f32,
   /// Whether the current hold has already fired (so it fires once).
   fired: bool,
   /// Seconds since the burst started, while one is animating.
   burst_age: Option<f32>,
}

/// A [`HoldBurst::Lightning`] strike, addressed by a derived id so it never
/// collides with a host's own [`Lightning`].
fn burst_lightning(id: Id, area: Rect, settings: &HoldSettings, color: Color32) -> Lightning {
   let intensity = settings.burst_intensity.clamp(0.0, 1.0);
   Lightning::new(("hold_burst", id))
      .color(color.gamma_multiply(intensity))
      .bolt_count(settings.burst_bolts)
      .branch_depth(settings.burst_branch_depth)
      .thickness(settings.burst_thickness)
      .jitter(settings.burst_jitter)
      .duration_ms(settings.burst_ms)
      .flash_intensity(HOLD_BURST_FLASH * intensity)
      .area(area)
}

/// Paints one frame of a completion burst on `ui`'s painter.
fn paint_burst(
   ui: &Ui,
   settings: &HoldSettings,
   id: Id,
   rect: Rect,
   expansion: f32,
   age: f32,
   color: Color32,
) {
   if !settings.burst_visible() {
      return;
   }

   let t = (age / settings.burst_secs()).clamp(0.0, 1.0);
   let intensity = settings.burst_intensity.clamp(0.0, 1.0);
   match settings.burst {
      HoldBurst::Off => {}
      HoldBurst::Glow => {
         // Grow and fade the shared glow fan; `size_attenuation` keeps the peak
         // alpha in family with the Light effect instead of a second falloff.
         let short = rect.width().min(rect.height()).max(1.0);
         let (from, to) = settings.glow_radius_range();
         let radius = short * (from + (to - from) * t);
         let attenuation = size_attenuation(radius, short * from, short * to);
         let tint = color.gamma_multiply((1.0 - t) * attenuation * intensity);
         if tint.a() > 0 {
            ui.painter().add(Shape::mesh(glow_mesh_with(
               rect.center(),
               radius,
               tint,
               GlowProfile::softness(settings.burst_glow_softness),
            )));
         }
      }
      HoldBurst::Lightning => {
         burst_lightning(id, rect.expand(expansion), settings, color).show(ui.ctx());
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
   layout: WidgetAtom<'a>,
   visuals: Option<ButtonVisuals>,
   bg_color: Option<Color32>,
   badge: Option<Badge>,
   small: bool,
   frame_when_inactive: bool,
   min_size: Vec2,
   label_align: Align,
   selected: bool,
   image_tint_follows_text_color: bool,
   limit_image_size: bool,
   hold: Option<HoldSettings>,
   push: Option<PushStyle>,
}

impl<'a> Button<'a> {
   /// Create a button from atoms (text, images, or a mix).
   pub fn new(atoms: impl IntoAtoms<'a>) -> Self {
      Self {
         layout: WidgetAtom::new(atoms.into_atoms())
            .sense(Sense::click())
            .fallback_font(TextStyle::Button),
         visuals: None,
         bg_color: None,
         badge: None,
         small: false,
         frame_when_inactive: true,
         min_size: Vec2::ZERO,
         label_align: Align::Min,
         selected: false,
         image_tint_follows_text_color: false,
         limit_image_size: false,
         hold: None,
         push: None,
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

   /// Align the button's atoms within its [`Self::min_size`] rect.
   ///
   /// Defaults to [`Align::Min`], which leaves the atoms where the surrounding
   /// [`Ui`] layout puts them — the rendering from before this builder existed.
   ///
   /// The implementation sets the horizontal component of the atom layout's
   /// [`WidgetAtom::align2`]: it moves the measured block of atoms inside the
   /// allocated rect. It does **not** use the layout's main-axis justification,
   /// which would stretch the gap *between* atoms instead of moving the block.
   ///
   /// A trailing grow atom — added by [`Self::right_text`] or
   /// [`Self::shortcut_text`] — makes the atom block fill the rect, so there is
   /// no leftover space for this alignment to move and the trailing text stays
   /// pinned to the right edge under every alignment.
   #[inline]
   pub fn label_align(mut self, align: Align) -> Self {
      self.label_align = align;
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
   /// The widget tracks the press itself: it latches egui's hit test at press
   /// time and then follows the raw pointer, so **the hold works for durations
   /// longer than egui's `max_click_duration`** (0.8 s by default), past the
   /// point where egui stops treating the press as a potential click. Releasing
   /// the button, or dragging the pointer further than `max_click_dist` away
   /// from it, cancels the hold.
   ///
   /// Note for touch screens: egui may additionally report a long touch as a
   /// secondary-click (context menu) once the press passes `max_click_duration`;
   /// this widget does not consume that.
   ///
   /// While the hold progresses the frame's fill is replaced by a left-to-right
   /// sweep that grows as the hold completes; when it fills, the button fires
   /// once and plays its [`HoldSettings::burst`]. Pressing Enter or Space on a
   /// focused button confirms immediately, like egui's keyboard activation for
   /// buttons.
   ///
   /// This shortcut only sets [`HoldSettings::seconds`]; every other field keeps
   /// its current value, so shortcuts compose in call order.
   pub fn hold(mut self, seconds: f32) -> Self {
      let mut settings = self.hold.unwrap_or_else(HoldSettings::default);
      settings.seconds = seconds;
      self.hold = Some(settings);
      self
   }

   /// Set a solid hold-sweep colour, and the default tint of a
   /// [`HoldBurst::Glow`] burst.
   ///
   /// This sets [`HoldSettings::fill_from`] and clears
   /// [`HoldSettings::fill_to`], so the sweep stays one colour; use
   /// [`Self::hold_settings`] for a colour that shifts as the hold fills.
   ///
   /// Defaults to the installed theme's
   /// [`colors.accent`](crate::theme::ThemeColors::accent). This shortcut does
   /// **not** arm the hold on its own — call [`Self::hold`] or
   /// [`Self::hold_settings`] too.
   pub fn hold_fill(mut self, color: Color32) -> Self {
      let settings = self.hold.get_or_insert_with(|| HoldSettings {
         seconds: 0.0,
         ..Default::default()
      });
      settings.fill_from = Some(color);
      settings.fill_to = None;
      self
   }

   /// Set the completion burst played when the hold fires.
   ///
   /// This sets [`HoldSettings::burst`]. Default: [`HoldBurst::Glow`]. This
   /// shortcut does **not** arm the hold on its own — call [`Self::hold`] or
   /// [`Self::hold_settings`] too.
   pub fn hold_burst(mut self, burst: HoldBurst) -> Self {
      let settings = self.hold.get_or_insert_with(|| HoldSettings {
         seconds: 0.0,
         ..Default::default()
      });
      settings.burst = burst;
      self
   }

   /// Replace the whole hold configuration with `settings`.
   ///
   /// Copies every field, including [`HoldSettings::seconds`]; a value of
   /// `0.0` or less leaves the button as an ordinary click button. The most
   /// recent of this call and the per-field shortcuts wins for the fields it
   /// sets.
   pub fn hold_settings(mut self, settings: &HoldSettings) -> Self {
      self.hold = Some(*settings);
      self
   }

   /// Lift the face above a solid base that collapses while the button is
   /// pressed.
   ///
   /// `depth` is the height of the base in points — and the distance the face
   /// travels down when pressed. `0.0` or less leaves the button flat.
   ///
   /// The widget reserves `depth` extra height below the face, so a following
   /// row never lands on the base, and the whole rectangle (base included) is
   /// the click target. [`Self::min_size`] keeps sizing the face, not the
   /// widget. While armed the base replaces the theme's soft shadow; the face
   /// keeps its themed fill, border and hover/press colours, so the 3D edge is
   /// styled by [`ButtonVisuals`] like every other part of the button.
   ///
   /// ```
   /// # use egui::__run_test_ui;
   /// # use egui_elements::widgets::Button;
   /// # __run_test_ui(|ui| {
   /// ui.add(Button::new("Save").push(4.0));
   /// # });
   /// ```
   pub fn push(mut self, depth: f32) -> Self {
      let mut style = self.push.unwrap_or_default();
      style.depth = depth;
      self.push = Some(style);
      self
   }

   /// Set the colour of the base painted by [`Self::push`].
   ///
   /// Does **not** arm the effect on its own — call [`Self::push`] or
   /// [`Self::push_style`] too, like [`Self::hold_fill`]. Defaults to the
   /// installed theme's
   /// [`colors.accent`](crate::theme::ThemeColors::accent).
   pub fn push_color(mut self, color: Color32) -> Self {
      let style = self.push.get_or_insert_with(|| PushStyle {
         depth: 0.0,
         ..Default::default()
      });
      style.color = Some(color);
      self
   }

   /// Replace the whole push configuration with `style`.
   ///
   /// Copies every field, including [`PushStyle::depth`]; a value of `0.0` or
   /// less leaves the button flat. The most recent of this call and the
   /// per-field shortcuts wins for the fields it sets.
   pub fn push_style(mut self, style: &PushStyle) -> Self {
      self.push = Some(*style);
      self
   }

   /// Show the button and return a [`WidgetAtomResponse`] for painting custom contents.
   pub fn atom_ui(self, ui: &mut Ui) -> WidgetAtomResponse {
      let Button {
         mut layout,
         bg_color,
         badge,
         small,
         visuals,
         frame_when_inactive,
         mut min_size,
         label_align,
         selected,
         image_tint_follows_text_color,
         limit_image_size,
         hold,
         push,
      } = self;

      let hold_settings = hold.filter(|settings| settings.seconds > 0.0);

      let visuals = visuals.or_else(|| Theme::button_visuals_from_ctx(ui.ctx()));
      let has_frame_margin = visuals.is_some() || ui.visuals().button_frame;
      // A frameless button has nothing to raise, and `push_color` alone must not
      // arm the effect, so keep only a style that will paint.
      let push = push.filter(|style| style.depth > 0.0 && has_frame_margin);

      if !small {
         min_size.y = min_size.y.at_least(ui.spacing().interact_size.y);
      }
      if let Some(style) = push {
         // The base is layout, not overflow: the widget owns the band so a
         // following row never lands on top of it. `min_size` stays the face.
         min_size.y += style.depth;
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

      // Only the horizontal component is overridden: `Align::Min` keeps the
      // surrounding layout's own alignment, so the default emits the same shapes
      // as before this knob existed. The vertical component always follows the
      // layout, exactly as it did when `align2` was left unset.
      if label_align != Align::Min {
         let vertical = ui.layout().vertical_align();
         layout = layout.align2(Align2([label_align, vertical]));
      }

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

         // ---- hold-to-confirm gesture ------------------------------------
         // egui abandons `is_pointer_button_down_on` once the press outlives
         // `max_click_duration` (it backs the long-touch context menu there), so
         // use it only to latch press ownership at press time and track the hold
         // from the raw pointer afterwards.
         let mut hold = HoldState::default();
         let mut fired_now = false;
         let mut holding = false;
         if let Some(settings) = hold_settings {
            let id = prepared.response.id;
            let seconds = settings.seconds;
            let dt = ui.ctx().input(|i| i.stable_dt).min(HOLD_MAX_DT);
            let (pointer_pos, primary_down) =
               ui.ctx().input(|i| (i.pointer.interact_pos(), i.pointer.primary_down()));
            let max_click_dist = ui.ctx().options(|o| o.input_options.max_click_dist);
            let rect = prepared.response.rect;
            hold = ui.ctx().data_mut(|data| {
               let state = data.get_temp_mut_or_insert_with(id, HoldState::default);
               if prepared.response.is_pointer_button_down_on() {
                  state.pressing = true;
               }
               let held = state.pressing
                  && primary_down
                  && pointer_pos.is_some_and(|p| rect.expand(max_click_dist).contains(p));
               holding = held;
               if held {
                  state.progress = (state.progress + dt / seconds).clamp(0.0, 1.0);
               } else {
                  // Release and drag-off both land here.
                  state.progress = 0.0;
                  state.fired = false;
                  if !primary_down {
                     // The latch must not leak into the next press.
                     state.pressing = false;
                  }
               }
               if state.progress >= 1.0 && !state.fired {
                  state.fired = true;
                  if settings.burst_visible() {
                     state.burst_age = Some(0.0);
                  }
                  fired_now = true;
               }
               let snapshot = *state;
               // Advance the burst clock for the next frame; the snapshot above
               // is this frame's age, so the burst paints before it advances.
               if let Some(age) = state.burst_age.as_mut() {
                  *age += dt;
                  if *age >= settings.burst_secs() {
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
            if fired_now && settings.burst == HoldBurst::Lightning && settings.burst_visible() {
               let colors = Theme::colors_from_ctx(ui.ctx());
               let accent = colors.accent;
               let fill = settings.fill_color(accent, 1.0);
               let color = settings.burst_color.unwrap_or(fill);
               burst_lightning(
                  id,
                  prepared.response.rect.expand(interact_visuals.expansion),
                  &settings,
                  color,
               )
               .strike(ui.ctx());
            }
         }

         // The button keeps its pressed chrome for the whole hold; egui's own
         // active flag stops mid-press at `max_click_duration`. Once the hold
         // fires the pressed look is released, while the sweep and burst keep
         // following `holding`.
         let is_active = if hold_settings.is_some() {
            holding && !hold.fired
         } else {
            prepared.response.is_pointer_button_down_on()
         };
         let is_hovered = prepared.response.hovered();

         // The widget reserves `depth` below the face; the face travels that far
         // down while pressed. Painting follows the face while interaction stays
         // on the whole rectangle, so the base is clickable.
         let full_rect = prepared.response.rect;
         let face_rect = push.map_or(full_rect, |style| {
            let face = Rect::from_min_max(
               full_rect.min,
               Pos2::new(full_rect.max.x, full_rect.max.y - style.depth),
            );
            if is_active {
               face.translate(Vec2::new(0.0, style.depth))
            } else {
               face
            }
         });
         prepared.response.rect = face_rect;

         // Eased idle → hover → press. `is_active` / `is_hovered` stay the caller's own state selection; only the
         // colours between states are blended, so a hold gesture keeps its pressed chrome exactly as before.
         let across = InteractT::of(
            ui,
            prepared.response.id,
            is_hovered,
            is_active,
            Motion::from_ui(ui),
         );
         let (fill, stroke, corner_radius) = if let Some(v) = &visuals {
            (
               across.blend(bg_color.unwrap_or(v.bg), v.bg_hover, v.bg_click),
               across.blend(v.border, v.border_hover, v.border_click),
               v.corner_radius,
            )
         } else {
            let widgets = &ui.style().visuals.widgets;
            (
               across.blend(
                  widgets.inactive.weak_bg_fill,
                  widgets.hovered.weak_bg_fill,
                  widgets.active.weak_bg_fill,
               ),
               across.blend(
                  widgets.inactive.bg_stroke,
                  widgets.hovered.bg_stroke,
                  widgets.active.bg_stroke,
               ),
               interact_visuals.corner_radius,
            )
         };

         // Selection is instant by design: a selected button changes fill in one step.
         let fill = match selected {
            false => fill,
            true => {
               visuals.as_ref().map(|v| v.bg_selected).unwrap_or(interact_visuals.weak_bg_fill)
            }
         };

         let text_color = visuals.as_ref().map(|v| v.text).unwrap_or(interact_visuals.text_color());
         let shadow = visuals.as_ref().map(|v| v.shadow).unwrap_or(Shadow::NONE);
         // The plate *is* this style's elevation; a soft shadow would double the
         // edge and blur the hard base.
         let shadow = if push.is_some() { Shadow::NONE } else { shadow };

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

            // Painted before the frame fills the face: the same rounded rect
            // shifted down by `depth`. A pressed face lands exactly on it, so
            // it is skipped there instead of painted and covered.
            if !is_active {
               if let Some(style) = push {
                  let color =
                     style.color.unwrap_or_else(|| Theme::colors_from_ctx(ui.ctx()).accent);
                  let base = face_rect
                     .expand(interact_visuals.expansion)
                     .translate(Vec2::new(0.0, style.depth));
                  ui.painter().add(Shape::rect_filled(base, corner_radius, color));
               }
            }
         }

         // ---- sweep + burst painting -------------------------------------
         // The fill clears the moment the hold fires; the burst takes over.
         let sweep_active = hold.progress > 0.0 && !hold.fired;
         let burst_active = hold.burst_age.is_some();
         let hold_color = hold_settings.filter(|_| sweep_active || burst_active).map(|settings| {
            let colors = Theme::colors_from_ctx(ui.ctx());
            let accent = colors.accent;
            settings.fill_color(accent, hold.progress)
         });

         if let Some(hold_color) = hold_color {
            if visible_frame {
               // Give up the frame's own fill *and* shadow so the base fill,
               // sweep and edge can be painted under the frame; the frame's
               // stroke still lands on top. The shadow is taken over here and
               // painted before the base fill, exactly as `Frame::paint` would
               // have ordered it — otherwise its opaque spread would cover the
               // alpha-filled button and the sweep.
               prepared.frame = prepared.frame.fill(Color32::TRANSPARENT).shadow(Shadow::NONE);

               let rect = prepared.response.rect;
               let expanded = rect.expand(interact_visuals.expansion);
               if shadow != Shadow::NONE {
                  ui.painter().add(shadow.as_shape(expanded, corner_radius));
               }
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

            if let (Some(settings), Some(age)) = (hold_settings, hold.burst_age) {
               let burst_color = settings.burst_color.unwrap_or(hold_color);
               paint_burst(
                  ui,
                  &settings,
                  prepared.response.id,
                  prepared.response.rect,
                  interact_visuals.expansion,
                  age,
                  burst_color,
               );
            }
         }

         // Repaint only while the hold progresses or a burst animates; a full,
         // held sweep is static, so it does not pin the frame loop.
         if (sweep_active && hold.progress < 1.0) || burst_active {
            ui.ctx().request_repaint();
         }

         let mut painted = prepared.paint(ui);
         // Report the whole widget — base included — while painting followed the
         // face, so layout, tooltips and the badge see the full extent.
         painted.response.rect = full_rect;
         if let Some(badge) = &badge {
            badge.paint_at_id(ui, painted.response.rect, painted.response.id);
         }
         painted
      } else {
         WidgetAtomResponse::empty(prepared.response)
      };

      response.response.widget_info(|| {
         if let Some(text) = &text {
            WidgetInfo::labeled(Role::Button, ui.is_enabled(), text)
         } else {
            WidgetInfo::new(Role::Button)
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

   /// egui's click ceiling, read from the live context so the tests stay
   /// meaningful if the default ever changes.
   fn max_click_duration(ctx: &Context) -> f64 {
      ctx.options(|o| o.input_options.max_click_duration)
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

   /// Fill rect of the first `RectShape` painted in `color`.
   fn fill_rect(output: &FullOutput, color: Color32) -> Option<Rect> {
      output.shapes.iter().find_map(|clipped| match &clipped.shape {
         Shape::Rect(rect) if rect.fill == color => Some(rect.rect),
         _ => None,
      })
   }

   fn has_mesh(output: &FullOutput) -> bool {
      output.shapes.iter().any(|clipped| matches!(clipped.shape, Shape::Mesh(_)))
   }
   /// Highest vertex alpha of any glow mesh in the frame (`0` when none).
   fn mesh_alpha(output: &FullOutput) -> u8 {
      output
         .shapes
         .iter()
         .filter_map(|clipped| match &clipped.shape {
            Shape::Mesh(mesh) => Some(mesh.vertices.iter().map(|vertex| vertex.color.a()).max()),
            _ => None,
         })
         .flatten()
         .max()
         .unwrap_or(0)
   }

   /// Number of polylines painted this frame (a lightning strike's bolts).
   fn path_count(output: &FullOutput) -> usize {
      output
         .shapes
         .iter()
         .filter(|clipped| matches!(clipped.shape, Shape::Path(_)))
         .count()
   }

   /// Whether the frame paints any completion-burst shape (glow mesh or bolt).
   fn has_burst_shape(output: &FullOutput) -> bool {
      has_mesh(output) || path_count(output) > 0
   }

   /// Advance passes until the hold fires, returning that frame's output and time.
   fn advance_until_fired(
      ctx: &Context,
      time: &mut f64,
      build: &impl Fn() -> Button<'static>,
   ) -> (FullOutput, f64) {
      for _ in 0..12 {
         *time += f64::from(STEP);
         let (output, frame) = run_pass(ctx, vec![], *time, || build());
         if frame.clicked() {
            return (output, *time);
         }
      }
      panic!("the hold must complete");
   }

   /// Press a button built from `settings` and hold it to completion.
   fn complete_hold(settings: HoldSettings) -> (Context, FullOutput, Response, f64) {
      let ctx = Context::default();
      let build = move || Button::new("Hold").hold_settings(&settings);
      let (_, warm) = run_pass(&ctx, vec![], 0.0, build);
      let center = warm.rect.center();
      let mut time = f64::from(STEP);
      run_pass(&ctx, vec![press(center)], time, build);
      let (completed, completion_time) = advance_until_fired(&ctx, &mut time, &build);
      (ctx, completed, warm, completion_time)
   }

   /// The press frame's progress and the number of further passes until the
   /// hold reports `clicked()`.
   fn press_outcome(build: impl Fn() -> Button<'static> + Copy) -> (f32, usize) {
      let ctx = Context::default();
      let (_, warm) = run_pass(&ctx, vec![], 0.0, build);
      let center = warm.rect.center();
      let mut time = f64::from(STEP);
      run_pass(&ctx, vec![press(center)], time, build);
      let first = hold_state(&ctx, warm.id).expect("hold state").progress;
      let mut passes = 0;
      loop {
         time += f64::from(STEP);
         passes += 1;
         let (_, frame) = run_pass(&ctx, vec![], time, build);
         if frame.clicked() {
            break;
         }
         assert!(
            passes < 12,
            "the hold did not complete after {passes} passes"
         );
      }
      (first, passes)
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
   fn the_burst_lifetime_is_honoured_and_costs_no_idle_frames() {
      // Case: the_burst_ends
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

      // Case: the_burst_honours_its_duration
      /// Long enough that the old 300 ms constant would be long gone.
      const BURST_MS: f32 = 600.0;
      let settings = HoldSettings {
         seconds: 0.5,
         burst: HoldBurst::Glow,
         burst_ms: BURST_MS,
         ..Default::default()
      };
      let (ctx, _, _, completion_time) = complete_hold(settings);
      let build = move || Button::new("Hold").hold_settings(&settings);
      let mut time = completion_time;

      // ~375 ms after completion: the burst must still be visible.
      for _ in 0..3 {
         time += f64::from(STEP);
         run_pass(&ctx, vec![], time, build);
      }
      let (still, _) = run_pass(&ctx, vec![], time, build);
      assert!(
         mesh_alpha(&still) > 0,
         "the {BURST_MS} ms burst was already invisible ~400 ms after completion"
      );

      // ~750 ms after completion: the burst must be over.
      for _ in 0..2 {
         time += f64::from(STEP);
         run_pass(&ctx, vec![], time, build);
      }
      let (gone, _) = run_pass(&ctx, vec![], time, build);
      assert_eq!(
         mesh_alpha(&gone),
         0,
         "the {BURST_MS} ms burst still painted ~700 ms after completion"
      );

      // Case: burst_off_paints_nothing_and_costs_no_frames
      let settings = HoldSettings {
         seconds: 0.5,
         burst: HoldBurst::Off,
         ..Default::default()
      };
      let (ctx, completed, warm, mut time) = complete_hold(settings);
      assert!(
         !has_burst_shape(&completed),
         "a burst set to Off must not paint a completion shape"
      );

      // Still held: nothing animates, so the widget must not pin the frame loop.
      let build = move || Button::new("Hold").hold_settings(&settings);
      time += f64::from(STEP);
      let (after, _) = run_pass(&ctx, vec![], time, build);
      assert_eq!(
         hold_state(&ctx, warm.id).expect("hold state").progress,
         1.0,
         "the pointer is still down after completion"
      );
      assert!(
         !has_burst_shape(&after),
         "an Off burst must stay unpainted while the completed hold is held"
      );
      let delay = after.viewport_output[&ViewportId::ROOT].repaint_delay;
      assert_eq!(
         delay,
         Duration::MAX,
         "an Off burst requested a repaint after {delay:?}"
      );
   }

   #[test]
   fn hold_outlives_egui_max_click_duration() {
      const SECONDS: f32 = 1.5;
      let ctx = Context::default();
      let ceiling = max_click_duration(&ctx);
      assert!(
         f64::from(SECONDS) > ceiling,
         "this test only bites while its {SECONDS}s hold outlives egui's {ceiling}s click ceiling"
      );
      let build = || Button::new("Hold").hold(SECONDS);
      let (_, warm) = run_pass(&ctx, vec![], 0.0, build);
      let id = warm.id;
      let center = warm.rect.center();

      let mut time = f64::from(STEP);
      let (_, pressed) = run_pass(&ctx, vec![press(center)], time, build);
      assert!(!pressed.clicked(), "a fresh press must not click");
      let press_time = time;

      let max_frames = (SECONDS / HOLD_MAX_DT).ceil() as usize + 2;
      let mut completion_time = time;
      let mut fired = false;
      for _ in 0..max_frames {
         time += f64::from(STEP);
         let (_, frame) = run_pass(&ctx, vec![], time, build);
         let progress = hold_state(&ctx, id).expect("hold state").progress;
         if progress < 1.0 {
            assert!(
               !frame.clicked(),
               "the hold clicked while the sweep was only {progress} full"
            );
         } else {
            assert!(
               frame.clicked(),
               "the hold must click on the frame its sweep completes"
            );
            fired = true;
            completion_time = time;
            break;
         }
      }
      assert!(
         fired,
         "a {SECONDS}s hold must complete past egui's {ceiling}s click ceiling"
      );
      assert!(
         completion_time - press_time > ceiling,
         "the hold finished in {}s, inside egui's {ceiling}s click ceiling",
         completion_time - press_time
      );

      // Releasing after completion resets the gesture and must not re-fire.
      time = completion_time + f64::from(STEP);
      let (_, released) = run_pass(&ctx, vec![release(center)], time, build);
      assert!(
         !released.clicked(),
         "a completed hold must not click again on release"
      );
      assert_eq!(
         hold_state(&ctx, id).expect("hold state").progress,
         0.0,
         "a release must reset the completed hold"
      );
   }

   #[test]
   fn a_second_press_starts_a_fresh_hold() {
      const SECONDS: f32 = 1.0;
      let build = || Button::new("Hold").hold(SECONDS);
      let ctx = Context::default();
      let (_, warm) = run_pass(&ctx, vec![], 0.0, build);
      let id = warm.id;
      let center = warm.rect.center();
      let max_frames = (SECONDS / HOLD_MAX_DT).ceil() as usize + 2;

      let mut time = f64::from(STEP);
      run_pass(&ctx, vec![press(center)], time, build);

      // First gesture: hold to completion.
      let mut completion_time = time;
      let mut first_fired = false;
      for _ in 0..max_frames {
         time += f64::from(STEP);
         let (_, frame) = run_pass(&ctx, vec![], time, build);
         if frame.clicked() {
            first_fired = true;
            completion_time = time;
            break;
         }
      }
      assert!(first_fired, "the first hold must fire");
      assert_eq!(
         hold_state(&ctx, id).expect("hold state").progress,
         1.0
      );

      // Release: the latch must clear so the next press starts from zero.
      time = completion_time + f64::from(STEP);
      let (_, released) = run_pass(&ctx, vec![release(center)], time, build);
      assert!(!released.clicked(), "a release must not click");
      assert_eq!(
         hold_state(&ctx, id).expect("hold state").progress,
         0.0,
         "a release must reset the first hold"
      );

      // A second press must accumulate from zero and fire on its own.
      time += f64::from(STEP);
      let (_, second) = run_pass(&ctx, vec![press(center)], time, build);
      assert!(
         !second.clicked(),
         "the second press must not click immediately"
      );
      let second_progress = hold_state(&ctx, id).expect("hold state").progress;
      assert!(
         second_progress > 0.0 && second_progress < 1.0,
         "the second gesture must start from zero, got {second_progress}"
      );

      let mut second_fired = false;
      for _ in 0..max_frames {
         time += f64::from(STEP);
         let (_, frame) = run_pass(&ctx, vec![], time, build);
         if frame.clicked() {
            second_fired = true;
            break;
         }
      }
      assert!(
         second_fired,
         "the second hold must fire on its own"
      );
   }

   #[test]
   fn hold_settings_sets_the_duration() {
      let settings = HoldSettings {
         seconds: 0.5,
         ..Default::default()
      };
      let from_settings = press_outcome(move || Button::new("Hold").hold_settings(&settings));
      let from_shortcut = press_outcome(|| Button::new("Hold").hold(0.5));
      assert_eq!(
         from_settings, from_shortcut,
         "hold_settings(seconds = 0.5) must fire exactly like hold(0.5)"
      );
      assert_eq!(
         from_settings.0, 0.2,
         "the first press frame must accumulate one capped step"
      );
   }

   #[test]
   fn shortcuts_and_settings_compose_in_call_order() {
      // The later `hold(0.5)` wins over `hold(2.0)` for `seconds`.
      let later_shortcut = press_outcome(|| Button::new("Hold").hold(2.0).hold(0.5));
      let plain = press_outcome(|| Button::new("Hold").hold(0.5));
      assert_eq!(
         later_shortcut, plain,
         "`.hold(2.0).hold(0.5)` must behave like `.hold(0.5)`"
      );

      /// The shortcut's fill, replaced by `hold_settings`.
      const SHORTCUT_FILL: Color32 = Color32::from_rgb(220, 20, 20);
      /// The settings' fill, which must win.
      const SETTINGS_FILL: Color32 = Color32::from_rgb(20, 220, 20);
      let settings = HoldSettings {
         seconds: 0.5,
         fill_from: Some(SETTINGS_FILL),
         fill_to: None,
         ..Default::default()
      };
      let build = move || Button::new("Hold").hold_fill(SHORTCUT_FILL).hold_settings(&settings);
      let ctx = Context::default();
      let (_, warm) = run_pass(&ctx, vec![], 0.0, build);
      let (pressed, _) = run_pass(
         &ctx,
         vec![press(warm.rect.center())],
         f64::from(STEP),
         build,
      );
      assert!(
         sweep_shape(&pressed, SETTINGS_FILL).is_some(),
         "`.hold_fill(a).hold_settings(&s)` must use the settings' fill"
      );
      assert!(
         sweep_shape(&pressed, SHORTCUT_FILL).is_none(),
         "the earlier shortcut fill must have been replaced"
      );

      // Fill before `hold` still composes: `.hold_fill(a).hold(0.5)` keeps `a`.
      let before_hold = || Button::new("Hold").hold_fill(SHORTCUT_FILL).hold(0.5);
      let ctx = Context::default();
      let (_, warm) = run_pass(&ctx, vec![], 0.0, before_hold);
      let (pressed, _) = run_pass(
         &ctx,
         vec![press(warm.rect.center())],
         f64::from(STEP),
         before_hold,
      );
      assert!(
         sweep_shape(&pressed, SHORTCUT_FILL).is_some(),
         "`.hold_fill(a).hold(seconds)` must keep the earlier fill"
      );
   }

   /// Base colour of the push tests; distinct from every paint the harness makes.
   const BASE: Color32 = Color32::from_rgb(7, 11, 13);
   /// Face fill of the push tests.
   const FACE: Color32 = Color32::from_rgb(23, 29, 31);

   /// A raised button whose face and base paints are identifiable in `shapes`.
   fn push_button() -> Button<'static> {
      Button::new("Push").push(6.0).push_color(BASE).visuals(ButtonVisuals {
         bg: FACE,
         bg_hover: FACE,
         bg_click: FACE,
         ..Default::default()
      })
   }

   #[test]
   fn the_base_sits_below_the_face_at_rest() {
      let ctx = Context::default();
      let (output, response) = run_pass(&ctx, vec![], 0.0, push_button);

      let base = fill_rect(&output, BASE).expect("the resting button paints its base");
      let face = fill_rect(&output, FACE).expect("the resting button paints its face");

      assert_eq!(
         base.size(),
         face.size(),
         "the base is a shifted copy of the face"
      );
      assert_eq!(
         base.top(),
         face.top() + 6.0,
         "the base hangs one depth below the face"
      );
      assert_eq!(
         face.top(),
         response.rect.top(),
         "the face keeps the top of the widget"
      );
      assert_eq!(
         base.bottom(),
         response.rect.bottom(),
         "the base ends the widget, so the band is layout"
      );
   }

   #[test]
   fn pressing_collapses_the_base_and_drops_the_face() {
      let ctx = Context::default();
      let (_, warm) = run_pass(&ctx, vec![], 0.0, push_button);
      let center = warm.rect.center();
      let (pressed, response) = run_pass(
         &ctx,
         vec![press(center)],
         f64::from(STEP),
         push_button,
      );

      assert!(
         fill_rect(&pressed, BASE).is_none(),
         "a pressed face lands on the base, so the base must not be painted"
      );
      let face = fill_rect(&pressed, FACE).expect("the pressed face still paints");
      assert_eq!(
         face.top(),
         warm.rect.top() + 6.0,
         "the face travels exactly one depth down"
      );
      assert_eq!(face.bottom(), warm.rect.bottom());
      assert_eq!(
         response.rect, warm.rect,
         "the widget keeps its whole rect while pressed"
      );
   }

   #[test]
   fn a_flat_depth_reserves_no_layout_band() {
      let plain_ctx = Context::default();
      let (_, plain) = run_pass(&plain_ctx, vec![], 0.0, || Button::new("Flat"));

      let zero_ctx = Context::default();
      let (output, zero) = run_pass(&zero_ctx, vec![], 0.0, || {
         Button::new("Flat").push(0.0).push_color(BASE)
      });
      assert_eq!(
         zero.rect.height(),
         plain.rect.height(),
         "0 depth is flat"
      );
      assert!(
         fill_rect(&output, BASE).is_none(),
         "and paints no base"
      );

      let color_only_ctx = Context::default();
      let (output, color_only) = run_pass(&color_only_ctx, vec![], 0.0, || {
         Button::new("Flat").push_color(BASE)
      });
      assert_eq!(
         color_only.rect.height(),
         plain.rect.height(),
         "a colour alone does not arm the effect"
      );
      assert!(fill_rect(&output, BASE).is_none());
   }

   #[test]
   fn the_base_is_part_of_the_click_target() {
      let ctx = Context::default();
      let (_, warm) = run_pass(&ctx, vec![], 0.0, push_button);
      // Below the face's bottom edge, inside the exposed band.
      let point = Pos2::new(warm.rect.center().x, warm.rect.bottom() - 1.0);

      let mut time = f64::from(STEP);
      run_pass(&ctx, vec![press(point)], time, push_button);
      time += f64::from(STEP);
      let (_, released) = run_pass(&ctx, vec![release(point)], time, push_button);
      assert!(
         released.clicked(),
         "the base band must push the button"
      );
   }

   #[test]
   fn min_size_sizes_the_face_not_the_widget() {
      let ctx = Context::default();
      let (output, response) = run_pass(&ctx, vec![], 0.0, || {
         Button::new("Push")
            .min_size(vec2(120.0, 36.0))
            .push(6.0)
            .visuals(ButtonVisuals {
               bg: FACE,
               ..Default::default()
            })
      });

      let face = fill_rect(&output, FACE).expect("the face fill");
      assert_eq!(
         face.height(),
         36.0,
         "`min_size` is the face height"
      );
      assert_eq!(
         response.rect.height(),
         42.0,
         "the widget owns the band too"
      );
   }
}
