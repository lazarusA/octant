//! Data store, file tree, cache, and filesystem procedural vector icons.

use egui::{Color32, Painter, Pos2, Rect, Stroke, StrokeKind, pos2};

pub fn draw_folder(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Folder body with top tab
    let body_pts = vec![
        p(0.14, 0.28),
        p(0.42, 0.28),
        p(0.50, 0.38),
        p(0.86, 0.38),
        p(0.86, 0.82),
        p(0.14, 0.82),
    ];

    painter.add(egui::Shape::convex_polygon(body_pts, fill, stroke));
}

pub fn draw_folder_open(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Back tab
    let back_pts = vec![
        p(0.14, 0.24),
        p(0.42, 0.24),
        p(0.50, 0.34),
        p(0.84, 0.34),
        p(0.84, 0.50),
        p(0.14, 0.50),
    ];
    painter.add(egui::Shape::convex_polygon(back_pts, fill, stroke));

    // Front tilted flap
    let front_pts = vec![p(0.10, 0.82), p(0.24, 0.44), p(0.90, 0.44), p(0.76, 0.82)];
    painter.add(egui::Shape::convex_polygon(
        front_pts,
        stroke.color.gamma_multiply(0.20),
        stroke,
    ));
}

pub fn draw_variable_doc(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Sheet with dog-ear folded corner
    let sheet_pts = vec![
        p(0.22, 0.16),
        p(0.58, 0.16),
        p(0.78, 0.36),
        p(0.78, 0.84),
        p(0.22, 0.84),
    ];
    painter.add(egui::Shape::convex_polygon(sheet_pts, fill, stroke));

    // Dog-ear fold triangle
    let fold_pts = vec![p(0.58, 0.16), p(0.78, 0.36), p(0.58, 0.36)];
    painter.add(egui::Shape::convex_polygon(
        fold_pts,
        stroke.color.gamma_multiply(0.30),
        stroke,
    ));

    // Internal text lines
    let line_stroke = Stroke::new(stroke.width * 0.9, stroke.color.gamma_multiply(0.60));
    painter.line_segment([p(0.32, 0.48), p(0.68, 0.48)], line_stroke);
    painter.line_segment([p(0.32, 0.62), p(0.60, 0.62)], line_stroke);
}

pub fn draw_icechunk(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // 3D Isometric Crystal Cube / Icechunk
    let top_face = vec![p(0.50, 0.15), p(0.82, 0.33), p(0.50, 0.51), p(0.18, 0.33)];
    let left_face = vec![p(0.18, 0.33), p(0.50, 0.51), p(0.50, 0.85), p(0.18, 0.67)];
    let right_face = vec![p(0.50, 0.51), p(0.82, 0.33), p(0.82, 0.67), p(0.50, 0.85)];

    painter.add(egui::Shape::convex_polygon(top_face, fill, stroke));
    painter.add(egui::Shape::convex_polygon(
        left_face,
        stroke.color.gamma_multiply(0.22),
        stroke,
    ));
    painter.add(egui::Shape::convex_polygon(
        right_face,
        stroke.color.gamma_multiply(0.40),
        stroke,
    ));
}

pub fn draw_catalog(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // 3 Standing / Leaning Books
    let book1 = Rect::from_min_max(p(0.16, 0.22), p(0.36, 0.82));
    let book2 = Rect::from_min_max(p(0.40, 0.18), p(0.60, 0.82));

    painter.rect(book1, 1.0, fill, stroke, StrokeKind::Inside);
    painter.rect(
        book2,
        1.0,
        stroke.color.gamma_multiply(0.25),
        stroke,
        StrokeKind::Inside,
    );

    // Leaning book 3
    let b3_pts = vec![p(0.64, 0.32), p(0.82, 0.22), p(0.86, 0.82), p(0.68, 0.82)];
    painter.add(egui::Shape::convex_polygon(b3_pts, fill, stroke));
}

