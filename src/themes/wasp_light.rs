//! Wasp (light). Source: <https://github.com/santiyounger/Wasp-Obsidian-Theme>.

use crate::overlay::OverlayManager;
use crate::theme::{Theme, ThemeColors, ThemeKind, ThemeVisuals, Typography};
use crate::visuals::*;
use egui::{
   Color32, CornerRadius, Frame, Margin, Shadow, Spacing, Stroke, Style, Visuals,
   style::{Selection, WidgetVisuals, Widgets},
   vec2,
};

// Color Palette — Wasp (Obsidian, light). Ink on cream paper,
// rust focus, sage links. Honey gold fails AA as a glyph on cream,
// so focus uses --text-accent and gold stays hover chrome / selection.
// https://github.com/santiyounger/Wasp-Obsidian-Theme

/// --color-base-10 / --background-primary
const CREAM: Color32 = Color32::from_rgba_premultiplied(250, 246, 240, 255);
/// --color-base-05. Elevated surface — tiny lift, no new plane.
const PAPER: Color32 = Color32::from_rgba_premultiplied(255, 252, 247, 255);
/// --color-base-20 / --background-secondary. Hover lift — gray step, no chroma.
const HUSK: Color32 = Color32::from_rgba_premultiplied(232, 223, 208, 255);
/// --text-normal / --color-base-100
const INK: Color32 = Color32::from_rgba_premultiplied(44, 36, 25, 255);
/// --text-muted
const MUTED: Color32 = Color32::from_rgba_premultiplied(92, 80, 68, 255);
/// --text-accent. Focus / primary action (gold fails AA on cream).
const RUST: Color32 = Color32::from_rgba_premultiplied(155, 58, 26, 255);
/// --inline-code. Links / info — cooler than rust.
const SAGE: Color32 = Color32::from_rgba_premultiplied(61, 107, 92, 255);
/// --button-hover-bg-color. Caution — darkened honey that clears AA.
const HONEY: Color32 = Color32::from_rgba_premultiplied(168, 92, 24, 255);
/// Sting red — Wasp light has no error token.
const STING: Color32 = Color32::from_rgba_premultiplied(160, 40, 30, 255);
/// Leaf olive, distinct from sage.
const PINE: Color32 = Color32::from_rgba_premultiplied(56, 118, 64, 255);
/// --color-base-30. Structural stroke / shadow.
const LINEN: Color32 = Color32::from_rgba_premultiplied(201, 184, 168, 255);
const FADED_LINEN: Color32 = Color32::from_rgba_premultiplied(176, 160, 144, 118);
/// --text-selection. Selected / emphasis fill, not hover.
const HIGHLIGHT: Color32 = Color32::from_rgba_premultiplied(228, 192, 122, 255);
/// --background-modifier-border. Hover border chrome only — not a ThemeColors slot.
const HOVER_CHROME: Color32 = Color32::from_rgba_premultiplied(200, 118, 40, 255);

const TITLE_BAR: Color32 = CREAM;
const MAIN_BG: Color32 = CREAM;
const WIDGET_BG: Color32 = PAPER;
const HOVER: Color32 = HUSK;
const TEXT: Color32 = INK;
const TEXT_MUTED: Color32 = MUTED;
const BORDER: Color32 = LINEN;
const ACCENT: Color32 = RUST;
const ERROR: Color32 = STING;
const WARNING: Color32 = HONEY;
const SUCCESS: Color32 = PINE;
const INFO: Color32 = SAGE;

const CORNER_RADIUS: u8 = 6;
const INNER_MARGIN: i8 = 10;
const OUTER_MARGIN: i8 = 5;

/// Return this theme
pub fn theme() -> Theme {
   Theme {
      dark: false,
      overlay_manager: OverlayManager::new(),
      image_tint_recommended: false,
      kind: ThemeKind::WaspLight,
      colors: colors(),
      typography: typography(),
      window_frame: window_frame(&colors()),
      frame1: frame1(&colors()),
      frame2: frame2(&colors()),
      visuals: theme_visuals(),
      corner_radius: CORNER_RADIUS,
      inner_margin: INNER_MARGIN,
      outer_margin: OUTER_MARGIN,
      button_padding: vec2(10.0, 8.0),
      #[cfg(feature = "elegance")]
      elegance_key: None,
   }
}

