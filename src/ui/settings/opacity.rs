use crate::app::OctantApp;
use crate::ui::icons::{Icon, IconSize, IconTone, UiIconExt};
use crate::utils::colormap::{AlphaInterp, alpha::PRESETS};

/// Global opacity slider and the opacity curve over the data range.
pub(crate) fn show_opacity_controls(app: &mut OctantApp, ui: &mut egui::Ui) {
    let resp = ui
        .add(egui::Slider::new(&mut app.color_opacity, 0.0..=1.0).text("Opacity"))
        .on_hover_text("Opacity of colormapped values. NaN and clip colors keep their own alpha. Double-click to reset.");
    if resp.double_clicked() {
        app.color_opacity = 1.0;
    }

    ui.add_space(4.0);
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label("Alpha curve");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if !app.colormaps.alpha.text.is_empty() && ui.close_button("Clear curve").clicked() {
                app.colormaps.alpha.text.clear();
                changed = true;
            }
            ui.menu_button(egui::RichText::new("Presets").small(), |ui| {
                for (label, text, interp) in PRESETS {
                    if ui.button(label).clicked() {
                        app.colormaps.alpha.text = text.to_string();
                        app.colormaps.alpha.interp = interp;
                        changed = true;
                        ui.close();
                    }
                }
            });
            for (interp, label) in [(AlphaInterp::Step, "Step"), (AlphaInterp::Linear, "Linear")] {
                if ui
                    .selectable_label(app.colormaps.alpha.interp == interp, label)
                    .clicked()
                {
                    app.colormaps.alpha.interp = interp;
                    changed = true;
                }
            }
        });
    });
    let edit = egui::TextEdit::singleline(&mut app.colormaps.alpha.text)
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
    if changed {
        app.apply_alpha_curve();
    }
    if let Some(error) = &app.colormaps.alpha.error {
        ui.horizontal(|ui| {
            ui.icon_toned(Icon::Warning, IconSize::Sm, IconTone::Error);
            ui.small(error.as_str());
        });
    }
}
