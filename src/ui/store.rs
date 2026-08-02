use crate::app::{OctantApp, StoreKind};

pub fn show_store_menu(app: &mut OctantApp, ui: &mut egui::Ui) {
    ui.menu_button("🌐 Store", |ui| {
        ui.set_min_width(280.0);
        ui.label(egui::RichText::new("Data Store Provider").strong());

        let old_store_kind = app.selected_store_kind;
        egui::ComboBox::from_id_salt("top_store_kind_select")
            .selected_text(match app.selected_store_kind {
                StoreKind::RemoteZarr => "🌐 Remote Zarr (HTTP/S3)",
                StoreKind::LocalZarr => "📁 Local Zarr (FileSystem)",
                StoreKind::RemoteIcechunk => "🧊 Remote Icechunk (HTTP/S3)",
                StoreKind::LocalIcechunk => "🧊 Local Icechunk (FileSystem)",
                StoreKind::ProceduralRandom => "🎲 Procedural Random Test",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut app.selected_store_kind, StoreKind::RemoteZarr, "🌐 Remote Zarr (HTTP/S3)");
                ui.selectable_value(&mut app.selected_store_kind, StoreKind::LocalZarr, "📁 Local Zarr (FileSystem)");
                ui.selectable_value(&mut app.selected_store_kind, StoreKind::RemoteIcechunk, "🧊 Remote Icechunk (HTTP/S3)");
                ui.selectable_value(&mut app.selected_store_kind, StoreKind::LocalIcechunk, "🧊 Local Icechunk (FileSystem)");
                ui.selectable_value(&mut app.selected_store_kind, StoreKind::ProceduralRandom, "🎲 Procedural Random Test");
            });

        if old_store_kind != app.selected_store_kind {
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
        ui.label(egui::RichText::new("Target URL / Path:").strong());
        let res = ui.text_edit_singleline(&mut app.store_target_input);
        if res.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            app.inspect_active_store();
        }

        ui.add_space(4.0);
        let btn_label = if app.is_loading {
            "⏳ Fetching Store Metadata..."
        } else {
            "🔍 Fetch / Load Store Metadata"
        };
        if ui.add_enabled(!app.is_loading, egui::Button::new(egui::RichText::new(btn_label).strong())).clicked() {
            app.inspect_active_store();
        }

        ui.separator();
        ui.menu_button("ℹ About Store", |ui| {
            ui.set_min_width(260.0);
            ui.label(egui::RichText::new("Store Information").strong());
            ui.separator();

            if let Some(metadata) = &app.active_dataset_metadata {
                ui.small(format!("Provider: {}", metadata.store_type));
                ui.small(format!("Dataset: {}", metadata.name));
                ui.small(format!("Variables: {}", metadata.variables.len()));
            } else {
                ui.small("No dataset metadata loaded.");
            }

            ui.add_space(4.0);
            ui.label(egui::RichText::new("Location Target:").small().strong());
            ui.small(&app.store_target_input);

            ui.add_space(6.0);
            if ui.button("🔄 Refresh Store Metadata").clicked() {
                app.inspect_active_store();
                ui.close_menu();
            }
        });
    });
}
