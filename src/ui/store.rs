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
                ui.heading("🗂️ Stores");
                ui.add_space(4.0);
                ui.label("Choose a data source and load metadata into the current session.");
                ui.separator();

                let mut selected = app.selected_store_kind;
                egui::ComboBox::from_id_salt("left_store_kind_select")
                    .selected_text(match selected {
                        StoreKind::RemoteZarr => "🌐 Remote Zarr",
                        StoreKind::LocalZarr => "📁 Local Zarr",
                        StoreKind::RemoteIcechunk => "🧊 Remote Icechunk",
                        StoreKind::LocalIcechunk => "🧊 Local Icechunk",
                        StoreKind::ProceduralRandom => "🎲 Procedural Random",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut selected, StoreKind::RemoteZarr, "🌐 Remote Zarr (HTTP/S3)");
                        ui.selectable_value(&mut selected, StoreKind::LocalZarr, "📁 Local Zarr (FileSystem)");
                        ui.selectable_value(&mut selected, StoreKind::RemoteIcechunk, "🧊 Remote Icechunk (HTTP/S3)");
                        ui.selectable_value(&mut selected, StoreKind::LocalIcechunk, "🧊 Local Icechunk (FileSystem)");
                        ui.selectable_value(&mut selected, StoreKind::ProceduralRandom, "🎲 Procedural Random Test");
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
                        StoreKind::ProceduralRandom => {
                            app.store_target_input = "procedural://random".to_string();
                        }
                    }
                }

                ui.add_space(6.0);
                ui.label(egui::RichText::new("Target URL / Path").strong());
                let res = ui.text_edit_singleline(&mut app.store_target_input);
                if res.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    app.inspect_active_store();
                }

                ui.add_space(6.0);
                let btn_label = if app.is_loading { "⏳ Fetching..." } else { "🔍 Load Metadata" };
                if ui.add_enabled(!app.is_loading, egui::Button::new(egui::RichText::new(btn_label).strong())).clicked() {
                    app.inspect_active_store();
                }

                ui.add_space(6.0);
                if ui.button(egui::RichText::new("📚 Open Catalog").strong()).clicked() {
                    app.show_catalog_window = true;
                }

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

                        let datasets: Vec<(String, String, String, Option<crate::stores::DatasetMetadata>, crate::data::DataSourceKind)> = app
                            .dataset_manager
                            .iter()
                            .map(|d| (
                                d.id.clone(),
                                d.source.display_name.clone(),
                                d.source.uri.clone(),
                                d.metadata.clone(),
                                d.source.kind.clone(),
                            ))
                            .collect();

                        for (_id, display_name, uri, metadata, kind) in datasets {
                            let is_active = app.store_target_input == uri;
                            let label_text = format!("• {} [{}]", display_name, uri);

                            if ui.selectable_label(is_active, egui::RichText::new(label_text).strong()).clicked() {
                                app.store_target_input = uri;
                                match kind {
                                    crate::data::DataSourceKind::RemoteZarr => app.selected_store_kind = StoreKind::RemoteZarr,
                                    crate::data::DataSourceKind::LocalZarr => app.selected_store_kind = StoreKind::LocalZarr,
                                    crate::data::DataSourceKind::RemoteIcechunk => app.selected_store_kind = StoreKind::RemoteIcechunk,
                                    crate::data::DataSourceKind::LocalIcechunk => app.selected_store_kind = StoreKind::LocalIcechunk,
                                    _ => app.selected_store_kind = StoreKind::ProceduralRandom,
                                }

                                if let Some(meta) = metadata {
                                    self_activate_dataset_metadata(app, meta);
                                } else {
                                    app.inspect_active_store();
                                }
                            }
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

fn self_activate_dataset_metadata(app: &mut OctantApp, metadata: crate::stores::DatasetMetadata) {
    app.status_message = format!(
        "Activated dataset '{}' (Found {} variables)",
        metadata.name,
        metadata.variables.len()
    );
    app.show_variables_overlay = true;
    if let Some(first_var) = metadata.variables.first() {
        let rank = first_var.shape.len();
        app.selected_dim_indices = vec![0; rank];
        app.selected_dim_ranges = first_var
            .shape
            .iter()
            .map(|&s| (0, (s as usize).saturating_sub(1)))
            .collect();
        app.dim_config = vec![
            crate::app::DimConfig {
                spatial: crate::app::SpatialRole::None,
                animation: crate::app::AnimationRole::None,
                active: false,
            };
            rank
        ];
        app.spatial_dims.clear();
        app.animated_dim = None;
    }
    app.active_dataset_metadata = Some(metadata);
    app.selected_variable_idx = 0;
}