/// Return the theme colors for this theme
fn colors() -> ThemeColors {
   ThemeColors {
      title_bar: TITLE_BAR,
      bg: MAIN_BG,
      widget_bg: WIDGET_BG,
      hover: HOVER,
      text: TEXT,
      text_muted: TEXT_MUTED,
      highlight: HIGHLIGHT,
      border: BORDER,
      accent: ACCENT,
      error: ERROR,
      warning: WARNING,
      success: SUCCESS,
      info: INFO,
   }
}

fn typography() -> Typography {
   Typography {
      very_small: 12.0,
      small: 14.0,
      normal: 16.0,
      large: 18.0,
      very_large: 20.0,
      heading: 26.0,
   }
}

pub fn window_frame(colors: &ThemeColors) -> Frame {
   let shadow = Shadow {
      offset: (0, 0).into(),
      blur: 3,
      spread: 0,
      color: colors.border,
   };
   Frame {
      corner_radius: CornerRadius::same(CORNER_RADIUS),
      inner_margin: Margin::same(INNER_MARGIN),
      fill: colors.bg,
      stroke: Stroke::new(1.0, colors.border),
      shadow: shadow,
      ..Default::default()
   }
}

/// Base container frame for major UI sections.
pub fn frame1(colors: &ThemeColors) -> Frame {
   let shadow = Shadow {
      offset: (0, 0).into(),
      blur: 2,
      spread: 0,
      color: colors.border,
   };
   Frame {
      corner_radius: CornerRadius::same(CORNER_RADIUS),
      inner_margin: Margin::same(INNER_MARGIN),
      fill: colors.widget_bg,
      stroke: Stroke::NONE,
      shadow: shadow,
      ..Default::default()
   }
}

pub fn frame1_visuals(colors: &ThemeColors) -> FrameVisuals {
   FrameVisuals {
      bg: colors.widget_bg,
      bg_hover: colors.hover,
      bg_click: colors.widget_bg,
      bg_selected: colors.highlight,
      border: Stroke::NONE,
      border_hover: Stroke::new(0.0, colors.highlight),
      border_click: Stroke::new(0.0, colors.highlight),
      corner_radius: CornerRadius::same(CORNER_RADIUS),
      shadow: Shadow {
         offset: (0, 0).into(),
         blur: 2,
         spread: 0,
         color: colors.border,
      },
   }
}

/// Frame for nested elements, like individual list items.
pub fn frame2(colors: &ThemeColors) -> Frame {
   let shadow = Shadow {
      offset: (0, 0).into(),
      blur: 2,
      spread: 0,
      color: colors.border,
   };
   Frame {
      corner_radius: CornerRadius::same(CORNER_RADIUS),
      inner_margin: Margin::same(INNER_MARGIN),
      fill: colors.bg,
      stroke: Stroke::NONE,
      shadow: shadow,
      ..Default::default()
   }
}

pub fn frame2_visuals(colors: &ThemeColors) -> FrameVisuals {
    FrameVisuals {
        bg: colors.bg,
        bg_hover: colors.hover,
        bg_click: colors.bg,
        bg_selected: colors.highlight,
        border: Stroke::NONE,
        border_hover: Stroke::new(0.0, colors.highlight),
        border_click: Stroke::new(0.0, colors.highlight),
        corner_radius: CornerRadius::same(CORNER_RADIUS),
        shadow: Shadow {
            offset: (0, 0).into(),
            blur: 2,
            spread: 0,
            color: colors.border,
        },
    }
}

pub fn button_visuals() -> ButtonVisuals {
   ButtonVisuals {
      text: TEXT,
      bg: WIDGET_BG,
      bg_hover: HOVER,
      bg_click: WIDGET_BG,
      bg_selected: HIGHLIGHT,
      border: Stroke::new(1.0, Color32::TRANSPARENT),
      border_hover: Stroke::new(1.0, HOVER_CHROME),
      border_click: Stroke::new(1.0, Color32::TRANSPARENT),
      corner_radius: CornerRadius::same(3),
      shadow: Shadow {
         offset: (0, 0).into(),
         blur: 2,
         spread: 1,
         color: LINEN,
      },
   }
}

pub fn combo_box_visuals() -> ComboBoxVisuals {
   ComboBoxVisuals {
      bg: WIDGET_BG,
      icon: TEXT,
      bg_hover: HOVER,
      bg_open: WIDGET_BG,
      border: Stroke::new(1.0, Color32::TRANSPARENT),
      border_hover: Stroke::new(1.0, HOVER_CHROME),
      border_open: Stroke::new(1.0, Color32::TRANSPARENT),
      corner_radius: CornerRadius::same(CORNER_RADIUS),
      shadow: Shadow {
         offset: (0, 0).into(),
         blur: 2,
         spread: 1,
         color: LINEN,
      },
   }
}

