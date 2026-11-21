//! Modal dialog — a centered themed card over a dimmed backdrop.
//!
//! Painted in two layers: a full-viewport dimmed backdrop that swallows
//! clicks (and closes the modal when clicked), and a centered card-like
//! window with an optional heading row and a close "×" button.
//! Press `Esc` to dismiss, whether or not the dialog is closable.
//!
//! Forked from <https://github.com/stephenberry/egui-elegance>

use egui::{
   Align, Align2, Area, Color32, Context, CornerRadius, Frame, Id, Image, Key, Layout, Margin,
   Order, Pos2, Rect, Response, RichText, Role, Sense, Shape, Stroke, Ui, UiBuilder, Vec2,
   WidgetInfo, WidgetText, accesskit, vec2,
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
///
/// # Accessibility
///
/// While it is open the modal owns egui's modal layer, so nothing behind it
/// can take keyboard focus: `Tab` stays inside the dialog and a screen reader
/// cannot reach the page underneath. The dialog exposes
/// `accesskit::Role::Dialog` (or `accesskit::Role::AlertDialog` with
/// [`Modal::alert`]), named by [`Modal::accessible_name`] when that is set and by
/// the heading otherwise.
///
/// `Esc` closes the modal whenever [`Modal::close_on_escape`] is `true` — the
/// default, and independent of [`Modal::closable`], so a dialog with no "×"
/// still has a keyboard exit. When it opens, focus moves to
/// [`Modal::initial_focus`], falling back to the close "×" button.
#[must_use = "Call `.show(ctx, |ui| { ... })` to render the modal."]
pub struct Modal<'a> {
   id_salt: Id,
   heading: Option<WidgetText>,
   subtitle: Option<WidgetText>,
   accessible_name: Option<String>,
   header_icon: Option<Image<'a>>,
   frame: Option<Frame>,
   center_header_icon: bool,
   backdrop_order: Order,
   content_order: Order,
   open: &'a mut bool,
   max_width: Option<f32>,
   width: Option<f32>,
   closable: bool,
   close_on_backdrop: bool,
   close_on_escape: bool,
   initial_focus: Option<Id>,
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
         .field("accessible_name", &self.accessible_name)
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
         .field("width", &self.width)
         .field("closable", &self.closable)
         .field("close_on_backdrop", &self.close_on_backdrop)
         .field("close_on_escape", &self.close_on_escape)
         .field("initial_focus", &self.initial_focus)
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

/// Id source accepted by [`Modal::new`].
///
/// Reproduces exactly the ids this crate produced against egui 0.36, when
/// `&str` converted to [`Id`] and the constructor hashed the input a second
/// time. Implemented for the two forms callers used: an [`Id`] and a string.
///
/// The string form is hashed **twice** on purpose: in egui 0.36,
/// `&str: Into<Id>` already called `Id::new(s)`, and the modal constructor
/// then called `Id::new` on that id — two hashes. An [`Id`] passes through
/// `Into<Id>` unchanged, so it is hashed once. "Simplifying" the string
/// impl to a single hash would move every existing modal to a new id, and
/// with it all of its persisted state.
pub trait IntoModalId {
   fn into_modal_id(self) -> Id;
}

impl IntoModalId for Id {
   fn into_modal_id(self) -> Id {
      Id::unique(self)
   }
}

impl IntoModalId for &str {
   fn into_modal_id(self) -> Id {
      Id::unique(Id::unique(self))
   }
}

impl IntoModalId for String {
   fn into_modal_id(self) -> Id {
      Id::unique(Id::unique(self))
   }
}

impl<'a> Modal<'a> {
   /// Create a modal keyed by `id_salt` whose visibility is bound to `open`.
   ///
   /// `id_salt` accepts the same forms callers used before egui 0.37 — see
   /// [`IntoModalId`], which reproduces the historical ids exactly.
   pub fn new(id: impl IntoModalId, open: &'a mut bool) -> Self {
      Self {
         id_salt: id.into_modal_id(),
         heading: None,
         subtitle: None,
         accessible_name: None,
         header_icon: None,
         frame: None,
         center_header_icon: false,
         backdrop_order: Order::Middle,
         content_order: Order::Foreground,
         open,
         max_width: None,
         width: None,
         closable: true,
         close_on_backdrop: true,
         close_on_escape: true,
         initial_focus: None,
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

   /// The accessible name the dialog exposes to assistive technology when it
   /// has no visible [`Modal::heading`] — and the name it exposes *instead of*
   /// the heading when one is set.
   ///
   /// A dialog with neither has no name at all: a screen reader announces the
   /// role and nothing else, which is why a body-only confirmation should name
   /// itself here. Default: the heading's text.
   pub fn accessible_name(mut self, label: impl Into<String>) -> Self {
      self.accessible_name = Some(label.into());
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

   /// Exact card width in points.
   ///
   /// The card fills this width and stays centered; it is clamped to the screen minus 32
   /// points. Takes precedence over [`Self::max_width`]. Default: shrink-wrap to the content.
   pub fn width(mut self, width: f32) -> Self {
      self.width = Some(width);
      self
   }

   /// Whether the user may dismiss the modal with the close "×" button or a
   /// backdrop click. When `false`, the "×" is hidden and
   /// [`Modal::close_on_backdrop`] is ignored.
   ///
   /// `Esc` is not part of this: it closes the modal whenever
   /// [`Modal::close_on_escape`] is `true` (the default), closable or not — a
   /// keyboard user always needs a way out. Use `.closable(false)` against
   /// `.close_on_escape(false)` for a dialog no key dismisses.
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

   /// Whether pressing `Esc` dismisses the modal, independently of
   /// [`Modal::closable`]: it closes a `.closable(false)` dialog too, so the
   /// keyboard always has an exit. Default: `true`.
   pub fn close_on_escape(mut self, close: bool) -> Self {
      self.close_on_escape = close;
      self
   }

   /// Which widget receives keyboard focus when the dialog opens. Pass a
   /// widget id — the `response.id` of a field drawn in the body, or
   /// `Id::unique(..)` for a widget created through a fixed scope.
   ///
   /// Default: the close "×" button, and nothing at all when the dialog has no
   /// close button ([`Modal::closable`]`(false)`) — a body-only dialog should
   /// point at its first control. The request is made on the first *painted*
   /// pass, so the invisible sizing pass a shrink-wrapping modal takes cannot
   /// withdraw it.
   pub fn initial_focus(mut self, id: impl Into<Id>) -> Self {
      self.initial_focus = Some(id.into());
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
      let focus_storage = Id::unique(("elegance_modal_focus", self.id_salt));
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
         focus_state.focus_pending = true;
         focus_state.pending_focus = self.initial_focus;
         ctx.data_mut(|d| d.insert_temp(focus_storage, focus_state));
      }

      let theme = Theme::current(ctx);
      let mut should_close = false;
      let mut close_btn_id: Option<Id> = None;
      let closable = self.closable;

      let window_id = Id::unique("elegance_modal_window").with(self.id_salt);
      let measure_key = window_id.with("egui_elements_modal_measure");
      let mut measure: ModalMeasure = ctx.data(|d| d.get_temp(measure_key)).unwrap_or_default();
      let measuring = self.max_width.is_none()
         && self.width.is_none()
         && measure.passes < SHRINK_WRAP_MEASURE_PASSES;
      let measure_width = if measure.passes == 0 {
         0.0
      } else {
         measure.width
      };

      // --- Areas -------------------------------------------------------
      // Both areas are built before either is shown: the modal layer must be
      // known before the content area runs, and it must be the lower of the
      // two. egui only lets widgets on the modal layer and above it take
      // input, so a higher modal layer would drop the backdrop below it and
      // the click that dismisses the modal would never arrive.
      let screen = ctx.content_rect();
      let backdrop_id = Id::unique("elegance_modal_backdrop").with(self.id_salt);
      let backdrop_area = Area::new(backdrop_id)
         .fixed_pos(screen.min)
         .order(self.backdrop_order)
         .sizing_pass(measuring);
      let mut area = Area::new(window_id)
         .order(self.content_order)
         .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
         .pivot(Align2::CENTER_CENTER);
      let backdrop_layer = backdrop_area.layer();
      let content_layer = area.layer();
      let modal_layer = if backdrop_layer.order <= content_layer.order {
         backdrop_layer
      } else {
         content_layer
      };
      ctx.memory_mut(|m| m.set_modal_layer(modal_layer));

      // --- Backdrop ----------------------------------------------------
      let backdrop = backdrop_area.show(ctx, |ui| {
         ui.painter().rect_filled(
            screen,
            CornerRadius::ZERO,
            Color32::from_rgba_premultiplied(0, 0, 0, 150),
         );
         // `Sense::CLICK`, not `Sense::click()`: the latter adds FOCUSABLE, which
         // would make the full-screen backdrop a tab stop inside the modal layer -
         // an unnamed, full-screen focus target with nothing to activate. It still
         // senses the click that dismisses the dialog.
         ui.allocate_rect(screen, Sense::CLICK)
      });
      if closable && self.close_on_backdrop && backdrop.inner.clicked() {
         should_close = true;
      }

      // --- Content -----------------------------------------------------
      let alert = self.alert;
      // The dialog's accessible name: an explicit `.accessible_name(..)` wins
      // over the heading, and a dialog with neither has no name to announce.
      let dialog_label = self
         .accessible_name
         .clone()
         .or_else(|| self.heading.as_ref().map(|h| h.text().to_string()));
      // Shrink-wrap (no `max_width`) has to measure the content before it
      // can anchor, and the first measurement is wrong: egui's own first-pass
      // sizing pass measures at width 0, where the body wraps into a too-tall
      // box, so the next pass would anchor to that stale height — the
      // one-frame off-centre jump. Spend one more invisible sizing pass, at
      // the width the first pass measured, to get the true height; the pass
      // after it is the first painted frame and is already centred. The
      // backdrop is gated on this same flag so the dim overlay cannot appear
      // a frame before the dialog does.
      // `max_width(Some(_))` callers keep the single-pass behaviour.
      let fixed_width = self
         .width
         .or(self.max_width)
         .map(|width| width.min(ctx.content_rect().width() - 32.0));
      area = match fixed_width {
         Some(w) => area.default_width(w),
         None if measuring => area.default_width(measure_width).sizing_pass(true),
         None => area.default_width(0.0),
      };
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
         let label = dialog_label.clone();
         ui.ctx().accesskit_node_builder(ui.unique_id(), |node| {
            node.set_role(role);
            if let Some(label) = label {
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
         // Content is laid out and measured: remember it so the next pass
         // paints instead of taking another (invisible) sizing pass.
         if measuring {
            measure.passes += 1;
            measure.width = card_rect.width();
            ui.ctx().data_mut(|d| d.insert_temp(measure_key, measure));
         }
         #[cfg(test)]
         ui.ctx().data_mut(|d| {
            d.insert_temp(
               Id::unique("egui_elements_test_card_rect"),
               card_rect,
            );
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

      if self.close_on_escape && ctx.input(|i| i.key_pressed(Key::Escape)) {
         should_close = true;
      }

      // Move keyboard focus into the dialog so Tab starts inside it. An
      // explicit [`Modal::initial_focus`] wins; otherwise the close "×" takes
      // focus, and a dialog without one leaves focus to the caller. The
      // request waits for a painted pass: egui withdraws a focus request made
      // for a pass that only measures, and the request is cleared once made so
      // it cannot pull focus back on later frames.
      if focus_state.focus_pending && !measuring {
         if let Some(id) = focus_state.pending_focus.or(close_btn_id) {
            ctx.memory_mut(|m| m.request_focus(id));
         }
         focus_state.focus_pending = false;
         focus_state.pending_focus = None;
         ctx.data_mut(|d| d.insert_temp(focus_storage, focus_state));
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
   /// Whether the opening focus grab still has to run. It is deferred to the
   /// first painted pass, because a request made on a sizing pass is
   /// withdrawn before any widget can take it.
   focus_pending: bool,
   /// The widget that grab targets ([`Modal::initial_focus`]). `None` falls
   /// back to the close button.
   pending_focus: Option<Id>,
}

/// Number of invisible sizing passes a shrink-wrapping modal spends before
/// its first painted frame: one at width 0 (egui's own first pass measures
/// there) and one at the width that pass measured, which is the only way to
/// learn the card's real height before it is anchored.
const SHRINK_WRAP_MEASURE_PASSES: u32 = 2;

/// Progress of a shrink-wrapping modal through those sizing passes.
#[derive(Clone, Copy, Default, Debug)]
struct ModalMeasure {
   /// How many sizing passes have run so far.
   passes: u32,
   /// Card width measured by the last sizing pass.
   width: f32,
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
         Id::unique("egui_elements_test_close_rect"),
         inner.rect,
      );
   });
   let enabled = inner.enabled();
   inner.widget_info(|| WidgetInfo::labeled(Role::Button, enabled, "Close"));
   inner
}

#[cfg(test)]
mod tests {
   use super::*;
   use egui::accesskit::Node;
   use egui::{Event, FullOutput, Modifiers, PointerButton, RawInput};

   fn run_until_painted(mut f: impl FnMut(&Context)) {
      let ctx = Context::default();
      ctx.set_fonts(egui::FontDefinitions::empty());
      // Three passes: egui's own width-0 sizing pass, our sizing pass at the
      // measured width, then the first painted frame.
      for _ in 0..3 {
         ctx.run_ui(Default::default(), |ui| f(ui.ctx())).drop_without_applying_deltas();
      }
   }

   /// The three passes a shrink-wrapping modal takes, with `input` delivered on
   /// each; returns the painted frame's output.
   fn painted_output(
      ctx: &Context,
      input: impl Fn() -> RawInput,
      mut f: impl FnMut(&Context),
   ) -> FullOutput {
      for _ in 0..2 {
         ctx.run_ui(input(), |ui| f(ui.ctx())).drop_without_applying_deltas();
      }
      ctx.run_ui(input(), |ui| f(ui.ctx()))
   }

   /// The AccessKit tree the painted frame produced.
   fn painted_accesskit_nodes(mut f: impl FnMut(&Context)) -> Vec<Node> {
      let ctx = Context::default();
      ctx.set_fonts(egui::FontDefinitions::empty());
      ctx.enable_accesskit();
      let mut output = painted_output(&ctx, RawInput::default, |ctx| f(ctx));
      let update = output
         .platform_output
         .accesskit_update
         .take()
         .expect("an AccessKit-enabled painted frame produces a tree update");
      output.drop_without_applying_deltas();
      update.nodes.into_iter().map(|(_, node)| node).collect()
   }

   fn node_with_role(nodes: &[Node], role: Role) -> &Node {
      nodes
         .iter()
         .find(|node| node.role() == role)
         .expect("no node with the expected role")
   }

   fn escape_press() -> Event {
      Event::Key {
         key: Key::Escape,
         physical_key: None,
         pressed: true,
         repeat: false,
         modifiers: Modifiers::NONE,
      }
   }

   fn pointer(pos: Pos2, pressed: bool) -> Event {
      Event::PointerButton {
         pos,
         button: PointerButton::Primary,
         pressed,
         modifiers: Modifiers::NONE,
      }
   }

   #[test]
   fn modal_centers_with_max_width() {
      let mut last: Option<(Rect, Rect)> = None;
      run_until_painted(|ctx| {
         let mut open = true;
         Modal::new("modal_center_cap", &mut open)
            .heading("Title")
            .max_width(600.0)
            .show(ctx, |ui| {
               ui.set_min_width(520.0);
               ui.label("wide body");
            });
         let card: Rect = ctx
            .data(|d| d.get_temp(Id::unique("egui_elements_test_card_rect")))
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

   #[test]
   fn modal_takes_an_exact_width() {
      let mut width = 0.0;
      run_until_painted(|ctx| {
         let mut open = true;
         Modal::new("modal_exact_width", &mut open)
            .heading("Title")
            .width(520.0)
            .show(ctx, |ui| {
               ui.label("short body");
            });

         let card: Rect = ctx
            .data(|d| d.get_temp(Id::unique("egui_elements_test_card_rect")))
            .expect("card rect");
         width = card.width();
      });

      assert!((width - 520.0).abs() <= 2.0, "card width {width}");
   }

   #[test]
   fn accessible_name_names_a_dialog_without_a_heading() {
      let nodes = painted_accesskit_nodes(|ctx| {
         let mut open = true;
         let _ = Modal::new("modal_accessible_name", &mut open)
            .accessible_name("Delete all contacts")
            .show(ctx, |ui| {
               ui.label("…");
            });
      });

      let node = node_with_role(&nodes, Role::Dialog);
      assert_eq!(node.label(), Some("Delete all contacts"));
   }

   #[test]
   fn a_heading_names_the_dialog_by_default() {
      let nodes = painted_accesskit_nodes(|ctx| {
         let mut open = true;
         let modal =
            Modal::new("modal_accessibility_heading", &mut open).heading("Discard changes?");
         let _ = modal.show(ctx, |ui| {
            ui.label("…");
         });
      });

      let node = node_with_role(&nodes, Role::Dialog);
      assert_eq!(node.label(), Some("Discard changes?"));
   }

   #[test]
   fn a_non_closable_modal_still_closes_on_escape() {
      let ctx = Context::default();
      ctx.set_fonts(egui::FontDefinitions::empty());
      let mut open = true;
      painted_output(
         &ctx,
         || RawInput {
            events: vec![escape_press()],
            ..RawInput::default()
         },
         |ctx| {
            let modal = Modal::new("modal_escape_non_closable", &mut open).closable(false);
            let _ = modal.show(ctx, |ui| {
               ui.label("blocking");
            });
         },
      )
      .drop_without_applying_deltas();

      // The dialog hides its "×", so `Esc` is its only way out.
      assert!(!open);
   }

   #[test]
   fn escape_is_ignored_when_close_on_escape_is_off() {
      let ctx = Context::default();
      ctx.set_fonts(egui::FontDefinitions::empty());
      let mut open = true;
      painted_output(
         &ctx,
         || RawInput {
            events: vec![escape_press()],
            ..RawInput::default()
         },
         |ctx| {
            let modal = Modal::new("modal_escape_off", &mut open).close_on_escape(false);
            let _ = modal.show(ctx, |ui| {
               ui.label("…");
            });
         },
      )
      .drop_without_applying_deltas();

      // `.close_on_escape(false)` is the host's "keep it open" switch.
      assert!(open);
   }

   #[test]
   fn a_backdrop_click_still_closes_a_closable_modal() {
      let ctx = Context::default();
      ctx.set_fonts(egui::FontDefinitions::empty());
      let mut open = true;
      // A corner of the screen: outside the card, inside the backdrop.
      let at = ctx.content_rect().left_top() + vec2(4.0, 4.0);
      // Three passes paint the dialog and register the backdrop's rect, then a
      // hover, a press and a release — a click reports on the release frame.
      let frames: [Vec<Event>; 6] = [
         vec![],
         vec![],
         vec![],
         vec![Event::PointerMoved(at)],
         vec![pointer(at, true)],
         vec![pointer(at, false)],
      ];
      for events in frames {
         let input = RawInput {
            events,
            ..RawInput::default()
         };
         ctx.run_ui(input, |ui| {
            let _ = Modal::new("modal_backdrop_click", &mut open).show(ui.ctx(), |ui| {
               ui.label("body");
            });
         })
         .drop_without_applying_deltas();
      }

      // A click on the backdrop must still dismiss a closable modal.
      assert!(!open);
   }

   #[test]
   fn the_modal_layer_is_set_while_the_modal_is_open() {
      let ctx = Context::default();
      ctx.set_fonts(egui::FontDefinitions::empty());
      let mut open = true;
      painted_output(&ctx, RawInput::default, |ctx| {
         let _ = Modal::new("modal_layer", &mut open).show(ctx, |ui| {
            ui.label("body");
         });
      })
      .drop_without_applying_deltas();

      // egui promotes the layer at the end of the pass, so the painted frame
      // is enough to observe it.
      let layer = ctx.memory(|m| m.top_modal_layer()).expect("an open modal sets a modal layer");
      // The lower of the two areas: the backdrop's `Middle`, not the card's
      // `Foreground` — a higher layer would put the backdrop out of reach.
      assert_eq!(layer.order, Order::Middle);

      open = false;
      painted_output(&ctx, RawInput::default, |ctx| {
         let _ = Modal::new("modal_layer", &mut open).show(ctx, |ui| {
            ui.label("body");
         });
      })
      .drop_without_applying_deltas();

      assert_eq!(ctx.memory(|m| m.top_modal_layer()), None);
   }

   #[test]
   fn initial_focus_takes_the_caret_when_the_dialog_opens() {
      let ctx = Context::default();
      ctx.set_fonts(egui::FontDefinitions::empty());
      let mut open = true;
      let target: std::cell::Cell<Option<Id>> = std::cell::Cell::new(None);
      let seen: std::cell::RefCell<Vec<Id>> = std::cell::RefCell::new(Vec::new());

      let run = |ctx: &Context, open: &mut bool| {
         let focus = target.get();
         let mut modal = Modal::new("modal_initial_focus", open);
         if let Some(id) = focus {
            modal = modal.initial_focus(id);
         }
         let _ = modal.show(ctx, |ui| {
            let response = ui.add(egui::Button::new("Rename"));
            target.set(Some(response.id));
            seen.borrow_mut().push(response.id);
         });
      };

      // The first open only exists to learn the body widget's id.
      painted_output(&ctx, RawInput::default, |ctx| run(ctx, &mut open))
         .drop_without_applying_deltas();
      let target_id = target.get().expect("the body's widget id");

      // Close, then reopen with `.initial_focus(..)` — the host's situation.
      open = false;
      painted_output(&ctx, RawInput::default, |ctx| run(ctx, &mut open))
         .drop_without_applying_deltas();
      open = true;
      painted_output(&ctx, RawInput::default, |ctx| run(ctx, &mut open))
         .drop_without_applying_deltas();

      let ids = seen.into_inner();
      assert!(
         ids.iter().all(|id| *id == target_id),
         "the body widget keeps one id across the close and reopen, got {ids:?}"
      );
      assert_eq!(ctx.memory(|m| m.focused()), Some(target_id));
   }
}
