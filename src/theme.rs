use crate::overlay::OverlayManager;
use crate::themes::tokyo_night;
use crate::visuals::*;
use crate::utils::*;
use egui::{Color32, Context, Frame, Id, Style};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThemeKind {
    /// Inspired by the https://github.com/tokyo-night/tokyo-night-vscode-theme
    ///
    /// With some slight palette adjustments
    TokyoNight,
}

impl ThemeKind {
    pub fn to_str(&self) -> &str {
        match self {
            ThemeKind::TokyoNight => "Tokyo Night",
        }
    }

    pub fn to_vec() -> Vec<Self> {
        vec![Self::TokyoNight]
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct Theme {
    /// True if the theme is dark
    pub dark_mode: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub overlay_manager: OverlayManager,

    /// True if a tint is recomended to be applied to images
    /// to soften the contrast between the image and the background
    ///
    /// This is usually true for themes with very dark background
    pub image_tint_recommended: bool,

    pub kind: ThemeKind,
    pub style: Style,
    pub colors: ThemeColors,
    pub typography: Typography,

    /// Used for [Frame] not native windows
    pub window_frame: Frame,
    /// Base container frame for major UI sections.
    pub frame1: Frame,
    /// Frame for nested elements, like individual list items.
    pub frame2: Frame,

    pub frame1_visuals: FrameVisuals,
    pub frame2_visuals: FrameVisuals,
}

impl PartialEq for Theme {
    fn eq(&self, other: &Self) -> bool {
        self.dark_mode == other.dark_mode
            && self.kind == other.kind
            && self.style == other.style
            && self.colors == other.colors
            && self.typography == other.typography
            && self.window_frame == other.window_frame
            && self.frame1 == other.frame1
            && self.frame2 == other.frame2
            && self.frame1_visuals == other.frame1_visuals
            && self.frame2_visuals == other.frame2_visuals
    }
}

impl Eq for Theme {}

impl Theme {
    pub fn new(kind: ThemeKind) -> Self {
        let theme = match kind {
            ThemeKind::TokyoNight => tokyo_night::theme(),
        };

        theme
    }

    pub fn button_visuals(&self) -> ButtonVisuals {
        self.colors.button_visuals
    }

    pub fn label_visuals(&self) -> ButtonVisuals {
        self.colors.label_visuals
    }

    pub fn combo_box_visuals(&self) -> ComboBoxVisuals {
        self.colors.combo_box_visuals
    }

    pub fn text_edit_visuals(&self) -> TextEditVisuals {
        self.colors.text_edit_visuals
    }

    fn storage_id() -> Id {
        Id::new("elements::theme")
    }

    /// Install this theme into the given egui context
    pub fn install(self, ctx: &Context) {
        let unchanged = ctx.data(|d| {
            d.get_temp::<Theme>(Self::storage_id())
                .is_some_and(|t| t == self)
        });

        if unchanged {
            return;
        }

        ctx.set_global_style(self.style.clone());
        ctx.data_mut(|d| d.insert_temp(Self::storage_id(), self));
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
            &mut self.frame1_visuals,
            old.hover,
            new.hover,
            old.widget_bg,
            new.widget_bg,
            old.highlight,
            new.highlight,
        );
        remap_frame_visuals(
            &mut self.frame2_visuals,
            old.hover,
            new.hover,
            old.bg,
            new.bg,
            old.highlight,
            new.highlight,
        );
    }
}

/// This is the color palette of the theme
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ThemeColors {
    pub button_visuals: ButtonVisuals,

    pub label_visuals: LabelVisuals,

    pub combo_box_visuals: ComboBoxVisuals,

    pub text_edit_visuals: TextEditVisuals,

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

    /// Highlight color
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
