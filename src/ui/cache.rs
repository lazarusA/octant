use crate::ui::number_input::LocalizedInput;
use crate::{
    app::OctantApp,
    i18n::{CommonKey, DatasetKey, fill, tr},
    ui::icons::{Icon, IconSize, IconTone, ToolbarButton, UiIconExt},
    utils::stack_str,
};

/// The cache slider's top. In the browser `usize` is 32 bits, so 4096 MB in
/// bytes wraps to 0 and nothing stays cached; wasm memory ends at 4 GiB anyway.
const MAX_CACHE_MB: usize = if cfg!(target_arch = "wasm32") {
    2048
} else {
    8192
};

/// The cache button and its menu; the button's response.
pub fn show_cache_menu(app: &mut OctantApp, ui: &mut egui::Ui, compact: bool) -> egui::Response {
    let button_response = ui.add(
        ToolbarButton::new(Icon::Cache, tr(CommonKey::Cache))
            .compact(compact)
            .owns_popup(),
    );
    egui::Popup::menu(&button_response).show(|ui| {
        ui.set_min_width(360.0);
        ui.horizontal(|ui| {
            ui.icon(Icon::Cache, IconSize::Sm);
            ui.label(egui::RichText::new(tr(DatasetKey::CacheMenuTitle)).strong());
        });
        ui.separator();

        let block_bytes = app.block_cache.current_bytes();
        let pyramid_bytes = app
            .layers
            .base
            .data
            .pyramid
            .as_ref()
            .map(|p| p.bytes_size())
            .unwrap_or(0);
        let total_bytes = block_bytes + pyramid_bytes;

        let total_mb = total_bytes as f64 / (1024.0 * 1024.0);
        let block_mb = block_bytes as f64 / (1024.0 * 1024.0);
        let pyramid_mb = pyramid_bytes as f64 / (1024.0 * 1024.0);

        let max_bytes = app.block_cache.max_bytes();
        let fraction = (total_bytes as f32 / max_bytes.max(1) as f32).clamp(0.0, 1.0);

        let (mut a, mut b, mut buf) = ([0u8; 16], [0u8; 16], [0u8; 128]);
        ui.label(egui::RichText::new(tr(DatasetKey::GlobalMemoryUsage)).small());
        let total = stack_str(
            &mut a,
            format_args!(
                "{}",
                crate::i18n::number::Num(format_args!("{total_mb:.2}"))
            ),
        );
        let percent = stack_str(
            &mut b,
            format_args!(
                "{}",
                crate::i18n::number::Num(format_args!("{:.1}", fraction * 100.0))
            ),
        );
        let usage = fill(
            &mut buf,
            tr(DatasetKey::CacheUsagePercent),
            &[&total, &app.max_cache_mb, &percent],
        );
        ui.add(egui::ProgressBar::new(fraction).text(usage));

        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(tr(DatasetKey::MemoryBreakdown))
                .strong()
                .small(),
        );
        ui.horizontal(|ui| {
            ui.icon(Icon::Bullet, IconSize::Xs);
            let mb = stack_str(
                &mut a,
                format_args!(
                    "{}",
                    crate::i18n::number::Num(format_args!("{block_mb:.2}"))
                ),
            );
            let count = app.block_cache.cached_count();
            ui.small(fill(
                &mut buf,
                tr(DatasetKey::ResidentHyperslabs),
                &[&mb, &count],
            ));
        });
        if pyramid_bytes > 0 {
            ui.horizontal(|ui| {
                ui.icon(Icon::Bullet, IconSize::Xs);
                let mb = stack_str(
                    &mut a,
                    format_args!(
                        "{}",
                        crate::i18n::number::Num(format_args!("{pyramid_mb:.2}"))
                    ),
                );
                ui.small(fill(&mut buf, tr(DatasetKey::MultiResPyramid), &[&mb]));
            });
        }

        ui.add_space(4.0);
        let rate = stack_str(
            &mut b,
            format_args!(
                "{}",
                crate::i18n::number::Num(format_args!("{:.1}", app.block_cache.hit_rate()))
            ),
        );
        let (hits, misses) = (app.block_cache.hits(), app.block_cache.misses());
        ui.small(fill(
            &mut buf,
            tr(DatasetKey::CacheHitsMisses),
            &[&hits, &misses, &rate],
        ));

        let pending = app.block_prefetcher.pending_count();
        if pending > 0 {
            ui.horizontal(|ui| {
                ui.icon_toned(Icon::Bolt, IconSize::Xs, IconTone::Success);
                ui.small(fill(
                    &mut buf,
                    tr(DatasetKey::PrefetchInFlight),
                    &[&pending],
                ));
            });
        } else {
            ui.horizontal(|ui| {
                ui.icon_toned(Icon::Check, IconSize::Xs, IconTone::Muted);
                ui.small(tr(DatasetKey::AllBlocksCached));
            });
        }

        ui.separator();
        ui.label(
            egui::RichText::new(tr(DatasetKey::SummaryPerVariable))
                .strong()
                .small(),
        );

        let summaries = app.block_cache.summary_per_variable();
        if summaries.is_empty() {
            ui.horizontal(|ui| {
                ui.add_space(6.0);
                ui.small(tr(DatasetKey::NoVariablesCached));
            });
        } else {
            egui::ScrollArea::vertical()
                .max_height(140.0)
                .show(ui, |ui| {
                    let mut var_to_clear: Option<(String, String)> = None;

                    for s in &summaries {
                        ui.push_id(("var_cache_row", &s.source_id, &s.variable_name), |ui| {
                            ui.horizontal(|ui| {
                                let mb = s.bytes as f64 / (1024.0 * 1024.0);
                                ui.label(
                                    egui::RichText::new(&s.variable_name)
                                        .monospace()
                                        .color(ui.visuals().strong_text_color()),
                                );
                                let mb = stack_str(
                                    &mut a,
                                    format_args!(
                                        "{}",
                                        crate::i18n::number::Num(format_args!("{mb:.1}"))
                                    ),
                                );
                                ui.small(fill(
                                    &mut buf,
                                    tr(DatasetKey::VariableCacheSize),
                                    &[&mb, &s.block_count],
                                ));
                                if ui
                                    .icon_button(Icon::Trash, "")
                                    .on_hover_text(tr(DatasetKey::ClearVariableCacheHover))
                                    .clicked()
                                {
                                    var_to_clear =
                                        Some((s.source_id.clone(), s.variable_name.clone()));
                                }
                            });
                        });
                    }

                    if let Some((source_id, var_name)) = var_to_clear {
                        app.block_cache.clear_variable(&source_id, &var_name);
                    }
                });
        }

        ui.separator();
        ui.label(egui::RichText::new(tr(DatasetKey::PrefetchCapacitySettings)).strong());
        ui.checkbox(
            &mut app.playback.enable_prefetch,
            tr(DatasetKey::EnablePrefetching),
        );
        let old_mb = app.max_cache_mb;
        ui.add(
            egui::Slider::new(&mut app.max_cache_mb, 256..=MAX_CACHE_MB)
                .localized()
                .suffix(tr(DatasetKey::CacheLimitSuffix)),
        );
        if old_mb != app.max_cache_mb {
            app.block_cache
                .set_max_bytes(app.max_cache_mb.saturating_mul(1024 * 1024));
        }

        ui.add(
            egui::Slider::new(&mut app.block_window_size, 8..=128)
                .localized()
                .suffix(tr(DatasetKey::WindowSizeSuffix)),
        );

        let old_threads = app.prefetch_threads;
        ui.add(
            egui::Slider::new(&mut app.prefetch_threads, 2..=32)
                .localized()
                .suffix(tr(DatasetKey::PrefetchWorkersSuffix)),
        );
        if old_threads != app.prefetch_threads {
            app.block_prefetcher
                .set_max_concurrent_threads(app.prefetch_threads);
        }

        ui.add_space(4.0);
        if ui
            .icon_button(Icon::Trash, tr(DatasetKey::FlushAllCaches))
            .clicked()
        {
            app.block_cache.clear();
            app.layers.base.data.pyramid = None;
            ui.close();
        }
    });
    button_response
}
