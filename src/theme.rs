//! Installed look: palette, typography, frames, and per-widget visuals.
//!
//! Construct with [`Theme::new`], then [`Theme::install`] so custom widgets
//! can read visuals from [`egui::Context`]. [`Theme::current`] reads that
//! stored copy (or falls back to Tokyo Night).
//!
//! Built-in palettes live in [`crate::themes`]. Slot roles are documented
//! in the crate `docs/theme.md`.

use crate::gradient::GradientDir;
use crate::themes::{
   mclaren_650gts_gt3, reverie, shade_sanctuary, tokyo_night, tokyo_night_light, wasp, wasp_light,
};
use crate::utils::*;
use crate::visuals::*;
use egui::{
   Color32, Context, CornerRadius, Frame, Id, Margin, Shadow, Spacing, Stroke, Style, Vec2,
   Visuals,
   style::{Selection, WidgetVisuals, Widgets},
   vec2,
};

/// Scroll-bar width the theme pins on the global style, in points.
///
/// egui's default floating bar is 10 points wide; panes share one narrow bar so
/// a list does not change shape when it is long enough to scroll.
const SCROLL_BAR_WIDTH: f32 = 6.0;

/// Gap between the content and the scroll bar, in points.
const SCROLL_BAR_INNER_MARGIN: f32 = 2.0;

/// Gap outside the scroll bar, in points. Only allocates space for a solid bar,
/// but pinned here so a theme that turns floating off still gets the gutter.
const SCROLL_BAR_OUTER_MARGIN: f32 = 4.0;

#[cfg(feature = "serde")]
use crate::gradient::DEFAULT_GRADIENT_STRENGTH;

#[cfg(feature = "elegance")]
use elegance::{Palette, Theme as EleganceTheme, Typography as EleganceTypography};

/// Temp-data key used to store the mirrored `egui-elegance` theme.
#[cfg(feature = "elegance")]
pub fn elegance_theme_key() -> Id {
   Id::unique("elegance::theme")
}

/// Theme colors to convert [Theme] to [EleganceTheme]
#[cfg(feature = "elegance")]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct EleganceThemeKey {
   dark: bool,
   bg: Color32,
   widget_bg: Color32,
   border: Color32,
   text: Color32,
   text_muted: Color32,
   accent: Color32,
   info: Color32,
   success: Color32,
   error: Color32,
   warning: Color32,
}

#[cfg(feature = "elegance")]
impl EleganceThemeKey {
   fn from_theme(theme: &Theme) -> Self {
      let c = &theme.colors;
      Self {
         dark: theme.dark,
         bg: c.bg,
         widget_bg: c.widget_bg,
         border: c.border,
         text: c.text,
         text_muted: c.text_muted,
         accent: c.accent,
         info: c.info,
         success: c.success,
         error: c.error,
         warning: c.warning,
      }
   }
}

/// Built-in palette. See each variant for the source look.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThemeKind {
   /// Based on <https://github.com/tokyo-night/tokyo-night-vscode-theme>
   TokyoNight,
   /// Based on <https://github.com/tokyo-night/tokyo-night-vscode-theme> (light)
   TokyoNightLight,
   /// Based on the 2015 McLaren 650S GT3
   McLaren650Gts,
   /// Based on <https://github.com/santiyounger/Reverie-Obsidian-Theme>
   Reverie,
   /// Based on <https://github.com/Elevict/Shade-Sanctuary>
   ShadeSanctuary,
   /// Based on <https://github.com/santiyounger/Wasp-Obsidian-Theme> (dark)
   Wasp,
   /// Based on <https://github.com/santiyounger/Wasp-Obsidian-Theme> (light)
   WaspLight,
   /// A theme defined at runtime, identified by name.
   ///
   /// The name is only an identity: the definition is the [`Theme`] it is
   /// serialized with. A `Custom` kind cannot be rebuilt by name, so
   /// [`Theme::new`] falls back to Tokyo Night and [`Theme::try_new`]
   /// returns `None`.
   Custom(String),
}

impl ThemeKind {
   /// Human-readable name, used by [`crate::utils::theme_switcher`].
   pub fn to_str(&self) -> &str {
      match self {
         ThemeKind::TokyoNight => "Tokyo Night",
         ThemeKind::TokyoNightLight => "Tokyo Night Light",
         ThemeKind::McLaren650Gts => "McLaren 650GTS",
         ThemeKind::Reverie => "Reverie",
         ThemeKind::ShadeSanctuary => "Shade Sanctuary",
         ThemeKind::Wasp => "Wasp",
         ThemeKind::WaspLight => "Wasp Light",
         ThemeKind::Custom(name) => name,
      }
   }

   /// Every built-in kind, in switcher order.
   pub fn to_vec() -> Vec<Self> {
      vec![
         Self::TokyoNight,
         Self::TokyoNightLight,
         Self::McLaren650Gts,
         Self::Reverie,
         Self::ShadeSanctuary,
         Self::Wasp,
         Self::WaspLight,
      ]
   }
}

/// How the themed fills are painted.
///
/// Fill slots store a solid palette colour; the variant decides whether a widget paints that
/// colour flat or as a soft two-stop gradient of it. Switching variant is one field, not a new
/// visuals set: every widget reads the graded fill as it paints.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ThemeVariant {
   /// Every fill is the palette's flat colour.
   #[default]
   Solid,
   /// Every fill is a bevel of that colour, clamped so the label ink keeps AA contrast.
   Gradient,
}

impl ThemeVariant {
   /// Human-readable name, for the editor's selector.
   pub fn to_str(&self) -> &str {
      match self {
         ThemeVariant::Solid => "Solid",
         ThemeVariant::Gradient => "Gradient",
      }
   }

   /// Both variants, in selector order.
   pub fn to_vec() -> Vec<Self> {
      vec![Self::Solid, Self::Gradient]
   }
}

/// Complete look: colors, typography, frames, and widget visuals.
///
/// `Theme` is `Clone`. [`Theme::install`] takes `&mut self` (the elegance
/// path records a key on `self`). Clone before install if you need to keep
/// an unmodified copy:
///
/// ```no_run
/// # use egui::Context;
/// # use egui_elements::theme::{Theme, ThemeKind};
/// # let ctx = Context::default();
/// let mut theme = Theme::new(ThemeKind::TokyoNight);
/// theme.install(&ctx);
/// ```
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone)]
pub struct Theme {
   /// True if the theme is dark
   pub dark: bool,

   /// True if a tint is recomended to be applied to images
   /// to soften the contrast between the image and the background
   ///
   /// This is usually true for themes with very dark background
   pub image_tint_recommended: bool,

   /// Which built-in palette this theme was constructed from.
   pub kind: ThemeKind,

   /// How the themed fills are painted: the flat palette colour, or a gradient of it.
   #[cfg_attr(feature = "serde", serde(default))]
   pub variant: ThemeVariant,

   /// Depth of a gradient fill: the share of the way to white / black a ramp may travel.
   ///
   /// Each end is clamped further by contrast, so a tight palette gets a subtler ramp instead of
   /// an unreadable one. Ignored while [`Self::variant`] is [`ThemeVariant::Solid`].
   #[cfg_attr(feature = "serde", serde(default = "gradient_strength"))]
   pub gradient_strength: f32,

   /// Axis a gradient fill runs along: which edge is lifted toward white.
   ///
   /// Ignored while [`Self::variant`] is [`ThemeVariant::Solid`]. A fill that is already a
   /// gradient keeps its own axis.
   #[cfg_attr(feature = "serde", serde(default))]
   pub gradient_dir: GradientDir,

