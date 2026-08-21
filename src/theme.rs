use crate::overlay::OverlayManager;
use crate::themes::{mclaren_650gts_gt3, reverie, tokyo_night};
use crate::utils::*;
use crate::visuals::*;
use egui::{Color32, Context, Frame, Id, Style, Vec2};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThemeKind {
    /// Based on https://github.com/tokyo-night/tokyo-night-vscode-theme
    TokyoNight,
    /// Based on the 2015 McLaren 650S GT3
    McLaren650GtsGt3,
    /// Based on https://github.com/santiyounger/Reverie-Obsidian-Theme
    Reverie,
}

impl ThemeKind {
    pub fn to_str(&self) -> &str {
        match self {
            ThemeKind::TokyoNight => "Tokyo Night",
            ThemeKind::McLaren650GtsGt3 => "McLaren 650GTS GT3",
            ThemeKind::Reverie => "Reverie",
        }
    }

    pub fn to_vec() -> Vec<Self> {
        vec![Self::TokyoNight, Self::McLaren650GtsGt3, Self::Reverie]
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct Theme {
    /// True if the theme is dark
    pub dark: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub overlay_manager: OverlayManager,

    /// True if a tint is recomended to be applied to images
    /// to soften the contrast between the image and the background
    ///
    /// This is usually true for themes with very dark background
    pub image_tint_recommended: bool,

    pub kind: ThemeKind,
    pub colors: ThemeColors,
    pub visuals: ThemeVisuals,
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
    pub fn new(kind: ThemeKind) -> Self {
        let theme = match kind {
            ThemeKind::TokyoNight => tokyo_night::theme(),
            ThemeKind::McLaren650GtsGt3 => mclaren_650gts_gt3::theme(),
            ThemeKind::Reverie => reverie::theme(),
        };

        theme
    }

    pub fn style(&self) -> Style {
        match self.kind {
            ThemeKind::TokyoNight => tokyo_night::style(),
            ThemeKind::McLaren650GtsGt3 => mclaren_650gts_gt3::style(),
            ThemeKind::Reverie => reverie::style(),
        }
    }

    pub fn button_visuals(&self) -> ButtonVisuals {
        self.visuals.button_visuals
    }

    pub fn label_visuals(&self) -> LabelVisuals {
        self.visuals.label_visuals
    }

    pub fn combo_box_visuals(&self) -> ComboBoxVisuals {
        self.visuals.combo_box_visuals
    }

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

    pub fn label_visuals_from_ctx(ctx: &Context) -> Option<LabelVisuals> {
        ctx.data(|d| d.get_temp(Self::label_visuals_id()))
    }

    pub fn combo_box_visuals_from_ctx(ctx: &Context) -> Option<ComboBoxVisuals> {
        ctx.data(|d| d.get_temp(Self::combo_box_visuals_id()))
    }

    pub fn text_edit_visuals_from_ctx(ctx: &Context) -> Option<TextEditVisuals> {
        ctx.data(|d| d.get_temp(Self::text_edit_visuals_id()))
    }

    /// Install this theme into the given egui context.
    ///
    /// Widgets then pick up their visuals from `ctx` automatically.
    pub fn install(self, ctx: &Context) {
        let button_visuals = self.button_visuals();
        let label_visuals = self.label_visuals();
        let combo_box_visuals = self.combo_box_visuals();
        let text_edit_visuals = self.text_edit_visuals();
        let style = self.style();

        ctx.set_global_style(style);
        ctx.data_mut(|d| d.insert_temp(Self::storage_id(), self));

        ctx.data_mut(|d| d.insert_temp(Self::button_visuals_id(), button_visuals));
        ctx.data_mut(|d| d.insert_temp(Self::label_visuals_id(), label_visuals));
        ctx.data_mut(|d| d.insert_temp(Self::combo_box_visuals_id(), combo_box_visuals));
        ctx.data_mut(|d| d.insert_temp(Self::text_edit_visuals_id(), text_edit_visuals));
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
        remap_frame(&mut self.frame2, old.bg, new.bg, old.border, new.border);
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
    pub button_visuals: ButtonVisuals,
    pub label_visuals: LabelVisuals,
    pub combo_box_visuals: ComboBoxVisuals,
    pub text_edit_visuals: TextEditVisuals,
    pub frame1_visuals: FrameVisuals,
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

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Typography {
    pub very_small: f32,
    pub small: f32,
    pub normal: f32,
    pub large: f32,
    pub very_large: f32,
    pub heading: f32,
}
