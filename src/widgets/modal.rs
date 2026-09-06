//! Modal dialog — a centered themed card over a dimmed backdrop.
//!
//! Painted in two layers: a full-viewport dimmed backdrop that swallows
//! clicks (and closes the modal when clicked), and a centered card-like
//! window with an optional heading row and a close "×" button.
//! Press `Esc` to dismiss.
//!
//! Forked from <https://github.com/stephenberry/egui-elegance>

use egui::{
   Align, Align2, Area, Color32, Context, CornerRadius, Frame, Id, Image, Key, Layout, Margin,
   Order, Pos2, Rect, Response, RichText, Sense, Shape, Stroke, Ui, UiBuilder, Vec2, WidgetInfo,
   WidgetText, WidgetType, accesskit, vec2,
};

use super::button::Button;
use crate::theme::Theme;

/// Boxed `FnOnce(&mut Ui)` callback used by the footer slots.
type UiFn<'a> = Box<dyn FnOnce(&mut Ui) + 'a>;

/// A centered modal dialog.
///
/// The `open` flag drives visibility: when it's `false` on entry to
/// [`Modal::show`], nothing is rendered; when the user clicks the backdrop,
/// presses `Esc`, or clicks the "×" button, it's flipped to `false`.
///
/// ```no_run
/// # use egui_elements::widgets::Modal;
/// # let ctx = egui::Context::default();
/// # let mut open = true;
/// Modal::new("stats", &mut open)
///     .heading("Run Summary")
///     .show(&ctx, |ui| {
///         ui.label("…");
///     });
/// ```
#[must_use = "Call `.show(ctx, |ui| { ... })` to render the modal."]
pub struct Modal<'a> {
   id_salt: Id,
   heading: Option<WidgetText>,
   subtitle: Option<WidgetText>,
   header_icon: Option<Image<'a>>,
   frame: Option<Frame>,
   center_header_icon: bool,
   backdrop_order: Order,
   content_order: Order,
   open: &'a mut bool,
   max_width: Option<f32>,
   closable: bool,
   close_on_backdrop: bool,
   close_on_escape: bool,
   alert: bool,
   center_header: bool,
   header_separator: bool,
   footer_separator: bool,
   footer: Option<UiFn<'a>>,
   footer_left: Option<UiFn<'a>>,
}

impl<'a> std::fmt::Debug for Modal<'a> {
   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      f.debug_struct("Modal")
         .field("id_salt", &self.id_salt)
         .field(
            "heading",
            &self.heading.as_ref().map(|h| h.text()),
         )
         .field(
            "subtitle",
            &self.subtitle.as_ref().map(|h| h.text()),
         )
         .field(
            "header_icon",
            &self.header_icon.as_ref().map(|_| "<image>"),
         )
         .field("frame", &self.frame)
         .field("center_header_icon", &self.center_header_icon)
         .field("backdrop_order", &self.backdrop_order)
         .field("content_order", &self.content_order)
         .field("open", &*self.open)
         .field("max_width", &self.max_width)
         .field("closable", &self.closable)
         .field("close_on_backdrop", &self.close_on_backdrop)
         .field("close_on_escape", &self.close_on_escape)
         .field("alert", &self.alert)
         .field("center_header", &self.center_header)
         .field("header_separator", &self.header_separator)
         .field("footer_separator", &self.footer_separator)
         .field(
            "footer",
            &self.footer.as_ref().map(|_| "<closure>"),
         )
         .field(
            "footer_left",
            &self.footer_left.as_ref().map(|_| "<closure>"),
         )
         .finish()
   }
}

impl<'a> Modal<'a> {
   /// Create a modal keyed by `id_salt` whose visibility is bound to `open`.
   pub fn new(id: impl Into<Id>, open: &'a mut bool) -> Self {
      Self {
         id_salt: Id::new(id.into()),
         heading: None,
         subtitle: None,
         header_icon: None,
         frame: None,
         center_header_icon: false,
         backdrop_order: Order::Middle,
         content_order: Order::Foreground,
         open,
         max_width: None,
         closable: true,
         close_on_backdrop: true,
         close_on_escape: true,
         alert: false,
         center_header: false,
         header_separator: true,
         footer_separator: true,
         footer: None,
         footer_left: None,
      }
   }

   /// Show a strong heading at the top of the modal, alongside the close button.
   pub fn heading(mut self, heading: impl Into<WidgetText>) -> Self {
      self.heading = Some(heading.into());
      self
   }