   /// Semantic color slots.
   pub colors: ThemeColors,
   /// Per-widget chrome (button, label, combo, text edit, frames).
   pub visuals: ThemeVisuals,
   /// Font sizes used by this crate's widgets and the demo.
   pub typography: Typography,

   /// Layout rhythm (gaps between siblings and groups).
   ///
   /// Widget chrome padding stays on [`Theme::inner_margin`] /
   /// [`Theme::button_padding`]. Apps should pick a step from this scale
   /// instead of one-off `4.0` / `12.0` / `16.0` literals.
   #[cfg_attr(feature = "serde", serde(default))]
   pub spacing: ThemeSpacing,

   /// Used for [Frame] not native windows
   pub window_frame: Frame,
   /// Base container frame for major UI sections.
   pub frame1: Frame,
   /// Frame for nested elements, like individual list items.
   pub frame2: Frame,

   /// Corner radius for widgets
   pub corner_radius: u8,

   /// Inner margin for widgets
   pub inner_margin: i8,

   /// Outer margin for widgets
   pub outer_margin: i8,

   /// Padding inside buttons
   ///
   /// Must use this from [egui::Ui::spacing_mut] to change the padding
   pub button_padding: Vec2,

   #[cfg(feature = "elegance")]
   #[cfg_attr(feature = "serde", serde(skip))]
   /// Snapshot used to skip reinstalling an unchanged elegance theme.
   pub elegance_key: Option<EleganceThemeKey>,
}

impl PartialEq for Theme {
   fn eq(&self, other: &Self) -> bool {
      self.dark == other.dark
         && self.kind == other.kind
         && self.variant == other.variant
         && self.gradient_strength == other.gradient_strength
         && self.gradient_dir == other.gradient_dir
         && self.colors == other.colors
         && self.typography == other.typography
         && self.spacing == other.spacing
         && self.window_frame == other.window_frame
         && self.frame1 == other.frame1
         && self.frame2 == other.frame2
         && self.visuals == other.visuals
   }
}

impl Eq for Theme {}

/// Serde default for [`Theme::gradient_strength`], so a theme saved before gradients loads flat.
#[cfg(feature = "serde")]
fn gradient_strength() -> f32 {
   DEFAULT_GRADIENT_STRENGTH
}

impl Theme {
   /// Construct a built-in theme.
   ///
   /// A [`ThemeKind::Custom`] carries only a name, never a palette, so this
   /// falls back to [`ThemeKind::TokyoNight`]. Use [`Theme::try_new`] when
   /// the caller must know that nothing could be rebuilt.
   pub fn new(kind: ThemeKind) -> Self {
      Self::try_new(kind).unwrap_or_else(tokyo_night::theme)
   }

   /// Like [`Theme::new`], but `None` for [`ThemeKind::Custom`].
   ///
   /// A custom theme's definition is the serialized [`Theme`] itself: load it
   /// with `serde_json::from_str::<Theme>`, or rebuild a built-in with
   /// [`Theme::new`] and tag it with [`Theme::rename`]. The kind that comes
   /// back from either path keeps the name across saves.
   pub fn try_new(kind: ThemeKind) -> Option<Self> {
      let theme = match kind {
         ThemeKind::TokyoNight => tokyo_night::theme(),
         ThemeKind::TokyoNightLight => tokyo_night_light::theme(),
         ThemeKind::McLaren650Gts => mclaren_650gts_gt3::theme(),
         ThemeKind::Reverie => reverie::theme(),
         ThemeKind::ShadeSanctuary => shade_sanctuary::theme(),
         ThemeKind::Wasp => wasp::theme(),
         ThemeKind::WaspLight => wasp_light::theme(),
         ThemeKind::Custom(_) => return None,
      };

      Some(theme)
   }

   /// Re-tag this theme as a runtime-defined [`ThemeKind::Custom`].
   ///
   /// Only the name changes; colors, typography, spacing, frames and visuals
   /// are left untouched. Persist the whole theme: the kind records the name,
   /// not the palette.
   pub fn rename(&mut self, name: impl Into<String>) {
      self.kind = ThemeKind::Custom(name.into());
   }

   /// This theme with its fills painted as `variant`.
   pub fn with_variant(mut self, variant: ThemeVariant) -> Self {
      self.variant = variant;
      self
   }

   /// This theme with gradient fills aimed along `dir`.
   pub fn with_gradient_dir(mut self, dir: GradientDir) -> Self {
      self.gradient_dir = dir;
      self
   }

   /// Switch how every themed fill is painted.
   ///
   /// The fills are stored solid and graded as they are read, so the next frame is enough — there
   /// is nothing to rebuild, and the editor keeps editing the same colours.
   pub fn set_variant(&mut self, variant: ThemeVariant) {
      self.variant = variant;
   }

   /// Stock [`egui::Style`] for this kind (widgets, selection, window fill).
   ///
   /// A built-in kind returns its palette module's hand-tuned style; a
   /// [`ThemeKind::Custom`] derives one from its own slots and frames.
   pub fn style(&self) -> Style {
      match &self.kind {
         ThemeKind::TokyoNight => tokyo_night::style(),
         ThemeKind::TokyoNightLight => tokyo_night_light::style(),
         ThemeKind::McLaren650Gts => mclaren_650gts_gt3::style(),
         ThemeKind::Reverie => reverie::style(),
         ThemeKind::ShadeSanctuary => shade_sanctuary::style(),
         ThemeKind::Wasp => wasp::style(),
         ThemeKind::WaspLight => wasp_light::style(),
         ThemeKind::Custom(_) => self.custom_style(),
      }
   }

   /// Stock [`egui::Style`] derived from this theme's own slots and frames.
   ///
   /// [`ThemeKind::Custom`] has no palette module to ask, so the stock egui
   /// visuals are rebuilt from `colors`, `dark`, `corner_radius` and the
   /// stored frames. Built-in kinds keep their hand-tuned [`Theme::style`].
   fn custom_style(&self) -> Style {
      let colors = &self.colors;
      let radius = CornerRadius::same(self.corner_radius);
      let base = WidgetVisuals {
         bg_fill: colors.widget_bg,
         weak_bg_fill: colors.bg,
         bg_stroke: Stroke::new(1.0, colors.border),
         corner_radius: radius,
         fg_stroke: Stroke::new(1.0, colors.text),
         expansion: 0.0,
      };

      // Stock sliders and switches read the inactive fill; without the
      // highlight slot they disappear into the canvas.
      let mut inactive = base.clone();
      inactive.bg_fill = colors.highlight;

      let widgets = Widgets {
         noninteractive: base.clone(),
         inactive,
         hovered: WidgetVisuals {
            bg_fill: colors.widget_bg,
            weak_bg_fill: colors.hover,
            bg_stroke: Stroke::new(1.0, colors.hover),
            ..base.clone()
         },
         active: WidgetVisuals {
            bg_fill: colors.widget_bg,
            weak_bg_fill: colors.widget_bg,
            bg_stroke: Stroke::new(1.0, colors.border),
            ..base.clone()
         },
         open: WidgetVisuals {
            bg_fill: colors.widget_bg,
            weak_bg_fill: colors.widget_bg,
            bg_stroke: Stroke::new(1.0, colors.border),
            ..base
         },
      };

      let visuals = Visuals {
         dark_mode: self.dark,
         override_text_color: Some(colors.text),
         widgets,
         selection: Selection {
            bg_fill: colors.highlight,
            stroke: Stroke::new(1.0, colors.accent),
         },
         hyperlink_color: colors.info,
         faint_bg_color: colors.bg,
         extreme_bg_color: colors.widget_bg,
         code_bg_color: colors.bg,
         warn_fg_color: colors.warning,
         error_fg_color: colors.error,
         window_corner_radius: radius,
         window_shadow: self.window_frame.shadow,
         window_fill: self.window_frame.fill,
         window_stroke: self.window_frame.stroke,
         panel_fill: colors.bg,
         ..Default::default()
      };

      Style {
         visuals,
         animation_time: 0.3,
         spacing: Spacing {
            window_margin: Margin::same(self.inner_margin),
            ..Default::default()
         },
         ..Default::default()
      }
   }

