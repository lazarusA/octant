use crate::app::OctantApp;

pub fn show_animation_controls(app: &mut OctantApp, ui: &mut egui::Ui) {
    let max_steps = app.animated_dim_extent();

    let is_3d_plot = app.active_plot_type == crate::plots::PlotType::Volume
        || app.active_plot_type == crate::plots::PlotType::PointCloud;

    ui.horizontal(|ui| {
        // Play / Pause Button
        let play_text = if app.is_playing {
            "⏸ Pause"
        } else {
            "▶ Play"
        };
        let mut play_btn = ui.add_enabled(
            !is_3d_plot,
            egui::Button::new(egui::RichText::new(play_text).strong()),
        );
        if is_3d_plot {
            play_btn = play_btn.on_hover_text("Playback disabled for 3D Volume/Point Cloud plots");
        }
        if play_btn.clicked() {
            app.is_playing = !app.is_playing;
            app.last_step_time = std::time::Instant::now();
        }

        let source_id = format!(
            "{:?}:{}",
            app.plotted_store_kind, app.plotted_store_target_input
        );
        let var_name = app
            .plotted_dataset_metadata
            .as_ref()
            .and_then(|m| m.variables.get(app.plotted_variable_idx))
            .map(|v| v.name.clone());

        // Step Prev
        if ui.button("◀").on_hover_text("Previous Step").clicked() {
            let target_step = if app.current_timestep > 0 {
                app.current_timestep - 1
            } else if max_steps > 0 {
                max_steps - 1
            } else {
                0
            };

            let is_cached = if let Some(ref name) = var_name {
                app.block_cache
                    .covers(&source_id, name, app.plotted_animated_dim, target_step)
            } else {
                false
            };

            if is_cached {
                app.current_timestep = target_step;
                app.load_selected_variable_block();
            } else {
                app.prefetch_block_window_for_next_steps(target_step);
            }
        }

        // Step Next
        if ui.button("▶").on_hover_text("Next Step").clicked() {
            let target_step = if max_steps > 0 {
                (app.current_timestep + 1) % max_steps
            } else {
                0
            };

            let is_cached = if let Some(ref name) = var_name {
                app.block_cache
                    .covers(&source_id, name, app.plotted_animated_dim, target_step)
            } else {
                false
            };

            if is_cached {
                app.current_timestep = target_step;
                app.load_selected_variable_block();
            } else {
                app.prefetch_block_window_for_next_steps(target_step);
            }
        }

        // Loop Toggle Checkbox
        ui.checkbox(&mut app.loop_playback, "🔄");

        // Step Timeline Slider
        let slider_max = max_steps.saturating_sub(1);
        ui.add_space(4.0);

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

        // FPS Speed Menu Dropdown
        ui.menu_button(format!("{:.0} FPS", app.playback_fps), |ui| {
            ui.label(egui::RichText::new("Playback Speed").strong());
            ui.add(egui::Slider::new(&mut app.playback_fps, 1.0..=60.0).suffix(" FPS"));
        });
    });
}
