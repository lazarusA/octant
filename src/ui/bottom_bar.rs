use crate::app::OctantApp;
use super::cache;

pub fn show_bottom_bar(app: &mut OctantApp, ctx: &egui::Context) {
    egui::TopBottomPanel::bottom("octant_bottom_bar")
        .exact_height(38.0)
        .show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                // 1. Play / Pause Button
                let play_text = if app.is_playing { "⏸ Pause" } else { "▶ Play" };
                if ui.button(egui::RichText::new(play_text).strong()).clicked() {
                    app.is_playing = !app.is_playing;
                    app.last_step_time = std::time::Instant::now();
                }

                let max_steps = app.matrix_data.as_ref().map(|h| h.max_timesteps).unwrap_or(1);

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

                // 5. Timestep timeline slider
                let slider_max = max_steps.saturating_sub(1);
                ui.label(egui::RichText::new(format!("Step {} / {}", app.current_timestep + 1, max_steps)).small().monospace());

                let slider_res = ui.add(
                    egui::Slider::new(&mut app.current_timestep, 0..=slider_max)
                        .show_value(false)
                        .trailing_fill(true),
                );
                if slider_res.drag_stopped() || (slider_res.changed() && !app.is_playing) {
                    app.load_selected_variable_slice();
                }

                ui.separator();

                // 6. Playback speed slider
                ui.menu_button(format!("{:.0} FPS", app.playback_fps), |ui| {
                    ui.label(egui::RichText::new("Playback Speed").strong());
                    ui.add(egui::Slider::new(&mut app.playback_fps, 1.0..=60.0).suffix(" FPS"));
                });

                // 7. Bottom Right: Cache Menu Dropdown
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    cache::show_cache_menu(app, ui);
                });
            });
        });
}
