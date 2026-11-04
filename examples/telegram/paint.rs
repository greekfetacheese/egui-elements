//! Hand-rolled painting for the Telegram demo: wallpaper, a procedural photo,
//! a waveform, read ticks, an icon button and a soft shadow.
//!
//! Everything is drawn from palette colours and deterministic geometry — the
//! demo ships no binary assets and never calls an RNG.

use eframe::egui::epaint::RectShape;
use eframe::egui::*;
use egui_elements::egui_lucide::Lucide;
use egui_elements::theme::Theme;
use egui_elements::utils::mix;

use crate::style;

/// Rounded bars in a voice message.
const WAVEFORM_BARS: usize = 28;
/// Vertices on a bloom's rim; enough that the falloff reads smooth at any size.
const BLOOM_SEGMENTS: usize = 40;
/// Ring around the sun disc, as a share of its radius.
const SUN_GLOW: f32 = 2.2;
/// Petals in the wallpaper's decorative flower.
const MOTIF_PETALS: usize = 5;
/// Peak alpha of a petal; the motif must stay under the bubbles.
const MOTIF_ALPHA: u8 = 14;

/// The conversation wallpaper: a quiet vertical wash with three huge blooms
/// and a faint vignette, quiet enough for bubbles to read on top.
pub fn wallpaper(ui: &Ui, rect: Rect, theme: &Theme) {
   let colors = theme.colors;
   let painter = ui.painter().with_clip_rect(rect);

   let top = mix(colors.bg, colors.accent, 0.16);
   let bottom = mix(colors.bg, colors.info, 0.10);
   vertical(&painter, rect, top, bottom);

   let radius = rect.width().max(rect.height()) * 0.55;
   bloom(
      &painter,
      at(rect, 0.16, 0.22),
      radius * 0.90,
      colors.accent,
   );
   bloom(
      &painter,
      at(rect, 0.86, 0.10),
      radius * 0.70,
      colors.info,
   );
   bloom(
      &painter,
      at(rect, 0.74, 0.94),
      radius * 0.80,
      colors.success,
   );
   // One large, barely-there flower, echoing Telegram's own wallpaper. It sits
   // under the bubbles and stays far quieter than the bloom peaks.
   motif(
      &painter,
      at(rect, 0.73, 0.42),
      rect.width().max(rect.height()) * 0.30,
      mix(colors.accent, colors.bg, 0.55),
   );
   vignette(&painter, rect, colors.bg);
}