   /// Frame 1 visuals from [`ThemeVisuals`], graded when [`Self::variant`] is
   /// [`ThemeVariant::Gradient`].
   pub fn frame1_visuals(&self) -> FrameVisuals {
      let visuals = self.visuals.frame1_visuals;
      let ink = self.colors.text;

      match self.variant {
         ThemeVariant::Solid => visuals,
         ThemeVariant::Gradient => visuals.graded(ink, self.gradient_strength, self.gradient_dir),
      }
   }

   /// Frame 2 visuals from [`ThemeVisuals`], graded when [`Self::variant`] is
   /// [`ThemeVariant::Gradient`].
   pub fn frame2_visuals(&self) -> FrameVisuals {
      let visuals = self.visuals.frame2_visuals;
      let ink = self.colors.text;

      match self.variant {
         ThemeVariant::Solid => visuals,
         ThemeVariant::Gradient => visuals.graded(ink, self.gradient_strength, self.gradient_dir),
      }
   }

   /// The frame for a [`Surface`].
   pub fn surface_frame(&self, surface: Surface) -> Frame {
      match surface {
         Surface::Canvas => Frame::NONE.fill(self.colors.bg),
         Surface::Panel => self.frame1,
         Surface::Well => self.frame2,
         Surface::Overlay => self.window_frame,
      }
   }

   /// Fill colour of a [`Surface`].
   pub fn surface_fill(&self, surface: Surface) -> Color32 {
      self.surface_frame(surface).fill
   }

   /// Hover/click visuals when a surface is interactive.
   pub fn surface_visuals(&self, surface: Surface) -> Option<FrameVisuals> {
      match surface {
         Surface::Panel => Some(self.frame1_visuals()),
         Surface::Well => Some(self.frame2_visuals()),
         Surface::Canvas | Surface::Overlay => None,
      }
   }

   /// Fill for `surface` in `state`, on the themed button recipe.
   ///
   /// Hover and press pull the surface 16 % / 28 % toward [`ThemeColors::bg`] (the mixes
   /// [`ButtonVisuals::filled`] uses); selection is [`ThemeColors::highlight`]; disabled keeps
   /// the idle fill and only mutes the ink ([`Self::state_ink`]).
   pub fn state_fill(&self, surface: Surface, state: SurfaceState) -> Color32 {
      let fill = self.surface_fill(surface);
      match state {
         SurfaceState::Idle | SurfaceState::Disabled => fill,
         SurfaceState::Hover => mix(fill, self.colors.bg, 0.16),
         SurfaceState::Pressed => mix(fill, self.colors.bg, 0.28),
         SurfaceState::Selected => self.colors.highlight,
      }
   }

   /// Readable ink for [`Self::state_fill`] on `surface` in `state`.
   pub fn state_ink(&self, surface: Surface, state: SurfaceState) -> Color32 {
      match state {
         SurfaceState::Selected => self.colors.ink_on(self.colors.highlight),
         SurfaceState::Disabled => self.colors.text_muted,
         SurfaceState::Idle | SurfaceState::Hover | SurfaceState::Pressed => {
            self.colors.ink_on(self.surface_fill(surface))
         }
      }
   }

   /// Shadow for an [`Elevation`] step, scaled from the palette's window shadow.
   ///
   /// Raised is half the shadow, Floating is the window shadow itself, Overlay is double. The
   /// colour is untouched, so a light palette keeps its soft neutral shadow.
   pub fn shadow(&self, elevation: Elevation) -> Shadow {
      self.shadow_scaled(elevation, 1.0)
   }

   /// [`Self::shadow`] with a caller-supplied strength multiplier.
   ///
   /// `1.0` is the step's normal shadow, `0.0` flattens it, and values above `1.0` lift a card
   /// further than its step allows. The palette still owns the colour.
   pub fn shadow_scaled(&self, elevation: Elevation, strength: f32) -> Shadow {
      let base = self.style().visuals.window_shadow;
      let scale = elevation.scale() * strength.max(0.0);

      Shadow {
         offset: [
            (f32::from(base.offset[0]) * scale).round() as i8,
            (f32::from(base.offset[1]) * scale).round() as i8,
         ],
         blur: (f32::from(base.blur) * scale).round() as u8,
         spread: (f32::from(base.spread) * scale).round() as u8,
         color: base.color,
      }
   }

   /// Button visuals, graded when [`Self::variant`] is [`ThemeVariant::Gradient`].
   ///
   /// The fills are stored solid on [`ThemeVisuals`]; this is where the variant turns them into
   /// ramps, clamped against the label's own ink.
   pub fn button_visuals(&self) -> ButtonVisuals {
      let visuals = self.visuals.button_visuals;

      match self.variant {
         ThemeVariant::Solid => visuals,
         ThemeVariant::Gradient => visuals.graded(
            visuals.text,
            self.gradient_strength,
            self.gradient_dir,
         ),
      }
   }

   /// Label visuals (same shape as button visuals), graded the same way.
   pub fn label_visuals(&self) -> LabelVisuals {
      let visuals = self.visuals.label_visuals;

      match self.variant {
         ThemeVariant::Solid => visuals,
         ThemeVariant::Gradient => visuals.graded(
            visuals.text,
            self.gradient_strength,
            self.gradient_dir,
         ),
      }
   }

   /// Combo-box visuals, graded against [`ThemeColors::text`] (the combo carries no ink slot of
   /// its own).
   pub fn combo_box_visuals(&self) -> ComboBoxVisuals {
      let visuals = self.visuals.combo_box_visuals;
      let ink = self.colors.text;

      match self.variant {
         ThemeVariant::Solid => visuals,
         ThemeVariant::Gradient => visuals.graded(ink, self.gradient_strength, self.gradient_dir),
      }
   }

   /// Text-edit visuals, graded against the typed-text colour.
   pub fn text_edit_visuals(&self) -> TextEditVisuals {
      let visuals = self.visuals.text_edit_visuals;

      match self.variant {
         ThemeVariant::Solid => visuals,
         ThemeVariant::Gradient => visuals.graded(
            visuals.text,
            self.gradient_strength,
            self.gradient_dir,
         ),
      }
   }

   fn storage_id() -> Id {
      Id::unique("elements::theme")
   }

   fn button_visuals_id() -> Id {
      Id::unique("elements::button_visuals")
   }

   fn label_visuals_id() -> Id {
      Id::unique("elements::label_visuals")
   }

   fn combo_box_visuals_id() -> Id {
      Id::unique("elements::combo_box_visuals")
   }

   fn text_edit_visuals_id() -> Id {
      Id::unique("elements::text_edit_visuals")
   }

   fn theme_colors_id() -> Id {
      Id::unique("elements::theme_colors")
   }

