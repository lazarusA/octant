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
        if ui.add(play_button).clicked() {
            app.is_playing = !app.is_playing;
            app.last_step_time = std::time::Instant::now();
        }

        let max_steps = app
            .matrix_data
            .as_ref()
            .map(|h| h.max_timesteps)
            .unwrap_or(1);

        // 2. Prev Step
        if ui.button("◀").on_hover_text("Previous Step").clicked() {
            if app.current_timestep > 0 {
                app.current_timestep -= 1;
            } else if max_steps > 0 {
                app.current_timestep = max_steps - 1;
            }
            app.load_selected_variable_slice();
        }

        // 3. Next Step
        if ui.button("▶").on_hover_text("Next Step").clicked() {
            if max_steps > 0 {
                app.current_timestep = (app.current_timestep + 1) % max_steps;
            }
            app.load_selected_variable_slice();
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

        // 6. Timestep timeline slider & Dimension-Agnostic Axis Reading
        let active_var_info = app
            .active_dataset_metadata
            .as_ref()
            .and_then(|m| m.variables.get(app.selected_variable_idx));

        let active_anim_dim = active_var_info
            .and_then(|v| v.dimension_names.first().cloned())
            .unwrap_or_else(|| "time".to_string());

        let direct_coord_label = app
            .active_dataset_metadata
            .as_ref()
            .and_then(|m| m.dimension_coordinates.get(&active_anim_dim.to_lowercase()))
            .and_then(|coords| coords.get(app.current_timestep).cloned());

        let formatted_axis = if let Some(coord_str) = direct_coord_label {
            coord_str
        } else {
            let active_dim_name = active_var_info.and_then(|v| v.dimension_names.first().cloned());
            let active_units = active_var_info.and_then(|v| v.units.as_deref());
            let time_start = active_var_info.and_then(|v| v.time_coverage_start.as_deref());
            let temp_res = active_var_info.and_then(|v| v.temporal_resolution.as_deref());

            crate::utils::units::format_axis_value(
                app.current_timestep,
                max_steps,
                active_dim_name.as_deref(),
                active_units,
                time_start,
                temp_res,
                Some(&app.store_target_input),
            )
        };

        let slider_max = max_steps.saturating_sub(1);
        ui.label(
            egui::RichText::new(format!("📅 {}", formatted_axis))
                .small()
                .monospace()
                .strong(),
        )
        .on_hover_text(format!("Step {} / {}", app.current_timestep + 1, max_steps));

        let slider_res = ui.add(
            egui::Slider::new(&mut app.current_timestep, 0..=slider_max)
                .show_value(false)
                .trailing_fill(true),
        );
        if slider_res.drag_stopped() || (slider_res.changed() && !app.is_playing) {
            app.load_selected_variable_slice();
        }

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