   /// Show a muted subtitle line under the heading.
   pub fn subtitle(mut self, subtitle: impl Into<WidgetText>) -> Self {
      self.subtitle = Some(subtitle.into());
      self
   }

   /// Show an image in the header. Alignment is independent of the title
   /// — see [`Modal::center_header_icon`]. Pass a sized/tinted
   /// [`egui::Image`], for example a Lucide SVG:
   ///
   /// ```ignore
   /// .header_icon(Lucide::ShieldCheck.size(28.0).color(theme.colors.accent).image())
   /// ```
   pub fn header_icon(mut self, icon: impl Into<Image<'a>>) -> Self {
      self.header_icon = Some(icon.into());
      self
   }

   /// Override the [Frame] used for the modal's card.
   ///
   /// By default it creates a frame from the theme's [Theme::current].
   pub fn frame(mut self, frame: Frame) -> Self {
      self.frame = Some(frame);
      self
   }

   /// Center the header image on its own row. Default: `false` (left).
   /// Does not change title alignment — that is [`Modal::center_header`].
   pub fn center_header_icon(mut self, center: bool) -> Self {
      self.center_header_icon = center;
      self
   }

   /// Override the [Order] used for the backdrop of the modal
   pub fn backdrop_order(mut self, order: Order) -> Self {
      self.backdrop_order = order;
      self
   }

   /// Override the [Order] used for the content of the modal
   pub fn content_order(mut self, order: Order) -> Self {
      self.content_order = order;
      self
   }

   /// Cap the modal card width in points. When unset the card shrink-wraps
   /// to its contents and stays centered. Default: no cap.
   pub fn max_width(mut self, max_width: f32) -> Self {
      self.max_width = Some(max_width);
      self
   }

   /// Whether the user may dismiss the modal at all. When `false`, the
   /// close "×" button is hidden and `Esc` / backdrop clicks are ignored —
   /// regardless of [`Modal::close_on_backdrop`] / [`Modal::close_on_escape`].
   ///
   /// Use this to force the user to see an in-progress action through (or
   /// cancel it via an explicit footer button) — for example, blocking
   /// dismissal while a long-running task runs, instead of juggling the
   /// `open` flag with an external "is it running?" guard.
   ///
   /// This only removes the *user-driven* dismissal affordances; the caller
   /// is still free to set the bound `open` flag to `false` programmatically
   /// to close the modal from code. Default: `true`.
   pub fn closable(mut self, closable: bool) -> Self {
      self.closable = closable;
      self
   }

   /// Whether clicking the dimmed backdrop dismisses the modal. Default: `true`.
   pub fn close_on_backdrop(mut self, close: bool) -> Self {
      self.close_on_backdrop = close;
      self
   }

   /// Whether pressing `Esc` dismisses the modal. Default: `true`.
   pub fn close_on_escape(mut self, close: bool) -> Self {
      self.close_on_escape = close;
      self
   }

   /// Mark this modal as an *alert dialog* — a dialog that demands the
   /// user's attention to proceed, such as a destructive confirmation or
   /// an unsaved-changes prompt. Screen readers announce alert dialogs
   /// more assertively than ordinary dialogs. Default: `false`.
   ///
   /// Under the hood this exposes `accesskit::Role::AlertDialog` on the
   /// modal's root node instead of the default `Role::Dialog`.
   pub fn alert(mut self, alert: bool) -> Self {
      self.alert = alert;
      self
   }

   /// Center the heading and subtitle in the header band. The close
   /// button stays top-right. Default: `false` (left-aligned). Icon
   /// alignment is separate — see [`Modal::center_header_icon`].
   pub fn center_header(mut self, center: bool) -> Self {
      self.center_header = center;
      self
   }

   /// Draw a horizontal divider under the header. Default: `true`.
   pub fn header_separator(mut self, show: bool) -> Self {
      self.header_separator = show;
      self
   }

   /// Draw a horizontal divider above the footer. Default: `true`.
   pub fn footer_separator(mut self, show: bool) -> Self {
      self.footer_separator = show;
      self
   }

