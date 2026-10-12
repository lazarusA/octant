use crate::app::OctantApp;
use crate::i18n::{DatasetKey, fill, tr};
use crate::utils::stack_str;

/// Renders a compact cache memory usage bar and block cache hit rate summary.
pub fn show_cache_stats(app: &OctantApp, ui: &mut egui::Ui) {
    let current_bytes = app.block_cache.current_bytes();
    let max_bytes = app.block_cache.max_bytes();
    let current_mb = current_bytes as f64 / (1024.0 * 1024.0);
    let max_mb = max_bytes as f64 / (1024.0 * 1024.0);
    let fraction = (current_bytes as f32 / max_bytes.max(1) as f32).clamp(0.0, 1.0);

    let (mut cur_buf, mut max_buf, mut buf) = ([0u8; 16], [0u8; 16], [0u8; 96]);
    let current = stack_str(
        &mut cur_buf,
        format_args!(
            "{}",
            crate::i18n::number::Num(format_args!("{current_mb:.1}"))
        ),
    );
    let max = stack_str(&mut max_buf, format_args!("{max_mb:.0}"));
    ui.add(
        egui::ProgressBar::new(fraction)
            .text(fill(
                &mut buf,
                tr(DatasetKey::CacheUsage),
                &[&current, &max],
            ))
            .show_percentage(),
    );

    ui.add_space(2.0);
    let mut rate_buf = [0u8; 16];
    let rate = stack_str(
        &mut rate_buf,
        format_args!(
            "{}",
            crate::i18n::number::Num(format_args!("{:.1}", app.block_cache.hit_rate()))
        ),
    );
    let mut buf = [0u8; 128];
    ui.label(
        egui::RichText::new(fill(
            &mut buf,
            tr(DatasetKey::CacheBlocksHitRate),
            &[&app.block_cache.cached_count(), &rate],
        ))
        .weak(),
    );
}
