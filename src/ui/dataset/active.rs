use crate::app::OctantApp;
use crate::app::StoreKind;
use crate::ui::icons::{Icon, IconSize, IconTone, UiIconExt};

/// Renders the currently loaded dataset summary card and quick actions.
pub fn show_active_dataset(app: &mut OctantApp, ui: &mut egui::Ui) {
    let Some(meta) = &app.selected.metadata else {
        show_active_empty(ui);
        return;
    };
    let store_kind = app.selected.store_kind;
    let target = app.selected.store_target.as_str();
    let name = meta.name.as_str();
    let var_count = meta.variables.len();
    let store_type = meta.store_type.as_str();

    let action = show_active_card(ui, store_kind, target, name, var_count, store_type);
    if action.inspect_variables {
        app.layout.show_variables_overlay = true;
    }
    if action.reload {
        let reload_target = app.selected.store_target.clone();
        app.submit_or_activate_source(&reload_target, Some(app.selected.store_kind));
    }
}

#[derive(Default)]
struct ActiveAction {
    inspect_variables: bool,
    reload: bool,
}

fn show_active_empty(ui: &mut egui::Ui) {
    egui::Frame::default()
        .fill(ui.visuals().faint_bg_color)
        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
        .corner_radius(6.0)
        .inner_margin(egui::Margin::symmetric(10, 8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.icon_toned(Icon::Info, IconSize::Sm, IconTone::Muted);
                ui.label(egui::RichText::new("No dataset currently loaded").weak());
            });
        });
}

fn store_badge(kind: StoreKind) -> (Icon, &'static str, IconTone) {
    match kind {
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
    }
}

fn show_active_card(
    ui: &mut egui::Ui,
    kind: StoreKind,
    uri: &str,
    name: &str,
    var_count: usize,
    store_type: &str,
) -> ActiveAction {
    let (badge_icon, badge_text, badge_tone) = store_badge(kind);
    let mut action = ActiveAction::default();

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
                        .strong()
                        .color(badge_tone.color(ui.visuals())),
                );
            });

            show_card_metadata(ui, name, uri, var_count, store_type);
            show_card_buttons(ui, &mut action);
        });

    action
}

fn show_card_metadata(
    ui: &mut egui::Ui,
    name: &str,
    uri: &str,
    var_count: usize,
    store_type: &str,
) {
    ui.add(
        egui::Label::new(egui::RichText::new(name).strong()).wrap_mode(egui::TextWrapMode::Wrap),
    );

    if !uri.is_empty() {
        ui.add(
            egui::Label::new(
                egui::RichText::new(uri)
                    .monospace()
                    .color(ui.visuals().weak_text_color()),
            )
            .wrap_mode(egui::TextWrapMode::Wrap),
        )
        .on_hover_ui(|ui| {
            ui.label(format!("Target URI:\n{uri}"));
        });
    }

    ui.horizontal(|ui| {
        let plural = if var_count == 1 { "" } else { "s" };
        ui.label(egui::RichText::new(format!("{var_count} variable{plural}")).weak());
        if !store_type.is_empty() {
            ui.label(egui::RichText::new(format!("• {store_type}")).weak());
        }
    });
}

fn show_card_buttons(ui: &mut egui::Ui, action: &mut ActiveAction) {
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        if ui
            .icon_button(Icon::Variables, "Inspect Variables")
            .clicked()
        {
            action.inspect_variables = true;
        }
        if ui.icon_button(Icon::Reset, "Reload").clicked() {
            action.reload = true;
        }
    });
}
