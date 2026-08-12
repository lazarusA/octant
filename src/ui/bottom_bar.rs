use super::cache;
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
    let is_3d_plot = app.active_plot_type == crate::plots::PlotType::Volume
        || app.active_plot_type == crate::plots::PlotType::PointCloud;

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        // 1. Play / Pause Button for Timestep Animation across all plot types
        let play_text = if app.is_playing { "⏸" } else { "▶" };
        let play_button = egui::Button::new(
            egui::RichText::new(format!(
                "{} {}",
                play_text,
                if app.is_playing { "Pause" } else { "Play" }
            ))
            .strong(),
        );
        let mut play_btn_res = ui.add_enabled(!is_3d_plot, play_button);
        if is_3d_plot {
            play_btn_res =
                play_btn_res.on_hover_text("Playback disabled for 3D Volume/Point Cloud plots");
        }
        if play_btn_res.clicked() {
            app.is_playing = !app.is_playing;
            app.last_step_time = std::time::Instant::now();
        }

        let max_steps = app.animated_dim_extent();

        // 2. Prev Step
        if ui.button("◀").on_hover_text("Previous Step").clicked() {
            if app.current_timestep > 0 {
                app.current_timestep -= 1;
            } else if max_steps > 0 {
                app.current_timestep = max_steps - 1;
            }
            app.load_selected_variable_block();
        }

        // 3. Next Step
        if ui.button("▶").on_hover_text("Next Step").clicked() {
            if max_steps > 0 {
                app.current_timestep = (app.current_timestep + 1) % max_steps;
            }
            app.load_selected_variable_block();
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

        let direct_coord_label = app
            .plotted_dataset_metadata
            .as_ref()
            .and_then(|m| m.dimension_coordinates.get(&active_anim_dim.to_lowercase()))
            .and_then(|coords| coords.get(app.current_timestep).cloned());

        let formatted_axis = if let Some(coord_str) = direct_coord_label {
            coord_str
        } else {
            crate::utils::units::format_axis_value(
                app.current_timestep,
                max_steps,
                active_dim_name.as_deref(),
                active_units.as_deref(),
                time_start.as_deref(),
                temp_res.as_deref(),
                Some(&app.plotted_store_target_input),
            )
        };

        let start_date_str = if let Some(coord_str) = app
            .plotted_dataset_metadata
            .as_ref()
            .and_then(|m| m.dimension_coordinates.get(&active_anim_dim.to_lowercase()))
            .and_then(|coords| coords.first().cloned())
        {
            coord_str
        } else {
            crate::utils::units::format_axis_value(
                0,
                max_steps,
                active_dim_name.as_deref(),
                active_units.as_deref(),
                time_start.as_deref(),
                temp_res.as_deref(),
                Some(&app.plotted_store_target_input),
            )
        };

        let end_date_str = if let Some(coord_str) = app
            .plotted_dataset_metadata
            .as_ref()
            .and_then(|m| m.dimension_coordinates.get(&active_anim_dim.to_lowercase()))
            .and_then(|coords| coords.last().cloned())
        {
            coord_str
        } else {
            crate::utils::units::format_axis_value(
                max_steps.saturating_sub(1),
                max_steps,
                active_dim_name.as_deref(),
                active_units.as_deref(),
                time_start.as_deref(),
                temp_res.as_deref(),
                Some(&app.plotted_store_target_input),
            )
        };

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

        let source_id = format!(
            "{:?}:{}",
            app.plotted_store_kind, app.plotted_store_target_input
        );
        let var_name = app
            .plotted_dataset_metadata
            .as_ref()
            .and_then(|m| m.variables.get(app.plotted_variable_idx))
            .map(|v| v.name.clone());

        let mut displayed_step = app.current_timestep;
        let slider_res = ui.add(
            egui::Slider::new(&mut displayed_step, 0..=slider_max)
                .show_value(false)
                .trailing_fill(true),
        );

        if slider_res.changed() {
            let is_cached = if let Some(ref name) = var_name {
                app.block_cache
                    .covers(&source_id, name, app.plotted_animated_dim, displayed_step)
            } else {
                false
            };

            if is_cached {
                app.current_timestep = displayed_step;
                app.load_selected_variable_block();
            } else if slider_res.drag_stopped() || !app.is_playing {
                app.prefetch_block_window_for_next_steps(displayed_step);
            }
        }

        ui.small(format!("⏱ {}", step_size_str));
        ui.small(format!("[{}]", end_date_str));

        ui.separator();

        // 7. Playback speed slider
        ui.menu_button(format!("{:.0} FPS", app.playback_fps), |ui| {
            ui.label(egui::RichText::new("Playback Speed").strong());
            ui.add(egui::Slider::new(&mut app.playback_fps, 1.0..=60.0).suffix(" FPS"));
        });

        // 8. Bottom Right: Cache Menu Dropdown
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            cache::show_cache_menu(app, ui);
        });
    });
}
