use crate::app::OctantApp;

/// Renders a compact cache memory usage bar and block cache hit rate summary.
pub fn show_cache_stats(app: &OctantApp, ui: &mut egui::Ui) {
    let current_bytes = app.block_cache.current_bytes();
    let max_bytes = app.block_cache.max_bytes();
    let current_mb = current_bytes as f64 / (1024.0 * 1024.0);
    let max_mb = max_bytes as f64 / (1024.0 * 1024.0);
    let fraction = (current_bytes as f32 / max_bytes.max(1) as f32).clamp(0.0, 1.0);

    ui.add(
        egui::ProgressBar::new(fraction)
            .text(format!("{:.1} MB / {:.0} MB", current_mb, max_mb))
            .show_percentage(),
    );

    ui.add_space(2.0);
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!(
                "{} blocks • {:.1}% hit rate",
                app.block_cache.cached_count(),
                app.block_cache.hit_rate()
            ))
            .weak(),
        );
    });
}
