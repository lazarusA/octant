//! The cache menu: memory use, what is cached per variable, and its
//! settings (`settings.rs`). At most 80 % of the window wide and tall
//! (`menu_width`, `scroll_menu`), scrolling past that.

mod settings;

use crate::{
    app::OctantApp,
    i18n::{CommonKey, DatasetKey, fill, tr},
    ui::icons::{Icon, IconSize, IconTone, ToolbarButton, UiIconExt},
    utils::stack_str,
};

/// Width of the cache menu where the window has room for it.
const MENU_WIDTH: f32 = 360.0;
/// Share of the window the cache menu may take at most.
const MAX_WINDOW_SHARE: f32 = 0.8;

/// A menu's contents, the menu growing with them up to [`MAX_WINDOW_SHARE`]
/// of the window's height, then scrolling. Call it before drawing anything
/// else in the menu: an egui menu offers only last frame's height, and making
/// room moves the layout back to the menu's top.
pub fn scroll_menu<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let max_height = ui.ctx().content_rect().height() * MAX_WINDOW_SHARE;
    ui.set_max_height(max_height);
    egui::ScrollArea::vertical()
        .max_height(max_height)
        .show(ui, add_contents)
        .inner
}

/// Width inside the cache menu: [`MENU_WIDTH`], the whole menu at most
/// [`MAX_WINDOW_SHARE`] of the window.
pub fn menu_width(ctx: &egui::Context) -> f32 {
    let style = ctx.global_style();
    // The popup frame's margins and border count toward the share.
    let frame = style.spacing.menu_margin.sum().x + 2.0 * style.visuals.window_stroke.width;
    MENU_WIDTH.min(ctx.content_rect().width() * MAX_WINDOW_SHARE - frame)
}

/// The cache button and its menu; the button's response.
pub fn show_cache_menu(app: &mut OctantApp, ui: &mut egui::Ui, compact: bool) -> egui::Response {
    let button_response = ui.add(
        ToolbarButton::new(Icon::Cache, tr(CommonKey::Cache))
            .compact(compact)
            .owns_popup(),
    );
    egui::Popup::menu(&button_response).show(|ui| {
        ui.set_width(menu_width(ui.ctx()));
        scroll_menu(ui, |ui| {
            ui.horizontal(|ui| {
                ui.icon(Icon::Cache, IconSize::Sm);
                ui.label(egui::RichText::new(tr(DatasetKey::CacheMenuTitle)).strong());
            });
            ui.separator();
            show_cache_contents(app, ui);
        });
    });
    button_response
}

/// Bytes in a MB, as the menu counts them.
const MB: f64 = 1024.0 * 1024.0;

/// The cache menu below its title: memory use, what is cached per variable,
/// and the prefetch and capacity settings. Its caller sets the width
/// ([`menu_width`]).
pub fn show_cache_contents(app: &mut OctantApp, ui: &mut egui::Ui) {
    let block_bytes = app.block_cache.current_bytes();
    let pyramid_bytes = app
        .layers
        .base
        .data
        .pyramid
        .as_ref()
        .map(|p| p.bytes_size())
        .unwrap_or(0);
    show_usage(app, ui, block_bytes + pyramid_bytes);
    ui.add_space(4.0);
    show_breakdown(app, ui, block_bytes, pyramid_bytes);
    ui.add_space(4.0);
    show_activity(app, ui);
    ui.separator();
    show_variables(app, ui);
    ui.separator();
    settings::show_settings(app, ui);
}

/// `value` with `decimals` places, in the language's number format.
fn num(buf: &mut [u8; 16], value: f64, decimals: usize) -> &str {
    let value = crate::i18n::number::Num(format_args!("{value:.decimals$}"));
    stack_str(buf, format_args!("{value}"))
}

