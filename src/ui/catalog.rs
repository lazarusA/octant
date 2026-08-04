use crate::app::{OctantApp, StoreKind};
use crate::catalog::{CatalogCategoryFilter, ICECHUNK_CATALOG, ZARR_CATALOG, get_catalog_entries};

pub fn show_catalog_window(app: &mut OctantApp, ctx: &egui::Context) {
    if !app.show_catalog_window {
        return;
    }

    let mut open = app.show_catalog_window;
    let mut should_close = false;

    egui::Window::new("📚 Dataset Catalog")
        .open(&mut open)
        .default_size([780.0, 560.0])
        .min_size([500.0, 380.0])
        .resizable(true)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.heading("🌐 Zarr & 🧊 Icechunk Dataset Catalog");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let total_count = ZARR_CATALOG.len() + ICECHUNK_CATALOG.len();
                    ui.small(format!("{} total stores", total_count));
                });
            });
            ui.label(
                egui::RichText::new("Browse curated cloud Zarr and Icechunk datasets. Select any entry to load it into Octant.")
                    .small()
                    .italics(),
            );
            ui.add_space(6.0);

            // Filter and Search Row
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("🔍 Search:").strong());
                ui.add(
                    egui::TextEdit::singleline(&mut app.catalog_search_query)
                        .hint_text("Filter by name, key, description or URL...")
                        .desired_width(280.0),
                );

                ui.separator();
                ui.label(egui::RichText::new("Category:").strong());

                let zarr_count = ZARR_CATALOG.len();
                let icechunk_count = ICECHUNK_CATALOG.len();
                let total_count = zarr_count + icechunk_count;

                ui.selectable_value(
                    &mut app.catalog_category_filter,
                    CatalogCategoryFilter::All,
                    format!("All ({})", total_count),
                );
                ui.selectable_value(
                    &mut app.catalog_category_filter,
                    CatalogCategoryFilter::Zarr,
                    format!("🌐 Zarr ({})", zarr_count),
                );
                ui.selectable_value(
                    &mut app.catalog_category_filter,
                    CatalogCategoryFilter::Icechunk,
                    format!("🧊 Icechunk ({})", icechunk_count),
                );
            });

            ui.separator();
            ui.add_space(4.0);

            let query = app.catalog_search_query.trim().to_lowercase();
            let entries = get_catalog_entries(app.catalog_category_filter);

            let filtered_entries: Vec<_> = entries
                .into_iter()
                .filter(|e| {
                    if query.is_empty() {
                        return true;
                    }
                    e.key.to_lowercase().contains(&query)
                        || e.label.to_lowercase().contains(&query)
                        || e.subtitle.to_lowercase().contains(&query)
                        || e.store.to_lowercase().contains(&query)
                })
                .collect();

            if filtered_entries.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    ui.label(egui::RichText::new("🚫 No catalog entries found matching your query.").strong());
                    if ui.button("Clear Search").clicked() {
                        app.catalog_search_query.clear();
                    }
                });
            } else {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for entry in filtered_entries {
                            ui.group(|ui| {
                                ui.set_width(ui.available_width());
                                ui.horizontal(|ui| {
                                    // Store type badge
                                    let badge_text = match entry.store_kind {
                                        StoreKind::RemoteZarr => "🌐 Zarr",
                                        StoreKind::RemoteIcechunk => "🧊 Icechunk",
                                        _ => "Store",
                                    };
                                    let badge_color = match entry.store_kind {
                                        StoreKind::RemoteZarr => egui::Color32::from_rgb(40, 130, 220),
                                        StoreKind::RemoteIcechunk => egui::Color32::from_rgb(140, 70, 230),
                                        _ => egui::Color32::GRAY,
                                    };

                                    ui.label(
                                        egui::RichText::new(format!("[{}]", badge_text))
                                            .strong()
                                            .color(badge_color),
                                    );

                                    ui.label(egui::RichText::new(entry.label).strong().size(14.0));

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        let btn = egui::Button::new(
                                            egui::RichText::new("⚡ Select & Load").strong(),
                                        );
                                        if ui.add(btn).clicked() {
                                            app.selected_store_kind = entry.store_kind;
                                            app.store_target_input = entry.store.to_string();
                                            should_close = true;
                                            app.inspect_active_store();
                                        }
                                        ui.small(egui::RichText::new(format!("key: {}", entry.key)).monospace());
                                    });
                                });

                                if !entry.subtitle.is_empty() {
                                    ui.label(egui::RichText::new(entry.subtitle).small().italics());
                                }

                                ui.add_space(2.0);
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("URL:").small().strong());
                                    ui.label(
                                        egui::RichText::new(entry.store)
                                            .small()
                                            .monospace()
                                            .color(egui::Color32::LIGHT_BLUE),
                                    );
                                });
                            });
                            ui.add_space(4.0);
                        }
                    });
            }
        });

    if should_close {
        open = false;
    }
    app.show_catalog_window = open;
}
