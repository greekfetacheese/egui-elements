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
- `Chip` / `ChipRole` — an auto-width chip.
- `Slider` / `SliderVisuals` — the Web Awesome slider.
- `Badge`, `BadgeAnchor`, `BadgeCorner`, `BadgePulse`, `BadgeBounce`, `BadgeBounceTrigger` — inside and
  straddling badges with a pulsing halo and a one-shot hop.
- `Indicator` / `IndicatorState` — a status dot with an optional pulse.
- `Avatar` / `AvatarGroup` / `AvatarVisuals` — an overlapping stack that fans out on hover.
- `Toast` / `Toasts` — a toast stack with auto-dismiss and a persistent variant.
- `Menu`, `MenuItem`, `MenuSection`, `SubMenuItem` — the forked menu family.
- `FpsMeter` / `FpsStats` / `Spread` — an FPS and CPU frame-time window.
- `Spinner` / `Ripple` / `Orb` / `SpinnerShadow` — the loader family.
- `TabsStyle::Underline` and `TabsVisuals::underline_from_theme` — the fitted underline strip beside the
  segmented one.
- `Modal::max_width`, `Frame::fit` and the `TextEdit` / `TextEditOutput` names for the themed field.

**Theme**

- `Surface`, `SurfaceState` and `Elevation` — the named surface ladder and the shadow steps behind
  `Theme::surface_frame`, `surface_fill`, `surface_visuals`, `state_fill`, `state_ink` and `shadow`.
- `ColorRole`, `ColorSlot` and `DerivedColor` — named palette roles, slot iteration and checked ink
  (`Theme::ink_on`) for chromatic fills.
- `Gradient`, `GradientDir` and `Fill` — every themed fill is now a flat colour or a two-stop gradient;
  `ThemeVariant::Gradient`, `Theme::gradient_strength` and `Theme::with_gradient_dir` drive it.
- `ThemeKind::Custom` — a runtime-named theme that keeps its palette when serialized.

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

### Breaking

- `widgets::spinner` is now `widgets::loaders`; the `Spinner` type keeps its name and crate-root export.
- Themed visual structs carry `Fill` instead of a bare `Color32` in their fill fields; construct them with
  `Fill::solid(..)` / `.into()`, or set the palette colour and let `Theme` grade it.
- `Theme` and `ThemeSpacing` own the scroll-bar geometry and the gradient fields, so code that built a
  theme or spacing with a struct literal must add `..Default::default()` (or the new fields).
