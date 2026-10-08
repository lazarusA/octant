use crate::app::OctantApp;

pub(crate) fn show_coastline_controls(app: &mut OctantApp, ui: &mut egui::Ui) {
    if app.is_ome_dataset() || !app.layers.base.composite.channel_configs.is_empty() {
        return;
    }
    ui.add_space(2.0);
    ui.horizontal(|ui| {
        ui.checkbox(&mut app.show_coastlines, "Coastlines")
            .on_hover_text("Overlay Natural Earth coastlines on geographic plots.");

        if app.show_coastlines {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if app.coastline_color.is_some()
                    && ui
                        .small_button("reset")
                        .on_hover_text("Reset coastline color to theme default")
                        .clicked()
                {
                    app.coastline_color = None;
                }

                let default_color = if ui.visuals().dark_mode {
                    [1.0_f32, 1.0, 1.0, 0.75]
                } else {
                    [0.15_f32, 0.15, 0.15, 0.85]
                };
                let mut color = app.coastline_color.unwrap_or(default_color);
                let initial_color = color;
                crate::ui::color_picker::ShapeColorPicker::new(
                    "settings_coastline_color_picker",
                    &mut color,
                    crate::ui::color_picker::ColorShape::Rect(3.0),
                )
                .size(egui::vec2(18.0, 16.0))
                .tooltip("Coastline line color. Click to select.")
                .anchor_offset(egui::vec2(-240.0, -100.0))
                .show(ui);

                if color != initial_color {
                    app.coastline_color = Some(color);
                }
            });
        }
    });

    if app.show_coastlines {
        show_coastline_details(app, ui);
    }
}

fn show_coastline_details(app: &mut OctantApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Resolution:").small().weak());
        ui.spacing_mut().item_spacing.x = 2.0;

        if app.coastline_is_loading {
            ui.spinner();
            ui.label(egui::RichText::new("Loading...").small().weak());
        } else {
            let lods = [
                (crate::plots::CoastlineLod::Lod110m, "110m"),
                (crate::plots::CoastlineLod::Lod50m, "50m"),
                (crate::plots::CoastlineLod::Lod10m, "10m"),
            ];
            for (lod, label) in lods {
                if ui
                    .selectable_label(
                        app.coastline_current_lod == lod,
                        egui::RichText::new(label).small(),
                    )
                    .on_hover_text(match lod {
                        crate::plots::CoastlineLod::Lod110m => {
                            "110 m — fast, always available (embedded)"
                        }
                        crate::plots::CoastlineLod::Lod50m => {
                            "50 m — medium detail (loaded in background)"
                        }
                        crate::plots::CoastlineLod::Lod10m => {
                            "10 m — high detail (loaded in background)"
                        }
                    })
                    .clicked()
                {
                    app.reload_coastline_lod(lod);
                }
            }
        }
    });

    ui.add(
        egui::Slider::new(&mut app.coastline_line_width, 1.0..=4.0)
            .step_by(0.1)
            .text("Line Width"),
    );

    ui.checkbox(
        &mut app.coastline_crop_to_data_domain,
        "Crop to data domain",
    )
    .on_hover_text("Clip coastlines strictly to the spatial boundary of the active dataset.");
}