   /// Add a footer row at the bottom of the modal. The closure runs in a
   /// right-to-left layout, so widgets added in source order land
   /// rightmost-first — matching the typical "Cancel | Confirm" reading.
   /// The footer renders below an optional horizontal divider (see
   /// [`Modal::footer_separator`]) and over a slightly recessed fill,
   /// separating it visually from the body.
   pub fn footer<F: FnOnce(&mut Ui) + 'a>(mut self, add_footer: F) -> Self {
      self.footer = Some(Box::new(add_footer));
      self
   }

   /// Add a left-aligned slot to the footer (only rendered when
   /// [`Modal::footer`] is also set). Useful for an "export before delete"
   /// checkbox or a keyboard-shortcut hint that should sit opposite the
   /// action buttons.
   pub fn footer_left<F: FnOnce(&mut Ui) + 'a>(mut self, add_left: F) -> Self {
      self.footer_left = Some(Box::new(add_left));
      self
   }

   /// Render the modal. Returns `None` if the modal was suppressed because
   /// the bound `open` flag was `false`; otherwise returns `Some(R)` with
   /// the content closure's return value.
   pub fn show<R>(self, ctx: &Context, add_contents: impl FnOnce(&mut Ui) -> R) -> Option<R> {
      // --- Focus lifecycle ------------------------------------------------
      // Track the open/closed transition so we can (a) record which widget
      // had keyboard focus before the modal opened and (b) restore that
      // focus when the modal closes. Without this the user's focus is
      // visually eclipsed by the modal but structurally remains behind it —
      // Tab would navigate widgets on the underlying page.
      let focus_storage = Id::new(("elegance_modal_focus", self.id_salt));
      let mut focus_state: ModalFocusState =
         ctx.data(|d| d.get_temp(focus_storage).unwrap_or_default());
      let is_open = *self.open;

      if focus_state.was_open && !is_open {
         // Just closed this frame — return focus to whatever had it before.
         if let Some(prev) = focus_state.prev_focus {
            ctx.memory_mut(|m| m.request_focus(prev));
         }
         ctx.data_mut(|d| d.insert_temp(focus_storage, ModalFocusState::default()));
         return None;
      }

      if !is_open {
         return None;
      }

      let just_opened = !focus_state.was_open;
      if just_opened {
         focus_state.prev_focus = ctx.memory(|m| m.focused());
         focus_state.was_open = true;
         ctx.data_mut(|d| d.insert_temp(focus_storage, focus_state));
      }

      let theme = Theme::current(ctx);
      let mut should_close = false;
      let mut close_btn_id: Option<Id> = None;
      let closable = self.closable;

      // --- Backdrop ----------------------------------------------------
      let screen = ctx.content_rect();
      let backdrop_id = Id::new("elegance_modal_backdrop").with(self.id_salt);
      let backdrop =
         Area::new(backdrop_id)
            .fixed_pos(screen.min)
            .order(self.backdrop_order)
            .show(ctx, |ui| {
               ui.painter().rect_filled(
                  screen,
                  CornerRadius::ZERO,
                  Color32::from_rgba_premultiplied(0, 0, 0, 150),
               );
               ui.allocate_rect(screen, Sense::click())
            });
      if closable && self.close_on_backdrop && backdrop.inner.clicked() {
         should_close = true;
      }

      // --- Content -----------------------------------------------------
      let window_id = Id::new("elegance_modal_window").with(self.id_salt);
      let alert = self.alert;
      let heading_text: Option<String> = self.heading.as_ref().map(|h| h.text().to_string());
      let mut area = Area::new(window_id)
         .order(self.content_order)
         .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
         .pivot(Align2::CENTER_CENTER);
      // 0 = shrink-wrap the first frame so a missing max_width still
      // measures true content width; Area re-anchors on the next pass.
      area = area.default_width(self.max_width.unwrap_or(0.0));
      let result = area.show(ctx, |ui| {
         // Upgrade this Ui's accesskit role from `GenericContainer`
         // (set automatically by `Ui::new`) to a dialog role, so
         // screen readers announce the modal correctly and
         // platforms that support dialog focus tracking (AT-SPI)
         // treat it as a window-like surface.
         let role = if alert {
            accesskit::Role::AlertDialog
         } else {
            accesskit::Role::Dialog
         };
         let heading_for_label = heading_text.clone();
         ui.ctx().accesskit_node_builder(ui.unique_id(), |node| {
            node.set_role(role);
            if let Some(label) = heading_for_label {
               node.set_label(label);
            }
         });

         if let Some(max_width) = self.max_width {
            ui.set_max_width(max_width);
         }

         let frame = self.frame.unwrap_or_else(|| {
            Frame::new()
               .fill(theme.colors.bg)
               .stroke(Stroke::new(1.0, theme.colors.border))
               .corner_radius(theme.frame1.corner_radius)
         });

         let card = frame.show(ui, |ui| {
            let pad = theme.frame1.inner_margin.left;
            let has_heading = self.heading.is_some();
            let has_icon = self.header_icon.is_some();
            if has_heading || has_icon {
               // Header band — same horizontal padding as body,
               // tighter bottom so the optional separator + body
               // continue to read as one block.
               Frame::new()
                  .inner_margin(Margin {
                     left: pad as i8,
                     right: pad as i8,
                     top: pad as i8,
                     bottom: 0,
                  })
                  .show(ui, |ui| {
                     let needs_full_width = self.center_header || self.center_header_icon;
                     if needs_full_width {
                        ui.set_min_width(ui.available_width());
                     }

                     let stacked_icon = has_icon && (self.center_header_icon || self.center_header);
                     if stacked_icon {
                        if let Some(icon) = self.header_icon.clone() {
                           if self.center_header_icon {
                              ui.vertical_centered(|ui| {
                                 ui.add(icon);
                              });
                           } else {
                              ui.add(icon);
                           }
                        }
                     }

                     if self.center_header {
                        ui.vertical_centered(|ui| {
                           if let Some(h) = &self.heading {
                              ui.add(egui::Label::new(h.clone()).halign(Align::Center));
                           }
                           if let Some(sub) = &self.subtitle {
                              ui.add(egui::Label::new(sub.clone()).halign(Align::Center));
                           }
                        });
                     } else {
                        ui.horizontal_top(|ui| {
                           if has_icon && !stacked_icon {
                              if let Some(icon) = &self.header_icon {
                                 ui.add(icon.clone());
                                 ui.add_space(10.0);
                              }
                           }
                           ui.vertical(|ui| {
                              if let Some(h) = &self.heading {
                                 ui.add(egui::Label::new(h.clone()));
                              }
                              if let Some(sub) = &self.subtitle {
                                 ui.add(egui::Label::new(sub.clone()));
                              }
                           });
                           ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                              // A non-closable modal shows no "×":
                              // there's no user-driven way out, so
                              // an affordance would only mislead.
                              if closable {
                                 let resp = close_button(ui, &theme);
                                 if resp.clicked() {
                                    should_close = true;
                                 }
                                 close_btn_id = Some(resp.id);
                              }
                           });
                        });
                     }
                  });
               if self.header_separator {
                  ui.add_space(6.0);
                  ui.separator();
                  ui.add_space(10.0);
               } else {
                  ui.add_space(10.0);
               }
            }
            // --- Body ---
            let body_result = Frame::new()
               .inner_margin(Margin {
                  left: pad as i8,
                  right: pad as i8,
                  top: if has_heading || has_icon {
                     0
                  } else {
                     pad as i8
                  },
                  bottom: if self.footer.is_some() {
                     pad as i8 / 2
                  } else {
                     pad as i8
                  },
               })
               .show(ui, |ui| add_contents(ui))
               .inner;

            // --- Footer ---
            if let Some(footer) = self.footer {
               if self.footer_separator {
                  ui.separator();
               }
               // The recessed footer fill is painted by hand rather
               // than via the frame's own `.fill`. A plain frame
               // fill is a square-cornered rectangle flush with the
               // card edges, so it paints over the card's rounded
               // bottom corners and bottom border — the non-round
               // corners reported in issue #7. Instead we lay the
               // footer out with no fill, then drop a rounded fill
               // into a slot reserved *behind* the content, tucked
               // one pixel inside the 1px border so the border (and
               // its rounded corners) stays unbroken all the way
               // around.
               let footer_fill = theme.colors.widget_bg;
               let fill_idx = ui.painter().add(Shape::Noop);
               let footer_rect = Frame::new()
                  .fill(theme.colors.bg)
                  .inner_margin(Margin::symmetric(pad as i8, pad as i8 * 3 / 4))
                  .show(ui, |ui| {
                     ui.horizontal(|ui| {
                        if let Some(left) = self.footer_left {
                           left(ui);
                        }
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                           footer(ui);
                        });
                     });
                  })
                  .response
                  .rect;
               // Round the bottom corners one pixel tighter than the
               // card so the fill follows the inside of the border's
               // curve; leave the top flush with the divider above.
               let card_radius = theme.frame1.corner_radius.ne as f32;
               let r = (card_radius - 1.0).max(0.0) as u8;
               let fill_rect = Rect::from_min_max(
                  Pos2::new(footer_rect.left() + 1.0, footer_rect.top()),
                  Pos2::new(
                     footer_rect.right() - 1.0,
                     footer_rect.bottom() - 1.0,
                  ),
               );
               ui.painter().set(
                  fill_idx,
                  Shape::rect_filled(
                     fill_rect,
                     CornerRadius {
                        nw: 0,
                        ne: 0,
                        sw: r,
                        se: r,
                     },
                     footer_fill,
                  ),
               );
            }
            body_result
         });

         let card_rect = card.response.rect;
         ui.expand_to_include_rect(card_rect);
         #[cfg(test)]
         ui.ctx().data_mut(|d| {
            d.insert_temp(Id::new("egui_elements_test_card_rect"), card_rect);
         });

         // Centered headers skip the in-flow close button so a wide
         // body can size the card first. Pin × to that final corner.
         if closable && self.center_header {
            let pad = theme.frame1.inner_margin.left as f32;
            let close_rect = Rect::from_min_size(
               Pos2::new(
                  card_rect.right() - pad - HEADER_CLOSE_SLOT,
                  card_rect.top() + pad,
               ),
               vec2(HEADER_CLOSE_SLOT, HEADER_CLOSE_SLOT),
            );
            ui.scope_builder(
               UiBuilder::new().max_rect(close_rect).layout(Layout::right_to_left(Align::Min)),
               |ui| {
                  let resp = close_button(ui, &theme);
                  if resp.clicked() {
                     should_close = true;
                  }
                  close_btn_id = Some(resp.id);
               },
            );
         }
         card.inner
      });

      if closable && self.close_on_escape && ctx.input(|i| i.key_pressed(Key::Escape)) {
         should_close = true;
      }

      // On the first frame a modal is open, move keyboard focus into it so
      // Tab navigates within the dialog rather than the background. We
      // target the close button when a heading is present (it has a
      // stable id and is always interactive); without a heading there's
      // no intrinsic focus target, so focus is left to the caller.
      if just_opened && let Some(id) = close_btn_id {
         ctx.memory_mut(|m| m.request_focus(id));
      }

      if should_close {
         *self.open = false;
         // Restore focus and clear the lifecycle state right now — in the
         // same frame the close is triggered — rather than deferring to the
         // `was_open && !is_open` branch on a subsequent `show()`. Callers
         // routinely drop the modal the instant it closes (the natural
         // `if let Some(m) = &self.modal { … }` + `self.modal = None`
         // pattern), so `show()` is never called again. A deferred cleanup
         // would then silently leak: focus is never returned to the
         // pre-modal widget, and the stale `was_open = true` defeats the
         // just-opened focus grab the next time a modal with this salt opens.
         if let Some(prev) = focus_state.prev_focus {
            ctx.memory_mut(|m| m.request_focus(prev));
         }
         ctx.data_mut(|d| d.insert_temp(focus_storage, ModalFocusState::default()));
      }

      Some(result.inner)
   }
}

