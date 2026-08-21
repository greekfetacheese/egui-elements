use std::f32::consts::{FRAC_PI_2, PI, TAU};

use egui::{Color32, CornerRadius, Mesh, Painter, Pos2, Rect, Sense, Shape, Ui, Vec2, pos2};

/// Axis of a linear gradient across a rectangle.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GradientDir {
    /// `from` at the top edge, `to` at the bottom.
    #[default]
    TopDown,
    /// `from` at the bottom edge, `to` at the top.
    BottomUp,
    /// `from` at the left edge, `to` at the right.
    LeftToRight,
    /// `from` at the right edge, `to` at the left.
    RightToLeft,
}

/// Two-color linear gradient, painted as a vertex-colored mesh.
///
/// egui fills are a single [`Color32`]. A gradient has to go through
/// [`Painter`]: vertices are colored, the GPU interpolates between them.
///
/// This is the primitive for replacing solid widget/frame fills later.
/// It is [`Copy`] so it can sit next to [`Color32`] in visuals.
///
/// Cardinal linear gradients stay exact under barycentric interpolation
/// even on a rounded mesh: each vertex is sampled in the same space.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Gradient {
    pub from: Color32,
    pub to: Color32,
    pub dir: GradientDir,
}

impl Gradient {
    pub const fn new(from: Color32, to: Color32) -> Self {
        Self {
            from,
            to,
            dir: GradientDir::TopDown,
        }
    }

    /// `from` at the top, `to` at the bottom.
    pub const fn vertical(from: Color32, to: Color32) -> Self {
        Self {
            from,
            to,
            dir: GradientDir::TopDown,
        }
    }

    /// `from` at the left, `to` at the right.
    pub const fn horizontal(from: Color32, to: Color32) -> Self {
        Self {
            from,
            to,
            dir: GradientDir::LeftToRight,
        }
    }

    pub const fn with_dir(self, dir: GradientDir) -> Self {
        Self {
            from: self.from,
            to: self.to,
            dir,
        }
    }

    pub const fn reverse(self) -> Self {
        Self {
            from: self.to,
            to: self.from,
            dir: self.dir,
        }
    }

    /// Color at `t` in `0..=1` along the gradient (gamma-space lerp).
    ///
    /// GPU interpolation of the mesh is the same channel lerp, so this
    /// matches what you see on an axis-aligned 2-stop fill.
    pub fn sample(self, t: f32) -> Color32 {
        self.from.lerp_to_gamma(self.to, t.clamp(0.0, 1.0))
    }

    /// `0` at `from`, `1` at `to`, from `pos` in `rect`.
    pub fn t_along(self, rect: Rect, pos: Pos2) -> f32 {
        let t = match self.dir {
            GradientDir::TopDown => {
                let h = rect.height();
                if h <= 0.0 {
                    0.0
                } else {
                    (pos.y - rect.top()) / h
                }
            }
            GradientDir::BottomUp => {
                let h = rect.height();
                if h <= 0.0 {
                    0.0
                } else {
                    (rect.bottom() - pos.y) / h
                }
            }
            GradientDir::LeftToRight => {
                let w = rect.width();
                if w <= 0.0 {
                    0.0
                } else {
                    (pos.x - rect.left()) / w
                }
            }
            GradientDir::RightToLeft => {
                let w = rect.width();
                if w <= 0.0 {
                    0.0
                } else {
                    (rect.right() - pos.x) / w
                }
            }
        };
        t.clamp(0.0, 1.0)
    }

    pub fn sample_at(self, rect: Rect, pos: Pos2) -> Color32 {
        self.sample(self.t_along(rect, pos))
    }

    /// Corner colors `(left_top, right_top, left_bottom, right_bottom)`.
    pub const fn corner_colors(self) -> [Color32; 4] {
        match self.dir {
            GradientDir::TopDown => [self.from, self.from, self.to, self.to],
            GradientDir::BottomUp => [self.to, self.to, self.from, self.from],
            GradientDir::LeftToRight => [self.from, self.to, self.from, self.to],
            GradientDir::RightToLeft => [self.to, self.from, self.to, self.from],
        }
    }

