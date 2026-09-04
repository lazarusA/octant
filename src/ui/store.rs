use crate::app::{OctantApp, StoreKind};

pub fn show_left_panel(app: &mut OctantApp, ui: &mut egui::Ui) {
    // Extract to a local bool to avoid split-borrow: we can't hold &mut app.field
    // AND also borrow all of app inside the closure at the same time.
    let mut show = app.show_left_panel;

    egui::Panel::left("octant_left_store_panel")
        .resizable(true)
        .default_size(280.0)
        .size_range(180.0..=420.0)
        .show_collapsible(ui, &mut show, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(4.0);
                if ui.button(egui::RichText::new("📚 Open Catalog").strong()).clicked() {
                    app.show_catalog_window = true;
                }
                ui.add_space(4.0);
                ui.separator();

                let mut selected = app.selected_store_kind;
                egui::ComboBox::from_id_salt("left_store_kind_select")
                    .selected_text(match selected {
                        StoreKind::RemoteZarr => "🌐 Remote Zarr",
                        StoreKind::LocalZarr => "📁 Local Zarr",
                        StoreKind::RemoteIcechunk => "🧊 Remote Icechunk",
                        StoreKind::LocalIcechunk => "🧊 Local Icechunk",
                        StoreKind::LocalNetCdf => "📁 Local NetCDF",
                        StoreKind::ProceduralVolume4D => "🌐 4D Known-Truth Volume",
                        StoreKind::ProceduralRandom => "🎲 2D Procedural Matrix",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut selected, StoreKind::RemoteZarr, "🌐 Remote Zarr (HTTP/S3)");
                        ui.selectable_value(&mut selected, StoreKind::LocalZarr, "📁 Local Zarr (FileSystem)");
                        ui.selectable_value(&mut selected, StoreKind::RemoteIcechunk, "🧊 Remote Icechunk (HTTP/S3)");
                        ui.selectable_value(&mut selected, StoreKind::LocalIcechunk, "🧊 Local Icechunk (FileSystem)");
                        ui.selectable_value(&mut selected, StoreKind::LocalNetCdf, "📁 Local NetCDF (.nc/.cdf)");
                        ui.separator();
                        ui.selectable_value(&mut selected, StoreKind::ProceduralVolume4D, "🌐 4D Known-Truth Volume (Procedural)");
                        ui.selectable_value(&mut selected, StoreKind::ProceduralRandom, "🎲 2D Procedural Matrix (Test)");
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
                let btn_label = if app.is_loading { "⏳ Loading..." } else { "⬇️ Load" };
                if ui.add_enabled(!app.is_loading, egui::Button::new(egui::RichText::new(btn_label).strong())).clicked() {
                    let target = app.store_target_input.clone();
                    app.submit_or_activate_source(&target, Some(app.selected_store_kind));
                }

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

                ui.collapsing("🧊 Dataset Manager", |ui| {
                    if app.dataset_manager.is_empty() {
                        ui.label("No active datasets in DatasetManager.");
                    } else {
                        ui.label(format!("Active Datasets: {}", app.dataset_manager.len()));
                        ui.add_space(4.0);

                        let mut to_activate: Option<(String, StoreKind)> = None;

                        for d in app.dataset_manager.iter() {
                            let is_active = app.store_target_input == d.source.uri;
                            let label_text = format!("• {} [{}]", d.source.display_name, d.source.uri);

                            if ui.selectable_label(is_active, egui::RichText::new(label_text).strong()).clicked() {
                                to_activate = Some((
                                    d.source.uri.clone(),
                                    StoreKind::from_data_source_kind(&d.source.kind),
                                ));
                            }
                        }

                        if let Some((uri, kind)) = to_activate {
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
    if ui
        .button(egui::RichText::new("🌐 Store").strong())
        .clicked()
    {
        app.show_left_panel = !app.show_left_panel;
    }
}
