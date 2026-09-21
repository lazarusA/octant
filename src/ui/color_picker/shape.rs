//! Shape definitions and painting for custom color picker trigger buttons.

use egui::{Color32, Mesh, Pos2, Rect, Shape, Stroke};
use std::sync::Arc;

/// Custom rendering closure type for `ColorShape::Custom`.
pub type CustomShapeFn = Arc<dyn Fn(&egui::Painter, Rect, Color32, Stroke) + Send + Sync>;

/// Shape definitions for custom color picker trigger buttons.
#[derive(Clone)]
pub enum ColorShape {
    /// Left-pointing triangle (◁) filling the rect
    LeftTriangle,
    /// Right-pointing triangle (▷) filling the rect
    RightTriangle,
    /// Up-pointing triangle (△) filling the rect
    UpTriangle,
    /// Down-pointing triangle (▽) filling the rect
    DownTriangle,
    /// Circle centered inside the rect
    Circle,
    /// Rounded rectangle with specified corner radius
    Rect(f32),
    /// Custom polygon points normalized to `[0.0, 1.0]` relative to the rect bounding box
    NormalizedPolygon(Vec<Pos2>),
    /// Custom drawing callback allowing arbitrary rendering logic
    Custom(CustomShapeFn),
}

impl ColorShape {
    /// Convenient constructor for arbitrary custom shape rendering logic
    pub fn custom(
        f: impl Fn(&egui::Painter, Rect, Color32, Stroke) + Send + Sync + 'static,
    ) -> Self {
        Self::Custom(Arc::new(f))
    }

    /// Helper to paint the shape with given color and stroke inside `rect`.
    pub fn paint(&self, painter: &egui::Painter, rect: Rect, color: Color32, stroke: Stroke) {
        match self {
            Self::LeftTriangle => {
                let tip = Pos2::new(rect.min.x, rect.center().y);
                let top = Pos2::new(rect.max.x, rect.min.y);
                let bottom = Pos2::new(rect.max.x, rect.max.y);
                paint_triangle(painter, tip, top, bottom, color, stroke);
            }
            Self::RightTriangle => {
                let top = Pos2::new(rect.min.x, rect.min.y);
                let tip = Pos2::new(rect.max.x, rect.center().y);
                let bottom = Pos2::new(rect.min.x, rect.max.y);
                paint_triangle(painter, top, tip, bottom, color, stroke);
            }
            Self::UpTriangle => {
                let tip = Pos2::new(rect.center().x, rect.min.y);
                let bottom_right = Pos2::new(rect.max.x, rect.max.y);
                let bottom_left = Pos2::new(rect.min.x, rect.max.y);
                paint_triangle(painter, tip, bottom_right, bottom_left, color, stroke);
            }
            Self::DownTriangle => {
                let top_left = Pos2::new(rect.min.x, rect.min.y);
                let top_right = Pos2::new(rect.max.x, rect.min.y);
                let tip = Pos2::new(rect.center().x, rect.max.y);
                paint_triangle(painter, top_left, top_right, tip, color, stroke);
            }
            Self::Circle => {
                let radius = rect.width().min(rect.height()) / 2.0;
                painter.circle(rect.center(), radius, color, stroke);
            }
            Self::Rect(rounding) => {
                painter.rect(rect, *rounding, color, stroke, egui::StrokeKind::Middle);
            }
            Self::NormalizedPolygon(normalized_pts) => {
                if normalized_pts.len() >= 3 {
                    let mut mapped_pts = Vec::with_capacity(normalized_pts.len());
                    for p in normalized_pts {
                        mapped_pts.push(Pos2::new(
                            rect.min.x + p.x * rect.width(),
                            rect.min.y + p.y * rect.height(),
                        ));
                    }
                    painter.add(Shape::convex_polygon(mapped_pts, color, stroke));
                }
            }
            Self::Custom(cb) => {
                (cb)(painter, rect, color, stroke);
            }
        }
    }
}

/// Zero-allocation helper to paint a filled triangle with stroke outline.
fn paint_triangle(
    painter: &egui::Painter,
    p0: Pos2,
    p1: Pos2,
    p2: Pos2,
    color: Color32,
    stroke: Stroke,
) {
    let mut mesh = Mesh::default();
    mesh.colored_vertex(p0, color);
    mesh.colored_vertex(p1, color);
    mesh.colored_vertex(p2, color);
    mesh.add_triangle(0, 1, 2);
    painter.add(Shape::mesh(mesh));

    if stroke.width > 0.0 && stroke.color != Color32::TRANSPARENT {
        painter.line_segment([p0, p1], stroke);
        painter.line_segment([p1, p2], stroke);
        painter.line_segment([p2, p0], stroke);
    }
}