    /// Sharp rectangle (four vertices). Prefer [`Self::mesh_rounded`] for widgets.
    pub fn mesh(self, rect: Rect) -> Mesh {
        self.mesh_rounded(rect, CornerRadius::ZERO)
    }

    /// Convex rounded-rect, vertex-colored by [`Self::sample_at`].
    ///
    /// [`CornerRadius::ZERO`] uses the cheap four-vertex quad.
    pub fn mesh_rounded(self, rect: Rect, corner_radius: impl Into<CornerRadius>) -> Mesh {
        let mut mesh = Mesh::default();
        if rect.width() <= 0.0 || rect.height() <= 0.0 {
            return mesh;
        }

        let cr = corner_radius.into();
        if cr == CornerRadius::ZERO {
            let [lt, rt, lb, rb] = self.corner_colors();
            mesh.colored_vertex(rect.left_top(), lt);
            mesh.colored_vertex(rect.right_top(), rt);
            mesh.colored_vertex(rect.left_bottom(), lb);
            mesh.colored_vertex(rect.right_bottom(), rb);
            mesh.add_triangle(0, 1, 2);
            mesh.add_triangle(2, 1, 3);
            return mesh;
        }

        let outline = rounded_rect_outline(rect, cr);
        let n = outline.len();
        if n < 3 {
            return mesh;
        }

        mesh.reserve_vertices(n);
        mesh.reserve_triangles(n.saturating_sub(2));
        for pos in &outline {
            mesh.colored_vertex(*pos, self.sample_at(rect, *pos));
        }
        for i in 2..n as u32 {
            mesh.add_triangle(0, i - 1, i);
        }
        mesh
    }

    pub fn shape(self, rect: Rect) -> Shape {
        Shape::mesh(self.mesh(rect))
    }

    pub fn shape_rounded(self, rect: Rect, corner_radius: impl Into<CornerRadius>) -> Shape {
        Shape::mesh(self.mesh_rounded(rect, corner_radius))
    }

    pub fn paint(self, painter: &Painter, rect: Rect) {
        self.paint_rounded(painter, rect, CornerRadius::ZERO);
    }

    pub fn paint_rounded(
        self,
        painter: &Painter,
        rect: Rect,
        corner_radius: impl Into<CornerRadius>,
    ) {
        if rect.width() <= 0.0 || rect.height() <= 0.0 {
            return;
        }
        painter.add(self.shape_rounded(rect, corner_radius));
    }

    /// Allocate `size` and paint. Hover-sense only.
    pub fn show(self, ui: &mut Ui, size: Vec2) -> egui::Response {
        self.show_rounded(ui, size, CornerRadius::ZERO)
    }

    pub fn show_rounded(
        self,
        ui: &mut Ui,
        size: Vec2,
        corner_radius: impl Into<CornerRadius>,
    ) -> egui::Response {
        let cr = corner_radius.into();
        let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
        if ui.is_rect_visible(rect) {
            self.paint_rounded(ui.painter(), rect, cr);
        }
        response
    }
}

/// Solid color or gradient. Intended to replace [`Color32`] on fills.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Fill {
    Solid(Color32),
    Gradient(Gradient),
}

impl Fill {
    pub const fn solid(color: Color32) -> Self {
        Self::Solid(color)
    }

    /// Two-stop vertical fill (`from` at the top).
    pub const fn vertical(from: Color32, to: Color32) -> Self {
        Self::Gradient(Gradient::vertical(from, to))
    }

    /// Two-stop horizontal fill (`from` at the left).
    pub const fn horizontal(from: Color32, to: Color32) -> Self {
        Self::Gradient(Gradient::horizontal(from, to))
    }

    pub fn paint(self, painter: &Painter, rect: Rect) {
        self.paint_rounded(painter, rect, CornerRadius::ZERO);
    }

    pub fn paint_rounded(
        self,
        painter: &Painter,
        rect: Rect,
        corner_radius: impl Into<CornerRadius>,
    ) {
        if rect.width() <= 0.0 || rect.height() <= 0.0 {
            return;
        }
        match self {
            Self::Solid(color) => {
                painter.rect_filled(rect, corner_radius, color);
            }
            Self::Gradient(gradient) => gradient.paint_rounded(painter, rect, corner_radius),
        }
    }