pub fn draw_save(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Floppy disk body
    let disk_pts = vec![
        p(0.16, 0.16),
        p(0.72, 0.16),
        p(0.84, 0.28),
        p(0.84, 0.84),
        p(0.16, 0.84),
    ];
    painter.add(egui::Shape::convex_polygon(disk_pts, fill, stroke));

    // Top metal slider
    let slider = Rect::from_min_max(p(0.32, 0.16), p(0.68, 0.44));
    painter.rect_stroke(slider, 0.0, stroke, StrokeKind::Inside);

    // Bottom label area
    let label_rect = Rect::from_min_max(p(0.26, 0.54), p(0.74, 0.84));
    painter.rect_stroke(label_rect, 0.0, stroke, StrokeKind::Inside);
}

pub fn draw_snapshot(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Camera body with top prism
    let body_pts = vec![
        p(0.14, 0.34),
        p(0.32, 0.34),
        p(0.40, 0.22),
        p(0.60, 0.22),
        p(0.68, 0.34),
        p(0.86, 0.34),
        p(0.86, 0.80),
        p(0.14, 0.80),
    ];
    painter.add(egui::Shape::convex_polygon(body_pts, fill, stroke));

    // Center lens circle
    painter.circle_stroke(p(0.50, 0.57), rect.width() * 0.16, stroke);
}

pub fn draw_drop_tray(painter: &Painter, rect: Rect, stroke: Stroke) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // U-shaped tray bottom
    let tray_pts = [p(0.16, 0.52), p(0.16, 0.82), p(0.84, 0.82), p(0.84, 0.52)];
    for win in tray_pts.windows(2) {
        painter.line_segment([win[0], win[1]], stroke);
    }

    // Downward arrow
    let arrow_stem = [p(0.50, 0.18), p(0.50, 0.60)];
    painter.line_segment(arrow_stem, stroke);

    let arrow_head = [p(0.34, 0.44), p(0.50, 0.60), p(0.66, 0.44)];
    painter.line_segment([arrow_head[0], arrow_head[1]], stroke);
    painter.line_segment([arrow_head[1], arrow_head[2]], stroke);
}

pub fn draw_search(painter: &Painter, rect: Rect, stroke: Stroke) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Magnifying glass lens
    let center = p(0.44, 0.44);
    let r = rect.width() * 0.24;
    painter.circle_stroke(center, r, stroke);

    // Diagonal handle
    let handle_start = p(0.61, 0.61);
    let handle_end = p(0.84, 0.84);
    let handle_stroke = Stroke::new(stroke.width * 1.5, stroke.color);
    painter.line_segment([handle_start, handle_end], handle_stroke);
}

pub fn draw_trash(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Lid top bar and handle
    painter.line_segment([p(0.16, 0.28), p(0.84, 0.28)], stroke);
    let handle_pts = [p(0.40, 0.28), p(0.40, 0.18), p(0.60, 0.18), p(0.60, 0.28)];
    for win in handle_pts.windows(2) {
        painter.line_segment([win[0], win[1]], stroke);
    }

    // Can body
    let body_pts = vec![p(0.24, 0.28), p(0.76, 0.28), p(0.70, 0.84), p(0.30, 0.84)];
    painter.add(egui::Shape::convex_polygon(body_pts, fill, stroke));

    // Vertical flutes/ribs
    let flute_stroke = Stroke::new(stroke.width * 0.8, stroke.color.gamma_multiply(0.60));
    painter.line_segment([p(0.42, 0.38), p(0.42, 0.74)], flute_stroke);
    painter.line_segment([p(0.58, 0.38), p(0.58, 0.74)], flute_stroke);
}

pub fn draw_clipboard(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Clipboard body
    let board = Rect::from_min_max(p(0.20, 0.22), p(0.80, 0.86));
    painter.rect(board, 2.0, fill, stroke, StrokeKind::Inside);

    // Top clip clamp
    let clip = Rect::from_min_max(p(0.36, 0.14), p(0.64, 0.26));
    painter.rect(
        clip,
        1.0,
        stroke.color.gamma_multiply(0.30),
        stroke,
        StrokeKind::Inside,
    );

    // Document horizontal lines
    let line_stroke = Stroke::new(stroke.width * 0.8, stroke.color.gamma_multiply(0.55));
    painter.line_segment([p(0.32, 0.42), p(0.68, 0.42)], line_stroke);
    painter.line_segment([p(0.32, 0.56), p(0.68, 0.56)], line_stroke);
    painter.line_segment([p(0.32, 0.70), p(0.54, 0.70)], line_stroke);
}