/// How full the cache is, against its limit.
fn show_usage(app: &OctantApp, ui: &mut egui::Ui, total_bytes: usize) {
    let max_bytes = app.block_cache.max_bytes();
    let fraction = (total_bytes as f32 / max_bytes.max(1) as f32).clamp(0.0, 1.0);
    let (mut a, mut b, mut buf) = ([0u8; 16], [0u8; 16], [0u8; 128]);
    ui.label(egui::RichText::new(tr(DatasetKey::GlobalMemoryUsage)).small());
    let total = num(&mut a, total_bytes as f64 / MB, 2);
    let percent = num(&mut b, f64::from(fraction) * 100.0, 1);
    let usage = fill(
        &mut buf,
        tr(DatasetKey::CacheUsagePercent),
        &[&total, &app.max_cache_mb, &percent],
    );
    ui.add(egui::ProgressBar::new(fraction).text(usage));
}

/// The memory held by blocks and by the multiresolution pyramid.
fn show_breakdown(app: &OctantApp, ui: &mut egui::Ui, block_bytes: usize, pyramid_bytes: usize) {
    let (mut a, mut buf) = ([0u8; 16], [0u8; 128]);
    ui.label(
        egui::RichText::new(tr(DatasetKey::MemoryBreakdown))
            .strong()
            .small(),
    );
    ui.horizontal(|ui| {
        ui.icon(Icon::Bullet, IconSize::Xs);
        let size = num(&mut a, block_bytes as f64 / MB, 2);
        let count = app.block_cache.cached_count();
        ui.small(fill(
            &mut buf,
            tr(DatasetKey::ResidentHyperslabs),
            &[&size, &count],
        ));
    });
    if pyramid_bytes > 0 {
        ui.horizontal(|ui| {
            ui.icon(Icon::Bullet, IconSize::Xs);
            let size = num(&mut a, pyramid_bytes as f64 / MB, 2);
            ui.small(fill(&mut buf, tr(DatasetKey::MultiResPyramid), &[&size]));
        });
    }
}

/// Cache hits and misses, and the prefetches in flight.
fn show_activity(app: &OctantApp, ui: &mut egui::Ui) {
    let (mut a, mut buf) = ([0u8; 16], [0u8; 128]);
    let rate = num(&mut a, f64::from(app.block_cache.hit_rate()), 1);
    let (hits, misses) = (app.block_cache.hits(), app.block_cache.misses());
    ui.small(fill(
        &mut buf,
        tr(DatasetKey::CacheHitsMisses),
        &[&hits, &misses, &rate],
    ));

    let pending = app.block_prefetcher.pending_count();
    ui.horizontal(|ui| {
        if pending > 0 {
            ui.icon_toned(Icon::Bolt, IconSize::Xs, IconTone::Success);
            ui.small(fill(
                &mut buf,
                tr(DatasetKey::PrefetchInFlight),
                &[&pending],
            ));
        } else {
            ui.icon_toned(Icon::Check, IconSize::Xs, IconTone::Muted);
            ui.small(tr(DatasetKey::AllBlocksCached));
        }
    });
}

/// What each variable holds in the cache, each with a button clearing it.
fn show_variables(app: &mut OctantApp, ui: &mut egui::Ui) {
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
        return;
    }
    let (mut a, mut buf) = ([0u8; 16], [0u8; 128]);
    let mut var_to_clear: Option<(String, String)> = None;
    for s in &summaries {
        ui.push_id(("var_cache_row", &s.source_id, &s.variable_name), |ui| {
            // Right to left, so the name takes what is left and is cut to
            // fit (whole on hover) instead of widening the menu.
            let row = egui::Layout::right_to_left(egui::Align::Center);
            ui.horizontal(|ui| {
                ui.with_layout(row, |ui| {
                    if ui
                        .icon_button(Icon::Trash, "")
                        .on_hover_text(tr(DatasetKey::ClearVariableCacheHover))
                        .clicked()
                    {
                        var_to_clear = Some((s.source_id.clone(), s.variable_name.clone()));
                    }
                    let size = num(&mut a, s.bytes as f64 / MB, 1);
                    ui.small(fill(
                        &mut buf,
                        tr(DatasetKey::VariableCacheSize),
                        &[&size, &s.block_count],
                    ));
                    let name = egui::RichText::new(&s.variable_name)
                        .monospace()
                        .color(ui.visuals().strong_text_color());
                    ui.add(egui::Label::new(name).truncate());
                });
            });
        });
    }
    if let Some((source_id, var_name)) = var_to_clear {
        app.block_cache.clear_variable(&source_id, &var_name);
    }
}