   fn spacing_id() -> Id {
      Id::unique("elements::spacing")
   }

   /// Widget visuals stored by [`Theme::install`], if any.
   ///
   /// Widgets resolve in this order: `self.visuals` → context → [`egui::Style`].
   pub fn button_visuals_from_ctx(ctx: &Context) -> Option<ButtonVisuals> {
      ctx.data(|d| d.get_temp(Self::button_visuals_id()))
   }

   /// Same as [`Self::button_visuals_from_ctx`] for labels.
   pub fn label_visuals_from_ctx(ctx: &Context) -> Option<LabelVisuals> {
      ctx.data(|d| d.get_temp(Self::label_visuals_id()))
   }

   /// Same as [`Self::button_visuals_from_ctx`] for combo boxes.
   pub fn combo_box_visuals_from_ctx(ctx: &Context) -> Option<ComboBoxVisuals> {
      ctx.data(|d| d.get_temp(Self::combo_box_visuals_id()))
   }

   /// Same as [`Self::button_visuals_from_ctx`] for text edits.
   pub fn text_edit_visuals_from_ctx(ctx: &Context) -> Option<TextEditVisuals> {
      ctx.data(|d| d.get_temp(Self::text_edit_visuals_id()))
   }

   pub fn colors_from_ctx(ctx: &Context) -> ThemeColors {
      let c = ctx.data(|d| d.get_temp(Self::theme_colors_id()));
      c.unwrap_or_else(|| Theme::new(ThemeKind::TokyoNight).colors)
   }

   pub fn spacing_from_ctx(ctx: &Context) -> ThemeSpacing {
      let sp = ctx.data(|d| d.get_temp(Self::spacing_id()));
      sp.unwrap_or_default()
   }

   /// Install this theme into the given egui context.
   ///
   /// Widgets then pick up their visuals from `ctx` automatically.
   ///
   /// # Keep in mind
   ///
   /// If you don't call this on app creation or whenever you change the theme
   /// widgets will pick up `default/outdated` values
   pub fn install(&mut self, ctx: &Context) {
      #[cfg(feature = "elegance")]
      self.install_elegance_theme(ctx);

      let spacing = self.spacing;
      let colors = self.colors;
      let button_visuals = self.button_visuals();
      let label_visuals = self.label_visuals();
      let combo_box_visuals = self.combo_box_visuals();
      let text_edit_visuals = self.text_edit_visuals();
      let mut style = self.style();
      self.apply_spacing(&mut style);
      let theme = self.clone();

      ctx.set_global_style(style);
      ctx.data_mut(|d| d.insert_temp(Self::storage_id(), theme));

      ctx.data_mut(|d| d.insert_temp(Self::spacing_id(), spacing));
      ctx.data_mut(|d| d.insert_temp(Self::theme_colors_id(), colors));
      ctx.data_mut(|d| d.insert_temp(Self::button_visuals_id(), button_visuals));
      ctx.data_mut(|d| d.insert_temp(Self::label_visuals_id(), label_visuals));
      ctx.data_mut(|d| d.insert_temp(Self::combo_box_visuals_id(), combo_box_visuals));
      ctx.data_mut(|d| d.insert_temp(Self::text_edit_visuals_id(), text_edit_visuals));
   }

   #[cfg(feature = "elegance")]
   /// Convert the current theme into an elegance theme and install it
   ///
   /// If the elegance theme is already installed, this is no-op.
   pub fn install_elegance_theme(&mut self, ctx: &Context) {
      let key = EleganceThemeKey::from_theme(&self);
      if self.elegance_key == Some(key) {
         return;
      }

      let c = &self.colors;
      let mut pal = if key.dark {
         Palette::charcoal()
      } else {
         Palette::frost()
      };

      pal.is_dark = key.dark;
      pal.bg = c.bg;
      pal.card = c.widget_bg;
      pal.input_bg = c.widget_bg;
      pal.border = c.border;
      pal.text = c.text;
      pal.text_muted = c.text_muted;
      pal.text_faint = c.text_muted;
      pal.focus = c.accent;
      pal.blue = c.info;
      pal.green = c.success;
      pal.green_hover = c.success;
      pal.red = c.error;
      pal.red_hover = c.error;
      pal.amber = c.warning;
      pal.amber_hover = c.warning;
      pal.purple = c.accent;
      pal.purple_hover = c.accent;
      pal.success = c.success;
      pal.danger = c.error;
      pal.warning = c.warning;

      let elegance_typography = EleganceTypography {
         body: self.typography.normal,
         button: self.typography.normal,
         label: self.typography.normal,
         small: self.typography.small,
         heading: self.typography.heading,
         monospace: self.typography.small,
      };

      let elegance_theme = EleganceTheme {
         palette: pal,
         control_radius: self.corner_radius as f32,
         card_radius: self.corner_radius as f32,
         card_padding: self.frame1.inner_margin.top as f32,
         control_padding_y: self.button_padding.y as f32,
         control_padding_x: self.button_padding.x as f32,
         typography: elegance_typography,
      };

      ctx.data_mut(|d| d.insert_temp(elegance_theme_key(), elegance_theme));
      self.elegance_key = Some(key);
   }

   /// Read the current theme from the context
   /// if it exists, otherwise return the default theme
   pub fn current(ctx: &Context) -> Theme {
      ctx.data(|d| {
         d.get_temp::<Theme>(Self::storage_id())
            .unwrap_or_else(|| Theme::new(ThemeKind::TokyoNight))
      })
   }

   /// Push [`Theme::spacing`], button/window padding and scroll-bar geometry
   /// onto `style`.
   ///
   /// Called from [`Theme::install`]. `style()` from a `ThemeKind` only
   /// sets colors; layout tokens live on `Theme` so the editor can change
   /// them without rebuilding the palette. The scroll-bar geometry is global
   /// so every `egui::ScrollArea` in the app draws the same thin bar.
   fn apply_spacing(&self, style: &mut Style) {
      let s = &self.spacing;
      style.spacing.item_spacing = vec2(s.md, s.md);
      style.spacing.button_padding = self.button_padding;
      style.spacing.window_margin = Margin::same(self.inner_margin);

      style.spacing.scroll.bar_width = SCROLL_BAR_WIDTH;
      style.spacing.scroll.bar_inner_margin = SCROLL_BAR_INNER_MARGIN;
      style.spacing.scroll.bar_outer_margin = SCROLL_BAR_OUTER_MARGIN;
   }

   /// Keep derived frame colors in sync with a palette change.
   ///
   /// Only updates a color if it still matches the previous palette slot
   /// (e.g. `frame1.fill == old.widget_bg`). Custom colors and structural
   /// frame properties (margins, rounding, shadow offsets) are left alone.
   pub fn remap_derived_frames(&mut self, old: &ThemeColors) {
      let new = self.colors;
      if !frame_palette_changed(old, &new) {
         return;
      }

      remap_frame(
         &mut self.window_frame,
         old.title_bar,
         new.title_bar,
         old.border,
         new.border,
      );
      remap_frame(
         &mut self.frame1,
         old.widget_bg,
         new.widget_bg,
         old.border,
         new.border,
      );
      remap_frame(
         &mut self.frame2,
         old.bg,
         new.bg,
         old.border,
         new.border,
      );
      remap_frame_visuals(
         &mut self.visuals.frame1_visuals,
         old.hover,
         new.hover,
         old.widget_bg,
         new.widget_bg,
         old.highlight,
         new.highlight,
      );
      remap_frame_visuals(
         &mut self.visuals.frame2_visuals,
         old.hover,
         new.hover,
         old.bg,
         new.bg,
         old.highlight,
         new.highlight,
      );
   }
}