/// A procedural sunset: a calm sky wash, a hazy horizon, a soft sun and two
/// hill silhouettes, with rounded top corners (the bubble's own corners).
/// `seed` shifts the geometry, so two photos never look alike.
///
/// Every paint is mixed most of the way toward `bg`: the image sits under a
/// bubble on every palette, so it is a mood, not a postcard.
pub fn photo(ui: &Ui, rect: Rect, seed: u32, theme: &Theme) {
   let colors = theme.colors;
   let painter = ui.painter().with_clip_rect(rect);
   let radius = style::BUBBLE_RADIUS.min(rect.height() * 0.45) as u8;
   let corners = CornerRadius {
      nw: radius,
      ne: radius,
      sw: 0,
      se: 0,
   };

   let sky_top = mix(colors.bg, colors.accent, 0.50);
   let sky_low = mix(colors.bg, colors.warning, 0.45);
   painter.add(Shape::Rect(RectShape::filled(
      rect, corners, sky_top,
   )));

   // The gradient body starts below the rounded band, so its square corners hide.
   let body = Rect::from_min_max(
      pos2(rect.left(), rect.top() + radius as f32 + 1.0),
      rect.max,
   );
   if body.height() > 0.0 {
      vertical(&painter, body, sky_top, sky_low);
   }

   // A thin band of haze across the horizon, under the hills.
   let horizon = rect.top() + rect.height() * 0.62;
   let band_h = rect.height() * 0.16;
   let haze = style::translucent(mix(colors.bg, colors.warning, 0.30), 96);
   let clear = style::translucent(colors.bg, 0);
   vertical(
      &painter,
      Rect::from_min_max(
         pos2(rect.left(), horizon - band_h * 0.5),
         pos2(rect.right(), horizon),
      ),
      clear,
      haze,
   );
   vertical(
      &painter,
      Rect::from_min_max(
         pos2(rect.left(), horizon),
         pos2(rect.right(), horizon + band_h * 0.5),
      ),
      haze,
      clear,
   );

   let sun = mix(colors.warning, colors.text, 0.35);
   let sun_center = pos2(
      rect.left() + rect.width() * (0.26 + 0.42 * unit(seed, 1)),
      rect.top() + rect.height() * (0.30 + 0.14 * unit(seed, 2)),
   );
   let sun_radius = rect.height() * (0.08 + 0.04 * unit(seed, 0));
   painter.circle_filled(
      sun_center,
      sun_radius * SUN_GLOW,
      style::translucent(sun, 16),
   );
   painter.circle_filled(
      sun_center,
      sun_radius * 1.35,
      style::translucent(sun, 34),
   );
   painter.circle_filled(sun_center, sun_radius, sun);

   let far = mix(colors.bg, colors.info, 0.35);
   let near = mix(colors.bg, colors.success, 0.45);
   let phase = unit(seed, 4) * std::f32::consts::TAU;
   hill(
      &painter,
      rect,
      0.60 + 0.05 * unit(seed, 3),
      0.08,
      far,
      phase,
   );
   hill(
      &painter,
      rect,
      0.74 + 0.05 * unit(seed, 5),
      0.07,
      near,
      phase + std::f32::consts::FRAC_PI_2,
   );
}

/// A voice-message waveform: rounded bars with deterministic heights; bars
/// before `progress` are painted `accent`, the rest `rest`.
pub fn waveform(
   painter: &Painter,
   rect: Rect,
   seed: u32,
   progress: f32,
   accent: Color32,
   rest: Color32,
) {
   let count = WAVEFORM_BARS;
   let gap = rect.width() / (count as f32 * 2.6);
   let bar_w = ((rect.width() - gap * (count as f32 - 1.0)) / count as f32).max(1.0);
   let radius = CornerRadius::same((bar_w * 0.5).clamp(1.0, 255.0) as u8);
   let played = (progress.clamp(0.0, 1.0) * count as f32).round() as usize;

   for index in 0..count {
      let height = rect.height() * (0.22 + 0.78 * unit(seed, index as u32));
      let x = rect.left() + index as f32 * (bar_w + gap);
      let bar = Rect::from_min_max(
         pos2(x, rect.center().y - height * 0.5),
         pos2(x + bar_w, rect.center().y + height * 0.5),
      );
      let color = if index < played { accent } else { rest };
      painter.add(Shape::Rect(RectShape::filled(bar, radius, color)));
   }
}

/// Read ticks: one check when sent, two when read.
pub fn ticks(painter: &Painter, rect: Rect, read: bool, color: Color32) {
   let stroke = Stroke::new(1.6, color);
   let width = rect.width();
   let height = rect.height();
   let check = |offset: f32| {
      [
         pos2(rect.left() + offset, rect.top() + height * 0.56),
         pos2(
            rect.left() + offset + width * 0.24,
            rect.top() + height * 0.84,
         ),
         pos2(
            rect.left() + offset + width * 0.68,
            rect.top() + height * 0.20,
         ),
      ]
   };

   if read {
      let front = check(width * 0.28);
      painter.line_segment([front[0], front[1]], stroke);
      painter.line_segment([front[1], front[2]], stroke);
   }
   let back = check(width * 0.02);
   painter.line_segment([back[0], back[1]], stroke);
   painter.line_segment([back[1], back[2]], stroke);
}

