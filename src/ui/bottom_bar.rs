use crate::app::OctantApp;

pub fn show_bottom_bar(app: &mut OctantApp, ui: &mut egui::Ui) {
    // Extract to a local bool to avoid split-borrow: we can't hold &mut app.field
    // AND also borrow all of app inside the closure at the same time.
    let mut expanded = app.show_bottom_bar;

    egui::Panel::show_switched(
        ui,
        &mut expanded,
        egui::Panel::bottom("octant_bottom_bar_collapsed")
            .resizable(true)
            .exact_size(20.0),
        egui::Panel::bottom("octant_bottom_bar_expanded")
            .resizable(true)
            .size_range(38.0..=80.0),
        |ui, is_expanded| {
            if is_expanded {
                show_bottom_bar_content(app, ui);
            } else {
                ui.vertical_centered(|ui| {
                    ui.small("▲ Playback (drag to expand)");
                });
            }
        },
    );

    app.show_bottom_bar = expanded;
}

fn show_bottom_bar_content(app: &mut OctantApp, ui: &mut egui::Ui) {
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        // 1. Play / Pause Button for Timestep Animation across all plot types
        let play_text = if app.is_playing {
            "⏸ Pause"
        } else {
            "▶ Play"
        };
        let play_button = egui::Button::new(egui::RichText::new(play_text).strong());
        if ui.add(play_button).clicked() {
            app.is_playing = !app.is_playing;
            app.last_step_time = std::time::Instant::now();
        }

        let max_steps = app.animated_dim_extent();

        // 2. Prev Step
        if ui.button("◀").on_hover_text("Previous Step").clicked() {
            app.step_prev();
        }

        // 3. Next Step
        if ui.button("▶").on_hover_text("Next Step").clicked() {
            app.step_next();
        }

        // 4. Loop Toggle
        ui.checkbox(&mut app.loop_playback, "🔄 Loop");

        ui.separator();

        let status_text = if app.is_playing {
            "▶ Playing"
        } else {
            "⏸ Paused"
        };
        let status_color = if app.is_playing {
            egui::Color32::from_rgb(255, 99, 71)
        } else {
            egui::Color32::LIGHT_GRAY
        };
        ui.label(egui::RichText::new(status_text).small().color(status_color));

        ui.separator();

        // 6. Step timeline slider & Dimension-Agnostic Axis Reading
        let anim_dim_idx = app.plotted_animated_dim.unwrap_or(0);
        let (active_anim_dim, active_dim_name, active_units, time_start, temp_res) =
            if let Some(plotted_var_info) = app
                .plotted_dataset_metadata
                .as_ref()
                .and_then(|m| m.variables.get(app.plotted_variable_idx))
            {
                let name = plotted_var_info
                    .dimension_names
                    .get(anim_dim_idx)
                    .cloned()
                    .unwrap_or_else(|| "step".to_string());
                (
                    name.clone(),
                    Some(name),
                    plotted_var_info.units.clone(),
                    plotted_var_info.time_coverage_start.clone(),
                    plotted_var_info.temporal_resolution.clone(),
                )
            } else {
                ("step".to_string(), None, None, None, None)
            };

        let dim_coords = app
            .plotted_dataset_metadata
            .as_ref()
            .and_then(|m| m.dimension_coordinates.get(&active_anim_dim.to_lowercase()));

        let direct_coord_label =
            dim_coords.and_then(|coords| coords.get(app.current_timestep).cloned());
        let first_coord = dim_coords.and_then(|coords| coords.first().cloned());
        let last_coord = dim_coords.and_then(|coords| coords.last().cloned());

        let format_dim_step = |step: usize, default_coord: Option<String>| -> String {
            if let Some(ref coord) = default_coord {
                let is_raw_numeric = coord.parse::<f64>().is_ok()
                    && !coord.contains('-')
                    && !coord.contains(':')
                    && !coord.contains('/')
                    && !coord.contains('T');

                if !is_raw_numeric && !coord.trim().is_empty() {
                    return coord.clone();
                }
            }

            crate::utils::units::format_axis_value(
                step,
                max_steps,
                active_dim_name.as_deref(),
                active_units.as_deref(),
                time_start.as_deref(),
                temp_res.as_deref(),
                Some(&app.plotted_store_target_input),
            )
        };

        let formatted_axis = format_dim_step(app.current_timestep, direct_coord_label);
        let start_date_str = format_dim_step(0, first_coord);
        let end_date_str = format_dim_step(max_steps.saturating_sub(1), last_coord);

        let step_size_str = temp_res.unwrap_or_else(|| "Step: 1".to_string());

        ui.label(
            egui::RichText::new(format!(
                "📅 {} | Current: {}",
                start_date_str, formatted_axis
            ))
            .small()
            .monospace()
            .strong(),
        )
        .on_hover_text(format!(
            "Start: {} | End: {} | {}",
            start_date_str, end_date_str, step_size_str
        ));

        let slider_max = max_steps.saturating_sub(1);
        ui.small(format!("[{}]", start_date_str));

        let mut displayed_step = app.current_timestep;
        let slider_res = ui.add(
            egui::Slider::new(&mut displayed_step, 0..=slider_max)
                .show_value(false)
                .trailing_fill(true),
        );

        if slider_res.changed() {
            app.request_step_or_load(displayed_step);
        }

        ui.small(format!("⏱ {}", step_size_str));
        ui.small(format!("[{}]", end_date_str));

        ui.separator();

        // 7. Playback speed slider
        let fps_int = app.playback_fps.round() as u32;
        let fps_label = match fps_int {
            1 => "1 FPS",
            2 => "2 FPS",
            3 => "3 FPS",
            4 => "4 FPS",
            5 => "5 FPS",
            6 => "6 FPS",
            8 => "8 FPS",
            10 => "10 FPS",
            12 => "12 FPS",
            15 => "15 FPS",
            20 => "20 FPS",
            24 => "24 FPS",
            30 => "30 FPS",
            60 => "60 FPS",
            _ => "",
        };

        let menu_body = |ui: &mut egui::Ui| {
            ui.label(egui::RichText::new("Playback Speed").strong());
            ui.add(egui::Slider::new(&mut app.playback_fps, 1.0..=60.0).suffix(" FPS"));
        };

        if !fps_label.is_empty() {
            ui.menu_button(fps_label, menu_body);
        } else {
            ui.menu_button(format!("{} FPS", fps_int), menu_body);
        }
    });
}
