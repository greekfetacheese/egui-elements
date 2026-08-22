//! Installed look: palette, typography, frames, and per-widget visuals.
//!
//! Construct with [`Theme::new`], then [`Theme::install`] so custom widgets
//! can read visuals from [`egui::Context`]. [`Theme::current`] reads that
//! stored copy (or falls back to Tokyo Night).
//!
//! Built-in palettes live in [`crate::themes`]. Slot roles are documented
//! in the crate `docs/theme.md`.

use crate::overlay::OverlayManager;
use crate::themes::{
   mclaren_650gts_gt3, reverie, shade_sanctuary, tokyo_night, tokyo_night_light, wasp, wasp_light,
};
use crate::utils::*;
use crate::visuals::*;
use egui::{Color32, Context, Frame, Id, Style, Vec2};

#[cfg(feature = "elegance")]
use elegance::{Palette, Theme as EleganceTheme, Typography as EleganceTypography};

/// Temp-data key used to store the mirrored `egui-elegance` theme.
#[cfg(feature = "elegance")]
pub fn elegance_theme_key() -> Id {
   Id::new("elegance::theme")
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
   /// Tracks open windows so [`OverlayManager::paint_overlay`] can dim the canvas.
   #[cfg_attr(feature = "serde", serde(skip))]
   pub overlay_manager: OverlayManager,

   /// True if a tint is recomended to be applied to images
   /// to soften the contrast between the image and the background
   ///
   /// This is usually true for themes with very dark background
   pub image_tint_recommended: bool,

   /// Which built-in palette this theme was constructed from.
   pub kind: ThemeKind,
   /// Semantic color slots.
   pub colors: ThemeColors,
   /// Per-widget chrome (button, label, combo, text edit, frames).
   pub visuals: ThemeVisuals,
   /// Font sizes used by this crate's widgets and the demo.
   pub typography: Typography,

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
         && self.colors == other.colors
         && self.typography == other.typography
         && self.window_frame == other.window_frame
         && self.frame1 == other.frame1
         && self.frame2 == other.frame2
         && self.visuals == other.visuals
   }
}

impl Eq for Theme {}

impl Theme {
   /// Construct a built-in theme.
   pub fn new(kind: ThemeKind) -> Self {
      let theme = match kind {
         ThemeKind::TokyoNight => tokyo_night::theme(),
         ThemeKind::TokyoNightLight => tokyo_night_light::theme(),
         ThemeKind::McLaren650Gts => mclaren_650gts_gt3::theme(),
         ThemeKind::Reverie => reverie::theme(),
         ThemeKind::ShadeSanctuary => shade_sanctuary::theme(),
         ThemeKind::Wasp => wasp::theme(),
         ThemeKind::WaspLight => wasp_light::theme(),
      };

      theme
   }

   /// Stock [`egui::Style`] for this kind (widgets, selection, window fill).
   pub fn style(&self) -> Style {
      match self.kind {
         ThemeKind::TokyoNight => tokyo_night::style(),
         ThemeKind::TokyoNightLight => tokyo_night_light::style(),
         ThemeKind::McLaren650Gts => mclaren_650gts_gt3::style(),
         ThemeKind::Reverie => reverie::style(),
         ThemeKind::ShadeSanctuary => shade_sanctuary::style(),
         ThemeKind::Wasp => wasp::style(),
         ThemeKind::WaspLight => wasp_light::style(),
      }
   }

   /// Button visuals from [`ThemeVisuals`].
   pub fn button_visuals(&self) -> ButtonVisuals {
      self.visuals.button_visuals
   }

   /// Label visuals from [`ThemeVisuals`] (same shape as button visuals).
   pub fn label_visuals(&self) -> LabelVisuals {
      self.visuals.label_visuals
   }

   /// Combo-box visuals from [`ThemeVisuals`].
   pub fn combo_box_visuals(&self) -> ComboBoxVisuals {
      self.visuals.combo_box_visuals
   }

   /// Text-edit visuals from [`ThemeVisuals`].
   pub fn text_edit_visuals(&self) -> TextEditVisuals {
      self.visuals.text_edit_visuals
   }

   fn storage_id() -> Id {
      Id::new("elements::theme")
   }

   fn button_visuals_id() -> Id {
      Id::new("elements::button_visuals")
   }

   fn label_visuals_id() -> Id {
      Id::new("elements::label_visuals")
   }

   fn combo_box_visuals_id() -> Id {
      Id::new("elements::combo_box_visuals")
   }

   fn text_edit_visuals_id() -> Id {
      Id::new("elements::text_edit_visuals")
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

   /// Install this theme into the given egui context.
   ///
   /// Widgets then pick up their visuals from `ctx` automatically.
   pub fn install(&mut self, ctx: &Context) {
      #[cfg(feature = "elegance")]
      self.install_elegance_theme(ctx);

      let button_visuals = self.button_visuals();
      let label_visuals = self.label_visuals();
      let combo_box_visuals = self.combo_box_visuals();
      let text_edit_visuals = self.text_edit_visuals();
      let style = self.style();
      let theme = self.clone();

      ctx.set_global_style(style);
      ctx.data_mut(|d| d.insert_temp(Self::storage_id(), theme));

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
