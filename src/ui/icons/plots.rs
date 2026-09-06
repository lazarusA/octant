//! Plot types and scientific visualization procedural vector icons.

use egui::{Color32, Painter, Pos2, Rect, Stroke, StrokeKind, pos2};

pub fn draw_plot_plane(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // 2D Quad Flatmap with 2x2 colored quadrants
    let main_rect = Rect::from_min_max(p(0.15, 0.15), p(0.85, 0.85));
    painter.rect(main_rect, 2.0, fill, stroke, StrokeKind::Inside);

    // Cross division lines
    painter.line_segment([p(0.50, 0.15), p(0.50, 0.85)], stroke);
    painter.line_segment([p(0.15, 0.50), p(0.85, 0.50)], stroke);

    // Subtle shaded inner cell
    let inner = Rect::from_min_max(p(0.52, 0.17), p(0.83, 0.48));
    painter.rect_filled(inner, 1.0, stroke.color.gamma_multiply(0.28));
}

pub fn draw_plot_line(painter: &Painter, rect: Rect, stroke: Stroke) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Axes (L-shape)
    let axis_stroke = Stroke::new(stroke.width * 0.9, stroke.color.gamma_multiply(0.6));
    painter.line_segment([p(0.14, 0.14), p(0.14, 0.86)], axis_stroke);
    painter.line_segment([p(0.14, 0.86), p(0.88, 0.86)], axis_stroke);

    // Dynamic line curve
    let line_pts = [
        p(0.18, 0.74),
        p(0.38, 0.38),
        p(0.54, 0.58),
        p(0.72, 0.22),
        p(0.86, 0.32),
    ];

    for win in line_pts.windows(2) {
        painter.line_segment([win[0], win[1]], stroke);
    }

    // Small data point circles on peaks
    let dot_r = rect.width() * 0.05;
    painter.circle_filled(line_pts[1], dot_r, stroke.color);
    painter.circle_filled(line_pts[3], dot_r, stroke.color);
}

pub fn draw_plot_surface(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Isometric 3D Mountain / Terrain mesh
    let mountain_back = vec![p(0.15, 0.78), p(0.48, 0.22), p(0.85, 0.78)];
    painter.add(egui::Shape::convex_polygon(
        mountain_back,
        fill,
        Stroke::new(stroke.width * 0.8, stroke.color.gamma_multiply(0.5)),
    ));

    let mountain_ridge = [p(0.48, 0.22), p(0.44, 0.62), p(0.35, 0.78)];
    for win in mountain_ridge.windows(2) {
        painter.line_segment([win[0], win[1]], stroke);
    }

    // Foreground secondary ridge
    let ridge_front = vec![p(0.35, 0.78), p(0.62, 0.42), p(0.85, 0.78)];
    painter.add(egui::Shape::convex_polygon(
        ridge_front,
        stroke.color.gamma_multiply(0.15),
        stroke,
    ));
}

pub fn draw_plot_globe(painter: &Painter, rect: Rect, stroke: Stroke, _fill: Color32) {
    let center = rect.center();
    let r = rect.width().min(rect.height()) * 0.40;
    if r <= 1.0 {
        return;
    }

    // Outer circle
    painter.circle_stroke(center, r, stroke);

    // Equatorial horizontal line
    painter.line_segment(
        [pos2(center.x - r, center.y), pos2(center.x + r, center.y)],
        stroke,
    );

    // Meridian ellipse arc
    let n_pts = 12;
    let mut meridian = Vec::with_capacity(n_pts + 1);
    for i in 0..=n_pts {
        let frac = (i as f32) / (n_pts as f32);
        let angle = -std::f32::consts::FRAC_PI_2 + frac * std::f32::consts::PI;
        let (s, c) = angle.sin_cos();
        meridian.push(pos2(center.x + c * (r * 0.45), center.y + s * r));
    }
    for win in meridian.windows(2) {
        painter.line_segment([win[0], win[1]], stroke);
    }
}

pub fn draw_plot_volume(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // 3D Isometric Cube Box
    let top_face = vec![p(0.50, 0.16), p(0.82, 0.32), p(0.50, 0.48), p(0.18, 0.32)];
    let left_face = vec![p(0.18, 0.32), p(0.50, 0.48), p(0.50, 0.84), p(0.18, 0.68)];
    let right_face = vec![p(0.50, 0.48), p(0.82, 0.32), p(0.82, 0.68), p(0.50, 0.84)];

    painter.add(egui::Shape::convex_polygon(top_face, fill, stroke));
    painter.add(egui::Shape::convex_polygon(
        left_face,
        stroke.color.gamma_multiply(0.25),
        stroke,
    ));
    painter.add(egui::Shape::convex_polygon(
        right_face,
        stroke.color.gamma_multiply(0.10),
        stroke,
    ));
}

pub fn draw_plot_point_cloud(painter: &Painter, rect: Rect, color: Color32) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    let r_big = rect.width() * 0.08;
    let r_med = rect.width() * 0.06;
    let r_small = rect.width() * 0.045;

    // Scattered 3D constellation of points
    let points = [
        (p(0.30, 0.28), r_med),
        (p(0.70, 0.22), r_small),
        (p(0.50, 0.46), r_big),
        (p(0.24, 0.68), r_small),
        (p(0.74, 0.62), r_med),
        (p(0.48, 0.80), r_med),
    ];

    for (pos, r) in points {
        painter.circle_filled(pos, r, color);
    }
}

pub fn draw_colormap(painter: &Painter, rect: Rect, stroke: Stroke) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Palette strip container
    let strip_rect = Rect::from_min_max(p(0.15, 0.26), p(0.85, 0.74));
    painter.rect_stroke(strip_rect, 2.0, stroke, StrokeKind::Inside);

    // 4 Distinct gradient segment swatches inside
    let swatches = [
        (0.16, 0.33, Color32::from_rgb(68, 1, 84)), // Viridis dark purple
        (0.33, 0.50, Color32::from_rgb(49, 104, 142)), // Viridis blue
        (0.50, 0.67, Color32::from_rgb(53, 183, 121)), // Viridis green
        (0.67, 0.84, Color32::from_rgb(253, 231, 37)), // Viridis yellow
    ];

    for (x0, x1, col) in swatches {
        let sw_rect = Rect::from_min_max(p(x0, 0.28), p(x1, 0.72));
        painter.rect_filled(sw_rect, 0.0, col);
    }
}
