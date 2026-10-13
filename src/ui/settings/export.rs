use crate::app::OctantApp;
use crate::i18n::{SettingsKey as K, tr};
use crate::ui::flow_row;
use crate::ui::icons::{Icon, UiIconExt};

/// Narrowest the export folder field gets beside its label; any narrower
/// and it moves below the label.
const MIN_FIELD_W: f32 = 120.0;

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

            folder_row(ui, &mut app.export_settings.export_dir);

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

/// The export folder's label and a field filling the row, below the label
/// when the field would be narrower than `MIN_FIELD_W` beside it.
fn folder_row(ui: &mut egui::Ui, dir: &mut String) {
    let label = tr(K::ExportFolder);
    let [label_w] = flow_row::cached_widths(ui, "export_folder", label, |ui| {
        [flow_row::text_width(ui, label, egui::TextStyle::Body)]
    });
    let field = egui::TextEdit::singleline(dir).desired_width(f32::INFINITY);
    if flow_row::fits(ui, &[label_w, MIN_FIELD_W]) {
        ui.horizontal(|ui| {
            ui.label(label);
            ui.add(field);
        });
    } else {
        ui.label(label);
        ui.add(field);
    }
}