/// Persistent focus-lifecycle state for a single `Modal`, keyed by the
/// modal's `id_salt`. Stored via `ctx.data_mut`.
#[derive(Clone, Copy, Default, Debug)]
struct ModalFocusState {
   /// Whether the modal was rendered open last frame. Used to detect
   /// open/close transitions.
   was_open: bool,
   /// Which widget (if any) had keyboard focus at the moment the modal
   /// opened. Restored on close.
   prev_focus: Option<Id>,
}

/// Overlay slot for the close button in a centered header. Sized to
/// fit the 20×20 min button plus theme padding without clipping.
const HEADER_CLOSE_SLOT: f32 = 48.0;

/// Render the modal's close button. Returns its `Response` so the caller
/// can route focus to it and check `clicked()`. The accesskit label is
/// set to `"Close"` explicitly — without this, screen readers announce
/// the "×" glyph literally as "multiplication sign."
///
/// The button is scoped under a stable id (`"elegance_modal_close"`) so
/// focus requests targeting it survive layout changes.
fn close_button(ui: &mut Ui, theme: &Theme) -> Response {
   let inner = ui
      .push_id("modal_close", |ui| {
         let text = RichText::new("X").size(theme.typography.normal);
         let padding = vec2(theme.spacing.sm, theme.spacing.xs);
         ui.spacing_mut().button_padding = padding;
         ui.add(Button::new(text).min_size(vec2(20.0, 20.0)))
      })
      .inner;
   #[cfg(test)]
   ui.ctx().data_mut(|d| {
      d.insert_temp(
         Id::new("egui_elements_test_close_rect"),
         inner.rect,
      );
   });
   let enabled = inner.enabled();
   inner.widget_info(|| WidgetInfo::labeled(WidgetType::Button, enabled, "Close"));
   inner
}

