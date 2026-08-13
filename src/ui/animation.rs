use crate::app::OctantApp;

pub fn show_animation_controls(app: &mut OctantApp, ui: &mut egui::Ui) {
    let max_steps = app.animated_dim_extent();

    ui.horizontal(|ui| {
        // Play / Pause Button
        let play_text = if app.is_playing {
            "⏸ Pause"
        } else {
            "▶ Play"
        };
        if ui
            .button(egui::RichText::new(play_text).strong())
            .clicked()
        {
            app.is_playing = !app.is_playing;
            app.last_step_time = std::time::Instant::now();
        }

        // Step Prev
        if ui.button("◀").on_hover_text("Previous Step").clicked() {
            app.step_prev();
        }

        // Step Next
        if ui.button("▶").on_hover_text("Next Step").clicked() {
            app.step_next();
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
            app.request_step_or_load(displayed_step);
        }

        // FPS Speed Menu Dropdown
        ui.menu_button(format!("{:.0} FPS", app.playback_fps), |ui| {
            ui.label(egui::RichText::new("Playback Speed").strong());
            ui.add(egui::Slider::new(&mut app.playback_fps, 1.0..=60.0).suffix(" FPS"));
        });
    });
}
