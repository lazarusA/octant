use crate::app::OctantApp;

pub fn show_cache_menu(app: &mut OctantApp, ui: &mut egui::Ui) {
    ui.menu_button("🧠 Cache", |ui| {
        ui.set_min_width(360.0);
        ui.label(egui::RichText::new("Unified Multi-Variable Memory Cache").strong());
        ui.separator();

        let block_bytes = app.block_cache.current_bytes();
        let pyramid_bytes = app
            .active_pyramid
            .as_ref()
            .map(|p| p.bytes_size())
            .unwrap_or(0);
        let total_bytes = block_bytes + pyramid_bytes;

        let total_mb = total_bytes as f64 / (1024.0 * 1024.0);
        let block_mb = block_bytes as f64 / (1024.0 * 1024.0);
        let pyramid_mb = pyramid_bytes as f64 / (1024.0 * 1024.0);

        let max_bytes = app.block_cache.max_bytes();
        let fraction = (total_bytes as f32 / max_bytes.max(1) as f32).clamp(0.0, 1.0);

        ui.label(egui::RichText::new("Global RAM Memory Usage:").small());
        ui.add(egui::ProgressBar::new(fraction).text(format!(
            "{:.2} MB / {} MB ({:.1}%)",
            total_mb,
            app.max_cache_mb,
            fraction * 100.0
        )));

        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("Memory Breakdown by Kind:")
                .strong()
                .small(),
        );
        ui.horizontal(|ui| {
            ui.small(format!(
                "• Resident Hyperslabs: {:.2} MB ({} blocks)",
                block_mb,
                app.block_cache.cached_count()
            ));
        });
        if pyramid_bytes > 0 {
            ui.horizontal(|ui| {
                ui.small(format!("• Multi-Res Pyramid: {:.2} MB", pyramid_mb));
            });
        }

        ui.add_space(4.0);
        ui.small(format!(
            "Cache Hits: {} | Misses: {} (Hit Rate: {:.1}%)",
            app.block_cache.hits(),
            app.block_cache.misses(),
            app.block_cache.hit_rate()
        ));

        let pending = app.block_prefetcher.pending_count();
        if pending > 0 {
            ui.small(format!("🟢 Background Prefetching: {} in-flight", pending));
        } else {
            ui.small("⚪ Buffer Warm / All Blocks Cached");
        }

        ui.separator();
        ui.label(
            egui::RichText::new("Summary per Variable:")
                .strong()
                .small(),
        );

        let summaries = app.block_cache.summary_per_variable();
        if summaries.is_empty() {
            ui.small("  (No variables currently cached in memory)");
        } else {
            egui::ScrollArea::vertical()
                .max_height(140.0)
                .show(ui, |ui| {
                    let mut var_to_clear: Option<(String, String)> = None;

                    for s in &summaries {
                        ui.horizontal(|ui| {
                            let mb = s.bytes as f64 / (1024.0 * 1024.0);
                            ui.label(
                                egui::RichText::new(&s.variable_name)
                                    .monospace()
                                    .color(ui.visuals().strong_text_color()),
                            );
                            ui.small(format!("({:.1} MB, {} blks)", mb, s.block_count));
                            if ui
                                .small_button("🗑")
                                .on_hover_text("Clear cache for this variable")
                                .clicked()
                            {
                                var_to_clear = Some((s.source_id.clone(), s.variable_name.clone()));
                            }
                        });
                    }

                    if let Some((source_id, var_name)) = var_to_clear {
                        app.block_cache.clear_variable(&source_id, &var_name);
                    }
                });
        }

        ui.separator();
        ui.label(egui::RichText::new("Prefetch & Capacity Settings").strong());
        ui.checkbox(&mut app.enable_prefetch, "Enable Background Prefetching");
        let old_mb = app.max_cache_mb;
        ui.add(egui::Slider::new(&mut app.max_cache_mb, 256..=8192).suffix(" MB Limit"));
        if old_mb != app.max_cache_mb {
            app.block_cache
                .set_max_bytes(app.max_cache_mb * 1024 * 1024);
        }

        ui.add(
            egui::Slider::new(&mut app.block_window_size, 8..=128).suffix(" Window Size (steps)"),
        );

        ui.add_space(4.0);
        if ui.button("🗑 Flush & Clear All Caches").clicked() {
            app.block_cache.clear();
            app.active_pyramid = None;
            ui.close();
        }
    });
}
