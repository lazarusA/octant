use crate::app::OctantApp;

pub fn show_cache_menu(app: &mut OctantApp, ui: &mut egui::Ui) {
    ui.menu_button("🧠 Cache", |ui| {
        ui.set_min_width(280.0);
        ui.label(egui::RichText::new("LRU Memory Cache & Prefetcher").strong());
        ui.separator();

        let current_bytes = app.lru_cache.current_bytes();
        let current_mb = current_bytes as f64 / (1024.0 * 1024.0);
        let max_bytes = app.lru_cache.max_bytes();
        let fraction = (current_bytes as f32 / max_bytes as f32).clamp(0.0, 1.0);

        ui.label(egui::RichText::new("Memory Consumption (1GB Default Limit):").small());
        ui.add(
            egui::ProgressBar::new(fraction).text(format!(
                "{:.2} MB / {} MB ({:.1}%)",
                current_mb,
                app.max_cache_mb,
                fraction * 100.0
            )),
        );

        ui.add_space(6.0);
        ui.small(format!(
            "Cached Slices: {} | Target Lookahead: {} slices",
            app.lru_cache.cached_count(),
            app.prefetch_lookahead
        ));

        ui.small(format!(
            "Hits: {} | Misses: {} (Hit Rate: {:.1}%)",
            app.lru_cache.hits(),
            app.lru_cache.misses(),
            app.lru_cache.hit_rate()
        ));

        let pending = app.prefetcher.pending_count();
        if pending > 0 {
            ui.small(format!("🟢 Background Prefetching: {} queued", pending));
        } else {
            ui.small("⚪ Buffer Warm / Prefetch Idle");
        }

        ui.separator();
        ui.label(egui::RichText::new("Cache Capacity Settings").strong());
        let old_mb = app.max_cache_mb;
        ui.add(egui::Slider::new(&mut app.max_cache_mb, 256..=4096).suffix(" MB Limit"));
        if old_mb != app.max_cache_mb {
            app.lru_cache.set_max_bytes(app.max_cache_mb * 1024 * 1024);
        }

        ui.add(egui::Slider::new(&mut app.prefetch_lookahead, 12..=48).suffix(" Lookahead Slices"));

        ui.add_space(4.0);
        if ui.button("🗑 Flush & Clear Cache").clicked() {
            app.lru_cache.clear();
            ui.close_menu();
        }
    });
}
