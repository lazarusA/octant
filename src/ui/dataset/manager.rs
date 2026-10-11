use crate::app::OctantApp;
use crate::app::StoreKind;
use crate::data::{DataSourceKind, Dataset};
use crate::i18n::{DatasetKey, fill, tr};
use crate::ui::icons::{Icon, IconSize, UiIconExt};
use std::borrow::Cow;

/// Renders the multi-dataset manager list, switching active dataset, and removal.
pub fn show_dataset_manager(app: &mut OctantApp, ui: &mut egui::Ui) {
    if app.dataset_manager.is_empty() {
        ui.label(egui::RichText::new(tr(DatasetKey::NoDatasetsRegistered)).weak());
        return;
    }

    ui.horizontal(|ui| {
        let mut buf = [0u8; 64];
        let loaded = fill(
            &mut buf,
            tr(DatasetKey::LoadedCount),
            &[&app.dataset_manager.len()],
        );
        ui.label(egui::RichText::new(loaded).strong());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .icon_button(Icon::Trash, tr(DatasetKey::ClearAll))
                .on_hover_text(tr(DatasetKey::ClearAllHover))
                .clicked()
            {
                app.clear_all_datasets();
            }
        });
    });

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
        let avail_w = (ui.available_width() - 36.0).max(80.0);
        let approx_chars = ((avail_w - 16.0) / 7.5).floor() as usize;
        let short_name = truncate_display_name(&d.source.display_name, approx_chars.max(10));

        let btn = egui::Button::new(egui::RichText::new(short_name.as_ref()).strong())
            .selected(is_active);

        let item_btn = ui.add_sized([avail_w, 20.0], btn);
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

        if ui
            .icon_button(Icon::Trash, "")
            .on_hover_text(tr(DatasetKey::RemoveDatasetHover))
            .clicked()
        {
            *to_delete = Some(d.id.clone());
        }
    });
}

fn truncate_display_name(name: &str, max_chars: usize) -> Cow<'_, str> {
    if name.chars().count() > max_chars && max_chars > 3 {
        let mut truncated: String = name.chars().take(max_chars - 3).collect();
        truncated.push_str("...");
        Cow::Owned(truncated)
    } else {
        Cow::Borrowed(name)
    }
}