    pub fn show(self, ui: &mut Ui, size: Vec2) -> egui::Response {
        self.show_rounded(ui, size, CornerRadius::ZERO)
    }

    pub fn show_rounded(
        self,
        ui: &mut Ui,
        size: Vec2,
        corner_radius: impl Into<CornerRadius>,
    ) -> egui::Response {
        let cr = corner_radius.into();
        let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
        if ui.is_rect_visible(rect) {
            self.paint_rounded(ui.painter(), rect, cr);
        }
        response
    }

    pub fn shape_rounded(self, rect: Rect, corner_radius: impl Into<CornerRadius>) -> Shape {
        match self {
            Self::Solid(color) => Shape::rect_filled(rect, corner_radius, color),
            Self::Gradient(gradient) => gradient.shape_rounded(rect, corner_radius),
        }
    }

    pub fn gamma_multiply(self, opacity: f32) -> Self {
        match self {
            Self::Solid(color) => Self::Solid(color.gamma_multiply(opacity)),
            Self::Gradient(gradient) => Self::Gradient(Gradient {
                from: gradient.from.gamma_multiply(opacity),
                to: gradient.to.gamma_multiply(opacity),
                dir: gradient.dir,
            }),
        }
    }

    /// If a stop (or the solid color) equals `old`, replace it with `new`.
    pub fn remap_color(&mut self, old: Color32, new: Color32) {
        match self {
            Self::Solid(color) => {
                if *color == old {
                    *color = new;
                }
            }
            Self::Gradient(gradient) => {
                if gradient.from == old {
                    gradient.from = new;
                }
                if gradient.to == old {
                    gradient.to = new;
                }
            }
        }
    }
}

impl Default for Fill {
    fn default() -> Self {
        Self::Solid(Color32::TRANSPARENT)
    }
}

impl From<Color32> for Fill {
    fn from(color: Color32) -> Self {
        Self::Solid(color)
    }
}

impl From<Gradient> for Fill {
    fn from(gradient: Gradient) -> Self {
        Self::Gradient(gradient)
    }
}

/// Clockwise outline, y-down: NW → NE → SE → SW.
///
/// Segment counts match epaint's circle-quadrant cutoffs so a gradient
/// fill sits on the same silhouette as a solid `rect_filled`.
fn rounded_rect_outline(rect: Rect, cr: CornerRadius) -> Vec<Pos2> {
    let max_cr = (rect.width() * 0.5).min(rect.height() * 0.5).max(0.0);
    let nw = (cr.nw as f32).clamp(0.0, max_cr);
    let ne = (cr.ne as f32).clamp(0.0, max_cr);
    let se = (cr.se as f32).clamp(0.0, max_cr);
    let sw = (cr.sw as f32).clamp(0.0, max_cr);

    if nw == 0.0 && ne == 0.0 && se == 0.0 && sw == 0.0 {
        return vec![
            rect.left_top(),
            rect.right_top(),
            rect.right_bottom(),
            rect.left_bottom(),
        ];
    }

    let mut path = Vec::new();
    add_corner(
        &mut path,
        pos2(rect.left() + nw, rect.top() + nw),
        nw,
        PI,
        PI + FRAC_PI_2,
        rect.left_top(),
    );
    add_corner(
        &mut path,
        pos2(rect.right() - ne, rect.top() + ne),
        ne,
        PI + FRAC_PI_2,
        TAU,
        rect.right_top(),
    );
    add_corner(
        &mut path,
        pos2(rect.right() - se, rect.bottom() - se),
        se,
        0.0,
        FRAC_PI_2,
        rect.right_bottom(),
    );
    add_corner(
        &mut path,
        pos2(rect.left() + sw, rect.bottom() - sw),
        sw,
        FRAC_PI_2,
        PI,
        rect.left_bottom(),
    );
    path
}

fn add_corner(path: &mut Vec<Pos2>, center: Pos2, radius: f32, start: f32, end: f32, sharp: Pos2) {
    if radius <= 0.0 {
        push_unique(path, sharp);
        return;
    }
    let n = quadrant_steps(radius);
    for i in 0..=n {
        let t = i as f32 / n as f32;
        let a = start + (end - start) * t;
        push_unique(
            path,
            pos2(center.x + radius * a.cos(), center.y + radius * a.sin()),
        );
    }
}

