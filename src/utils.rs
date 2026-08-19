use crate::theme::*;
use crate::visuals::FrameVisuals;
use egui::{Color32, ComboBox, Frame, Ui};
use palette::{Hsl, IntoColor, Srgba};

/// Should work for most images that are shown on a very dark background
pub const TINT_1: Color32 = Color32::from_rgba_premultiplied(216, 216, 216, 255);

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct Hsla {
    /// Hue 0.0..=360.0
    pub h: f32,
    /// Saturation 0.0..=100.0
    pub s: f32,
    /// Lightness 0.0..=100.0
    pub l: f32,
    /// Alpha 0.0..=1.0
    pub a: f32,
}

impl Hsla {
    pub fn from_color32(c: Color32) -> Self {
        let srgba = Srgba::new(
            c.r() as f32 / 255.0,
            c.g() as f32 / 255.0,
            c.b() as f32 / 255.0,
            c.a() as f32 / 255.0,
        );
        let hsl: Hsl = srgba.into_color();
        let (h, s, l) = hsl.into_components();
        // Normalize hue to [0, 360)
        let mut hue = h.into_degrees();
        hue = (hue % 360.0 + 360.0) % 360.0;
        Hsla {
            h: hue,
            s: s * 100.0,
            l: l * 100.0,
            a: srgba.alpha,
        }
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        match Color32::from_hex(hex) {
            Ok(c) => Some(Self::from_color32(c)),
            Err(_) => None,
        }
    }

    pub fn to_color32(&self) -> Color32 {
        let srgba = self.to_srgba();
        let (r, g, b, a) = srgba.into_components();
        Color32::from_rgba_unmultiplied(
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8,
            (a * 255.0) as u8,
        )
    }

    pub fn to_srgba(&self) -> Srgba {
        let hsl = Hsl::new(self.h, self.s / 100.0, self.l / 100.0);
        hsl.into_color()
    }

    pub fn to_rgba_components(&self) -> (u8, u8, u8, u8) {
        let srgba = self.to_srgba();
        let (r, g, b, a) = srgba.into_components();
        let r = (r * 255.0) as u8;
        let g = (g * 255.0) as u8;
        let b = (b * 255.0) as u8;
        let a = (a * 255.0) as u8;
        (r, g, b, a)
    }

    pub fn shades(&self, num_shades: usize, direction: ShadeDirection) -> Vec<Color32> {
        let mut shades = Vec::new();
        let step = if direction == ShadeDirection::Lighter {
            5.0
        } else {
            -5.0
        };
        let mut current = *self;
        for _ in 0..num_shades {
            shades.push(current.to_color32());
            current.l = (current.l as f32 + step).clamp(0.0, 100.0);
        }
        shades
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ShadeDirection {
    Lighter,
    Darker,
}

pub fn frame_palette_changed(old: &ThemeColors, new: &ThemeColors) -> bool {
    old.title_bar != new.title_bar
        || old.bg != new.bg
        || old.widget_bg != new.widget_bg
        || old.hover != new.hover
        || old.highlight != new.highlight
        || old.border != new.border
}

pub fn remap_if_eq(slot: &mut Color32, old: Color32, new: Color32) {
    if *slot == old {
        *slot = new;
    }
}

pub fn remap_frame(
    frame: &mut Frame,
    old_fill: Color32,
    new_fill: Color32,
    old_border: Color32,
    new_border: Color32,
) {
    remap_if_eq(&mut frame.fill, old_fill, new_fill);
    remap_if_eq(&mut frame.stroke.color, old_border, new_border);
    remap_if_eq(&mut frame.shadow.color, old_border, new_border);
}

pub fn remap_frame_visuals(
    visuals: &mut FrameVisuals,
    old_hover: Color32,
    new_hover: Color32,
    old_click: Color32,
    new_click: Color32,
    old_highlight: Color32,
    new_highlight: Color32,
) {
    remap_if_eq(&mut visuals.bg_on_hover, old_hover, new_hover);
    remap_if_eq(&mut visuals.bg_on_click, old_click, new_click);
    remap_if_eq(&mut visuals.border_on_hover.1, old_highlight, new_highlight);
    remap_if_eq(&mut visuals.border_on_click.1, old_highlight, new_highlight);
}

/// Show a ComboBox to change the theme
///
/// Returns the new theme if we select one, the new theme is also applied to the [egui::Context]
pub fn change_theme(current_theme: &Theme, ui: &mut Ui) -> Option<Theme> {
    let mut new_theme_opt = None;
    ComboBox::from_label("Theme")
        .selected_text(current_theme.kind.to_str())
        .show_ui(ui, |ui| {
            for kind in ThemeKind::to_vec() {
                if ui
                    .selectable_label(current_theme.kind == kind, kind.to_str())
                    .clicked()
                {
                    let new_theme = Theme::new(kind);
                    ui.ctx().set_global_style(new_theme.style.clone());
                    new_theme_opt = Some(new_theme);
                }
            }
        });
    new_theme_opt
}
