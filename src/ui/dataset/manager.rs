use crate::app::OctantApp;
use crate::app::StoreKind;
use crate::data::{DataSourceKind, Dataset};
use crate::i18n::{DatasetKey, fill, tr};
use crate::ui::flow_row;
use crate::ui::icons::{Icon, IconSize, UiIconExt, icon_button_width};

/// Renders the multi-dataset manager list, switching active dataset, and removal.
pub fn show_dataset_manager(app: &mut OctantApp, ui: &mut egui::Ui) {
    if app.dataset_manager.is_empty() {
        ui.label(egui::RichText::new(tr(DatasetKey::NoDatasetsRegistered)).weak());
        return;
    }

    let mut buf = [0u8; 64];
    let loaded = fill(
        &mut buf,
        tr(DatasetKey::LoadedCount),
        &[&app.dataset_manager.len()],
    );
    let clear = tr(DatasetKey::ClearAll);
    let [loaded_w, clear_w] =
        flow_row::cached_widths(ui, "dataset_loaded_row", (loaded, clear), |ui| {
            [
                flow_row::text_width(ui, loaded, egui::TextStyle::Body),
                icon_button_width(ui, clear),
            ]
        });
    let mut clear_all = false;
    flow_row::split(
        ui,
        loaded_w,
        &[clear_w],
        |ui| {
            ui.label(egui::RichText::new(loaded).strong());
        },
        |ui, _| {
            clear_all = ui
                .icon_button(Icon::Trash, clear)
                .on_hover_text(tr(DatasetKey::ClearAllHover))
                .clicked();
        },
    );
    if clear_all {
        app.clear_all_datasets();
    }

    ui.add_space(4.0);

    let mut to_activate: Option<(String, StoreKind)> = None;
    let mut to_delete: Option<String> = None;

    egui::ScrollArea::vertical()
        .max_height(160.0)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for d in app.dataset_manager.iter() {
                let is_active = app.selected.store_target == d.source.uri;
                show_dataset_row(ui, d, is_active, &mut to_activate, &mut to_delete);
            }
        });

    if let Some(id) = to_delete {
        app.remove_dataset(&id);
    } else if let Some((uri, kind)) = to_activate {
        app.submit_or_activate_source(&uri, Some(kind));
    }
}

fn source_kind_icon(kind: &DataSourceKind) -> Icon {
    match kind {
        DataSourceKind::RemoteZarr => Icon::Globe,
        DataSourceKind::LocalZarr => Icon::Folder,
        DataSourceKind::RemoteIcechunk | DataSourceKind::LocalIcechunk => Icon::Icechunk,
        DataSourceKind::NetCdf => Icon::Folder,
        DataSourceKind::RemoteGeoTiff => Icon::Globe,
        DataSourceKind::LocalGeoTiff => Icon::Folder,
        DataSourceKind::Procedural => Icon::PlotPlane,
        _ => Icon::PlotVolume,
    }
}

fn show_dataset_row(
    ui: &mut egui::Ui,
    d: &Dataset,
    is_active: bool,
    to_activate: &mut Option<(String, StoreKind)>,
    to_delete: &mut Option<String>,
) {
    ui.horizontal(|ui| {
        ui.icon(source_kind_icon(&d.source.kind), IconSize::Xs);
        // The remove button at the right end, then the name filling the rest
        // of the row, truncated to fit.
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .icon_button(Icon::Trash, "")
                .on_hover_text(tr(DatasetKey::RemoveDatasetHover))
                .clicked()
            {
                *to_delete = Some(d.id.clone());
            }
            let name = egui::RichText::new(d.source.display_name.as_str()).strong();
            let btn = egui::Button::new(name).truncate().selected(is_active);
            let item_btn = ui.add_sized([ui.available_width(), 20.0], btn);
            if item_btn
                .on_hover_ui(|ui| {
                    let mut buf = [0u8; 1024];
                    let args: [&dyn std::fmt::Display; 2] = [&d.source.display_name, &d.source.uri];
                    ui.label(fill(&mut buf, tr(DatasetKey::DatasetRowHover), &args));
                })
                .clicked()
            {
                *to_activate = Some((
                    d.source.uri.clone(),
                    StoreKind::from_data_source_kind(&d.source.kind),
                ));
            }
        });
    });
}