/// Theme visuals
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ThemeVisuals {
   /// Visuals for [`crate::widgets::Button`].
   pub button_visuals: ButtonVisuals,
   /// Visuals for [`crate::widgets::Label`].
   pub label_visuals: LabelVisuals,
   /// Visuals for [`crate::widgets::ComboBox`].
   pub combo_box_visuals: ComboBoxVisuals,
   /// Visuals for [`crate::widgets::SecureTextEdit`].
   pub text_edit_visuals: TextEditVisuals,
   /// Hover/click fills for [`Theme::frame1`].
   pub frame1_visuals: FrameVisuals,
   /// Hover/click fills for [`Theme::frame2`].
   pub frame2_visuals: FrameVisuals,
}

/// Named layer in the surface ladder.
///
/// Frames carry fill and chrome; this names the role, so nesting can be described without
/// remembering which of `frame1` / `frame2` goes on which.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Surface {
   /// Page background ([`ThemeColors::bg`]).
   #[default]
   Canvas,
   /// Major card on the canvas ([`Theme::frame1`]).
   Panel,
   /// Recessed area inside a panel ([`Theme::frame2`]).
   Well,
   /// Window or modal surface ([`Theme::window_frame`]).
   Overlay,
}

impl Surface {
   /// The surface a card placed on this one should use.
   ///
   /// Alternates panel ↔ well, so a card stays distinguishable from the surface under it.
   pub fn card(self) -> Surface {
      match self {
         Surface::Canvas | Surface::Well | Surface::Overlay => Surface::Panel,
         Surface::Panel => Surface::Well,
      }
   }
}

/// Interaction state of a hand-rolled surface.
///
/// Use with [`Theme::state_fill`] / [`Theme::state_ink`] so a row, plate or icon button that
/// paints itself still follows the themed button recipe.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SurfaceState {
   /// Nothing interacting with it.
   #[default]
   Idle,
   /// Pointer over it.
   Hover,
   /// Pointer held on it.
   Pressed,
   /// Chosen / active.
   Selected,
   /// Not interactive, but still shown.
   Disabled,
}

/// Shadow weight for an elevated surface.
///
/// Each step scales the palette's `egui::Visuals::window_shadow`, so the colour stays the
/// theme's (a light palette's soft shadow does not become a black drop shadow) and only the
/// spread grows with the layer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Elevation {
   /// Resting card: half the window shadow.
   Raised,
   /// Floating layer: the window shadow itself.
   #[default]
   Floating,
   /// Modal or popover: twice the window shadow.
   Overlay,
}

impl Elevation {
   fn scale(self) -> f32 {
      match self {
         Self::Raised => 0.5,
         Self::Floating => 1.0,
         Self::Overlay => 2.0,
      }
   }
}

/// This is the color palette of the theme
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ThemeColors {
   /// The color for the title bar of the app (if using custom window frame)
   pub title_bar: Color32,

   /// Main BG color of the theme
   pub bg: Color32,

   /// Widget BG color
   ///
   /// This is the color of the widget backgrounds
   pub widget_bg: Color32,

   /// The color to use when hovering over a widget
   pub hover: Color32,

   /// Main text color
   pub text: Color32,

   /// Muted text color
   ///
   /// For example a hint inside a text field
   pub text_muted: Color32,

   /// Selected / emphasis fill. Distinct from [`Self::hover`].
   pub highlight: Color32,

   /// Border color
   pub border: Color32,

   /// Accent color
   pub accent: Color32,

   /// Error color
   ///
   /// Can be used to indicate something bad or to highlight a dangerous action
   pub error: Color32,

   /// Warning color
   pub warning: Color32,

   /// Success color
   ///
   /// Can be used to indicate something good or to highlight a successful action
   pub success: Color32,

   /// Info color
   ///
   /// Can be used for hyperlinks or to highlight something important
   pub info: Color32,
}

impl ThemeColors {
   /// Share of the surface colour in a tinted fill.
   const FILL_MIX: f32 = 0.86;
   /// Share of [`Self::text`] in tinted ink.
   const INK_MIX: f32 = 0.30;

   /// Tinted fill and readable ink for `role` on the canvas.
   ///
   /// `fill` is [`Self::bg`] pulled most of the way toward `role`; `ink` is `role` pulled a
   /// little of the way toward [`Self::text`], so the chip keeps the role's hue instead of
   /// reading as plain text. If that ink misses [`CONTRAST_AA_TEXT`] on the fill, it falls back
   /// to [`Self::ink_on`], which picks whichever of [`Self::text`] / [`Self::bg`] reads best.
   pub fn tint(&self, role: Color32) -> (Color32, Color32) {
      let fill = mix(role, self.bg, Self::FILL_MIX);
      let ink = mix(role, self.text, Self::INK_MIX);
      let ink = if contrast_ratio(ink, fill) >= CONTRAST_AA_TEXT {
         ink
      } else {
         self.ink_on(fill)
      };
      (fill, ink)
   }

   /// [`Self::widget_bg`] pulled `strength` toward `role` (0 = the widget, 1 = the role).
   ///
   /// The recipe a chat bubble, a selected row or a badge wash uses; naming it keeps the
   /// strength beside the decision instead of inlined at a paint site.
   pub fn wash(&self, role: Color32, strength: f32) -> Color32 {
      mix(self.widget_bg, role, strength.clamp(0.0, 1.0))
   }

   /// [`Self::text`] or [`Self::bg`], whichever has more contrast against `fill`.
   ///
   /// The ink rule for every solid chromatic fill: [`Self::text`] is a surface ink and does not
   /// read on `accent`, `error`, `warning`, `success` or `info` in any built-in palette
   /// (Tokyo Night's `text` on `accent` is 1.06:1). [`Self::audit`] checks the result against
   /// each of those fills. Alpha is ignored; pass an opaque fill.
   pub fn ink_on(&self, fill: Color32) -> Color32 {
      if contrast_ratio(self.text, fill) >= contrast_ratio(self.bg, fill) {
         self.text
      } else {
         self.bg
      }
   }

   /// Every colour slot, in struct order, with the field name attached.
   ///
   /// The name is the field's, so an audit panel or a usage map can label a swatch without
   /// hard-coding the slot list.
   pub fn slots(&self) -> [ColorSlot; 13] {
      [
         ColorSlot {
            name: "title_bar",
            color: self.title_bar,
         },
         ColorSlot {
            name: "bg",
            color: self.bg,
         },
         ColorSlot {
            name: "widget_bg",
            color: self.widget_bg,
         },
         ColorSlot {
            name: "hover",
            color: self.hover,
         },
         ColorSlot {
            name: "text",
            color: self.text,
         },
         ColorSlot {
            name: "text_muted",
            color: self.text_muted,
         },
         ColorSlot {
            name: "highlight",
            color: self.highlight,
         },
         ColorSlot {
            name: "border",
            color: self.border,
         },
         ColorSlot {
            name: "accent",
            color: self.accent,
         },
         ColorSlot {
            name: "error",
            color: self.error,
         },
         ColorSlot {
            name: "warning",
            color: self.warning,
         },
         ColorSlot {
            name: "success",
            color: self.success,
         },
         ColorSlot {
            name: "info",
            color: self.info,
         },
      ]
   }

