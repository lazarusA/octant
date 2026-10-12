use super::gated;
use super::support::{OptionSupport, Support};
use crate::app::OctantApp;
use crate::app::layers::{AlphaCurveState, LayerId};
use crate::i18n::{SettingsKey as K, tr};
use crate::ui::icons::{Icon, IconSize, IconTone, UiIconExt};
use crate::ui::number_input::LocalizedInput;
use crate::utils::colormap::{AlphaInterp, alpha::PRESETS};

/// The names of the alpha curve `PRESETS`, in their order.
const PRESET_NAMES: [K; PRESETS.len()] = [
    K::PresetRampUp,
    K::PresetRampDown,
    K::PresetPeak,
    K::PresetSink,
    K::PresetHideLow,
    K::PresetHideCenter,
];

/// Layer `id`'s opacity and alpha curve, then (for the base layer, whose
/// plot type decides the canvas) the toggle that draws translucent meshes,
/// point clouds or volumes, each where the plot honors it.
pub(crate) fn show_transparency_settings(
    app: &mut OctantApp,
    ui: &mut egui::Ui,
    id: LayerId,
    support: &OptionSupport,
) {
    // The color mapping's note above already names an override they share.
    let opacity = match support.opacity.reason() {
        Some(reason) if support.color_mapping.reason() == Some(reason) => Support::No,
        _ => support.opacity,
    };
    gated(ui, opacity, |ui| show_opacity_controls(app, ui, id));
    if id != LayerId::BASE {
        return;
    }
    if support.transparency != Support::No {
        transparency_toggle(
            ui,
            &mut app.plot_configs.plot_transparency,
            support.transparency,
        )
        .on_hover_text(tr(K::TransparencyHover));
    }
    if support.volume_transparency != Support::No {
        transparency_toggle(
            ui,
            &mut app.plot_configs.volume.transparency,
            support.volume_transparency,
        );
        if app.plot_configs.volume.algorithm == 0 && app.plot_configs.volume.transparency {
            ui.checkbox(&mut app.plot_configs.volume.lighting, tr(K::Lighting))
                .on_hover_text(tr(K::LightingHover));
        }
    }
}

/// The Transparency checkbox, disabled with the reason when overridden.
fn transparency_toggle(ui: &mut egui::Ui, value: &mut bool, support: Support) -> egui::Response {
    let response = ui.add_enabled(
        support.is_yes(),
        egui::Checkbox::new(value, tr(K::Transparency)),
    );
    match support.reason() {
        Some(reason) => response.on_disabled_hover_text(reason),
        None => response,
    }
}

/// Layer `id`'s opacity slider and its opacity curve over the data range.
fn show_opacity_controls(app: &mut OctantApp, ui: &mut egui::Ui, id: LayerId) {
    let Some(layer) = app.layers.get_mut(id) else {
        return;
    };
    let color = &mut layer.color;
    let resp = ui
        .add(
            egui::Slider::new(&mut color.opacity, 0.0..=1.0)
                .localized()
                .text(tr(K::Opacity)),
        )
        .on_hover_text(tr(K::OpacityHover));
    if resp.double_clicked() {
        color.opacity = 1.0;
    }
    ui.add_space(4.0);
    let curve = &mut color.alpha;
    let mut changed = curve_header(ui, curve);
    let edit = egui::TextEdit::singleline(&mut curve.text)
        .hint_text(tr(K::AlphaCurveHint))
        .desired_width(f32::INFINITY);
    changed |= ui.add(edit).on_hover_text(tr(K::AlphaCurveHover)).changed();
    if let Some(error) = &curve.error {
        ui.horizontal(|ui| {
            ui.icon_toned(Icon::Warning, IconSize::Sm, IconTone::Error);
            let mut buf = [0; 256];
            ui.small(crate::utils::stack_str(&mut buf, format_args!("{error}")));
        });
    }
    if changed {
        app.apply_alpha_curve(id);
    }
}

/// The alpha curve's title row: clear, presets, and step or linear
/// interpolation. Whether the curve changed.
fn curve_header(ui: &mut egui::Ui, curve: &mut AlphaCurveState) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(tr(K::AlphaCurve));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if !curve.text.is_empty() && ui.close_button(tr(K::ClearCurveHover)).clicked() {
                curve.text.clear();
                changed = true;
            }
            ui.menu_button(egui::RichText::new(tr(K::Presets)).small(), |ui| {
                for (name, (_, text, interp)) in PRESET_NAMES.into_iter().zip(PRESETS) {
                    if ui.button(tr(name)).clicked() {
                        curve.text = text.to_string();
                        curve.interp = interp;
                        changed = true;
                        ui.close();
                    }
                }
            });
            let interps = [
                (AlphaInterp::Step, K::InterpStep),
                (AlphaInterp::Linear, K::InterpLinear),
            ];
            for (interp, label) in interps {
                if ui
                    .selectable_label(curve.interp == interp, tr(label))
                    .clicked()
                {
                    curve.interp = interp;
                    changed = true;
                }
            }
        });
    });
    changed
}