#[cfg(test)]
mod tests {
   use super::*;

   #[test]
   fn center_header_without_separators_renders() {
      egui::__run_test_ctx(|ctx| {
         let mut open = true;
         Modal::new("modal_opts", &mut open)
            .heading("Title")
            .subtitle("Subtitle")
            .header_icon(Image::new("file://modal_icon"))
            .center_header(true)
            .center_header_icon(true)
            .header_separator(false)
            .footer_separator(false)
            .footer(|ui| {
               ui.label("ok");
            })
            .show(ctx, |ui| {
               ui.label("body");
            });
         assert!(open);
      });
   }

   #[test]
   fn default_header_layout_still_renders() {
      egui::__run_test_ctx(|ctx| {
         let mut open = true;
         Modal::new("modal_default", &mut open)
            .heading("Title")
            .footer(|ui| {
               ui.label("ok");
            })
            .show(ctx, |ui| {
               ui.label("body");
            });
         assert!(open);
      });
   }

   #[test]
   fn centered_close_button_hugs_card_right() {
      egui::__run_test_ctx(|ctx| {
         let mut open = true;
         Modal::new("modal_corner", &mut open)
            .heading("Title")
            .center_header(true)
            .max_width(500.0)
            .show(ctx, |ui| {
               ui.set_min_width(480.0);
               ui.label("wide body");
            });
         let close: Rect = ctx
            .data(|d| d.get_temp(Id::new("egui_elements_test_close_rect")))
            .expect("close button rect");
         let card: Rect = ctx
            .data(|d| d.get_temp(Id::new("egui_elements_test_card_rect")))
            .expect("card rect");
         let gap = card.right() - close.right();
         assert!(
            gap >= 0.0 && gap < 56.0,
            "close button should hug the card's right edge, gap={gap}, close={close:?}, card={card:?}"
         );
         assert!(
            close.right() > card.center().x,
            "close button should be on the right half of the card"
         );
      });
   }

