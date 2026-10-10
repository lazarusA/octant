use crate::app::OctantApp;

pub(crate) fn show_line_options(app: &mut OctantApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.checkbox(&mut app.plot_configs.line.show_lines, "Lines")
            .on_hover_text("Show continuous lines connecting points");
        ui.checkbox(&mut app.plot_configs.line.show_points, "Scatter")
            .on_hover_text("Show scatter markers at each point location");

        if !app.plot_configs.line.show_lines && !app.plot_configs.line.show_points {
            app.plot_configs.line.show_lines = true;
        }
    });

    if app.plot_configs.line.show_points {
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            ui.label("Point Size:");
            ui.add(
                egui::Slider::new(&mut app.plot_configs.line.point_size, 2.0..=24.0)
                    .suffix(" px")
                    .show_value(true),
            );
        });
    }

    ui.add_space(2.0);
    ui.horizontal(|ui| {
        ui.checkbox(&mut app.plot_configs.line.use_custom_color, "Custom Color")
            .on_hover_text("Use a solid line/scatter color instead of colormap evaluation");

        if app.plot_configs.line.use_custom_color {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                crate::ui::color_picker::ShapeColorPicker::new(
                    "settings_line_color_picker",
                    &mut app.plot_configs.line.line_color,
                    crate::ui::color_picker::ColorShape::Rect(3.0),
                )
                .size(egui::vec2(18.0, 16.0))
                .tooltip("Line / scatter color. Click to select color.")
                .anchor_offset(egui::vec2(-240.0, -100.0))
                .show(ui);
            });
        }
    });

    show_line_profile_controls(app, ui);
}

fn show_line_profile_controls(app: &mut OctantApp, ui: &mut egui::Ui) {
    let has_z_dim = app
        .layers
        .base
        .data
        .volume
        .as_ref()
        .is_some_and(|v| v.depth > 1);
    let profile_controls = app
        .layers
        .base
        .data
        .matrix
        .as_ref()
        .map_or((false, 0usize), |matrix| {
            (
                matrix.width > 1 || matrix.height > 1 || has_z_dim,
                matrix.width.max(matrix.height),
            )
        });

    if !profile_controls.0 {
        return;
    }

    ui.separator();
    ui.label(egui::RichText::new("Line Profile").small().weak());

    let label_x = app.get_spatial_dim_label(0);
    let label_y = app.get_spatial_dim_label(1);
    let label_z = app.get_spatial_dim_label(2);

    let selected_text = match app.plot_configs.line.profile_dim_idx {
        2 if has_z_dim => &label_z,
        1 => &label_y,
        _ => &label_x,
    };

    let mut selected_dim_idx = app.plot_configs.line.profile_dim_idx;
    egui::ComboBox::from_id_salt("line_profile_dim_selector")
        .selected_text(selected_text)
        .show_ui(ui, |ui| {
            if ui
                .selectable_label(selected_dim_idx == 0, &label_x)
                .clicked()
            {
                selected_dim_idx = 0;
            }
            if ui
                .selectable_label(selected_dim_idx == 1, &label_y)
                .clicked()
            {
                selected_dim_idx = 1;
            }
            if has_z_dim
                && ui
                    .selectable_label(selected_dim_idx == 2, &label_z)
                    .clicked()
            {
                selected_dim_idx = 2;
            }
        });
    if selected_dim_idx != app.plot_configs.line.profile_dim_idx {
        app.plot_configs.line.profile_dim_idx = selected_dim_idx;
        app.plot_configs.line.profile_slice_idx = 0;
    }

    let mut all_series = app.plot_configs.line.all_series;
    if ui.checkbox(&mut all_series, "All Lines Series").changed() {
        app.plot_configs.line.all_series = all_series;
    }

    if !app.plot_configs.line.all_series {
        show_line_profile_slider(app, ui, has_z_dim);
    }
}

fn show_line_profile_slider(app: &mut OctantApp, ui: &mut egui::Ui, has_z_dim: bool) {
    let max_idx = match app.plot_configs.line.profile_dim_idx {
        2 if has_z_dim => app
            .layers
            .base
            .data
            .volume
            .as_ref()
            .map_or(0, |v| (v.width * v.height).saturating_sub(1)),
        1 => app
            .layers
            .base
            .data
            .matrix
            .as_ref()
            .map_or(0, |matrix| matrix.width.saturating_sub(1)),
        _ => app
            .layers
            .base
            .data
            .matrix
            .as_ref()
            .map_or(0, |matrix| matrix.height.saturating_sub(1)),
    };
    if max_idx > 0 {
        let mut slice_idx = app.plot_configs.line.profile_slice_idx;
        if ui
            .add(egui::Slider::new(&mut slice_idx, 0..=max_idx).text("Profile Index"))
            .changed()
        {
            app.plot_configs.line.profile_slice_idx = slice_idx;
        }
    } else {
        ui.label("Single profile available.");
    }
}

pub(crate) fn show_heatmap_options(app: &mut OctantApp, ui: &mut egui::Ui) {
    ui.add_space(2.0);
    ui.checkbox(&mut app.layout.enforce_data_aspect_ratio, "Aspect Ratio")
        .on_hover_text("If checked, 2D plots preserve matrix data aspect ratio (width/height). If unchecked, 2D plots expand to fill full canvas.");
}
