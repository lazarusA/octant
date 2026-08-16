use crate::app::OctantApp;

pub fn show_status_bar(app: &mut OctantApp, ui: &mut egui::Ui) {
    let pending_count = app.block_prefetcher.pending_count();
    if pending_count > 0 {
        let completed_bytes = app.block_prefetcher.completed_bytes();
        let total_bytes = app
            .block_prefetcher
            .total_bytes()
            .max(completed_bytes)
            .max(1);
        let progress = (completed_bytes as f32 / total_bytes as f32).clamp(0.0, 1.0);
        let formatted_completed = crate::ui::variables_panel::format_byte_size(completed_bytes);
        let formatted_total = crate::ui::variables_panel::format_byte_size(total_bytes);

        ui.horizontal(|ui| {
            // Abort button to interrupt ongoing calls
            let abort_btn = egui::Button::new(
                egui::RichText::new("⏹ Abort")
                    .small()
                    .strong()
                    .color(egui::Color32::from_rgb(255, 110, 110)),
            );
            if ui
                .add(abort_btn)
                .on_hover_text("Interrupt and abort ongoing data transfer")
                .clicked()
            {
                app.abort_current_fetch();
            }

            ui.add(egui::Spinner::new().size(12.0));
            ui.label(
                egui::RichText::new(format!(
                    "Fetching {} / {} ({:.0}%)",
                    formatted_completed,
                    formatted_total,
                    progress * 100.0
                ))
                .small()
                .strong()
                .color(egui::Color32::from_rgb(255, 205, 80)),
            );
        });
        ui.separator();
    }
}
