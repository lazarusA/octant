use crate::app::OctantApp;

/// Color scale (linear, log, symlog, sqrt, exponential) and its parameter.
pub(crate) fn show_scale_type_controls(app: &mut OctantApp, ui: &mut egui::Ui) {
    let is_valid_log =
        app.layers.base.color.range_min >= -1e-15 && app.layers.base.color.range_max > 0.0;
    if !is_valid_log && app.layers.base.color.scale_type == 1 {
        app.layers.base.color.scale_type = 0;
    }

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Scale").strong());
        egui::ComboBox::from_id_salt("settings_color_scale_dropdown")
            .selected_text(match app.layers.base.color.scale_type {
                1 => "Logarithmic",
                2 => "Symlog",
                3 => "Sqrt",
                4 => "Exponential",
                _ => "Linear",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut app.layers.base.color.scale_type, 0, "Linear");
                ui.add_enabled_ui(is_valid_log, |ui| {
                    ui.selectable_value(&mut app.layers.base.color.scale_type, 1, "Logarithmic")
                        .on_hover_text(if is_valid_log {
                            "Log scale (non-negative data)"
                        } else {
                            "Disabled: requires min >= 0. Use Symlog for negative data."
                        });
                });
                ui.selectable_value(&mut app.layers.base.color.scale_type, 2, "Symlog");
                ui.selectable_value(&mut app.layers.base.color.scale_type, 3, "Sqrt");
                ui.selectable_value(&mut app.layers.base.color.scale_type, 4, "Exponential");
            });
    });

    if app.layers.base.color.scale_type == 1
        || app.layers.base.color.scale_type == 2
        || app.layers.base.color.scale_type == 4
    {
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            ui.label("Param:");
            ui.add(
                egui::DragValue::new(&mut app.layers.base.color.scale_param)
                    .speed(0.01)
                    .range(0.0001..=100.0),
            );
        });
    }
}
