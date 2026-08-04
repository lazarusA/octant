use crate::app::OctantApp;

pub fn show_animation_controls(app: &mut OctantApp, ui: &mut egui::Ui) {
    let max_steps = app
        .matrix_data
        .as_ref()
        .map(|h| h.max_timesteps)
        .unwrap_or(1);

    ui.horizontal(|ui| {
        // Play / Pause Button
        let play_text = if app.is_playing {
            "⏸ Pause"
        } else {
            "▶ Play"
        };
        if ui.button(egui::RichText::new(play_text).strong()).clicked() {
            app.is_playing = !app.is_playing;
            app.last_step_time = std::time::Instant::now();
        }

        // Step Prev
        if ui.button("◀").on_hover_text("Previous Timestep").clicked() {
            if app.current_timestep > 0 {
                app.current_timestep -= 1;
            } else if max_steps > 0 {
                app.current_timestep = max_steps - 1;
            }
            app.load_selected_variable_slice();
        }

        // Step Next
        if ui.button("▶").on_hover_text("Next Timestep").clicked() {
            if max_steps > 0 {
                app.current_timestep = (app.current_timestep + 1) % max_steps;
            }
            app.load_selected_variable_slice();
        }

        // Loop Toggle Checkbox
        ui.checkbox(&mut app.loop_playback, "🔄");

        // Timestep Timeline Slider
        let slider_max = max_steps.saturating_sub(1);
        ui.add_space(4.0);
        let slider_res = ui.add(
            egui::Slider::new(&mut app.current_timestep, 0..=slider_max)
                .show_value(false)
                .trailing_fill(true),
        );
        if slider_res.drag_stopped() || (slider_res.changed() && !app.is_playing) {
            app.load_selected_variable_slice();
        }

        // FPS Speed Menu Dropdown
        ui.menu_button(format!("{:.0} FPS", app.playback_fps), |ui| {
            ui.label(egui::RichText::new("Playback Speed").strong());
            ui.add(egui::Slider::new(&mut app.playback_fps, 1.0..=60.0).suffix(" FPS"));
        });
    });
}
