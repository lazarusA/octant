//! Procedural isometric wireframe and solid octant cube renderer.

/// Draws a big wireframe cube made of all 8 unit sub-cubes, with one
/// full solid mini-cube ("the octant") touring all 8 positions.
/// All wireframe lines remain fully intact during transitions.
pub fn draw_octant_widget(
    ui: &mut egui::Ui,
    size: f32,
    filled: [f32; 3],
    extra_rot: f32,
    extra_scale: f32,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let is_dark = ui.visuals().dark_mode;
        let wire_color = ui.visuals().weak_text_color().gamma_multiply(0.40);
        let wire_stroke = egui::Stroke::new(0.8, wire_color);

        let cos30 = 0.8660254_f32;
        let sin30 = 0.5_f32;

        let iso = |x: f32, y: f32, z: f32| -> egui::Vec2 {
            egui::vec2((x - z) * cos30, (x + z) * sin30 - y)
        };

        let scale = size / 4.2;
        let project =
            |x: f32, y: f32, z: f32| -> egui::Pos2 { rect.center() + iso(x, y, z) * scale };

        // Draw all 8 wireframe cubes without skipping any lines
        let draw_wire_cube = |ix: f32, iy: f32, iz: f32| {
            let corners: [egui::Pos2; 8] = std::array::from_fn(|i| {
                let dx = (i & 1) as f32;
                let dy = ((i >> 1) & 1) as f32;
                let dz = ((i >> 2) & 1) as f32;
                project(ix + dx, iy + dy, iz + dz)
            });
            let edges = [
                (0, 1),
                (0, 2),
                (0, 4),
                (1, 3),
                (1, 5),
                (2, 3),
                (2, 6),
                (3, 7),
                (4, 5),
                (4, 6),
                (5, 7),
                (6, 7),
            ];
            for (a, b) in edges {
                painter.line_segment([corners[a], corners[b]], wire_stroke);
            }
        };

        for ix in [-1.0_f32, 0.0] {
            for iy in [-1.0_f32, 0.0] {
                for iz in [-1.0_f32, 0.0] {
                    draw_wire_cube(ix, iy, iz);
                }
            }
        }

        // Full solid moving mini-cube (the octant)
        let [fx, fy, fz] = filled;
        let anchor = project(fx + 0.5, fy + 0.5, fz + 0.5);
        let (s, c) = extra_rot.sin_cos();
        let wobble = |pt: egui::Pos2| -> egui::Pos2 {
            let d = pt - anchor;
            let d = egui::vec2(d.x * c - d.y * s, d.x * s + d.y * c) * extra_scale;
            anchor + d
        };
        let p = |dx: f32, dy: f32, dz: f32| wobble(project(fx + dx, fy + dy, fz + dz));

        // The 3 visible isometric faces of a unit cube:
        // Top face (Y = 1 plane)
        let face_top = vec![
            p(0.0, 1.0, 0.0),
            p(1.0, 1.0, 0.0),
            p(1.0, 1.0, 1.0),
            p(0.0, 1.0, 1.0),
        ];
        // Right face (X = 1 plane)
        let face_right = vec![
            p(1.0, 0.0, 0.0),
            p(1.0, 1.0, 0.0),
            p(1.0, 1.0, 1.0),
            p(1.0, 0.0, 1.0),
        ];
        // Front-left face (Z = 1 plane)
        let face_left = vec![
            p(0.0, 0.0, 1.0),
            p(1.0, 0.0, 1.0),
            p(1.0, 1.0, 1.0),
            p(0.0, 1.0, 1.0),
        ];

        let base = ui.visuals().strong_text_color();
        let shade = |c: egui::Color32, f: f32| {
            egui::Color32::from_rgba_unmultiplied(
                ((c.r() as f32) * f).round() as u8,
                ((c.g() as f32) * f).round() as u8,
                ((c.b() as f32) * f).round() as u8,
                c.a(),
            )
        };

        // Solid octant is black in light mode, off-white in dark mode.
        // In light mode, crisp panel-fill white seams separate the black facets cleanly.
        let fill_stroke = if is_dark {
            egui::Stroke::new(1.0, base)
        } else {
            egui::Stroke::new(1.3, ui.visuals().panel_fill)
        };

        painter.add(egui::Shape::convex_polygon(
            face_top,
            shade(base, 1.0),
            fill_stroke,
        ));
        painter.add(egui::Shape::convex_polygon(
            face_right,
            shade(base, 0.72),
            fill_stroke,
        ));
        painter.add(egui::Shape::convex_polygon(
            face_left,
            shade(base, 0.52),
            fill_stroke,
        ));
    }

    response
}
