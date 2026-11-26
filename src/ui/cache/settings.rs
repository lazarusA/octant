//! The cache menu's settings: prefetching, the cache limit, the block window
//! and the worker count, and flushing every cache.

use crate::app::OctantApp;
use crate::i18n::{DatasetKey, tr};
use crate::ui::icons::{Icon, UiIconExt};
use crate::ui::number_input::LocalizedInput;

/// The cache slider's top. In the browser `usize` is 32 bits, so 4096 MB in
/// bytes wraps to 0 and nothing stays cached; wasm memory ends at 4 GiB anyway.
const MAX_CACHE_MB: usize = if cfg!(target_arch = "wasm32") {
    2048
} else {
    8192
};

/// Prefetching, the cache limit, the block window and the worker count, and
/// flushing every cache.
pub(super) fn show_settings(app: &mut OctantApp, ui: &mut egui::Ui) {
    ui.label(egui::RichText::new(tr(DatasetKey::PrefetchCapacitySettings)).strong());
    let suffixes = [
        tr(DatasetKey::CacheLimitSuffix),
        tr(DatasetKey::WindowSizeSuffix),
        tr(DatasetKey::PrefetchWorkersSuffix),
    ];
    ui.spacing_mut().slider_width = slider_width(ui, &suffixes);
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
        // The menu, not the More menu's Cache section holding it.
        ui.close_kind(egui::UiKind::Menu);
    }
}

/// Narrowest slider bar kept however little room is left.
const MIN_SLIDER_WIDTH: f32 = 40.0;

/// The sliders' bar width: the default, narrowed so the value and the longest
/// of `suffixes` (the sliders' labels) fit beside it, as on a phone.
fn slider_width(ui: &egui::Ui, suffixes: &[&str]) -> f32 {
    let font = egui::TextStyle::Button.resolve(ui.style());
    let color = ui.visuals().text_color();
    let measure = |text: &str| {
        ui.painter()
            .layout_no_wrap(text.to_owned(), font.clone(), color)
            .size()
            .x
    };
    let label = suffixes.iter().map(|s| measure(s)).fold(0.0, f32::max);
    // The widest value, with room for a digit group separator.
    let value = measure("8 192") + 2.0 * ui.spacing().button_padding.x;
    let room = ui.available_width() - label - value - 2.0 * ui.spacing().item_spacing.x;
    room.clamp(MIN_SLIDER_WIDTH, ui.spacing().slider_width)
}
