use crate::app::layers::{ColorStyle, LayerId};
use crate::i18n::{SettingsKey as K, tr};
use crate::ui::number_input::LocalizedInput;
use crate::utils::colormap::ColorScale;

/// Layer `id`'s color scale (linear, log, symlog, sqrt, exponential) and its
/// parameter.
pub(crate) fn show_scale_type_controls(ui: &mut egui::Ui, color: &mut ColorStyle, id: LayerId) {
    let allows_log = color.allows_log();
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(tr(K::Scale)).strong());
        let mut picked = color.scale;
        egui::ComboBox::from_id_salt(("settings_color_scale_dropdown", id))
            .selected_text(tr(scale_key(color.scale)))
            .show_ui(ui, |ui| {
                for scale in ColorScale::ALL {
                    let enabled = scale != ColorScale::Log || allows_log;
                    ui.add_enabled_ui(enabled, |ui| {
                        let row = ui.selectable_value(&mut picked, scale, tr(scale_key(scale)));
                        if scale == ColorScale::Log {
                            row.on_hover_text(tr(if allows_log {
                                K::ScaleLogHover
                            } else {
                                K::ScaleLogDisabledHover
                            }));
                        }
                    });
                }
            });
        color.set_scale(picked);
    });

    if color.scale.has_param() {
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

/// The menu text of `scale`.
fn scale_key(scale: ColorScale) -> K {
    match scale {
        ColorScale::Linear => K::ScaleLinear,
        ColorScale::Log => K::ScaleLog,
        ColorScale::Symlog => K::ScaleSymlog,
        ColorScale::Sqrt => K::ScaleSqrt,
        ColorScale::Exponential => K::ScaleExponential,
    }
}
