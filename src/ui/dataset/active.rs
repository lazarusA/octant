use crate::app::OctantApp;
use crate::app::StoreKind;
use crate::ui::icons::{Icon, IconSize, IconTone, UiIconExt};

/// Renders the currently loaded dataset summary card and quick actions.
pub fn show_active_dataset(app: &mut OctantApp, ui: &mut egui::Ui) {
    let (name, var_count, store_type) = if let Some(meta) = &app.selected.metadata {
        (
            meta.name.clone(),
            meta.variables.len(),
            meta.store_type.clone(),
        )
    } else {
        egui::Frame::default()
            .fill(ui.visuals().faint_bg_color)
            .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
            .corner_radius(6.0)
            .inner_margin(egui::Margin::symmetric(10, 8))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.icon_toned(Icon::Info, IconSize::Sm, IconTone::Muted);
                    ui.label(
                        egui::RichText::new("No dataset currently loaded")
                            .small()
                            .weak(),
                    );
                });
            });
        return;
    };

    let (badge_icon, badge_text, badge_tone) = match app.selected.store_kind {
        StoreKind::RemoteZarr | StoreKind::LocalZarr => (Icon::Globe, "Zarr", IconTone::Accent),
        StoreKind::RemoteIcechunk | StoreKind::LocalIcechunk => {
            (Icon::Icechunk, "Icechunk", IconTone::Info)
        }
        StoreKind::RemoteGeoTiff | StoreKind::LocalGeoTiff => {
            (Icon::Folder, "GeoTIFF", IconTone::Success)
        }
        StoreKind::LocalNetCdf => (Icon::Folder, "NetCDF", IconTone::Muted),
        StoreKind::ProceduralVolume4D => (Icon::PlotVolume, "4D Volume", IconTone::Warning),
        StoreKind::ProceduralRandom => (Icon::PlotPlane, "2D Matrix", IconTone::Warning),
    };

    let mut should_reload = false;
    egui::Frame::default()
        .fill(ui.visuals().faint_bg_color)
        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
        .corner_radius(6.0)
        .inner_margin(egui::Margin::symmetric(8, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.icon_toned(badge_icon, IconSize::Xs, badge_tone);
                ui.label(
                    egui::RichText::new(badge_text)
                        .small()
                        .strong()
                        .color(badge_tone.color(ui.visuals())),
                );
                ui.label(egui::RichText::new(&name).strong());
            });

            let uri = &app.selected.store_target;
            if !uri.is_empty() {
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(uri)
                            .small()
                            .monospace()
                            .color(ui.visuals().weak_text_color()),
                    )
                    .truncate(),
                )
                .on_hover_text(format!("Target URI:\n{uri}"));
            }

            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!(
                        "{} variable{}",
                        var_count,
                        if var_count == 1 { "" } else { "s" }
                    ))
                    .small()
                    .weak(),
                );
                if !store_type.is_empty() {
                    ui.label(
                        egui::RichText::new(format!("• {store_type}"))
                            .small()
                            .weak(),
                    );
                }
            });

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if ui
                    .icon_button(Icon::Variables, "Inspect Variables")
                    .clicked()
                {
                    app.layout.show_variables_overlay = true;
                }
                if ui.icon_button(Icon::Reset, "Reload").clicked() {
                    should_reload = true;
                }
            });
        });

    if should_reload {
        let target = app.selected.store_target.clone();
        app.submit_or_activate_source(&target, Some(app.selected.store_kind));
    }
}
