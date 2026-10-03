//! Drawing primitives: lines, paths, arcs, fills and basic shapes, all in grid units.

use super::IconCanvas;
use super::geom::{dedup, dedup_closed, is_convex};
use egui::{Color32, Pos2, Rect, Shape, Stroke, StrokeKind, pos2};

/// Which ends of an open curve get round caps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Caps {
    Start,
    End,
    Both,
}

impl IconCanvas<'_> {
    /// Straight segment between grid points, pixel snapped.
    pub fn line(&self, a: (f32, f32), b: (f32, f32), stroke: Stroke) {
        let w = stroke.width;
        self.painter
            .line_segment([self.ps(a.0, a.1, w), self.ps(b.0, b.1, w)], stroke);
    }

    /// Grid points snapped for a stroke of `width` points.
    fn snapped(&self, pts: &[(f32, f32)], width: f32) -> Vec<Pos2> {
        pts.iter().map(|&(x, y)| self.ps(x, y, width)).collect()
    }

    /// Open polyline through grid points with joined corners, pixel snapped.
    pub fn path(&self, pts: &[(f32, f32)], stroke: Stroke) {
        self.curve(self.snapped(pts, stroke.width), stroke);
    }

    /// Closed polyline through grid points, pixel snapped.
    pub fn closed(&self, pts: &[(f32, f32)], stroke: Stroke) {
        self.curve_closed(self.snapped(pts, stroke.width), stroke);
    }

    /// Segment between screen points; egui still snaps it when axis aligned.
    pub fn seg(&self, a: Pos2, b: Pos2, stroke: Stroke) {
        self.painter.line_segment([a, b], stroke);
    }

    /// Open polyline through grid points with round caps, pixel snapped.
    pub fn path_round(&self, pts: &[(f32, f32)], stroke: Stroke) {
        self.curve_capped(self.snapped(pts, stroke.width), stroke, Caps::Both);
    }

    /// Open curve with round caps on the chosen ends (use with opaque
    /// strokes). Returns the first and last points so callers can attach
    /// arrowheads, or `None` for an empty curve.
    pub fn curve_capped(&self, pts: Vec<Pos2>, stroke: Stroke, caps: Caps) -> Option<(Pos2, Pos2)> {
        let ends = (*pts.first()?, *pts.last()?);
        self.curve(pts, stroke);
        let r = stroke.width * 0.5;
        if matches!(caps, Caps::Start | Caps::Both) {
            self.painter.circle_filled(ends.0, r, stroke.color);
        }
        if matches!(caps, Caps::End | Caps::Both) {
            self.painter.circle_filled(ends.1, r, stroke.color);
        }
        Some(ends)
    }

    /// Open curve through screen points (no snapping, so curves stay smooth).
    pub fn curve(&self, pts: Vec<Pos2>, stroke: Stroke) {
        let pts = dedup(pts);
        if pts.len() >= 2 {
            self.painter.add(Shape::line(pts, stroke));
        }
    }

    /// Closed curve through screen points.
    pub fn curve_closed(&self, pts: Vec<Pos2>, stroke: Stroke) {
        let pts = dedup_closed(pts);
        if pts.len() < 3 {
            return;
        }
        self.painter.add(Shape::closed_line(pts, stroke));
    }

    /// Point at grid radius `r` and angle `a` (0 = right, clockwise on screen)
    /// around grid center `c`.
    pub fn polar(&self, c: (f32, f32), r: f32, a: f32) -> Pos2 {
        self.p(c.0 + r * a.cos(), c.1 + r * a.sin())
    }

    /// Points on an ellipse arc centered at grid `(cx, cy)` with grid radii.
    /// Angles are in radians, 0 = right, increasing clockwise on screen.
    pub fn arc(&self, c: (f32, f32), r: (f32, f32), a0: f32, a1: f32) -> Vec<Pos2> {
        // About one segment per 1.5 physical pixels of arc: dense enough to look
        // round, sparse enough that joins never see sub-pixel segments.
        let span_px = (a1 - a0).abs() * r.0.max(r.1) * self.unit() * self.ppp;
        let steps = ((span_px / 1.5).ceil() as usize).clamp(6, 64);
        (0..=steps)
            .map(|i| {
                let a = a0 + (a1 - a0) * (i as f32 / steps as f32);
                self.p(c.0 + r.0 * a.cos(), c.1 + r.1 * a.sin())
            })
            .collect()
    }

    /// Fill a convex polygon of grid points, edges snapped to pixel boundaries.
    pub fn fill(&self, pts: &[(f32, f32)], color: Color32) {
        debug_assert!(
            is_convex(&pts.iter().map(|&(x, y)| pos2(x, y)).collect::<Vec<_>>()),
            "IconCanvas::fill needs a convex polygon"
        );
        let pts = pts.iter().map(|&(x, y)| self.pe(x, y)).collect();
        self.add_fill(pts, color);
    }

    /// Convex polygon of grid points: snapped fill plus one snapped outline.
    pub fn polygon(&self, pts: &[(f32, f32)], fill: Color32, stroke: Stroke) {
        self.fill(pts, fill);
        self.closed(pts, stroke);
    }

    /// Fill a convex polygon of screen points.
    pub fn fill_pts(&self, pts: Vec<Pos2>, color: Color32) {
        debug_assert!(is_convex(&pts), "IconCanvas::fill needs a convex polygon");
        self.add_fill(pts, color);
    }

    /// Add a convex fill, skipping polygons that snapping collapsed to a line.
    fn add_fill(&self, pts: Vec<Pos2>, color: Color32) {
        let pts = dedup_closed(pts);
        let n = pts.len();
        let twice_area: f32 = (0..n)
            .map(|i| {
                let (a, b) = (pts[i], pts[(i + 1) % n]);
                a.x * b.y - b.x * a.y
            })
            .sum();
        if n < 3 || twice_area.abs() < 1e-3 {
            return;
        }
        self.painter
            .add(Shape::convex_polygon(pts, color, Stroke::NONE));
    }

    /// Rounded rectangle between grid corners; the stroke is centered on the
    /// edge like every other outline. Radius is in grid units.
    pub fn rrect(&self, min: (f32, f32), max: (f32, f32), r: f32, fill: Color32, stroke: Stroke) {
        let w = stroke.width;
        let rect = Rect::from_min_max(self.ps(min.0, min.1, w), self.ps(max.0, max.1, w));
        let radius = r * self.unit();
        self.painter
            .rect(rect, radius, fill, stroke, StrokeKind::Middle);
    }

    /// Filled rounded rectangle, edges on pixel boundaries.
    pub fn rrect_fill(&self, min: (f32, f32), max: (f32, f32), r: f32, color: Color32) {
        let rect = Rect::from_min_max(self.pe(min.0, min.1), self.pe(max.0, max.1));
        self.painter.rect_filled(rect, r * self.unit(), color);
    }

    /// Circle at a grid center with a grid radius.
    pub fn circle(&self, c: (f32, f32), r: f32, fill: Color32, stroke: Stroke) {
        self.painter
            .circle(self.p(c.0, c.1), r * self.unit(), fill, stroke);
    }

    /// Solid dot, never smaller than one physical pixel across.
    pub fn dot(&self, c: (f32, f32), r: f32, color: Color32) {
        let radius = (r * self.unit()).max(0.5 / self.ppp);
        self.painter.circle_filled(self.p(c.0, c.1), radius, color);
    }

    /// Diamond with grid half extents.
    pub fn diamond(&self, c: (f32, f32), hw: f32, hh: f32, fill: Color32, stroke: Stroke) {
        let pts = vec![
            self.p(c.0, c.1 - hh),
            self.p(c.0 + hw, c.1),
            self.p(c.0, c.1 + hh),
            self.p(c.0 - hw, c.1),
        ];
        if fill != Color32::TRANSPARENT {
            self.fill_pts(pts.clone(), fill);
        }
        if stroke.width > 0.0 {
            self.curve_closed(pts, stroke);
        }
    }
}
