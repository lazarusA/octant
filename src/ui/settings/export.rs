use crate::app::OctantApp;
use crate::i18n::{SettingsKey as K, tr};
use crate::ui::icons::{Icon, UiIconExt};

pub(crate) fn show_export_preferences(app: &mut OctantApp, ui: &mut egui::Ui) {
    egui::CollapsingHeader::new(tr(K::Export))
        .id_salt("settings_export_section")
        .default_open(false)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(tr(K::ExportFormat));
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
                ui.label(tr(K::ExportFolder));
                let field = egui::TextEdit::singleline(&mut app.export_settings.export_dir)
                    .desired_width(f32::INFINITY);
                ui.add(field);
            });

            ui.horizontal_wrapped(|ui| {
                if ui.icon_button(Icon::Save, tr(K::OpenSaveDialog)).clicked() {
                    app.show_export_modal = true;
                }
                if ui.icon_button(Icon::Scissors, tr(K::CropTool)).clicked() {
                    app.show_crop_overlay = !app.show_crop_overlay;
                }
            });
        });
}
