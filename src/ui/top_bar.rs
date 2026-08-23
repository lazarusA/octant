use super::{colormap, plot_type, status, store};
use crate::app::OctantApp;

pub fn show_top_bar(app: &mut OctantApp, ui: &mut egui::Ui) {
    egui::Panel::top("octant_top_bar")
        .exact_size(34.0)
        .show(ui, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                // Octant Brand Header (clickable to toggle Hero / Landing view)
                let brand_resp = ui
                    .horizontal(|ui| {
                        octant_icon(ui, 24.0);
                        ui.label(egui::RichText::new("Octant").strong().heading());
                    })
                    .response
                    .interact(egui::Sense::click())
                    .on_hover_text("Toggle Hero / Landing View");

                if brand_resp.clicked() {
                    app.show_hero = !app.show_hero;
                }
                ui.separator();

                // Dropdown menus: Store, Variables, Dimensions, Plot, Settings
                store::show_store_menu(app, ui);

                if ui
                    .button(egui::RichText::new("📊 Variables").strong())
                    .clicked()
                {
                    app.show_variables_overlay = !app.show_variables_overlay;
                }

                if ui
                    .button(egui::RichText::new("🎛️ Dimensions").strong())
                    .on_hover_text("Toggle Variable Controls Panel")
                    .clicked()
                {
                    app.show_variable_controls = !app.show_variable_controls;
                }

                plot_type::show_plot_type_menu(app, ui);
                colormap::show_colormap_menu(app, ui);

                if ui
                    .button(egui::RichText::new("⚙️ Settings").strong())
                    .clicked()
                {
                    app.show_settings_panel = !app.show_settings_panel;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let is_dark = app.theme_preference == egui::ThemePreference::Dark;
                    let theme_label = if is_dark { "☀ Light" } else { "🌙 Dark" };
                    let theme_hover = if is_dark {
                        "Switch to Light mode"
                    } else {
                        "Switch to Dark mode"
                    };
                    if ui
                        .button(egui::RichText::new(theme_label))
                        .on_hover_text(theme_hover)
                        .clicked()
                    {
                        app.theme_preference = if is_dark {
                            egui::ThemePreference::Light
                        } else {
                            egui::ThemePreference::Dark
                        };
                        ui.ctx().set_theme(app.theme_preference);
                    }

                    ui.separator();
                    status::show_status_bar(app, ui);
                });
            });
        });
}

fn octant_icon(ui: &mut egui::Ui, size: f32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let stroke = egui::Stroke::new(0.6, ui.visuals().text_color().gamma_multiply(0.6));

        let cos30 = 0.8660254_f32;
        let sin30 = 0.5_f32;

        // Project a 3D point (x, y, z), each in [-1, 1], to a 2D offset.
        // Origin (0,0,0) is the center of the whole 2x2x2 cube.
        let iso = |x: f32, y: f32, z: f32| -> egui::Vec2 {
            egui::vec2((x - z) * cos30, (x + z) * sin30 - y)
        };

        let scale = size / 4.4;
        let project =
            |x: f32, y: f32, z: f32| -> egui::Pos2 { rect.center() + iso(x, y, z) * scale };

        // Wireframe unit cube whose lower corner is at (ix, iy, iz), each -1 or 0.
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
                painter.line_segment([corners[a], corners[b]], stroke);
            }
        };

        // Filled unit cube: shade the 3 faces on its *outer* corner (away from the big cube's center),
        // so the shape correctly reads regardless of which octant is filled.
        let draw_filled_cube = |ix: f32, iy: f32, iz: f32| {
            let p = |dx: f32, dy: f32, dz: f32| project(ix + dx, iy + dy, iz + dz);

            let dx_outer = if ix < 0.0 { 0.0 } else { 1.0 };
            let dy_outer = if iy < 0.0 { 0.0 } else { 1.0 };
            let dz_outer = if iz < 0.0 { 0.0 } else { 1.0 };

            let face_x = vec![
                p(dx_outer, 0.0, 0.0),
                p(dx_outer, 0.0, 1.0),
                p(dx_outer, 1.0, 1.0),
                p(dx_outer, 1.0, 0.0),
            ];
            let face_y = vec![
                p(0.0, dy_outer, 0.0),
                p(1.0, dy_outer, 0.0),
                p(1.0, dy_outer, 1.0),
                p(0.0, dy_outer, 1.0),
            ];
            let face_z = vec![
                p(0.0, 0.0, dz_outer),
                p(1.0, 0.0, dz_outer),
                p(1.0, 1.0, dz_outer),
                p(0.0, 1.0, dz_outer),
            ];

            let base = ui.visuals().strong_text_color();
            let fill_stroke = egui::Stroke::new(0.6, base);
            let shade = |c: egui::Color32, f: f32| {
                egui::Color32::from_rgb(
                    (c.r() as f32 * f) as u8,
                    (c.g() as f32 * f) as u8,
                    (c.b() as f32 * f) as u8,
                )
            };

            painter.add(egui::Shape::convex_polygon(
                face_y,
                shade(base, 1.0),
                fill_stroke,
            ));
            painter.add(egui::Shape::convex_polygon(
                face_x,
                shade(base, 0.7),
                fill_stroke,
            ));
            painter.add(egui::Shape::convex_polygon(
                face_z,
                shade(base, 0.55),
                fill_stroke,
            ));
        };

        // Octant from (-1,-1,-1) to (0,0,0) is filled.
        for ix in [-1.0, 0.0] {
            for iy in [-1.0, 0.0] {
                for iz in [-1.0, 0.0] {
                    let is_filled = ix == -1.0 && iy == -1.0 && iz == -1.0;
                    if !is_filled {
                        draw_wire_cube(ix, iy, iz);
                    }
                }
            }
        }
        draw_filled_cube(-1.0, -1.0, -1.0); // drawn last so its edges stay crisp
    }

    response
}
