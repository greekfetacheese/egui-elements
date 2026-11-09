//! [`MenuItem`]: one selectable row inside a [`Menu`](super::Menu).

use egui::{
   CornerRadius, Image, Pos2, Rect, Response, Role, Sense, Ui, Vec2, Widget, WidgetInfo, WidgetText,
};

use super::shared::{allocate_at_least, galley, with_alpha};
use crate::theme::Theme;

/// A single selectable row inside a [`Menu`](super::Menu).
///
/// Add with `ui.add(MenuItem::new("…"))` inside a menu body. The returned
/// [`Response`]'s `.clicked()` fires on activation.
///
/// The optional [`icon`](Self::icon), [`checked`](Self::checked), and
/// [`radio`](Self::radio) builders all reserve the same leading gutter, so
/// toggle items in a menu align cleanly with action items as long as every item
/// in that menu opts into one of them.
#[must_use = "Add with `ui.add(...)`."]
pub struct MenuItem {
   label: WidgetText,
   shortcut: Option<String>,
   danger: bool,
   enabled: bool,
   leading: Option<Leading>,
   submenu_arrow: bool,
}

#[derive(Clone)]
enum Leading {
   Icon(WidgetText),
   Image(Image<'static>),
   Checked(bool),
   Radio(bool),
}

impl std::fmt::Debug for MenuItem {
   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      f.debug_struct("MenuItem")
         .field("label", &self.label.text())
         .field("shortcut", &self.shortcut)
         .field("danger", &self.danger)
         .field("enabled", &self.enabled)
         .field("leading", &self.leading.is_some())
         .finish()
   }
}

impl MenuItem {
   /// Create a menu item with the given label.
   pub fn new(label: impl Into<WidgetText>) -> Self {
      Self {
         label: label.into(),
         shortcut: None,
         danger: false,
         enabled: true,
         leading: None,
         submenu_arrow: false,
      }
   }

   #[doc(hidden)]
   /// Render a body-sized right chevron in the right gutter to mark a submenu
   /// trigger. Hidden because callers should reach for
   /// [`SubMenuItem`](super::SubMenuItem) rather than building this on the raw
   /// `MenuItem`.
   pub fn with_submenu_arrow(mut self) -> Self {
      self.submenu_arrow = true;
      self
   }

   /// Display a keyboard-shortcut hint on the right (informational only — the
   /// actual shortcut is not bound).
   pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
      self.shortcut = Some(shortcut.into());
      self
   }

   /// Render the item in the danger tone — red label, red hover highlight. Use
   /// for destructive actions.
   pub fn danger(mut self) -> Self {
      self.danger = true;
      self
   }

   /// Disable the item. Disabled items do not fire `clicked()` and render with
   /// muted text.
   pub fn enabled(mut self, enabled: bool) -> Self {
      self.enabled = enabled;
      self
   }

   /// Show a leading icon (any text, typically a unicode glyph) in the gutter to
   /// the left of the label. Reserves the gutter even when the glyph is narrow,
   /// so adjacent items align.
   pub fn icon(mut self, icon: impl Into<WidgetText>) -> Self {
      self.leading = Some(Leading::Icon(icon.into()));
      self
   }

   /// Show a leading image (typically a Lucide icon) in the same gutter as [`icon`](Self::icon).
   ///
   /// The image is tinted with the muted text colour and drawn in the gutter's 16 pt box, so a menu
   /// of image rows and toggle rows keeps one aligned leading column.
   pub fn image(mut self, image: impl Into<Image<'static>>) -> Self {
      self.leading = Some(Leading::Image(image.into()));
      self
   }

   /// Render the item as a checkbox toggle: a checkmark in the leading gutter
   /// when `on`, an empty gutter when off. The item is announced via accesskit
   /// as a checkbox with the given selected state.
   pub fn checked(mut self, on: bool) -> Self {
      self.leading = Some(Leading::Checked(on));
      self
   }

   /// Render the item as a radio-button toggle: a filled dot in the leading
   /// gutter when `on`, an empty gutter when off. Use within a group of
   /// mutually-exclusive choices. Announced via accesskit as a radio button.
   pub fn radio(mut self, on: bool) -> Self {
      self.leading = Some(Leading::Radio(on));
      self
   }
}

