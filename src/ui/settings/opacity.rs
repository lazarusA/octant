use super::gated;
use super::support::{OptionSupport, Support};
use crate::app::OctantApp;
use crate::app::layers::{AlphaCurveState, LayerId};
use crate::ui::icons::{Icon, IconSize, IconTone, UiIconExt};
use crate::utils::colormap::{AlphaInterp, alpha::PRESETS};

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
        transparency_toggle(ui, &mut app.plot_transparency, support.transparency).on_hover_text(
            "With translucent colors (Opacity or Alpha curve), draw every layer \
                 instead of letting the nearest one hide those behind it.",
        );
    }
    if support.volume_transparency != Support::No {
        transparency_toggle(
            ui,
            &mut app.volume_transparency,
            support.volume_transparency,
        );
        if app.volume_algorithm == 0 && app.volume_transparency {
            ui.checkbox(&mut app.volume_lighting, "Lighting")
                .on_hover_text("Shade samples by their gradient so fronts and edges gain shape");
        }
    }
}

/// The Transparency checkbox, disabled with the reason when overridden.
fn transparency_toggle(ui: &mut egui::Ui, value: &mut bool, support: Support) -> egui::Response {
    let response = ui.add_enabled(support.is_yes(), egui::Checkbox::new(value, "Transparency"));
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
        .add(egui::Slider::new(&mut color.opacity, 0.0..=1.0).text("Opacity"))
        .on_hover_text("Opacity of colormapped values. NaN and clip colors keep their own alpha. Double-click to reset.");
    if resp.double_clicked() {
        color.opacity = 1.0;
    }
    ui.add_space(4.0);
    let curve = &mut color.alpha;
    let mut changed = curve_header(ui, curve);
    let edit = egui::TextEdit::singleline(&mut curve.text)
        .hint_text("0.1, 0.4, 0.3  or  0:0, 0.5:1, 1:0")
        .desired_width(f32::INFINITY);
    changed |= ui
        .add(edit)
        .on_hover_text(
            "Alpha over the colorbar, from its start to its end. Values are evenly \
             spaced (Step: one equal bin each); position:alpha stops are placed \
             along the bar. Empty for none.",
        )
        .changed();
    if let Some(error) = &curve.error {
        ui.horizontal(|ui| {
            ui.icon_toned(Icon::Warning, IconSize::Sm, IconTone::Error);
            ui.small(error.as_str());
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
        ui.label("Alpha curve");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if !curve.text.is_empty() && ui.close_button("Clear curve").clicked() {
                curve.text.clear();
                changed = true;
            }
            ui.menu_button(egui::RichText::new("Presets").small(), |ui| {
                for (label, text, interp) in PRESETS {
                    if ui.button(label).clicked() {
                        curve.text = text.to_string();
                        curve.interp = interp;
                        changed = true;
                        ui.close();
                    }
                }
            });
            for (interp, label) in [(AlphaInterp::Step, "Step"), (AlphaInterp::Linear, "Linear")] {
                if ui.selectable_label(curve.interp == interp, label).clicked() {
                    curve.interp = interp;
                    changed = true;
                }
            }
        });
    });
    changed
}
