use crate::app::OctantApp;

pub fn show_status_bar(app: &OctantApp, ui: &mut egui::Ui) {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        // Non-blocking fetch badge
        if app.is_fetching_slice {
            ui.label(
                egui::RichText::new("⏳ FETCHING")
                    .small()
                    .color(egui::Color32::GOLD),
            );
            ui.separator();
        }

        // Playback state indicator badge
        if app.is_playing {
            ui.label(
                egui::RichText::new("▶ PLAYING")
                    .small()
                    .color(egui::Color32::from_rgb(255, 99, 71)),
            );
        } else {
            ui.label(
                egui::RichText::new("⏸ PAUSED")
                    .small()
                    .color(egui::Color32::LIGHT_GRAY),
            );
        }
    });
}
