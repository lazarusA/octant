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
        let is_dark = ui.visuals().dark_mode;
        let wire_color = ui.visuals().weak_text_color().gamma_multiply(0.40);
        let wire_stroke = egui::Stroke::new(0.7, wire_color);

        let cos30 = 0.8660254_f32;
        let sin30 = 0.5_f32;

        let iso = |x: f32, y: f32, z: f32| -> egui::Vec2 {
            egui::vec2((x - z) * cos30, (x + z) * sin30 - y)
        };

        let scale = size / 4.2;
        let project =
            |x: f32, y: f32, z: f32| -> egui::Pos2 { rect.center() + iso(x, y, z) * scale };

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

        // Draw all 8 wireframe cubes without skipping any lines
        for ix in [-1.0_f32, 0.0] {
            for iy in [-1.0_f32, 0.0] {
                for iz in [-1.0_f32, 0.0] {
                    draw_wire_cube(ix, iy, iz);
                }
            }
        }

        // The filled octant at [-1.0, -1.0, -1.0]
        let p = |dx: f32, dy: f32, dz: f32| project(-1.0 + dx, -1.0 + dy, -1.0 + dz);

        let face_top = vec![
            p(0.0, 1.0, 0.0),
            p(1.0, 1.0, 0.0),
            p(1.0, 1.0, 1.0),
            p(0.0, 1.0, 1.0),
        ];
        let face_right = vec![
            p(1.0, 0.0, 0.0),
            p(1.0, 1.0, 0.0),
            p(1.0, 1.0, 1.0),
            p(1.0, 0.0, 1.0),
        ];
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
            egui::Stroke::new(0.8, base)
        } else {
            egui::Stroke::new(1.0, ui.visuals().panel_fill)
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
