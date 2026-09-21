use crate::app::OctantApp;
use crate::ui::settings::coastline::show_coastline_controls;

pub(crate) fn show_line_options(app: &mut OctantApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        let mut use_flat = app.active_colormap == 999;
        if ui
            .checkbox(&mut use_flat, "Solid Flat Line Color")
            .changed()
        {
            if use_flat {
                app.active_colormap = 999;
            } else {
                app.active_colormap = 0;
            }
        }
        ui.selectable_label(app.show_hover_card, "Hover Card")
            .on_hover_text(if app.show_hover_card {
                "Hide hover card"
            } else {
                "Show hover card"
            })
            .clicked()
            .then(|| app.show_hover_card = !app.show_hover_card);
    });

    show_line_profile_controls(app, ui);
}

fn show_line_profile_controls(app: &mut OctantApp, ui: &mut egui::Ui) {
    let has_z_dim = app.volume_data.as_ref().is_some_and(|v| v.depth > 1);
    let profile_controls = app.matrix_data.as_ref().map_or((false, 0usize), |matrix| {
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

    let selected_text = match app.line_profile_dim_idx {
        2 if has_z_dim => &label_z,
        1 => &label_y,
        _ => &label_x,
    };

    let mut selected_dim_idx = app.line_profile_dim_idx;
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
    if selected_dim_idx != app.line_profile_dim_idx {
        app.line_profile_dim_idx = selected_dim_idx;
        app.line_profile_slice_idx = 0;
    }

    let mut all_series = app.line_plot_all_series;
    if ui.checkbox(&mut all_series, "All Lines Series").changed() {
        app.line_plot_all_series = all_series;
    }

    if !app.line_plot_all_series {
        show_line_profile_slider(app, ui, has_z_dim);
    }
}

fn show_line_profile_slider(app: &mut OctantApp, ui: &mut egui::Ui, has_z_dim: bool) {
    let max_idx = match app.line_profile_dim_idx {
        2 if has_z_dim => app
            .volume_data
            .as_ref()
            .map_or(0, |v| (v.width * v.height).saturating_sub(1)),
        1 => app
            .matrix_data
            .as_ref()
            .map_or(0, |matrix| matrix.width.saturating_sub(1)),
        _ => app
            .matrix_data
            .as_ref()
            .map_or(0, |matrix| matrix.height.saturating_sub(1)),
    };
    if max_idx > 0 {
        let mut slice_idx = app.line_profile_slice_idx;
        if ui
            .add(egui::Slider::new(&mut slice_idx, 0..=max_idx).text("Profile Index"))
            .changed()
        {
            app.line_profile_slice_idx = slice_idx;
        }
    } else {
        ui.label("Single profile available.");
    }
}

pub(crate) fn show_heatmap_options(app: &mut OctantApp, ui: &mut egui::Ui) {
    ui.add_space(2.0);
    ui.horizontal(|ui| {
        ui.checkbox(&mut app.enforce_data_aspect_ratio, "Aspect Ratio")
            .on_hover_text("If checked, 2D plots preserve matrix data aspect ratio (width/height). If unchecked, 2D plots expand to fill full canvas.");
        ui.selectable_label(app.show_hover_card, "Hover Card")
            .on_hover_text(if app.show_hover_card { "Hide hover card" } else { "Show hover card" })
            .clicked().then(|| app.show_hover_card = !app.show_hover_card);
    });

    if app.has_rgb_bands() {
        super::composite::show_composite_controls(app, ui);
    } else if app.rgb_composite_mode {
        app.rgb_composite_mode = false;
        if app.active_colormap == 1000 {
            app.active_colormap = 0;
        }
    }

    show_coastline_controls(app, ui);
}
