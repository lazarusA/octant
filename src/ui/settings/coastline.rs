use crate::app::OctantApp;
use crate::i18n::{CommonKey, SettingsKey as K, tr};

pub(crate) fn show_coastline_controls(app: &mut OctantApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.checkbox(&mut app.show_coastlines, tr(K::Coastlines))
            .on_hover_text(tr(K::CoastlinesHover));

        if app.show_coastlines {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if app.coastline_color.is_some()
                    && ui
                        .small_button(tr(K::ResetCoastlineColor))
                        .on_hover_text(tr(K::ResetCoastlineColorHover))
                        .clicked()
                {
                    app.coastline_color = None;
                }

                let default_color = if ui.visuals().dark_mode {
                    [1.0_f32, 1.0, 1.0, 0.75]
                } else {
                    [0.15_f32, 0.15, 0.15, 0.85]
                };
                let mut color = app.coastline_color.unwrap_or(default_color);
                let initial_color = color;
                crate::ui::color_picker::ShapeColorPicker::new(
                    "settings_coastline_color_picker",
                    &mut color,
                    crate::ui::color_picker::ColorShape::Rect(3.0),
                )
                .size(egui::vec2(18.0, 16.0))
                .tooltip(tr(K::CoastlineColorHover))
                .anchor_offset(egui::vec2(-240.0, -100.0))
                .show(ui);

                if color != initial_color {
                    app.coastline_color = Some(color);
                }
            });
        }
    });

    if app.show_coastlines {
        show_coastline_details(app, ui);
    }
}

fn show_coastline_details(app: &mut OctantApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(tr(K::CoastlineResolution))
                .small()
                .weak(),
        );
        ui.spacing_mut().item_spacing.x = 2.0;

        if app.coastline_is_loading {
            ui.spinner();
            ui.label(egui::RichText::new(tr(CommonKey::Loading)).small().weak());
        } else {
            let lods = [
                (crate::plots::CoastlineLod::Lod110m, "110m"),
                (crate::plots::CoastlineLod::Lod50m, "50m"),
                (crate::plots::CoastlineLod::Lod10m, "10m"),
            ];
            for (lod, label) in lods {
                if ui
                    .selectable_label(
                        app.coastline_current_lod == lod,
                        egui::RichText::new(label).small(),
                    )
                    .on_hover_text(tr(match lod {
                        crate::plots::CoastlineLod::Lod110m => K::Lod110mHover,
                        crate::plots::CoastlineLod::Lod50m => K::Lod50mHover,
                        crate::plots::CoastlineLod::Lod10m => K::Lod10mHover,
                    }))
                    .clicked()
                {
                    app.reload_coastline_lod(lod);
                }
            }
        }
    });

    ui.add(
        egui::Slider::new(&mut app.coastline_line_width, 1.0..=4.0)
            .step_by(0.1)
            .text(tr(K::LineWidth)),
    );

    ui.checkbox(&mut app.coastline_crop_to_data_domain, tr(K::CropToDomain))
        .on_hover_text(tr(K::CropToDomainHover));
}
