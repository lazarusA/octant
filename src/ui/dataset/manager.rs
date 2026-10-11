use crate::app::OctantApp;
use crate::app::StoreKind;
use crate::ui::icons::{Icon, IconSize, UiIconExt};

/// Renders the multi-dataset manager list, switching active dataset, and removal.
pub fn show_dataset_manager(app: &mut OctantApp, ui: &mut egui::Ui) {
    if app.dataset_manager.is_empty() {
        ui.label(
            egui::RichText::new("No datasets currently registered in manager.")
                .small()
                .weak(),
        );
        return;
    }

    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!("Loaded ({}):", app.dataset_manager.len()))
                .strong()
                .small(),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .icon_button(Icon::Trash, "Clear All")
                .on_hover_text("Remove all datasets from manager")
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
                let icon = match d.source.kind {
                    crate::data::DataSourceKind::RemoteZarr => Icon::Globe,
                    crate::data::DataSourceKind::LocalZarr => Icon::Folder,
                    crate::data::DataSourceKind::RemoteIcechunk
                    | crate::data::DataSourceKind::LocalIcechunk => Icon::Icechunk,
                    crate::data::DataSourceKind::NetCdf => Icon::Folder,
                    crate::data::DataSourceKind::RemoteGeoTiff => Icon::Globe,
                    crate::data::DataSourceKind::LocalGeoTiff => Icon::Folder,
                    crate::data::DataSourceKind::Procedural => Icon::PlotPlane,
                    _ => Icon::PlotVolume,
                };

                ui.horizontal(|ui| {
                    ui.icon(icon, IconSize::Xs);
                    let avail_w = (ui.available_width() - 36.0).max(80.0);
                    let approx_chars = ((avail_w - 16.0) / 7.5).floor() as usize;
                    let short_name =
                        truncate_display_name(&d.source.display_name, approx_chars.max(10));

                    let btn = egui::Button::new(egui::RichText::new(short_name).small().strong())
                        .selected(is_active);

                    let item_btn = ui.add_sized([avail_w, 18.0], btn);
                    if item_btn
                        .on_hover_text(format!("{}\nURI: {}", d.source.display_name, d.source.uri))
                        .clicked()
                    {
                        to_activate = Some((
                            d.source.uri.clone(),
                            StoreKind::from_data_source_kind(&d.source.kind),
                        ));
                    }

                    if ui
                        .icon_button(Icon::Trash, "")
                        .on_hover_text("Remove this dataset")
                        .clicked()
                    {
                        to_delete = Some(d.id.clone());
                    }
                });
            }
        });

    if let Some(id) = to_delete {
        app.remove_dataset(&id);
    } else if let Some((uri, kind)) = to_activate {
        app.submit_or_activate_source(&uri, Some(kind));
    }
}

fn truncate_display_name(name: &str, max_chars: usize) -> String {
    if name.chars().count() > max_chars && max_chars > 3 {
        let mut truncated: String = name.chars().take(max_chars - 3).collect();
        truncated.push_str("...");
        truncated
    } else {
        name.to_string()
    }
}
