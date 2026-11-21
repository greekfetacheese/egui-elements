# Changelog

All notable changes to `egui-elements`, newest first. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the crate follows semantic versioning.

## [Unreleased]

Everything below landed after the 1.4.0 release (`4c663ae`, 2026-10-25). **The version is still 1.4.0
on purpose**: bumping it, tagging and publishing is the release step, not part of the feature work.

### Added

**Layout**

- `Axis`, `Justify`, `CrossAlign`, `ItemSize`, `ItemSpec` and `resolve` — the pure axis solver behind
  the flex containers.
- `Flex` / `Item` — a CSS-flexbox subset: `grow`, `Fixed` / `Auto` / `Fr`, gap, align and justify.
- `Table`, `TableHeader`, `TableRow`, `TableScroll` / `TableUi` — data-driven `fixed` / `auto` / `fr`
  columns, a pinned header, viewport-only rows, row dividers and per-cell alignment.

**Motion**

- `Motion`, `Easing`, `Lerp`, `animate`, `animate_with`, `animate_from` — eased values on the style clock.
- `InteractT` — eased idle / hover / press weights shared by the themed widgets.
- `Fade`, `FadeMode`, `Collapse` — fade and measured-height reveal containers.

**Widgets**

- `Row` — a full-width product row: leading icon, title, optional subtitle, trailing control and chevron
  on the themed hover/selection plate.
- `Popover` / `PopoverSide` — an anchored overlay that owns its surface, placement and dismissals;
  with `.surface(Surface)`, `.inner_margin(Margin)`, `.align(Align2)` and `.screen_margin(f32)`.
- `Checkbox`, `Radio`, `Switch` with the shared `ToggleVisuals` / `ToggleShadow` (the historic
  `CheckboxVisuals` / `CheckboxShadow` names stay as aliases).
- `Button::icon(image)` — the chrome-less icon form: no font-height clamp and symmetric padding.
- `Chip` / `ChipRole` — an auto-width chip.
- `Slider` / `SliderVisuals` — the Web Awesome slider.
- `Badge`, `BadgeAnchor`, `BadgeCorner`, `BadgePulse`, `BadgeBounce`, `BadgeBounceTrigger` — inside and
  straddling badges with a pulsing halo and a one-shot hop.
- `Indicator` / `IndicatorState` — a status dot with an optional pulse and an optional outline
  (`.ring(Stroke)`).
- `Label::text(text)` — the common label constructor without a `None` image argument.
- `Avatar` / `AvatarGroup` / `AvatarVisuals` — an overlapping stack that fans out on hover.
- `Toast` / `Toasts` — a toast stack with auto-dismiss and a persistent variant.
- `Menu`, `MenuItem`, `MenuSection`, `SubMenuItem` — the forked menu family.
- `FpsMeter` / `FpsStats` / `Spread` — an FPS and CPU frame-time window.
- `Spinner` / `Ripple` / `Orb` / `SpinnerShadow` — the loader family.
- `TabsStyle::Underline` and `TabsVisuals::underline_from_theme` — the fitted underline strip beside the
  segmented one; `Tabs::text_size` sets the label size and the strip's default height.
- `ComboBox::show_at` / `ComboBox::option` — place the button at an exact rect, and build the standard
  option row.
- `MenuItem::image` — a Lucide (or any) image in the leading gutter, beside `.icon()`.
- `Modal::max_width`, `Frame::fit` and the `TextEdit` / `TextEditOutput` names for the themed field.
- `Modal::accessible_name` — the dialog's accessible name, for the body-only dialogs that had none: it is used
  when there is no visible heading, and overrides the heading when there is one.
- `Modal::initial_focus` — the widget that takes keyboard focus when the dialog opens, ahead of the close
  "×" it falls back to.
- `Frame::to_egui()` and `From` conversions between `Frame` and `egui::Frame` — the static chrome
  egui can paint (a gradient collapses to its midpoint; hover / click state does not cross over).
