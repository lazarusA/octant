use crate::app::OctantApp;
use crate::ui::icons::{Icon, UiIconExt};

pub(crate) fn show_export_preferences(app: &mut OctantApp, ui: &mut egui::Ui) {
    egui::CollapsingHeader::new("Export")
        .default_open(false)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Format:");
                egui::ComboBox::from_id_salt("settings_export_format")
                    .selected_text(app.export_settings.format.label())
                    .show_ui(ui, |ui| {
                        for format in crate::export::ExportFormat::ALL {
                            ui.selectable_value(
                                &mut app.export_settings.format,
                                format,
                                format.label(),
                            );
                        }
                    });
            });

            ui.horizontal(|ui| {
                ui.label("Folder:");
                ui.text_edit_singleline(&mut app.export_settings.export_dir);
            });

            ui.horizontal(|ui| {
                if ui.icon_button(Icon::Save, "Open Save Dialog").clicked() {
                    app.show_export_modal = true;
                }
                if ui.icon_button(Icon::Scissors, "Crop Tool").clicked() {
                    app.show_crop_overlay = !app.show_crop_overlay;
                }
            });
        });
}