impl Widget for MenuItem {
   fn ui(self, ui: &mut Ui) -> Response {
      let theme = Theme::current(ui.ctx());
      let colors = &theme.colors;
      let typography = &theme.typography;

      let pad_x = 10.0;
      let pad_y = 6.0;
      let gap_x = 16.0;
      let gutter_w = 16.0; // Reserved leading-glyph slot.
      let gutter_gap = 8.0;

      let label_color = if !self.enabled {
         colors.text_muted
      } else if self.danger {
         colors.error
      } else {
         colors.text
      };

      let label_galley = galley(ui, self.label.text(), typography.normal, false);

      let shortcut_galley =
         self.shortcut.as_deref().map(|s| galley(ui, s, typography.small, false));

      // Submenu arrow: a `›` glyph rendered well above body size so it reads as a
      // flyout indicator rather than a small auxiliary mark.
      let submenu_arrow_galley = if self.submenu_arrow {
         Some(galley(ui, "\u{203A}", 24.0, false))
      } else {
         None
      };

      let leading_glyph_galley = match &self.leading {
         Some(Leading::Icon(icon)) => Some(galley(ui, icon.text(), typography.normal, false)),
         Some(Leading::Image(_)) => None,
         Some(Leading::Checked(true)) => Some(galley(ui, "\u{2713}", typography.normal, true)),
         Some(Leading::Radio(true)) => Some(galley(ui, "\u{2022}", typography.normal, true)),
         // Off-state toggles still reserve the gutter so siblings align.
         Some(Leading::Checked(false)) | Some(Leading::Radio(false)) => None,
         None => None,
      };

      let leading_offset = if self.leading.is_some() {
         gutter_w + gutter_gap
      } else {
         0.0
      };

      let trailing_w = shortcut_galley
         .as_ref()
         .map_or(0.0, |g| g.size().x + gap_x)
         .max(submenu_arrow_galley.as_ref().map_or(0.0, |g| g.size().x + gap_x));

      let content_w = leading_offset + label_galley.size().x + trailing_w;
      // Width is the natural content size: the parent menu's
      // `top_down_justified` layout stretches each row to match the widest one,
      // and the popup as a whole sizes to that maximum. Don't
      // `.max(available_width)` here — that would let each item greedily expand
      // to whatever space the parent offers, which makes the popup balloon out to
      // its container's width.
      let desired = Vec2::new(
         content_w + pad_x * 2.0,
         label_galley.size().y.max(typography.normal) + pad_y * 2.0,
      );

      let sense = if self.enabled {
         Sense::click()
      } else {
         Sense::hover()
      };

      // `allocate_at_least` returns the full layout slot rect (which, under
      // `top_down_justified`, expands to the widest sibling). We need that here
      // so the trailing shortcut right-aligns with the popup's right edge —
      // `allocate_exact_size` would clamp the returned rect to `desired` and the
      // shortcut would sit right after the label.
      let (rect, response) = allocate_at_least(ui, desired, sense, self.enabled);

      if ui.is_rect_visible(rect) {
         let is_hovered = response.hovered() && self.enabled;

         if is_hovered {
            let bg = if self.danger {
               with_alpha(colors.error, 40)
            } else {
               with_alpha(colors.accent, 28)
            };
            let radius = CornerRadius::same(theme.corner_radius.saturating_sub(2));

            ui.painter().rect_filled(rect, radius, bg);
         }

         if let Some(glyph) = leading_glyph_galley {
            // Centre the glyph within the gutter slot.
            let slot_x = rect.min.x + pad_x;
            let glyph_color = match &self.leading {
               Some(Leading::Checked(true)) | Some(Leading::Radio(true)) => colors.accent,
               _ => colors.text_muted,
            };
            let pos = Pos2::new(
               slot_x + (gutter_w - glyph.size().x) * 0.5,
               rect.center().y - glyph.size().y * 0.5,
            );

            ui.painter().galley(pos, glyph, glyph_color);
         }

         if let Some(Leading::Image(image)) = &self.leading {
            let slot = Rect::from_center_size(
               Pos2::new(
                  rect.min.x + pad_x + gutter_w * 0.5,
                  rect.center().y,
               ),
               Vec2::splat(gutter_w),
            );
            image.clone().tint(colors.text_muted).paint_at(ui, slot);
         }

         let label_pos = Pos2::new(
            rect.min.x + pad_x + leading_offset,
            rect.center().y - label_galley.size().y * 0.5,
         );

         ui.painter().galley(label_pos, label_galley, label_color);

         if let Some(galley) = shortcut_galley {
            let pos = Pos2::new(
               rect.max.x - pad_x - galley.size().x,
               rect.center().y - galley.size().y * 0.5,
            );
            let color = if self.enabled && self.danger {
               with_alpha(colors.error, 200)
            } else {
               colors.text_muted
            };

            ui.painter().galley(pos, galley, color);
         }

         if let Some(galley) = submenu_arrow_galley {
            let pos = Pos2::new(
               rect.max.x - pad_x - galley.size().x,
               rect.center().y - galley.size().y * 0.5,
            );

            ui.painter().galley(pos, galley, colors.text_muted);
         }
      }

      response.widget_info(|| match &self.leading {
         Some(Leading::Checked(on)) => WidgetInfo::selected(
            Role::CheckBox,
            self.enabled,
            *on,
            self.label.text(),
         ),
         Some(Leading::Radio(on)) => WidgetInfo::selected(
            Role::RadioButton,
            self.enabled,
            *on,
            self.label.text(),
         ),
         _ => WidgetInfo::labeled(Role::Button, self.enabled, self.label.text()),
      });

      response
   }
}