/// A chrome-less icon button: `size` is the icon, the hit box adds the theme's
/// small step on every side (a 16 pt icon yields a 32 pt button).
pub fn icon_button(
   ui: &mut Ui,
   icon: Lucide,
   size: f32,
   color: Color32,
   hover: Color32,
) -> Response {
   let pad = Theme::spacing_from_ctx(ui.ctx()).sm;
   let (rect, response) = ui.allocate_exact_size(Vec2::splat(size + pad * 2.0), Sense::click());
   if !ui.is_rect_visible(rect) {
      return response;
   }

   if response.hovered() {
      ui.painter().rect_filled(
         rect,
         CornerRadius::same(style::ICON_RADIUS),
         hover,
      );
      ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
   }

   let icon_rect = Rect::from_center_size(rect.center(), Vec2::splat(size));
   icon.size(size).color(color).image().paint_at(ui, icon_rect);
   response
}

/// A blurred rounded rect under a card; `radius` also scales the blur.
pub fn soft_shadow(painter: &Painter, rect: Rect, radius: f32, color: Color32) {
   let shadow = RectShape::filled(rect, CornerRadius::same(radius as u8), color)
      .with_blur_width(radius * 1.2);
   painter.add(Shape::Rect(shadow));
}

/// A vertical two-colour gradient filling `rect`.
fn vertical(painter: &Painter, rect: Rect, top: Color32, bottom: Color32) {
   let mut mesh = Mesh::default();
   mesh.colored_vertex(rect.left_top(), top);
   mesh.colored_vertex(rect.right_top(), top);
   mesh.colored_vertex(rect.right_bottom(), bottom);
   mesh.colored_vertex(rect.left_bottom(), bottom);
   mesh.add_triangle(0, 1, 2);
   mesh.add_triangle(0, 2, 3);
   painter.add(Shape::mesh(mesh));
}

/// A horizontal two-colour gradient filling `rect`.
fn horizontal(painter: &Painter, rect: Rect, left: Color32, right: Color32) {
   let mut mesh = Mesh::default();
   mesh.colored_vertex(rect.left_top(), left);
   mesh.colored_vertex(rect.right_top(), right);
   mesh.colored_vertex(rect.right_bottom(), right);
   mesh.colored_vertex(rect.left_bottom(), left);
   mesh.add_triangle(0, 1, 2);
   mesh.add_triangle(0, 2, 3);
   painter.add(Shape::mesh(mesh));
}

/// A daisy-like fan of overlapping petals: the decorative wallpaper motif.
fn motif(painter: &Painter, center: Pos2, radius: f32, color: Color32) {
   for index in 0..MOTIF_PETALS {
      let angle = index as f32 / MOTIF_PETALS as f32 * std::f32::consts::TAU;
      let offset = vec2(angle.cos(), angle.sin()) * radius * 0.52;
      petal(
         painter,
         center + offset,
         radius * 0.50,
         radius * 0.17,
         angle,
         style::translucent(color, MOTIF_ALPHA),
      );
   }
   painter.circle_filled(
      center,
      radius * 0.17,
      style::translucent(color, MOTIF_ALPHA + 4),
   );
}

/// One petal: an ellipse fan, rotated so its long axis points away from the
/// motif's centre.
fn petal(painter: &Painter, center: Pos2, rx: f32, ry: f32, angle: f32, color: Color32) {
   let (sin, cos) = angle.sin_cos();
   let mut mesh = Mesh::default();
   mesh.colored_vertex(center, color);
   for index in 0..=BLOOM_SEGMENTS {
      let step = index as f32 / BLOOM_SEGMENTS as f32 * std::f32::consts::TAU;
      let (step_sin, step_cos) = step.sin_cos();
      let local = vec2(step_cos * rx, step_sin * ry);
      let rotated = vec2(
         local.x * cos - local.y * sin,
         local.x * sin + local.y * cos,
      );
      mesh.colored_vertex(center + rotated, color);
   }
   for index in 0..BLOOM_SEGMENTS {
      mesh.add_triangle(0, index as u32 + 1, index as u32 + 2);
   }
   painter.add(Shape::mesh(mesh));
}