- `Frame`'s accessibility surface — `.accessible_name(``'static str``)`, `.labelled_by(Id)`,
  `.accessible_description(``'static str``)`, `.described_by(Id)`, `.accessible_role(..)` and
  `.hide_from_accessibility(..)` put one AccessKit node on the plate: an interactive frame is a
  `Button` (with `.selected(true)` reported as toggled), a named static frame is a `Label` readout,
  and a frame with nothing to say stays out of the tree. Names and descriptions are either static
  strings or references to a node the host already draws, so naming allocates nothing and `Frame`
  stays `Copy`.
- `Row`'s accessibility surface — a row is named "Title — Subtitle", so a readout row whose meaning
  rides the second line ("Load now" over "0.4, rising") is announced whole instead of as a bare title;
  `.read_only(true)` turns the row into a `Label` readout with a hover sense instead of a clickable
  `Button`, and a read-only row is not in the tab order.
- `Indicator::label(..)` — names what a light reports ("RPC: connected", "Delegation: delegated"),
  instead of leaving every light on a row saying "status: on".
- `Checkbox::name(..)` — the accessible name of the `without_text` form; a visible text always wins.
- The slider's AccessKit node carries the formatted value ("0.75 s"), so a reader speaks what the
  screen shows instead of the bare number, while the numeric value, the step and the `SetValue`
  action stay as they were.
- `CredentialsForm` announces the focused password field's length through its own `Announcer`, and a
  plain field's empty state (a reader key-echoes its characters already; its emptiness nothing else
  reports), so a host gets the feedback without wiring anything: `with_speech` / `set_speech` (also on
  `SecureInputField`) choose what assistive technology receives, and the row labels now name their
  input.
- `CredentialsForm::with_announce_text` / `set_announce_text` / `clear_announce_text` — replace the
  built-in wording with a closure over `Announcement` (focused field, character count, mask state),
  e.g. to append the attempts left or to speak another language; `Announcement::default_text` keeps
  the built-in sentence (`Display` prints it). The closure sees a count, never the secret.
- `Announcer` — a `Status` node marked `Live::Polite` whose text changes a screen reader speaks
  (AccessKit turns the name change into an AT-SPI announcement); an empty text keeps the node in the
  tree and says nothing. It exists because a text field's value does not reach the AT-SPI text
  interface, so a password field is otherwise silent under a reader that suppresses key echo.
- `SecureSpeech` / `SecureTextEdit::speech` — what a password field hands to assistive technology:
  the mask by default (one mask character per character) instead of the previous empty value, and the
  plaintext only when the host opts in with `SecureSpeech::Plaintext` (which drops the password role,
  since a password role lets the platform withhold the value). The masked field keeps that role on
  purpose: it is what keeps a reader's key echo — which otherwise reads every typed character of an
  ordinary text input — out of the field.
- `SecureTextEdit::accessible_name` / `accessible_hint` — the accessible name a reader announces for the field,
  and a spoken placeholder that can differ from the drawn `hint_text`. Without a name a field is
  announced as "text, empty"; the name stays opt-in, so a field with a visible label beside it keeps
  using `labelled_by`.
- `Button::alt(text)` — the text alternative for an **icon-only** button. A button built from an image
  and no text reaches the accessibility tree as a role with no name, so a reader says only "button";
  the alternative names the action (`.alt("Refresh")`) and the button's own text always wins over it.
  In a debug build an image-only button with neither now trips a `debug_assert!` in `atom_ui`.
- `QrImage::alt(text)` / `QrImage::hidden()` / `QrImage::show(ui)` — the text alternative, the switch
  that keeps the symbol out of the accessibility tree, and the drawing call that owns the node: it
  clears the label and description egui derives from the texture, so a pending QR is never announced
  as "Loading bytes://…" (a symbol encoding a secret would otherwise be spoken by its texture URI).
  `image()` stays the drawing method for a custom size and names the node through egui's `alt_text`.
- The crate's own chrome is named: the secure field's eye toggle is "Show credentials" / "Hide
  credentials" by masking state, its QR button is "Scan a QR code", and the demo's bell is
  "Notifications".
