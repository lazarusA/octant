use crate::app::layers::{ColorStyle, LayerId};
use crate::i18n::{SettingsKey as K, tr};
use crate::ui::number_input::LocalizedInput;

/// Layer `id`'s color scale (linear, log, symlog, sqrt, exponential) and its
/// parameter.
pub(crate) fn show_scale_type_controls(ui: &mut egui::Ui, color: &mut ColorStyle, id: LayerId) {
    let is_valid_log = color.range_min >= -1e-15 && color.range_max > 0.0;
    if !is_valid_log && color.scale_type == 1 {
        color.scale_type = 0;
    }

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(tr(K::Scale)).strong());
        egui::ComboBox::from_id_salt(("settings_color_scale_dropdown", id))
            .selected_text(tr(match color.scale_type {
                1 => K::ScaleLog,
                2 => K::ScaleSymlog,
                3 => K::ScaleSqrt,
                4 => K::ScaleExponential,
                _ => K::ScaleLinear,
            }))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut color.scale_type, 0, tr(K::ScaleLinear));
                ui.add_enabled_ui(is_valid_log, |ui| {
                    ui.selectable_value(&mut color.scale_type, 1, tr(K::ScaleLog))
                        .on_hover_text(tr(if is_valid_log {
                            K::ScaleLogHover
                        } else {
                            K::ScaleLogDisabledHover
                        }));
                });
                ui.selectable_value(&mut color.scale_type, 2, tr(K::ScaleSymlog));
                ui.selectable_value(&mut color.scale_type, 3, tr(K::ScaleSqrt));
                ui.selectable_value(&mut color.scale_type, 4, tr(K::ScaleExponential));
            });
    });

    if matches!(color.scale_type, 1 | 2 | 4) {
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            ui.label(tr(K::ScaleParam));
            ui.add(
                egui::DragValue::new(&mut color.scale_param)
                    .localized()
                    .speed(0.01)
                    .range(0.0001..=100.0),
            );
        });
    }
}