   /// Every text/surface pairing worth checking, in a stable order.
   ///
   /// Covers body and muted text on the canvas, widget, title-bar and hover surfaces, text on the
   /// selection fill; the five semantic roles used as text; the same five used as fills against
   /// [`Self::ink_on`]; and the tinted chip ink against the fill [`Self::tint`] returns. Each
   /// entry carries the measured ratio and the threshold it needs, so callers can filter failures
   /// or render the whole report.
   pub fn audit(&self) -> Vec<ContrastCheck> {
      self.audit_with(std::iter::empty())
   }

   /// [`Self::audit`] plus application-specific pairs.
   ///
   /// A derived fill (a bubble wash, a wallpaper plate) sits outside the palette's fixed pairs;
   /// append its checks so one report covers the installed theme and the app.
   pub fn audit_with(&self, extra: impl IntoIterator<Item = ContrastCheck>) -> Vec<ContrastCheck> {
      let check = ContrastCheck::text;

      let mut checks = vec![
         check("text on bg", self.text, self.bg),
         check("text on widget_bg", self.text, self.widget_bg),
         check("text on title_bar", self.text, self.title_bar),
         check("text on hover", self.text, self.hover),
         check("text on highlight", self.text, self.highlight),
         check("muted text on bg", self.text_muted, self.bg),
         check(
            "muted text on widget_bg",
            self.text_muted,
            self.widget_bg,
         ),
         check(
            "muted text on title_bar",
            self.text_muted,
            self.title_bar,
         ),
         check("accent on bg", self.accent, self.bg),
         check("accent on widget_bg", self.accent, self.widget_bg),
         check("info on bg", self.info, self.bg),
         check("info on widget_bg", self.info, self.widget_bg),
         check("success on bg", self.success, self.bg),
         check(
            "success on widget_bg",
            self.success,
            self.widget_bg,
         ),
         check("warning on bg", self.warning, self.bg),
         check(
            "warning on widget_bg",
            self.warning,
            self.widget_bg,
         ),
         check("error on bg", self.error, self.bg),
         check("error on widget_bg", self.error, self.widget_bg),
         self.ink_check("ink on accent", self.accent),
         self.ink_check("ink on error", self.error),
         self.ink_check("ink on warning", self.warning),
         self.ink_check("ink on success", self.success),
         self.ink_check("ink on info", self.info),
         self.tinted_check("tinted accent", self.accent),
         self.tinted_check("tinted info", self.info),
         self.tinted_check("tinted success", self.success),
         self.tinted_check("tinted warning", self.warning),
         self.tinted_check("tinted error", self.error),
      ];

      checks.extend(extra);
      checks
   }

   fn ink_check(&self, label: &'static str, fill: Color32) -> ContrastCheck {
      let foreground = self.ink_on(fill);
      ContrastCheck::text(label, foreground, fill)
   }

   fn tinted_check(&self, label: &'static str, role: Color32) -> ContrastCheck {
      let (background, foreground) = self.tint(role);
      ContrastCheck::text(label, foreground, background)
   }
}

/// One named colour slot of [`ThemeColors`].
///
/// Returned by [`ThemeColors::slots`]; the name is the struct field, so a palette audit can
/// label swatches without hard-coding the field list.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorSlot {
   /// Field name, e.g. `"bg"`.
   pub name: &'static str,
   /// The slot colour.
   pub color: Color32,
}

/// One foreground/background pair from [`ThemeColors::audit`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContrastCheck {
   /// Pair name, e.g. `"text on bg"`.
   pub label: &'static str,
   /// Text or icon colour.
   pub foreground: Color32,
   /// Surface behind it.
   pub background: Color32,
   /// WCAG 2.2 ratio, from [`crate::utils::contrast_ratio`].
   pub ratio: f32,
   /// Minimum ratio this pair needs.
   pub required: f32,
}

impl ContrastCheck {
   /// A pair measured against `required`.
   pub fn new(
      label: &'static str,
      foreground: Color32,
      background: Color32,
      required: f32,
   ) -> Self {
      Self {
         label,
         foreground,
         background,
         ratio: contrast_ratio(foreground, background),
         required,
      }
   }

   /// A pair measured against [`CONTRAST_AA_TEXT`].
   pub fn text(label: &'static str, foreground: Color32, background: Color32) -> Self {
      Self::new(label, foreground, background, CONTRAST_AA_TEXT)
   }

   /// `true` when the pair meets its threshold.
   pub fn passes(&self) -> bool {
      self.ratio >= self.required
   }
}

/// A palette role a [`DerivedColor`] mixes from.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorRole {
   /// [`ThemeColors::accent`].
   Accent,
   /// [`ThemeColors::info`].
   Info,
   /// [`ThemeColors::success`].
   Success,
   /// [`ThemeColors::warning`].
   Warning,
   /// [`ThemeColors::error`].
   Error,
   /// [`ThemeColors::highlight`].
   Highlight,
   /// [`ThemeColors::hover`].
   Hover,
}

impl ColorRole {
   /// The palette colour this role names.
   pub fn color(self, colors: ThemeColors) -> Color32 {
      match self {
         Self::Accent => colors.accent,
         Self::Info => colors.info,
         Self::Success => colors.success,
         Self::Warning => colors.warning,
         Self::Error => colors.error,
         Self::Highlight => colors.highlight,
         Self::Hover => colors.hover,
      }
   }
}

/// A named application colour derived from a palette role.
///
/// [`Self::fill`] is [`ThemeColors::widget_bg`] pulled `strength` toward the role — the
/// Telegram demo's outgoing bubble was `mix(widget_bg, accent, 0.32)` inline. Naming it keeps
/// the decision and its strength together, lets several widgets share one wash, and gives the
/// contrast audit a label: [`Self::check`] is a [`ContrastCheck`] named after the colour, ready
/// for [`ThemeColors::audit_with`].
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DerivedColor {
   /// Stable name, used as the contrast-check label.
   pub name: &'static str,
   /// Palette role the fill is mixed toward.
   pub role: ColorRole,
   /// Share of the role in the fill, `0.0..=1.0`.
   pub strength: f32,
}

impl DerivedColor {
   /// An application colour with a name, a role and a wash strength.
   pub const fn new(name: &'static str, role: ColorRole, strength: f32) -> Self {
      Self {
         name,
         role,
         strength,
      }
   }

   /// [`ThemeColors::widget_bg`] pulled `strength` toward the role.
   pub fn fill(self, colors: ThemeColors) -> Color32 {
      colors.wash(self.role.color(colors), self.strength)
   }

   /// Readable ink on [`Self::fill`].
   pub fn ink(self, colors: ThemeColors) -> Color32 {
      colors.ink_on(self.fill(colors))
   }

   /// Contrast pair named after this colour, for [`ThemeColors::audit_with`].
   pub fn check(self, colors: ThemeColors) -> ContrastCheck {
      ContrastCheck::text(self.name, self.ink(colors), self.fill(colors))
   }
}

/// Layout rhythm in points. Jobs, not one-off gaps.
///
/// | Token | Default | Job |
/// |-------|---------|-----|
/// | `xs` | 4 | Tight meta, icon+label, stacked lines in a row |
/// | `sm` | 8 | Dense lists, control padding y, related captions |
/// | `md` | 12 | Default [`egui::Spacing::item_spacing`], related group |
/// | `lg` | 16 | Section padding (header, pane inner) |
/// | `xl` | 24 | Between groups (heading → gallery) |
///
/// Widget chrome (`inner_margin`, `button_padding`) is separate.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ThemeSpacing {
   /// Tight: stacked meta, icon+label.
   pub xs: f32,
   /// Dense lists and compact control padding.
   pub sm: f32,
   /// Default item spacing / related group.
   pub md: f32,
   /// Section padding.
   pub lg: f32,
   /// Space between groups.
   pub xl: f32,
}