- `SecureInputField::mask_toggle` / `set_mask_toggle` / `close_qr_scanner` and
  `CredentialsForm::with_mask_toggle` / `set_mask_toggle` / `with_qr_enabled` / `set_qr_enabled` —
  a host can drop the visual-only show/hide and QR controls it cannot use (each costs a screen-reader
  user a focus stop and can never help them), per field or for all three credential rows in one call.
  Both flags default to `true`, so nothing changes unless a host opts in; disabling the QR flag also
  stops an overlay that was already open.

**Theme**

- `Surface`, `SurfaceState` and `Elevation` — the named surface ladder and the shadow steps behind
  `Theme::surface_frame`, `surface_fill`, `surface_visuals`, `state_fill`, `state_ink` and `shadow`.
- `ColorRole`, `ColorSlot` and `DerivedColor` — named palette roles, slot iteration and checked ink
  (`Theme::ink_on`) for chromatic fills.
- `Gradient`, `GradientDir` and `Fill` — every themed fill is now a flat colour or a two-stop gradient;
  `ThemeVariant::Gradient`, `Theme::gradient_strength` and `Theme::with_gradient_dir` drive it.
- `ThemeKind::Custom` — a runtime-named theme that keeps its palette when serialized.
- `Theme::shadow_scaled(elevation, strength)` — the elevation shadow with a caller-supplied strength,
  for a card that needs more lift than its step allows.
- `utils::with_alpha` / `utils::hex` — the two colour helpers every host otherwise writes.

### Changed

- Every themed widget eases its hover and press chrome through `Motion` / `InteractT`; selection stays
  instant.
- `ComboBox` gained an elevated popup surface, `.popup_gap`, `.popup_content_margin`, an animated
  disclosure chevron and the virtualized `show_rows_ui`.
- `Popover::width` now fills the card to that width, so a picker's content no longer shrink-wraps inside
  a wider card.
- `Table` pins its header, gives every row one pitch and lays out only the rows the viewport shows.
- `Tabs` can paint a fitted underline strip, and a click keeps its wash until release.
- The theme editor gained a live preview, naming, save/load and theme-following chrome; its host IO hooks
  are `Send + Sync`.
- Scroll-bar geometry moved onto the theme and `Theme::install` pins it, so every `egui::ScrollArea`
  draws the same bar.
- Documentation was rewritten as a landing-page README plus one reference page per area under `docs/`.
- `CredentialsForm`'s announcement hook is `Send + Sync`, so a host can keep the form inside a
  `static`. The crate's other host-IO hooks already carried the bound.
- `Modal` sets egui's modal layer while it is open, so keyboard focus cannot leave the dialog for the page
  behind it. The layer sits on the backdrop — the lower of the two areas — so a backdrop click still closes.
- `Modal`'s `Esc` no longer depends on `.closable(..)`: a dialog that hides the "×" and ignores backdrop
  clicks still closes on `Esc`, because a keyboard user needs a way out. Use
  `.closable(false).close_on_escape(false)` for a dialog no key dismisses.
- `Modal`'s opening focus grab runs on the first painted pass rather than the first pass, so the invisible
  sizing pass of a shrink-wrapping dialog no longer withdraws it; it targets `.initial_focus(..)` when
  given and the close "×" otherwise.

### Breaking

- `Indicator` is no longer `Copy`: it owns its accessibility label, so a value used twice needs
  `.clone()` (or a second builder chain).
- `widgets::spinner` is now `widgets::loaders`; the `Spinner` type keeps its name and crate-root export.
- Themed visual structs carry `Fill` instead of a bare `Color32` in their fill fields; construct them with
  `Fill::solid(..)` / `.into()`, or set the palette colour and let `Theme` grade it.
- `Theme` and `ThemeSpacing` own the scroll-bar geometry and the gradient fields, so code that built a
  theme or spacing with a struct literal must add `..Default::default()` (or the new fields).