/// A radial bloom: one fan from a barely-tinted centre to a clear rim.
fn bloom(painter: &Painter, center: Pos2, radius: f32, color: Color32) {
   let mut mesh = Mesh::default();
   mesh.colored_vertex(center, style::translucent(color, 24));
   for index in 0..=BLOOM_SEGMENTS {
      let angle = index as f32 / BLOOM_SEGMENTS as f32 * std::f32::consts::TAU;
      let rim = center + vec2(angle.cos(), angle.sin()) * radius;
      mesh.colored_vertex(rim, Color32::TRANSPARENT);
   }
   for index in 0..BLOOM_SEGMENTS {
      mesh.add_triangle(0, index as u32 + 1, index as u32 + 2);
   }
   painter.add(Shape::mesh(mesh));
}

/// Four faint edge bands pulling the wallpaper toward the canvas.
fn vignette(painter: &Painter, rect: Rect, color: Color32) {
   let edge = style::translucent(color, 70);
   let clear = style::translucent(color, 0);
   let band_w = rect.width() * 0.12;
   let band_h = rect.height() * 0.10;

   let top = Rect::from_min_max(
      rect.left_top(),
      pos2(rect.right(), rect.top() + band_h),
   );
   vertical(painter, top, edge, clear);
   let bottom = Rect::from_min_max(
      pos2(rect.left(), rect.bottom() - band_h),
      rect.right_bottom(),
   );
   vertical(painter, bottom, clear, edge);
   let left = Rect::from_min_max(
      rect.left_top(),
      pos2(rect.left() + band_w, rect.bottom()),
   );
   horizontal(painter, left, edge, clear);
   let right = Rect::from_min_max(
      pos2(rect.right() - band_w, rect.top()),
      rect.right_bottom(),
   );
   horizontal(painter, right, clear, edge);
}

/// A hill silhouette: the area under a gentle wave, as one triangle strip.
fn hill(painter: &Painter, rect: Rect, base: f32, amplitude: f32, color: Color32, phase: f32) {
   let steps = 18;
   let mut points = Vec::with_capacity(steps + 1);
   for index in 0..=steps {
      let t = index as f32 / steps as f32;
      let wave = (t * std::f32::consts::TAU + phase).sin() * amplitude;
      points.push(pos2(
         rect.left() + rect.width() * t,
         rect.top() + rect.height() * (base + wave),
      ));
   }

   let mut mesh = Mesh::default();
   for point in &points {
      mesh.colored_vertex(*point, color);
   }
   for point in &points {
      mesh.colored_vertex(pos2(point.x, rect.bottom()), color);
   }

   let count = points.len() as u32;
   for index in 0..count - 1 {
      mesh.add_triangle(index, index + 1, count + index + 1);
      mesh.add_triangle(index, count + index + 1, count + index);
   }
   painter.add(Shape::mesh(mesh));
}

/// A point at fractional coordinates inside `rect`.
fn at(rect: Rect, x: f32, y: f32) -> Pos2 {
   pos2(
      rect.left() + rect.width() * x,
      rect.top() + rect.height() * y,
   )
}

/// Deterministic `0.0..1.0` value for `(seed, index)` — no RNG, so a photo or
/// a waveform paints identically on every frame and every machine.
fn unit(seed: u32, index: u32) -> f32 {
   let mut hash = seed.wrapping_mul(0x9E37_79B9).wrapping_add(index.wrapping_mul(0x85EB_CA6B));
   hash ^= hash >> 15;
   hash = hash.wrapping_mul(0x2545_F491);
   hash ^= hash >> 13;
   (hash & 0xFFFF) as f32 / 65535.0
}