pub fn label_visuals() -> ButtonVisuals {
   ButtonVisuals {
      bg: Color32::TRANSPARENT,
      border: Stroke::new(1.0, Color32::TRANSPARENT),
      border_hover: Stroke::new(1.0, Color32::TRANSPARENT),
      border_click: Stroke::new(1.0, Color32::TRANSPARENT),
      ..button_visuals()
   }
}

pub fn text_edit_visuals() -> TextEditVisuals {
   TextEditVisuals {
      text: TEXT,
      bg: WIDGET_BG,
      border: Stroke::new(1.0, BORDER),
      border_hover: Stroke::new(1.0, RUST),
      border_open: Stroke::new(1.0, RUST),
      corner_radius: CornerRadius::same(CORNER_RADIUS),
      shadow: Shadow::NONE,
   }
}

pub fn style() -> Style {
   let widgets = widgets(colors());
   let visuals = visuals(widgets, &colors());
   let spacing = Spacing {
      window_margin: Margin::same(10),
      ..Default::default()
   };

   Style {
      visuals,
      animation_time: 0.3,
      spacing,
      ..Default::default()
   }
}

fn theme_visuals() -> ThemeVisuals {
   ThemeVisuals {
      button_visuals: button_visuals(),
      label_visuals: label_visuals(),
      combo_box_visuals: combo_box_visuals(),
      text_edit_visuals: text_edit_visuals(),
      frame1_visuals: frame1_visuals(&colors()),
      frame2_visuals: frame2_visuals(&colors()),
   }
}

fn visuals(widgets: Widgets, colors: &ThemeColors) -> Visuals {
   Visuals {
      dark_mode: false,
      override_text_color: Some(colors.text),
      widgets,
      selection: Selection {
         bg_fill: colors.highlight, // selected text / combo selected row
         stroke: Stroke::new(1.0, colors.accent), // also affects TextEdit border color when active
      },
      hyperlink_color: colors.info,
      faint_bg_color: colors.bg,
      extreme_bg_color: colors.widget_bg,
      code_bg_color: colors.bg,
      warn_fg_color: colors.warning,
      error_fg_color: colors.error,
      window_corner_radius: CornerRadius::same(CORNER_RADIUS),
      window_shadow: Shadow {
         offset: (0, 0).into(),
         blur: 3,
         spread: 0,
         color: FADED_LINEN,
      },
      window_fill: colors.bg,
      window_stroke: Stroke::new(1.0, Color32::TRANSPARENT),
      panel_fill: colors.bg,
      ..Default::default()
   }
}

fn widgets(colors: ThemeColors) -> Widgets {
   let base_visuals = WidgetVisuals {
      bg_fill: colors.widget_bg,
      weak_bg_fill: colors.bg,
      bg_stroke: Stroke::new(1.0, colors.border),
      corner_radius: CornerRadius::same(CORNER_RADIUS),
      fg_stroke: Stroke::new(1.0, colors.text),
      expansion: 0.0,
   };

   let mut non_interactive_base = base_visuals.clone();
   non_interactive_base.bg_stroke.width = 1.0;

   // Set inactive bg to highlight color
   // Because widgets like sliders dont get a border and it will not distinguish
   // from the bg color
   let mut inactive_visuals = base_visuals.clone();
   inactive_visuals.bg_fill = colors.highlight;

   Widgets {
      noninteractive: non_interactive_base,
      inactive: inactive_visuals,
      hovered: WidgetVisuals {
         bg_fill: colors.widget_bg,
         weak_bg_fill: colors.hover,
         bg_stroke: Stroke::new(1.0, colors.hover),
         ..base_visuals
      },
      active: WidgetVisuals {
         bg_fill: colors.widget_bg,
         weak_bg_fill: colors.widget_bg,
         bg_stroke: Stroke::new(1.0, colors.border),
         ..base_visuals
      },
      open: WidgetVisuals {
         bg_fill: colors.widget_bg,
         // egui Window title (on top) paints this, not title_frame.fill
         weak_bg_fill: colors.widget_bg,
         bg_stroke: Stroke::new(1.0, colors.border),
         ..base_visuals
      },
   }
}
