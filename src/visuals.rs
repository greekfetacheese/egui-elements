use crate::gradient::Fill;
use egui::{
    Color32, CornerRadius, Painter, Rect, Response, Shadow, Shape, Stroke, StrokeKind,
    epaint::RectShape,
};

pub type LabelVisuals = ButtonVisuals;

/// Visuals for a button
#[derive(Copy, Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ButtonVisuals {
    pub text: Color32,
    pub bg: Fill,
    pub bg_hover: Fill,
    pub bg_click: Fill,
    pub bg_selected: Fill,
    pub border: Stroke,
    pub border_hover: Stroke,
    pub border_click: Stroke,
    pub corner_radius: CornerRadius,
    pub shadow: Shadow,
}

impl PartialEq for ButtonVisuals {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
            && self.bg == other.bg
            && self.bg_hover == other.bg_hover
            && self.bg_click == other.bg_click
            && self.bg_selected == other.bg_selected
            && self.border == other.border
            && self.border_hover == other.border_hover
            && self.border_click == other.border_click
            && self.corner_radius == other.corner_radius
            && self.shadow == other.shadow
    }
}

impl Eq for ButtonVisuals {}

impl ButtonVisuals {
    pub fn bg_from_res(&self, res: &Response) -> Fill {
        if res.is_pointer_button_down_on() || res.has_focus() || res.clicked() {
            self.bg_click
        } else if res.hovered() || res.highlighted() {
            self.bg_hover
        } else {
            self.bg
        }
    }

    pub fn border_from_res(&self, res: &Response) -> Stroke {
        if res.is_pointer_button_down_on() || res.has_focus() || res.clicked() {
            self.border_click
        } else if res.hovered() || res.highlighted() {
            self.border_hover
        } else {
            self.border
        }
    }

    pub fn paint_at(&self, painter: &Painter, rect: Rect, fill: Fill, stroke: Stroke) {
        painter.add(paint_shape(
            rect,
            fill,
            stroke,
            self.corner_radius,
            self.shadow,
        ));
    }
}

/// Visuals for a TextEdit
#[derive(Copy, Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextEditVisuals {
    pub text: Color32,
    pub bg: Fill,
    pub border: Stroke,
    pub border_hover: Stroke,
    pub border_open: Stroke,
    pub corner_radius: CornerRadius,
    pub shadow: Shadow,
}

impl PartialEq for TextEditVisuals {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
            && self.bg == other.bg
            && self.border == other.border
            && self.border_hover == other.border_hover
            && self.border_open == other.border_open
            && self.corner_radius == other.corner_radius
            && self.shadow == other.shadow
    }
}

impl Eq for TextEditVisuals {}

impl TextEditVisuals {
    pub fn border_from_res(&self, res: &Response) -> Stroke {
        if res.is_pointer_button_down_on() || res.has_focus() || res.clicked() {
            self.border_open
        } else if res.hovered() || res.highlighted() {
            self.border_hover
        } else {
            self.border
        }
    }

    pub fn paint_at(&self, painter: &Painter, rect: Rect, fill: Fill, stroke: Stroke) {
        painter.add(paint_shape(
            rect,
            fill,
            stroke,
            self.corner_radius,
            self.shadow,
        ));
    }
}

/// Visuals for a ComboBox
#[derive(Copy, Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ComboBoxVisuals {
    pub bg: Fill,
    pub icon: Color32,
    pub bg_hover: Fill,
    pub bg_open: Fill,
    pub border: Stroke,
    pub border_hover: Stroke,
    pub border_open: Stroke,
    pub corner_radius: CornerRadius,
    pub shadow: Shadow,
}

impl PartialEq for ComboBoxVisuals {
    fn eq(&self, other: &Self) -> bool {
        self.bg == other.bg
            && self.icon == other.icon
            && self.bg_hover == other.bg_hover
            && self.bg_open == other.bg_open
            && self.border == other.border
            && self.border_hover == other.border_hover
            && self.border_open == other.border_open
            && self.corner_radius == other.corner_radius
            && self.shadow == other.shadow
    }
}

impl Eq for ComboBoxVisuals {}

