//! Plot types and scientific visualization procedural vector icons.
//! Precision-engineered for Octant following standardized 24-unit geometric keylines.

use super::grid_p;
use egui::{Color32, Painter, Rect, Stroke, pos2};

/// PlotLine: Analytical precision line chart with crosshair axes and vertex diamond markers (18x18dp keyline).
pub fn draw_plot_line(painter: &Painter, rect: Rect, stroke: Stroke) {
    let p = |gx: f32, gy: f32| grid_p(rect, gx, gy);

    // Axes (L-shape with end ticks)
    let axis_stroke = Stroke::new(stroke.width * 0.85, stroke.color.gamma_multiply(0.60));
    painter.line_segment([p(3.5, 3.5), p(3.5, 20.5)], axis_stroke);
    painter.line_segment([p(3.5, 20.5), p(20.5, 20.5)], axis_stroke);
    painter.line_segment([p(2.5, 3.5), p(4.5, 3.5)], axis_stroke);
    painter.line_segment([p(20.5, 19.5), p(20.5, 21.5)], axis_stroke);

    // Analytical curve trajectory
    let line_pts = [
        p(4.5, 17.5),
        p(8.5, 9.0),
        p(13.0, 13.5),
        p(17.0, 5.0),
        p(20.5, 7.5),
    ];

    for win in line_pts.windows(2) {
        painter.line_segment([win[0], win[1]], stroke);
    }

    // High-tech diamond vertex data points (3dp width)
    let d = rect.width() * (1.5 / 24.0);
    for &pt in &[line_pts[1], line_pts[3]] {
        let pts = vec![
            pos2(pt.x, pt.y - d),
            pos2(pt.x + d, pt.y),
            pos2(pt.x, pt.y + d),
            pos2(pt.x - d, pt.y),
        ];
        painter.add(egui::Shape::convex_polygon(pts, stroke.color, Stroke::NONE));
    }
}

/// PlotSurface: Axonometric topographic terrain mesh with dual-tone elevation ridges (18x16dp keyline).
pub fn draw_plot_surface(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |gx: f32, gy: f32| grid_p(rect, gx, gy);

    // Background ridge
    let mountain_back = vec![p(3.5, 19.5), p(11.5, 4.5), p(20.5, 19.5)];
    painter.add(egui::Shape::convex_polygon(
        mountain_back,
        fill,
        Stroke::new(stroke.width * 0.85, stroke.color.gamma_multiply(0.60)),
    ));

    // Central spine
    painter.line_segment([p(11.5, 4.5), p(9.0, 19.5)], stroke);

    // Foreground secondary ridge with specular illumination
    let ridge_front = vec![p(9.0, 19.5), p(15.5, 9.0), p(20.5, 19.5)];
    painter.add(egui::Shape::convex_polygon(
        ridge_front,
        stroke.color.gamma_multiply(0.28),
        stroke,
    ));

    // Foreground left slope
    let left_slope = vec![p(3.5, 19.5), p(11.5, 4.5), p(9.0, 19.5)];
    painter.add(egui::Shape::convex_polygon(
        left_slope,
        stroke.color.gamma_multiply(0.12),
        Stroke::NONE,
    ));
}

/// PlotGlobe: 3D wireframe geoid with dynamic latitude/longitude parallels (20dp circle keyline).
pub fn draw_plot_globe(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let center = rect.center();
    let r = rect.width().min(rect.height()) * (10.0 / 24.0);
    if r <= 1.0 {
        return;
    }

    // Outer circle
    painter.circle(center, r, fill, stroke);

    // Axial tilt angle ~23.5 deg
    let tilt_cos = 0.9171_f32;
    let tilt_sin = 0.3987_f32;

    // Tilted equatorial line
    painter.line_segment(
        [
            pos2(center.x - r * tilt_cos, center.y + r * tilt_sin),
            pos2(center.x + r * tilt_cos, center.y - r * tilt_sin),
        ],
        stroke,
    );

    // Tilted polar axis
    painter.line_segment(
        [
            pos2(center.x - r * tilt_sin, center.y - r * tilt_cos),
            pos2(center.x + r * tilt_sin, center.y + r * tilt_cos),
        ],
        Stroke::new(stroke.width * 0.85, stroke.color.gamma_multiply(0.65)),
    );

    // Longitudinal ellipse
    let n_pts = 12;
    let mut meridian = Vec::with_capacity(n_pts + 1);
    for i in 0..=n_pts {
        let frac = (i as f32) / (n_pts as f32);
        let angle = -std::f32::consts::FRAC_PI_2 + frac * std::f32::consts::PI;
        let (s, c) = angle.sin_cos();
        let lx = c * (r * 0.45);
        let ly = s * r;
        let rx = lx * tilt_cos - ly * tilt_sin;
        let ry = lx * tilt_sin + ly * tilt_cos;
        meridian.push(pos2(center.x + rx, center.y + ry));
    }
    for win in meridian.windows(2) {
        painter.line_segment([win[0], win[1]], stroke);
    }
}

/// PlotVolume: Full 3D Isometric Voxel Cube directly echoing the Octant brand logo (18x18dp keyline).
pub fn draw_plot_volume(painter: &Painter, rect: Rect, stroke: Stroke, _fill: Color32) {
    let p = |gx: f32, gy: f32| grid_p(rect, gx, gy);

    // 3 Visible isometric faces of an Octant voxel unit (Standard 30° angles)
    let top_face = vec![p(12.0, 3.0), p(20.0, 7.5), p(12.0, 12.0), p(4.0, 7.5)];
    let right_face = vec![p(12.0, 12.0), p(20.0, 7.5), p(20.0, 16.5), p(12.0, 21.0)];
    let left_face = vec![p(4.0, 7.5), p(12.0, 12.0), p(12.0, 21.0), p(4.0, 16.5)];

    let base = stroke.color;
    painter.add(egui::Shape::convex_polygon(
        top_face,
        base.gamma_multiply(0.40),
        stroke,
    ));
    painter.add(egui::Shape::convex_polygon(
        right_face,
        base.gamma_multiply(0.25),
        stroke,
    ));
    painter.add(egui::Shape::convex_polygon(
        left_face,
        base.gamma_multiply(0.12),
        stroke,
    ));
}

/// PlotPointCloud: LiDAR sensor coordinate constellation with clustered orbital points (20x20dp keyline).
pub fn draw_plot_point_cloud(painter: &Painter, rect: Rect, color: Color32) {
    let p = |gx: f32, gy: f32| grid_p(rect, gx, gy);

    let r_core = rect.width() * (2.0 / 24.0);
    let r_med = rect.width() * (1.5 / 24.0);
    let r_small = rect.width() * (1.1 / 24.0);

    // Technical crosshair reticle behind points
    let axis_stroke = Stroke::new(0.8, color.gamma_multiply(0.35));
    painter.line_segment([p(4.0, 12.0), p(20.0, 12.0)], axis_stroke);
    painter.line_segment([p(12.0, 4.0), p(12.0, 20.0)], axis_stroke);

    // Clustered 3D constellation
    let points = [
        (p(12.0, 12.0), r_core),
        (p(7.5, 7.5), r_med),
        (p(17.5, 6.5), r_small),
        (p(6.5, 16.5), r_small),
        (p(17.0, 16.0), r_med),
        (p(11.5, 19.5), r_small),
        (p(12.5, 5.5), r_small),
    ];

    for (pos, r) in points {
        painter.circle_filled(pos, r, color);
    }
}
