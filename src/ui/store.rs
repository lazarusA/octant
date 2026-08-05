use crate::app::{OctantApp, StoreKind};

pub fn show_left_panel(app: &mut OctantApp, ui: &mut egui::Ui) {
    if !app.show_left_panel {
        return;
    }

    let panel_width = 280.0;
    egui::Panel::left("octant_left_store_panel")
        .resizable(false)
        .default_size(panel_width)
        .show(ui, |ui| {
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
        });
}

pub fn show_store_menu(app: &mut OctantApp, ui: &mut egui::Ui) {
    if ui.button(egui::RichText::new("🌐 Store").strong()).clicked() {
        app.show_left_panel = !app.show_left_panel;
    }
}
