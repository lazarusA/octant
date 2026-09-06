//! Plot types and scientific visualization procedural vector icons.
//! Isometric, axonometric, and blueprint aesthetics inspired by the Octant logo.

use egui::{Color32, Painter, Pos2, Rect, Stroke, StrokeKind, pos2};

/// PlotPlane: Isometric 2.5D projected data plane with grid elevation and faceted quadrants.
pub fn draw_plot_plane(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Isometric 2.5D diamond plane
    let plane_pts = vec![p(0.50, 0.16), p(0.86, 0.44), p(0.50, 0.84), p(0.14, 0.44)];
    painter.add(egui::Shape::convex_polygon(plane_pts, fill, stroke));

    // Isometric grid crosshairs
    painter.line_segment([p(0.50, 0.16), p(0.50, 0.84)], stroke);
    painter.line_segment([p(0.14, 0.44), p(0.86, 0.44)], stroke);

    // Active illuminated quadrant (Top-right quadrant facet)
    let quad_top_right = vec![p(0.50, 0.16), p(0.86, 0.44), p(0.50, 0.50)];
    painter.add(egui::Shape::convex_polygon(
        quad_top_right,
        stroke.color.gamma_multiply(0.35),
        Stroke::NONE,
    ));
}

/// PlotLine: Analytical precision line chart with crosshair axes and technical vertex diamond markers.
pub fn draw_plot_line(painter: &Painter, rect: Rect, stroke: Stroke) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Axes (L-shape with end ticks)
    let axis_stroke = Stroke::new(stroke.width * 0.85, stroke.color.gamma_multiply(0.60));
    painter.line_segment([p(0.14, 0.14), p(0.14, 0.86)], axis_stroke);
    painter.line_segment([p(0.14, 0.86), p(0.88, 0.86)], axis_stroke);
    painter.line_segment([p(0.10, 0.14), p(0.18, 0.14)], axis_stroke);
    painter.line_segment([p(0.88, 0.82), p(0.88, 0.90)], axis_stroke);

    // Analytical curve trajectory
    let line_pts = [
        p(0.18, 0.74),
        p(0.36, 0.36),
        p(0.54, 0.56),
        p(0.72, 0.20),
        p(0.86, 0.32),
    ];

    for win in line_pts.windows(2) {
        painter.line_segment([win[0], win[1]], stroke);
    }

    // High-tech diamond vertex data points
    let d = rect.width() * 0.05;
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

/// PlotSurface: Axonometric topographic terrain mesh with dual-tone elevation ridges.
pub fn draw_plot_surface(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Background ridge
    let mountain_back = vec![p(0.14, 0.80), p(0.48, 0.18), p(0.86, 0.80)];
    painter.add(egui::Shape::convex_polygon(
        mountain_back,
        fill,
        Stroke::new(stroke.width * 0.85, stroke.color.gamma_multiply(0.60)),
    ));

    // Central spine
    painter.line_segment([p(0.48, 0.18), p(0.38, 0.80)], stroke);

    // Foreground secondary ridge with specular illumination
    let ridge_front = vec![p(0.38, 0.80), p(0.64, 0.38), p(0.86, 0.80)];
    painter.add(egui::Shape::convex_polygon(
        ridge_front,
        stroke.color.gamma_multiply(0.28),
        stroke,
    ));

    // Foreground left slope
    let left_slope = vec![p(0.14, 0.80), p(0.48, 0.18), p(0.38, 0.80)];
    painter.add(egui::Shape::convex_polygon(
        left_slope,
        stroke.color.gamma_multiply(0.12),
        Stroke::NONE,
    ));
}

/// PlotGlobe: 3D wireframe geoid with dynamic latitude/longitude parallels.
pub fn draw_plot_globe(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let center = rect.center();
    let r = rect.width().min(rect.height()) * 0.40;
    if r <= 1.0 {
        return;
    }

    // Outer circle
    painter.circle(center, r, fill, stroke);

    // Axial tilt angle ~23 deg
    let tilt_cos = 0.9205_f32;
    let tilt_sin = 0.3907_f32;

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

/// PlotVolume: Full 3D Isometric Voxel Cube directly echoing the Octant brand logo.
pub fn draw_plot_volume(painter: &Painter, rect: Rect, stroke: Stroke, _fill: Color32) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // 3 Visible isometric faces of an Octant voxel unit
    let top_face = vec![p(0.50, 0.14), p(0.84, 0.32), p(0.50, 0.50), p(0.16, 0.32)];
    let right_face = vec![p(0.50, 0.50), p(0.84, 0.32), p(0.84, 0.70), p(0.50, 0.88)];
    let left_face = vec![p(0.16, 0.32), p(0.50, 0.50), p(0.50, 0.88), p(0.16, 0.70)];

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

/// PlotPointCloud: LiDAR sensor coordinate constellation with clustered orbital points.
pub fn draw_plot_point_cloud(painter: &Painter, rect: Rect, color: Color32) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    let r_core = rect.width() * 0.085;
    let r_med = rect.width() * 0.06;
    let r_small = rect.width() * 0.045;

    // Technical crosshair reticle behind points
    let axis_stroke = Stroke::new(0.8, color.gamma_multiply(0.35));
    painter.line_segment([p(0.16, 0.50), p(0.84, 0.50)], axis_stroke);
    painter.line_segment([p(0.50, 0.16), p(0.50, 0.84)], axis_stroke);

    // Clustered 3D constellation
    let points = [
        (p(0.50, 0.50), r_core),
        (p(0.30, 0.30), r_med),
        (p(0.72, 0.26), r_small),
        (p(0.26, 0.68), r_small),
        (p(0.70, 0.66), r_med),
        (p(0.48, 0.80), r_small),
        (p(0.52, 0.22), r_small),
    ];

    for (pos, r) in points {
        painter.circle_filled(pos, r, color);
    }
}

/// Colormap: Precision spectral gradient cassette with calibrated tick divisions.
pub fn draw_colormap(painter: &Painter, rect: Rect, stroke: Stroke) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Palette strip container with subtle chamfer
    let strip_rect = Rect::from_min_max(p(0.14, 0.26), p(0.86, 0.74));
    painter.rect_stroke(strip_rect, 2.0, stroke, StrokeKind::Inside);

    // 4 Distinct spectral gradient swatches (Viridis thermal mapping)
    let swatches = [
        (0.15, 0.325, Color32::from_rgb(68, 1, 84)), // Deep purple
        (0.325, 0.50, Color32::from_rgb(49, 104, 142)), // Indigo blue
        (0.50, 0.675, Color32::from_rgb(53, 183, 121)), // Spectral green
        (0.675, 0.85, Color32::from_rgb(253, 231, 37)), // Solar yellow
    ];

    for (x0, x1, col) in swatches {
        let sw_rect = Rect::from_min_max(p(x0, 0.28), p(x1, 0.72));
        painter.rect_filled(sw_rect, 0.0, col);
    }

    // Top calibration marks
    let tick_stroke = Stroke::new(stroke.width * 0.8, stroke.color.gamma_multiply(0.80));
    for frac in [0.325, 0.50, 0.675] {
        painter.line_segment([p(frac, 0.26), p(frac, 0.34)], tick_stroke);
    }
}
