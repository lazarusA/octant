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
