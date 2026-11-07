use crate::app::OctantApp;
use crate::i18n::{SettingsKey as K, tr};
use crate::ui::flow_row;
use crate::ui::icons::{Icon, IconSize, ToolbarButton, UiIconExt};
use crate::ui::toast::Severity;

/// Narrowest the export folder field gets beside its label; any narrower
/// and it moves below the label.
const MIN_FIELD_W: f32 = 120.0;

/// Room kept between the panel title and the Share Link button.
const TITLE_GAP: f32 = 8.0;

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

/// The header's Share Link button, left of the grip in a right-to-left `ui`:
/// copies the plot's share link, disabled while there is nothing to share.
/// Its icon alone when the label would reach the panel title; either way
/// its tooltip reads "Share Link" in full.
pub(crate) fn copy_link_button(app: &mut OctantApp, ui: &mut egui::Ui) {
    let label = tr(K::Share);
    // The label is one word; the tooltip names the link in full.
    let button = ToolbarButton::new(Icon::Clipboard, label)
        .icon_size(IconSize::Sm)
        .hover(tr(K::ShareLinkHover));
    let [width] = flow_row::cached_widths(ui, "share_link", label, |ui| [button.width(ui)]);
    let button = button.compact(width + TITLE_GAP > ui.available_width());
    if ui.add_enabled(app.can_share(), button).clicked()
        && let Some(link) = app.share_link()
    {
        ui.ctx().copy_text(link.clone());
        app.notify(Severity::Success, tr(K::ShareLinkCopied), link);
    }
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