impl ComboBoxVisuals {
    pub fn bg_from_res(&self, res: &Response) -> Fill {
        if res.is_pointer_button_down_on() || res.has_focus() || res.clicked() {
            self.bg_open
        } else if res.hovered() || res.highlighted() {
            self.bg_hover
        } else {
            self.bg
        }
    }

    pub fn border_from_res(&self, res: &Response) -> Stroke {
        if res.is_pointer_button_down_on() || res.has_focus() || res.clicked() {
            self.border_open
        } else if res.hovered() || res.highlighted() {
            self.border_hover
        } else {
            self.border
        }
    }

    pub fn paint_at(&self, painter: &Painter, rect: Rect, fill: Fill, stroke: Stroke) {
        painter.add(paint_shape(
            rect,
            fill,
            stroke,
            self.corner_radius,
            self.shadow,
        ));
    }
}

/// Visuals for [`crate::widgets::Frame`].
///
/// Fill slots are [`Fill`] so a frame can be solid or a two-stop gradient.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Default)]
pub struct FrameVisuals {
    pub bg: Fill,
    pub bg_hover: Fill,
    pub bg_click: Fill,
    pub border: Stroke,
    pub border_hover: Stroke,
    pub border_click: Stroke,
    pub corner_radius: CornerRadius,
    pub shadow: Shadow,
}

impl PartialEq for FrameVisuals {
    fn eq(&self, other: &Self) -> bool {
        self.bg == other.bg
            && self.bg_hover == other.bg_hover
            && self.bg_click == other.bg_click
            && self.border == other.border
            && self.border_hover == other.border_hover
            && self.border_click == other.border_click
            && self.corner_radius == other.corner_radius
            && self.shadow == other.shadow
    }
}

impl Eq for FrameVisuals {}

impl FrameVisuals {
    pub fn bg_from_res(&self, res: &Response) -> Fill {
        if res.is_pointer_button_down_on() || res.has_focus() || res.clicked() {
            self.bg_click
        } else if res.hovered() || res.highlighted() {
            self.bg_hover
        } else {
            self.bg
        }
    }

    pub fn border_from_res(&self, res: &Response) -> Stroke {
        if res.is_pointer_button_down_on() || res.has_focus() || res.clicked() {
            self.border_click
        } else if res.hovered() || res.highlighted() {
            self.border_hover
        } else {
            self.border
        }
    }

    /// Shadow + fill + inside stroke on an already allocated widget rect.
    pub fn paint_at(&self, painter: &Painter, rect: Rect, fill: Fill, stroke: Stroke) {
        painter.add(paint_shape(
            rect,
            fill,
            stroke,
            self.corner_radius,
            self.shadow,
        ));
    }
}

pub(crate) fn paint_shape(
    rect: Rect,
    fill: Fill,
    stroke: Stroke,
    corner_radius: CornerRadius,
    shadow: Shadow,
) -> Shape {
    if rect.width() <= 0.0 || rect.height() <= 0.0 {
        return Shape::Noop;
    }

    let has_fill = match fill {
        Fill::Solid(color) => color != Color32::TRANSPARENT,
        Fill::Gradient(_) => true,
    };
    let has_shadow = shadow != Shadow::NONE && shadow.color != Color32::TRANSPARENT;
    if !has_fill && stroke.is_empty() && !has_shadow {
        return Shape::Noop;
    }

    let mut shapes = Vec::new();

    if has_shadow {
        shapes.push(Shape::from(shadow.as_shape(rect, corner_radius)));
    }

    match fill {
        Fill::Solid(color) => {
            if color != Color32::TRANSPARENT || !stroke.is_empty() {
                shapes.push(Shape::Rect(RectShape::new(
                    rect,
                    corner_radius,
                    color,
                    stroke,
                    StrokeKind::Inside,
                )));
            }
        }
        Fill::Gradient(gradient) => {
            shapes.push(gradient.shape_rounded(rect, corner_radius));
            if !stroke.is_empty() {
                shapes.push(Shape::Rect(RectShape::stroke(
                    rect,
                    corner_radius,
                    stroke,
                    StrokeKind::Inside,
                )));
            }
        }
    }

    match shapes.len() {
        0 => Shape::Noop,
        1 => shapes.remove(0),
        _ => Shape::Vec(shapes),
    }
}