fn push_unique(path: &mut Vec<Pos2>, p: Pos2) {
    if let Some(&last) = path.last() {
        if (last - p).length_sq() < 1e-8 {
            return;
        }
    }
    path.push(p);
}

fn quadrant_steps(radius: f32) -> usize {
    if radius <= 2.0 {
        2
    } else if radius <= 5.0 {
        4
    } else if radius < 18.0 {
        8
    } else if radius < 50.0 {
        16
    } else {
        32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{pos2, vec2};

    fn unit_rect() -> Rect {
        Rect::from_min_size(pos2(0.0, 0.0), vec2(10.0, 10.0))
    }

    #[test]
    fn sample_endpoints() {
        let from = Color32::from_rgb(0, 0, 0);
        let to = Color32::from_rgb(255, 255, 255);
        let g = Gradient::vertical(from, to);
        assert_eq!(g.sample(0.0), from);
        assert_eq!(g.sample(1.0), to);
        assert_eq!(g.sample(-1.0), from);
        assert_eq!(g.sample(2.0), to);
    }

    #[test]
    fn sample_at_matches_corners() {
        let from = Color32::RED;
        let to = Color32::BLUE;
        let g = Gradient::horizontal(from, to);
        let rect = unit_rect();
        assert_eq!(g.sample_at(rect, rect.left_top()), from);
        assert_eq!(g.sample_at(rect, rect.right_top()), to);
    }

    #[test]
    fn corner_colors_match_dir() {
        let from = Color32::RED;
        let to = Color32::BLUE;
        assert_eq!(
            Gradient::vertical(from, to).corner_colors(),
            [from, from, to, to]
        );
        assert_eq!(
            Gradient::horizontal(from, to).corner_colors(),
            [from, to, from, to]
        );
        assert_eq!(
            Gradient::new(from, to)
                .with_dir(GradientDir::BottomUp)
                .corner_colors(),
            [to, to, from, from]
        );
        assert_eq!(
            Gradient::new(from, to)
                .with_dir(GradientDir::RightToLeft)
                .corner_colors(),
            [to, from, to, from]
        );
    }

    #[test]
    fn reverse_swaps_stops() {
        let from = Color32::WHITE;
        let to = Color32::BLACK;
        let g = Gradient::horizontal(from, to).reverse();
        assert_eq!(g.from, to);
        assert_eq!(g.to, from);
        assert_eq!(g.dir, GradientDir::LeftToRight);
    }

    #[test]
    fn mesh_has_quad() {
        let g = Gradient::vertical(Color32::WHITE, Color32::BLACK);
        let mesh = g.mesh(unit_rect());
        assert_eq!(mesh.vertices.len(), 4);
        assert_eq!(mesh.indices.len(), 6);
        assert!(mesh.is_valid());
    }

    #[test]
    fn rounded_mesh_is_valid_and_inside_rect() {
        let g = Gradient::vertical(Color32::WHITE, Color32::BLACK);
        let rect = unit_rect();
        let mesh = g.mesh_rounded(rect, 4);
        assert!(mesh.vertices.len() > 4);
        assert!(mesh.is_valid());
        assert_eq!(mesh.indices.len() % 3, 0);
        for v in &mesh.vertices {
            assert!(rect.expand(0.01).contains(v.pos), "{:?}", v.pos);
        }
    }

    #[test]
    fn mixed_corners() {
        let g = Gradient::horizontal(Color32::WHITE, Color32::BLACK);
        let cr = CornerRadius {
            nw: 8,
            ne: 0,
            sw: 2,
            se: 4,
        };
        let mesh = g.mesh_rounded(unit_rect(), cr);
        assert!(mesh.is_valid());
        assert!(mesh.vertices.len() >= 4);
    }

    #[test]
    fn full_pill_clamps() {
        let g = Gradient::vertical(Color32::WHITE, Color32::BLACK);
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(20.0, 8.0));
        let mesh = g.mesh_rounded(rect, 255);
        assert!(mesh.is_valid());
        assert!(mesh.vertices.len() > 4);
        for v in &mesh.vertices {
            assert!(rect.expand(0.01).contains(v.pos));
        }
    }
}