impl Default for ThemeSpacing {
   fn default() -> Self {
      Self {
         xs: 4.0,
         sm: 8.0,
         md: 12.0,
         lg: 16.0,
         xl: 24.0,
      }
   }
}

impl ThemeSpacing {
   /// Uniform margin from one step (e.g. `space.margin(space.lg)`).
   pub fn margin(self, size: f32) -> Margin {
      Margin::same(size as i8)
   }

   /// Independent x/y margin (e.g. header `lg` × `md`).
   pub fn margin_xy(self, x: f32, y: f32) -> Margin {
      Margin::symmetric(x as i8, y as i8)
   }
}

/// Font sizes in points.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Typography {
   /// Tiny captions.
   pub very_small: f32,
   /// Secondary / hint text.
   pub small: f32,
   /// Body and button text.
   pub normal: f32,
   /// Emphasized body.
   pub large: f32,
   /// Section titles below a heading.
   pub very_large: f32,
   /// Page / modal headings.
   pub heading: f32,
}

#[cfg(test)]
mod tests {
   use super::*;
   use crate::gradient::Fill;

   #[test]
   fn install_pins_the_scroll_bar_geometry() {
      let ctx = Context::default();
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&ctx);

      let scroll = ctx.global_style().spacing.scroll;
      assert!((scroll.bar_width - SCROLL_BAR_WIDTH).abs() < f32::EPSILON);
      assert!((scroll.bar_inner_margin - SCROLL_BAR_INNER_MARGIN).abs() < f32::EPSILON);
      assert!((scroll.bar_outer_margin - SCROLL_BAR_OUTER_MARGIN).abs() < f32::EPSILON);
   }

   /// Every themed fill the variant grades, with the ink it carries.
   fn graded_fills(theme: &Theme) -> Vec<(&'static str, Fill, Color32)> {
      let button = theme.button_visuals();
      let label = theme.label_visuals();
      let combo = theme.combo_box_visuals();
      let edit = theme.text_edit_visuals();
      let frame1 = theme.frame1_visuals();
      let frame2 = theme.frame2_visuals();
      let frames_ink = theme.colors.text;

      vec![
         ("button bg", button.bg, button.text),
         ("button hover", button.bg_hover, button.text),
         ("button click", button.bg_click, button.text),
         ("button selected", button.bg_selected, button.text),
         ("label hover", label.bg_hover, label.text),
         ("label selected", label.bg_selected, label.text),
         ("combo bg", combo.bg, frames_ink),
         ("combo hover", combo.bg_hover, frames_ink),
         ("combo open", combo.bg_open, frames_ink),
         ("text edit bg", edit.bg, edit.text),
         ("frame1 bg", frame1.bg, frames_ink),
         ("frame1 hover", frame1.bg_hover, frames_ink),
         ("frame2 bg", frame2.bg, frames_ink),
         ("frame2 hover", frame2.bg_hover, frames_ink),
      ]
   }

   #[test]
   fn gradient_fills_keep_their_ink_readable_on_every_builtin_theme() {
      for kind in ThemeKind::to_vec() {
         let name = kind.to_str().to_owned();
         let theme = Theme::new(kind).with_variant(ThemeVariant::Gradient);

         for (slot, fill, ink) in graded_fills(&theme) {
            let worst = fill.min_contrast(ink);

            assert!(
               worst >= CONTRAST_AA_TEXT,
               "{name}: {slot} fell to {worst:.2}:1 on {fill:?}"
            );
         }
      }
   }

   #[test]
   fn the_gradient_direction_reaches_every_graded_fill() {
      let theme = Theme::new(ThemeKind::TokyoNight)
         .with_variant(ThemeVariant::Gradient)
         .with_gradient_dir(GradientDir::LeftToRight);

      for (slot, fill, _ink) in graded_fills(&theme) {
         let Fill::Gradient(gradient) = fill else {
            // A fill with no headroom stays solid; the direction cannot apply to it.
            continue;
         };

         assert_eq!(gradient.dir, GradientDir::LeftToRight, "{slot}");
      }
   }

   #[test]
   fn the_gradient_direction_defaults_to_top_down() {
      let theme = Theme::new(ThemeKind::TokyoNight).with_variant(ThemeVariant::Gradient);

      assert_eq!(theme.gradient_dir, GradientDir::TopDown);

      let Fill::Gradient(gradient) = theme.button_visuals().bg else {
         panic!("the idle fill must be a ramp");
      };

      assert_eq!(gradient.dir, GradientDir::TopDown);
   }

   #[test]
   fn the_gradient_variant_ramps_the_fills_it_can() {
      let theme = Theme::new(ThemeKind::TokyoNight).with_variant(ThemeVariant::Gradient);
      let button = theme.button_visuals();

      assert!(
         matches!(button.bg, Fill::Gradient(gradient) if !gradient.is_flat()),
         "the idle fill must be a ramp: {:?}",
         button.bg
      );
   }

   #[test]
   fn the_solid_variant_leaves_the_palette_fills_flat() {
      let theme = Theme::new(ThemeKind::TokyoNight);
      let button = theme.button_visuals();

      assert_eq!(button.bg, theme.visuals.button_visuals.bg);
      assert_eq!(button.bg, Fill::solid(theme.colors.widget_bg));
   }

   #[test]
   fn switching_variant_returns_the_same_stored_fill() {
      let mut theme = Theme::new(ThemeKind::Wasp);
      let solid = theme.button_visuals();

      theme.set_variant(ThemeVariant::Gradient);
      let _graded = theme.button_visuals();

      theme.set_variant(ThemeVariant::Solid);
      assert_eq!(theme.button_visuals(), solid);
   }

   #[test]
   fn a_tight_fill_stays_flat_under_the_gradient_variant() {
      let colors = Theme::new(ThemeKind::TokyoNight).colors;
      let tight = colors.text;

      // Text on its own colour has no headroom at all, so the bevel must collapse to a solid.
      assert_eq!(Fill::bevel(tight, tight, 0.5), Fill::Solid(tight));
   }

   #[test]
   fn tinted_ink_is_readable_on_every_builtin_theme() {
      for kind in ThemeKind::to_vec() {
         let name = kind.to_str().to_owned();
         let colors = Theme::new(kind).colors;

         for role in [
            colors.accent,
            colors.info,
            colors.success,
            colors.warning,
            colors.error,
         ] {
            let (fill, ink) = colors.tint(role);
            let ratio = contrast_ratio(fill, ink);
            assert!(
               ratio >= CONTRAST_AA_TEXT,
               "{name} role {role:?}: {ratio:.2} on {fill:?}"
            );
         }
      }
   }

   #[test]
   fn audit_flags_a_broken_palette() {
      let colors = Theme::new(ThemeKind::TokyoNight).colors;
      assert_eq!(colors.audit().len(), 28);

      let mut broken = colors;
      broken.text = broken.bg;
      let failures = broken.audit().into_iter().filter(|check| !check.passes()).count();
      assert!(failures >= 4, "only {failures} pairs failed");
   }

   #[test]
   fn chromatic_fills_carry_readable_ink_on_every_builtin_theme() {
      for kind in ThemeKind::to_vec() {
         let name = kind.to_str().to_owned();
         let colors = Theme::new(kind).colors;

         for fill in [
            colors.accent,
            colors.error,
            colors.warning,
            colors.success,
            colors.info,
         ] {
            let ink = colors.ink_on(fill);
            let ratio = contrast_ratio(ink, fill);
            assert!(
               ratio >= CONTRAST_AA_TEXT,
               "{name}: ink {ink:?} on fill {fill:?} is {ratio:.2}:1"
            );
         }
      }
   }

   #[test]
   fn the_surface_ladder_matches_the_theme_frames() {
      let theme = Theme::new(ThemeKind::TokyoNight);

      assert_eq!(
         theme.surface_fill(Surface::Canvas),
         theme.colors.bg
      );
      assert_eq!(
         theme.surface_fill(Surface::Panel),
         theme.frame1.fill
      );
      assert_eq!(
         theme.surface_fill(Surface::Well),
         theme.frame2.fill
      );
      assert_eq!(
         theme.surface_fill(Surface::Overlay),
         theme.window_frame.fill
      );

      assert_eq!(Surface::Canvas.card(), Surface::Panel);
      assert_eq!(Surface::Panel.card(), Surface::Well);
      assert_eq!(Surface::Well.card(), Surface::Panel);
      assert_eq!(Surface::Overlay.card(), Surface::Panel);
   }

   #[test]
   fn ink_on_picks_the_higher_contrast() {
      let colors = Theme::new(ThemeKind::TokyoNight).colors;
      assert_eq!(
         colors.ink_on(Color32::from_rgb(90, 90, 90)),
         colors.text
      );
      assert_eq!(colors.ink_on(colors.text), colors.bg);
   }

   #[test]
   fn every_slot_round_trips() {
      let colors = Theme::new(ThemeKind::TokyoNight).colors;
      let slots = colors.slots();
      assert_eq!(slots[0].name, "title_bar");
      assert_eq!(slots[1].color, colors.bg);
      let mut names: Vec<&str> = slots.iter().map(|slot| slot.name).collect();
      let total = names.len();
      names.sort_unstable();
      names.dedup();
      assert_eq!(names.len(), total);
   }

   #[test]
   fn extra_pairs_join_the_audit() {
      let colors = Theme::new(ThemeKind::TokyoNight).colors;
      let extra = ContrastCheck::new("app pair", colors.text, colors.widget_bg, 3.0);
      let checks = colors.audit_with([extra]);
      assert_eq!(checks.len(), colors.audit().len() + 1);
      assert_eq!(checks.last().copied(), Some(extra));
   }

   #[test]
   fn surface_states_follow_the_button_recipe() {
      let theme = Theme::new(ThemeKind::TokyoNight);
      let idle = theme.state_fill(Surface::Panel, SurfaceState::Idle);
      assert_eq!(idle, theme.surface_fill(Surface::Panel));
      assert_eq!(
         theme.state_fill(Surface::Panel, SurfaceState::Selected),
         theme.colors.highlight
      );
      assert_ne!(
         theme.state_fill(Surface::Panel, SurfaceState::Hover),
         idle
      );
      assert_ne!(
         theme.state_fill(Surface::Panel, SurfaceState::Pressed),
         theme.state_fill(Surface::Panel, SurfaceState::Hover)
      );
      assert_eq!(
         theme.state_ink(Surface::Panel, SurfaceState::Selected),
         theme.colors.ink_on(theme.colors.highlight)
      );
      assert_eq!(
         theme.state_ink(Surface::Panel, SurfaceState::Disabled),
         theme.colors.text_muted
      );
      let ink = theme.state_ink(Surface::Well, SurfaceState::Idle);
      let fill = theme.state_fill(Surface::Well, SurfaceState::Idle);
      assert!(contrast_ratio(ink, fill) >= CONTRAST_AA_TEXT);
   }

   #[test]
   fn elevation_scales_the_window_shadow() {
      let theme = Theme::new(ThemeKind::TokyoNight);
      let raised = theme.shadow(Elevation::Raised);
      let floating = theme.shadow(Elevation::Floating);
      let overlay = theme.shadow(Elevation::Overlay);

      assert_eq!(floating, theme.style().visuals.window_shadow);
      assert!(
         raised.blur < floating.blur,
         "raised must sit below floating"
      );
      assert!(
         floating.blur < overlay.blur,
         "overlay must sit above floating"
      );
      assert_eq!(
         raised.color, overlay.color,
         "the palette keeps the shadow colour"
      );
   }

   #[test]
   fn a_derived_colour_washes_widget_bg() {
      let colors = Theme::new(ThemeKind::TokyoNight).colors;
      let bubble = DerivedColor::new("bubble out", ColorRole::Accent, 0.32);

      assert_eq!(
         bubble.fill(colors),
         mix(colors.widget_bg, colors.accent, 0.32)
      );
      assert_eq!(
         DerivedColor::new("plain", ColorRole::Accent, 0.0).fill(colors),
         colors.widget_bg
      );
      assert_eq!(
         DerivedColor::new("full", ColorRole::Accent, 1.0).fill(colors),
         colors.accent
      );

      let check = bubble.check(colors);
      assert_eq!(check.label, "bubble out");
      assert!(
         check.passes(),
         "ink on the wash must read: {check:?}"
      );
   }

   #[test]
   fn a_custom_kind_names_itself_and_has_no_palette() {
      let kind = ThemeKind::Custom("Sunset".into());
      assert_eq!(kind.to_str(), "Sunset");

      assert!(
         Theme::try_new(kind.clone()).is_none(),
         "a name alone cannot rebuild a theme"
      );
      assert_eq!(
         Theme::new(kind).kind,
         ThemeKind::TokyoNight,
         "Theme::new falls back to Tokyo Night"
      );
   }

   #[test]
   fn a_custom_theme_derives_its_own_stock_style() {
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.colors.accent = Color32::from_rgb(12, 34, 56);
      theme.rename("Sunset");

      let style = theme.style();
      assert_eq!(style.visuals.panel_fill, theme.colors.bg);
      assert_eq!(
         style.visuals.selection.bg_fill,
         theme.colors.highlight
      );
      assert_eq!(style.visuals.dark_mode, theme.dark);
      assert_eq!(
         style.visuals.widgets.noninteractive.bg_stroke.color,
         theme.colors.border
      );
      assert_eq!(
         style.visuals.widgets.inactive.bg_fill,
         theme.colors.highlight
      );
   }

   #[cfg(feature = "serde")]
   #[test]
   fn a_custom_kind_round_trips_through_json() -> Result<(), serde_json::Error> {
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.colors.accent = Color32::from_rgb(9, 8, 7);
      theme.rename("Sunset");

      let json = serde_json::to_string(&theme)?;
      let restored: Theme = serde_json::from_str(&json)?;

      assert_eq!(restored.kind, ThemeKind::Custom("Sunset".into()));
      assert_eq!(restored.colors, theme.colors);
      assert_eq!(
         restored.style().visuals.panel_fill,
         restored.colors.bg
      );

      // Built-in kinds keep their wire format; a custom kind is tagged.
      assert_eq!(
         serde_json::from_str::<ThemeKind>("\"Wasp\"")?,
         ThemeKind::Wasp
      );
      assert_eq!(
         serde_json::from_str::<ThemeKind>(r#"{"Custom":"Sunset"}"#)?,
         ThemeKind::Custom("Sunset".into())
      );
      Ok(())
   }
}
