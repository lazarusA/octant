use super::gated;
use super::support::{OptionSupport, Support};
use crate::app::OctantApp;
use crate::app::layers::{AlphaCurveState, LayerId};
use crate::i18n::{SettingsKey as K, tr};
use crate::ui::flow_row::{self, SliderLabel, labeled_slider};
use crate::ui::icons::{Icon, IconSize, IconTone, ToolbarButton, UiIconExt};
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
    opacity_slider(ui, id, &mut color.opacity);
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

/// Layer `id`'s opacity slider; a double-click resets it to opaque.
pub(super) fn opacity_slider(ui: &mut egui::Ui, id: LayerId, opacity: &mut f32) {
    let resp = labeled_slider(
        ui,
        ("opacity", id),
        SliderLabel::After(tr(K::Opacity)),
        egui::Slider::new(opacity, 0.0..=1.0).localized(),
    )
    .on_hover_text(tr(K::OpacityHover));
    if resp.double_clicked() {
        *opacity = 1.0;
    }
}

/// The alpha curve's title row: clear, presets, and step or linear
/// interpolation, below the title when they don't fit beside it. Whether the
/// curve changed.
fn curve_header(ui: &mut egui::Ui, curve: &mut AlphaCurveState) -> bool {
    let (title, presets) = (tr(K::AlphaCurve), tr(K::Presets));
    let (step, linear) = (tr(K::InterpStep), tr(K::InterpLinear));
    let salt = "alpha_curve_header";
    let [title_w, widths @ ..] =
        flow_row::cached_widths(ui, salt, (title, presets, step, linear), |ui| {
            [
                flow_row::text_width(ui, title, egui::TextStyle::Body),
                ToolbarButton::side(IconSize::Sm),
                flow_row::button_width(ui, presets, egui::TextStyle::Small),
                flow_row::button_width(ui, step, egui::TextStyle::Button),
                flow_row::button_width(ui, linear, egui::TextStyle::Button),
            ]
        });
    // Item 0 (clear, the rightmost) shows only while there is a curve.
    let first = usize::from(curve.text.is_empty());
    let mut changed = false;
    let title_label = |ui: &mut egui::Ui| {
        ui.label(title);
    };
    flow_row::split(ui, title_w, &widths[first..], title_label, |ui, item| {
        changed |= curve_header_item(ui, item + first, curve);
    });
    changed
}

/// Item `item` of the alpha curve's title row: 0 clear, 1 presets, 2 step
/// and 3 linear interpolation. Whether the curve changed.
fn curve_header_item(ui: &mut egui::Ui, item: usize, curve: &mut AlphaCurveState) -> bool {
    match item {
        0 => {
            let clear = ui.close_button(tr(K::ClearCurveHover)).clicked();
            if clear {
                curve.text.clear();
            }
            clear
        }
        1 => presets_menu(ui, curve),
        _ => {
            let (interp, label) = if item == 2 {
                (AlphaInterp::Step, K::InterpStep)
            } else {
                (AlphaInterp::Linear, K::InterpLinear)
            };
            let picked = ui
                .selectable_label(curve.interp == interp, tr(label))
                .clicked();
            if picked {
                curve.interp = interp;
            }
            picked
        }
    }
}

/// The Presets menu; whether a preset replaced the curve.
fn presets_menu(ui: &mut egui::Ui, curve: &mut AlphaCurveState) -> bool {
    let mut picked = false;
    ui.menu_button(egui::RichText::new(tr(K::Presets)).small(), |ui| {
        for (name, (_, text, interp)) in PRESET_NAMES.into_iter().zip(PRESETS) {
            if ui.button(tr(name)).clicked() {
                curve.text = text.to_string();
                curve.interp = interp;
                picked = true;
                ui.close();
            }
        }
    });
    picked
}
