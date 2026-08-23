//! Wasp (dark). Source: <https://github.com/santiyounger/Wasp-Obsidian-Theme>.

use crate::overlay::OverlayManager;
use crate::theme::{Theme, ThemeColors, ThemeKind, ThemeVisuals, Typography};
use crate::visuals::*;
use egui::{
   Color32, CornerRadius, Frame, Margin, Shadow, Spacing, Stroke, Style, Visuals,
   style::{Selection, WidgetVisuals, Widgets},
   vec2,
};

// Color Palette — Wasp (Obsidian, dark). Warm parchment on charcoal,
// honey-gold focus, sage links.
// https://github.com/santiyounger/Wasp-Obsidian-Theme

/// --color-base-00 / --background-secondary
const NEST: Color32 = Color32::from_rgba_premultiplied(25, 25, 25, 255);
/// --color-base-10 / --background-primary
const COMB: Color32 = Color32::from_rgba_premultiplied(36, 36, 36, 255);
/// --color-base-25. Hover lift — gray step, no chroma.
const COMB2: Color32 = Color32::from_rgba_premultiplied(50, 50, 50, 255);
/// --text-normal / --color-base-100
const PARCHMENT: Color32 = Color32::from_rgba_premultiplied(250, 242, 214, 255);
/// --text-faint / --text-muted
const FAINT: Color32 = Color32::from_rgba_premultiplied(189, 174, 147, 255);
/// --interactive-accent / --h1-color. Focus / primary action.
const HONEY: Color32 = Color32::from_rgba_premultiplied(248, 197, 55, 255);
/// --text-link / --inline-code. Links / info — cooler than honey.
const SAGE: Color32 = Color32::from_rgba_premultiplied(131, 165, 152, 255);
/// --h3-color / --button-bg-color
const AMBER: Color32 = Color32::from_rgba_premultiplied(212, 147, 53, 255);
/// --text-highlight-bg (#640211) lifted so glyphs clear AA on nest.
const STING: Color32 = Color32::from_rgba_premultiplied(220, 96, 88, 255);
/// Gruvbox moss — Wasp has no success token.
const MOSS: Color32 = Color32::from_rgba_premultiplied(142, 192, 124, 255);
/// --color-base-40. Structural stroke / shadow.
const HUSK: Color32 = Color32::from_rgba_premultiplied(92, 92, 84, 255);
const FADED_HUSK: Color32 = Color32::from_rgba_premultiplied(92, 92, 84, 118);
/// --wasp-nav-active-background. Selected / emphasis fill, not hover.
const HIGHLIGHT: Color32 = Color32::from_rgba_premultiplied(122, 92, 20, 255);
/// --text-accent-hover. Hover border chrome only — not a ThemeColors slot.
const HOVER_CHROME: Color32 = Color32::from_rgba_premultiplied(212, 147, 53, 255);

const TITLE_BAR: Color32 = NEST;
const MAIN_BG: Color32 = NEST;
const WIDGET_BG: Color32 = COMB;
const HOVER: Color32 = COMB2;
const TEXT: Color32 = PARCHMENT;
const TEXT_MUTED: Color32 = FAINT;
const BORDER: Color32 = HUSK;
const ACCENT: Color32 = HONEY;
const ERROR: Color32 = STING;
const WARNING: Color32 = AMBER;
const SUCCESS: Color32 = MOSS;
const INFO: Color32 = SAGE;

const CORNER_RADIUS: u8 = 6;
const INNER_MARGIN: i8 = 10;
const OUTER_MARGIN: i8 = 5;

/// Return this theme
pub fn theme() -> Theme {
   Theme {
      dark: true,
      overlay_manager: OverlayManager::new(),
      image_tint_recommended: true,
      kind: ThemeKind::Wasp,
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
      bg_on_hover: colors.hover,
      bg_on_click: colors.widget_bg,
      border_on_hover: (0.0, colors.highlight),
      border_on_click: (0.0, colors.highlight),
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
      outer_margin: Margin::same(OUTER_MARGIN),
      fill: colors.bg,
      stroke: Stroke::NONE,
      shadow: shadow,
      ..Default::default()
   }
}

pub fn frame2_visuals(colors: &ThemeColors) -> FrameVisuals {
   FrameVisuals {
      bg_on_hover: colors.hover,
      bg_on_click: colors.bg,
      border_on_hover: (0.0, colors.highlight),
      border_on_click: (0.0, colors.highlight),
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
         color: HUSK,
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
         color: HUSK,
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
      border_hover: Stroke::new(1.0, HONEY),
      border_open: Stroke::new(1.0, HONEY),
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
      dark_mode: true,
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
         color: FADED_HUSK,
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
