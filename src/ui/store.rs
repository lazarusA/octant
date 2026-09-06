use crate::app::{OctantApp, StoreKind};
use crate::ui::icons::{Icon, UiIconExt};

pub fn show_left_panel(app: &mut OctantApp, ui: &mut egui::Ui) {
    // Extract to a local bool to avoid split-borrow: we can't hold &mut app.field
    // AND also borrow all of app inside the closure at the same time.
    let mut show = app.show_left_panel;

    egui::Panel::left("octant_left_store_panel")
        .resizable(true)
        .default_size(280.0)
        .size_range(180.0..=420.0)
        .show_collapsible(ui, &mut show, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                ui.add_space(4.0);
                if ui.icon_button(Icon::Catalog, "Open Catalog").clicked() {
                    app.show_catalog_window = true;
                }
                ui.add_space(4.0);
                ui.separator();

                let mut selected = app.selected_store_kind;
                egui::ComboBox::from_id_salt("left_store_kind_select")
                    .selected_text(match selected {
                        StoreKind::RemoteZarr => "Remote Zarr",
                        StoreKind::LocalZarr => "Local Zarr",
                        StoreKind::RemoteIcechunk => "Remote Icechunk",
                        StoreKind::LocalIcechunk => "Local Icechunk",
                        StoreKind::LocalNetCdf => "Local NetCDF / HDF5",
                        StoreKind::ProceduralVolume4D => "4D Known-Truth Volume",
                        StoreKind::ProceduralRandom => "2D Procedural Matrix",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut selected, StoreKind::RemoteZarr, "Remote Zarr (HTTP/S3)");
                        ui.selectable_value(&mut selected, StoreKind::LocalZarr, "Local Zarr (FileSystem)");
                        ui.selectable_value(&mut selected, StoreKind::RemoteIcechunk, "Remote Icechunk (HTTP/S3)");
                        ui.selectable_value(&mut selected, StoreKind::LocalIcechunk, "Local Icechunk (FileSystem)");
                        ui.selectable_value(&mut selected, StoreKind::LocalNetCdf, "Local NetCDF / HDF5 (.nc/.h5/.hdf5)");
                        ui.separator();
                        ui.selectable_value(&mut selected, StoreKind::ProceduralVolume4D, "4D Known-Truth Volume (Procedural)");
                        ui.selectable_value(&mut selected, StoreKind::ProceduralRandom, "2D Procedural Matrix (Test)");
                    });

                if selected != app.selected_store_kind {
                    app.selected_store_kind = selected;
                    match app.selected_store_kind {
                        StoreKind::RemoteZarr => {
                            app.store_target_input = "https://s3.bgc-jena.mpg.de:9000/esdl-esdc-v3.0.2/esdc-16d-2.5deg-46x72x1440-3.0.2.zarr".to_string();
                        }
                        StoreKind::LocalZarr => {
                            app.store_target_input = "./data/sample_dataset.zarr".to_string();
                        }
                        StoreKind::RemoteIcechunk => {
                            app.store_target_input = "https://s3.amazonaws.com/icechunk-demo/repository".to_string();
                        }
                        StoreKind::LocalIcechunk => {
                            app.store_target_input = "./data/icechunk_repo".to_string();
                        }
                        StoreKind::LocalNetCdf => {
                            app.store_target_input = "./data/sample.nc".to_string();
                        }
                        StoreKind::ProceduralVolume4D => {
                            app.submit_or_activate_source("procedural://volume4d", Some(StoreKind::ProceduralVolume4D));
                        }
                        StoreKind::ProceduralRandom => {
                            app.submit_or_activate_source("procedural://matrix2d", Some(StoreKind::ProceduralRandom));
                        }
                    }
                }

                ui.add_space(6.0);
                ui.label(egui::RichText::new("Target URL / Path").strong());
                let res = ui.text_edit_singleline(&mut app.store_target_input);
                if res.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    let target = app.store_target_input.clone();
                    app.submit_or_activate_source(&target, Some(app.selected_store_kind));
                }

                ui.add_space(6.0);
                let (btn_icon, btn_label) = if app.is_loading {
                    (Icon::Hourglass, "Loading...")
                } else {
                    (Icon::DropTray, "Load")
                };
                if ui
                    .add_enabled(!app.is_loading, |ui: &mut egui::Ui| {
                        ui.icon_button(btn_icon, btn_label)
                    })
                    .clicked()
                {
                    let target = app.store_target_input.clone();
                    app.submit_or_activate_source(&target, Some(app.selected_store_kind));
                }

                ui.add_space(8.0);
                crate::ui::drop_zone::show_drop_zone(ui, None, 68.0);

                ui.add_space(6.0);
                ui.separator();
                ui.collapsing("About this store", |ui| {
                    if let Some(metadata) = &app.active_dataset_metadata {
                        ui.label(format!("Provider: {}", metadata.store_type));
                        ui.label(format!("Dataset: {}", metadata.name));
                        ui.label(format!("Variables: {}", metadata.variables.len()));
                    } else {
                        ui.label("No dataset metadata loaded yet.");
                    }
                });

                ui.collapsing("Dataset Manager", |ui| {
                    if app.dataset_manager.is_empty() {
                        ui.label("No active datasets in DatasetManager.");
                    } else {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!("Active ({})", app.dataset_manager.len()))
                                    .strong(),
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
                            .max_height(200.0)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                for d in app.dataset_manager.iter() {
                                    let is_active = app.store_target_input == d.source.uri;
                                    let icon = match d.source.kind {
                                        crate::data::DataSourceKind::RemoteZarr => Icon::Globe,
                                        crate::data::DataSourceKind::LocalZarr => Icon::Folder,
                                        crate::data::DataSourceKind::RemoteIcechunk
                                        | crate::data::DataSourceKind::LocalIcechunk => Icon::Icechunk,
                                        crate::data::DataSourceKind::NetCdf => Icon::Folder,
                                        crate::data::DataSourceKind::Procedural => Icon::PlotPlane,
                                        _ => Icon::PlotVolume,
                                    };

                                    ui.horizontal(|ui| {
                                        ui.icon(icon, 13.0);
                                        let avail_w = (ui.available_width() - 44.0).max(60.0);
                                        let approx_chars = ((avail_w - 20.0) / 7.5).floor() as usize;
                                        let short_name = truncate_display_name(&d.source.display_name, approx_chars.max(10));

                                        let item_btn = ui.add_sized(
                                            [avail_w, 20.0],
                                            egui::Button::new(egui::RichText::new(short_name).strong())
                                                .selected(is_active),
                                        );
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
                    ui.separator();
                    ui.label(format!("BlockCache Entries: {}", app.block_cache.cached_count()));
                    ui.label(format!(
                        "BlockCache Size: {:.2} MB / {:.2} MB",
                        app.block_cache.current_bytes() as f64 / (1024.0 * 1024.0),
                        app.block_cache.max_bytes() as f64 / (1024.0 * 1024.0)
                    ));
                    ui.label(format!("BlockCache Hit Rate: {:.1}%", app.block_cache.hit_rate()));
                });
            });
        });

    app.show_left_panel = show;
}

pub fn show_store_menu(app: &mut OctantApp, ui: &mut egui::Ui) {
    if ui.icon_button(Icon::Globe, "Store").clicked() {
        app.show_left_panel = !app.show_left_panel;
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