   fn run_two_passes(mut f: impl FnMut(&Context)) {
      let ctx = Context::default();
      ctx.set_fonts(egui::FontDefinitions::empty());
      for _ in 0..2 {
         ctx.run_ui(Default::default(), |ui| f(ui.ctx())).drop_without_applying_deltas();
      }
   }

   #[test]
   fn modal_centers_without_max_width() {
      let mut last: Option<(Rect, Rect)> = None;
      run_two_passes(|ctx| {
         let mut open = true;
         Modal::new("modal_center", &mut open).heading("Title").show(ctx, |ui| {
            ui.set_min_width(520.0);
            ui.label("wide body");
         });
         let card: Rect = ctx
            .data(|d| d.get_temp(Id::new("egui_elements_test_card_rect")))
            .expect("card rect");
         last = Some((card, ctx.content_rect()));
      });
      let (card, screen) = last.expect("second pass");
      let dx = (card.center().x - screen.center().x).abs();
      let dy = (card.center().y - screen.center().y).abs();
      assert!(
         dx < 8.0 && dy < 8.0,
         "card should be centered without max_width, dx={dx}, dy={dy}, card={card:?}, screen={screen:?}"
      );
   }

   #[test]
   fn modal_centers_with_max_width() {
      let mut last: Option<(Rect, Rect)> = None;
      run_two_passes(|ctx| {
         let mut open = true;
         Modal::new("modal_center_cap", &mut open)
            .heading("Title")
            .max_width(600.0)
            .show(ctx, |ui| {
               ui.set_min_width(520.0);
               ui.label("wide body");
            });
         let card: Rect = ctx
            .data(|d| d.get_temp(Id::new("egui_elements_test_card_rect")))
            .expect("card rect");
         last = Some((card, ctx.content_rect()));
      });
      let (card, screen) = last.expect("second pass");
      let dx = (card.center().x - screen.center().x).abs();
      let dy = (card.center().y - screen.center().y).abs();
      assert!(
         dx < 8.0 && dy < 8.0,
         "card should be centered with max_width, dx={dx}, dy={dy}, card={card:?}, screen={screen:?}"
      );
   }
}
