use crate::app::OctantApp;
use crate::app::StoreKind;
use crate::i18n::{DatasetKey, fill, tr};
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
                ui.label(egui::RichText::new(tr(DatasetKey::NoDatasetLoaded)).weak());
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
        StoreKind::ProceduralVolume4D => (
            Icon::PlotVolume,
            tr(DatasetKey::BadgeVolume4D),
            IconTone::Warning,
        ),
        StoreKind::ProceduralRandom => (
            Icon::PlotPlane,
            tr(DatasetKey::BadgeMatrix2D),
            IconTone::Warning,
        ),
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
            let mut buf = [0u8; 1024];
            ui.label(fill(&mut buf, tr(DatasetKey::TargetUriHover), &[&uri]));
        });
    }

    ui.horizontal(|ui| {
        let template = if var_count == 1 {
            tr(DatasetKey::VariableCountOne)
        } else {
            tr(DatasetKey::VariableCountMany)
        };
        let mut buf = [0u8; 96];
        ui.label(egui::RichText::new(fill(&mut buf, template, &[&var_count])).weak());
        if !store_type.is_empty() {
            let mut buf = [0u8; 128];
            let text = crate::utils::stack_str(&mut buf, format_args!("• {store_type}"));
            ui.label(egui::RichText::new(text).weak());
        }
    });
}

fn show_card_buttons(ui: &mut egui::Ui, action: &mut ActiveAction) {
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        if ui
            .icon_button(Icon::Variables, tr(DatasetKey::InspectVariables))
            .clicked()
        {
            action.inspect_variables = true;
        }
        if ui
            .icon_button(Icon::Reset, tr(DatasetKey::Reload))
            .clicked()
        {
            action.reload = true;
        }
    });
}
