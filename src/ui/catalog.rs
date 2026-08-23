use crate::app::{OctantApp, StoreKind};
use crate::catalog::{
    CatalogCategoryFilter, ICECHUNK_CATALOG, PROCEDURAL_CATALOG, ZARR_CATALOG, get_catalog_entries,
};

pub fn show_catalog_window(app: &mut OctantApp, ctx: &egui::Context) {
    if !app.show_catalog_window {
        return;
    }

    let mut open = app.show_catalog_window;
    let mut should_close = false;

    let max_w = (ctx.viewport_rect().width() * 0.7).clamp(480.0, 950.0);

    let response = egui::Window::new("📚 Dataset Catalog")
        .open(&mut open)
        .default_size([max_w, 560.0])
        .max_width(max_w)
        .min_size([460.0, 380.0])
        .resizable(true)
        .collapsible(false)
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.heading("🌐 Zarr, 🧊 Icechunk & 🎲 Procedural Dataset Catalog");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let total_count =
                        ZARR_CATALOG.len() + ICECHUNK_CATALOG.len() + PROCEDURAL_CATALOG.len();
                    ui.small(format!("{} total stores", total_count));
                });
            });
            ui.label(
                egui::RichText::new("Browse curated cloud Zarr, Icechunk, and procedural ground-truth datasets. Select any entry to load it into Octant.")
                    .small()
                    .italics(),
            );
            ui.add_space(6.0);

            // Filter and Search Row
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("🔍 Search:").strong());
                ui.add(
                    egui::TextEdit::singleline(&mut app.catalog_search_query)
                        .hint_text("Filter by name, description or URL...")
                        .desired_width(260.0),
                );

                ui.separator();
                ui.label(egui::RichText::new("Category:").strong());

                let zarr_count = ZARR_CATALOG.len();
                let icechunk_count = ICECHUNK_CATALOG.len();
                let procedural_count = PROCEDURAL_CATALOG.len();
                let total_count = zarr_count + icechunk_count + procedural_count;

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
                ui.selectable_value(
                    &mut app.catalog_category_filter,
                    CatalogCategoryFilter::Procedural,
                    format!("🎲 Procedural ({})", procedural_count),
                );
            });

            ui.separator();
            ui.add_space(4.0);

            let query = app.catalog_search_query.trim();
            let entries = get_catalog_entries(app.catalog_category_filter);

            let filtered_entries: Vec<_> = entries
                .into_iter()
                .filter(|e| {
                    if query.is_empty() {
                        return true;
                    }
                    contains_ignore_ascii_case(e.key, query)
                        || contains_ignore_ascii_case(e.label, query)
                        || contains_ignore_ascii_case(e.subtitle, query)
                        || contains_ignore_ascii_case(e.store, query)
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
                            let trimmed_url = entry.store.trim();
                            ui.group(|ui| {
                                ui.set_width(ui.available_width());

                                ui.horizontal(|ui| {
                                    // Store type badge using system theme colors
                                    let badge_label = match entry.store_kind {
                                        StoreKind::RemoteZarr => "[🌐 Zarr]",
                                        StoreKind::RemoteIcechunk => "[🧊 Icechunk]",
                                        StoreKind::ProceduralVolume4D => "[🌐 4D Volume]",
                                        StoreKind::ProceduralRandom => "[🎲 2D Matrix]",
                                        _ => "[Store]",
                                    };
                                    let badge_color = match entry.store_kind {
                                        StoreKind::RemoteZarr => ui.visuals().selection.bg_fill,
                                        StoreKind::RemoteIcechunk => {
                                            ui.visuals().widgets.active.bg_fill
                                        }
                                        StoreKind::ProceduralVolume4D
                                        | StoreKind::ProceduralRandom => {
                                            ui.visuals().widgets.hovered.bg_fill
                                        }
                                        _ => ui.visuals().widgets.noninteractive.fg_stroke.color,
                                    };

                                    ui.label(
                                        egui::RichText::new(badge_label)
                                            .strong()
                                            .color(badge_color),
                                    );

                                    ui.label(egui::RichText::new(entry.label).strong().size(14.0));

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        let btn = egui::Button::new(
                                            egui::RichText::new("⚡ Select & Load").strong(),
                                        );
                                        if ui.add(btn).clicked() {
                                            app.submit_or_activate_source(trimmed_url, Some(entry.store_kind));
                                            should_close = true;
                                        }
                                    });
                                });

                                if !entry.subtitle.is_empty() {
                                    ui.label(egui::RichText::new(entry.subtitle).small().italics());
                                }

                                ui.add_space(2.0);
                                ui.horizontal_top(|ui| {
                                    ui.label(egui::RichText::new("URL:").small().strong());
                                    ui.add(
                                        egui::Label::new(
                                            egui::RichText::new(trimmed_url)
                                                .small()
                                                .monospace()
                                                .color(ui.visuals().hyperlink_color),
                                        )
                                        .wrap(),
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

    // Close when the user presses outside the catalog window.
    // We use primary_pressed() instead of any_click() to avoid a same-frame race:
    // button.clicked() fires on *release*, so on the frame the window first appears,
    // primary_pressed() is already false — preventing an immediate close.
    if let Some(r) = response
        && let Some(pos) = ctx.input(|i| i.pointer.interact_pos())
        && ctx.input(|i| i.pointer.primary_pressed())
        && !r.response.rect.contains(pos)
    {
        open = false;
    }

    app.show_catalog_window = open;
}

#[inline]
fn contains_ignore_ascii_case(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    if needle.len() > haystack.len() {
        return false;
    }
    haystack
        .as_bytes()
        .windows(needle.len())
        .any(|w| w.eq_ignore_ascii_case(needle.as_bytes()))
}
