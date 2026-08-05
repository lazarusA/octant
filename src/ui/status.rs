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
    });
}
