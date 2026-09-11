use crate::app::{OctantApp, StoreKind};
use crate::catalog::{
    CatalogCategoryFilter, ICECHUNK_CATALOG, PROCEDURAL_CATALOG, ZARR_CATALOG, get_catalog_entries,
};
use crate::ui::icons::{Icon, UiIconExt};

/// Render the responsive, centered Dataset Catalog modal dialog.
pub fn show_catalog_window(app: &mut OctantApp, ctx: &egui::Context) {
    if !app.show_catalog_window {
        return;
    }

    let screen_rect = ctx.viewport_rect();
    let is_mobile = screen_rect.width() < 520.0;

    let modal_w = (screen_rect.width() - 24.0).clamp(280.0, 780.0);
    let modal_h = (screen_rect.height() - 36.0).clamp(320.0, 680.0);
    let modal_rect =
        egui::Rect::from_center_size(screen_rect.center(), egui::vec2(modal_w, modal_h));

    let mut should_close = ctx.input(|i| i.key_pressed(egui::Key::Escape));

    // 1. Scrim / Backdrop overlay: dims background and dismisses modal on click outside
    let backdrop_id = egui::Id::new("catalog_modal_backdrop");
    egui::Area::new(backdrop_id)
        .fixed_pos(screen_rect.min)
        .order(egui::Order::Middle)
        .show(ctx, |ui| {
            let (resp, painter) = ui.allocate_painter(screen_rect.size(), egui::Sense::click());
            let dim_color = if ui.visuals().dark_mode {
                egui::Color32::from_black_alpha(150)
            } else {
                egui::Color32::from_black_alpha(70)
            };
            painter.rect_filled(screen_rect, 0.0, dim_color);
            if resp.clicked() {
                should_close = true;
            }
        });

    // 2. Centered Modal Dialog Card
    let modal_id = egui::Id::new("catalog_modal_dialog");
    egui::Area::new(modal_id)
        .fixed_pos(modal_rect.min)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            ui.set_width(modal_w);
            ui.set_height(modal_h);

            egui::Frame::window(ui.style())
                .inner_margin(egui::Margin::symmetric(14, 12))
                .corner_radius(8.0)
                .show(ui, |ui| {
                    let total_count =
                        ZARR_CATALOG.len() + ICECHUNK_CATALOG.len() + PROCEDURAL_CATALOG.len();

                    render_header(ui, total_count, &mut should_close);
                    ui.add_space(8.0);

                    render_search_and_filters(app, ui, is_mobile, total_count);
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(4.0);

                    render_entries_list(app, ui, is_mobile, &mut should_close);
                });
        });

    if should_close {
        app.show_catalog_window = false;
    }
}

fn render_header(ui: &mut egui::Ui, total_count: usize, should_close: &mut bool) {
    ui.horizontal(|ui| {
        ui.icon_colored(Icon::Catalog, 16.0, ui.visuals().strong_text_color());
        ui.heading("Dataset Catalog");

        let mut badge_buf = [0u8; 32];
        let badge_text = format_count(&mut badge_buf, total_count, " stores");
        ui.label(
            egui::RichText::new(badge_text)
                .small()
                .color(ui.visuals().weak_text_color()),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .icon_button(Icon::Cross, "")
                .on_hover_text("Close (Esc)")
                .clicked()
            {
                *should_close = true;
            }
        });
    });

    ui.add(
        egui::Label::new(
            egui::RichText::new(
                "Curated cloud Zarr, Icechunk, and procedural ground-truth datasets. Select any dataset to load.",
            )
            .small()
            .italics(),
        )
        .wrap_mode(egui::TextWrapMode::Wrap),
    );
}

fn render_search_and_filters(
    app: &mut OctantApp,
    ui: &mut egui::Ui,
    is_mobile: bool,
    total_count: usize,
) {
    let zarr_count = ZARR_CATALOG.len();
    let icechunk_count = ICECHUNK_CATALOG.len();
    let procedural_count = PROCEDURAL_CATALOG.len();

    // Search input row
    ui.horizontal(|ui| {
        ui.icon(Icon::Search, 13.0);
        let search_has_text = !app.catalog_search_query.is_empty();
        let right_pad = if search_has_text { 28.0 } else { 0.0 };
        let search_w = (ui.available_width() - right_pad).max(80.0);

        ui.add(
            egui::TextEdit::singleline(&mut app.catalog_search_query)
                .hint_text("Filter by name, description, or URL...")
                .desired_width(search_w),
        );

        if search_has_text
            && ui
                .icon_button(Icon::Cross, "")
                .on_hover_text("Clear search")
                .clicked()
        {
            app.catalog_search_query.clear();
        }
    });

    ui.add_space(4.0);

    // Filter categories row with stack-formatted labels
    let spacing = if is_mobile {
        egui::vec2(4.0, 4.0)
    } else {
        egui::vec2(6.0, 4.0)
    };
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = spacing;
        ui.label(egui::RichText::new("Category:").strong().small());

        let mut buf_all = [0u8; 32];
        let mut buf_zarr = [0u8; 32];
        let mut buf_ice = [0u8; 32];
        let mut buf_proc = [0u8; 32];

        ui.selectable_value(
            &mut app.catalog_category_filter,
            CatalogCategoryFilter::All,
            format_tab(&mut buf_all, "All", total_count),
        );
        ui.selectable_value(
            &mut app.catalog_category_filter,
            CatalogCategoryFilter::Zarr,
            format_tab(&mut buf_zarr, "Zarr", zarr_count),
        );
        ui.selectable_value(
            &mut app.catalog_category_filter,
            CatalogCategoryFilter::Icechunk,
            format_tab(&mut buf_ice, "Icechunk", icechunk_count),
        );
        ui.selectable_value(
            &mut app.catalog_category_filter,
            CatalogCategoryFilter::Procedural,
            format_tab(&mut buf_proc, "Procedural", procedural_count),
        );
    });
}

fn render_entries_list(
    app: &mut OctantApp,
    ui: &mut egui::Ui,
    is_mobile: bool,
    should_close: &mut bool,
) {
    let query = app.catalog_search_query.trim();
    let entries = get_catalog_entries(app.catalog_category_filter);

    let is_match = |e: &&crate::catalog::CatalogEntry| -> bool {
        if query.is_empty() {
            return true;
        }
        contains_ignore_ascii_case(e.key, query)
            || contains_ignore_ascii_case(e.label, query)
            || contains_ignore_ascii_case(e.subtitle, query)
            || contains_ignore_ascii_case(e.store, query)
    };

    let mut match_count = 0;
    for _ in entries.iter().copied().filter(is_match) {
        match_count += 1;
    }

    if match_count == 0 {
        ui.vertical_centered(|ui| {
            ui.add_space(32.0);
            ui.icon_colored(Icon::Info, 18.0, ui.visuals().weak_text_color());
            ui.add_space(6.0);
            ui.label(egui::RichText::new("No datasets found matching your search.").strong());
            ui.add_space(8.0);
            if ui.button("Clear Search").clicked() {
                app.catalog_search_query.clear();
            }
        });
        return;
    }

    let mut to_load: Option<(&str, StoreKind)> = None;

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for entry in entries.iter().copied().filter(is_match) {
                let trimmed_url = entry.store.trim();
                if render_entry_card(ui, entry, trimmed_url, is_mobile) {
                    to_load = Some((trimmed_url, entry.store_kind));
                }
                ui.add_space(6.0);
            }
        });

    if let Some((url, kind)) = to_load {
        app.submit_or_activate_source(url, Some(kind));
        *should_close = true;
    }
}

fn render_entry_card(
    ui: &mut egui::Ui,
    entry: &crate::catalog::CatalogEntry,
    trimmed_url: &str,
    is_mobile: bool,
) -> bool {
    let (badge_icon, badge_bracket, badge_color) = match entry.store_kind {
        StoreKind::RemoteZarr => (Icon::Globe, "[Zarr]", ui.visuals().selection.bg_fill),
        StoreKind::RemoteIcechunk => (
            Icon::Icechunk,
            "[Icechunk]",
            ui.visuals().widgets.active.bg_fill,
        ),
        StoreKind::ProceduralVolume4D => (
            Icon::PlotVolume,
            "[4D Volume]",
            ui.visuals().widgets.hovered.bg_fill,
        ),
        StoreKind::ProceduralRandom => (
            Icon::PlotPlane,
            "[2D Matrix]",
            ui.visuals().widgets.hovered.bg_fill,
        ),
        _ => (
            Icon::Folder,
            "[Store]",
            ui.visuals().widgets.noninteractive.fg_stroke.color,
        ),
    };

    let mut clicked_load = false;

    egui::Frame::default()
        .fill(ui.visuals().faint_bg_color)
        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
        .corner_radius(6.0)
        .inner_margin(egui::Margin::symmetric(10, 8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());

            if is_mobile {
                // Mobile layout: Stacked
                ui.horizontal(|ui| {
                    ui.icon_colored(badge_icon, 13.0, badge_color);
                    ui.label(
                        egui::RichText::new(badge_bracket)
                            .strong()
                            .small()
                            .color(badge_color),
                    );
                    ui.label(egui::RichText::new(entry.label).strong().size(13.5));
                });

                if !entry.subtitle.is_empty() {
                    ui.label(egui::RichText::new(entry.subtitle).small().italics());
                }

                ui.add_space(2.0);
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(trimmed_url)
                            .small()
                            .monospace()
                            .color(ui.visuals().hyperlink_color),
                    )
                    .wrap_mode(egui::TextWrapMode::Wrap),
                );

                ui.add_space(4.0);
                if ui
                    .icon_button(Icon::DropTray, "Select & Load Dataset")
                    .clicked()
                {
                    clicked_load = true;
                }
            } else {
                // Desktop layout: Side-by-side header with right-aligned button
                ui.horizontal(|ui| {
                    ui.icon_colored(badge_icon, 13.0, badge_color);
                    ui.label(
                        egui::RichText::new(badge_bracket)
                            .strong()
                            .small()
                            .color(badge_color),
                    );
                    ui.label(egui::RichText::new(entry.label).strong().size(14.0));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.icon_button(Icon::DropTray, "Select & Load").clicked() {
                            clicked_load = true;
                        }
                    });
                });

                if !entry.subtitle.is_empty() {
                    ui.label(egui::RichText::new(entry.subtitle).small().italics());
                }

                ui.add_space(2.0);
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(trimmed_url)
                            .small()
                            .monospace()
                            .color(ui.visuals().hyperlink_color),
                    )
                    .wrap_mode(egui::TextWrapMode::Wrap),
                );
            }
        });

    clicked_load
}

fn format_count<'a>(buf: &'a mut [u8; 32], count: usize, suffix: &str) -> &'a str {
    use std::io::Write;
    let mut cursor = std::io::Cursor::new(&mut buf[..]);
    let _ = write!(cursor, "{}{}", count, suffix);
    let len = cursor.position() as usize;
    std::str::from_utf8(&buf[..len]).unwrap_or("")
}

fn format_tab<'a>(buf: &'a mut [u8; 32], label: &str, count: usize) -> &'a str {
    use std::io::Write;
    let mut cursor = std::io::Cursor::new(&mut buf[..]);
    let _ = write!(cursor, "{} ({})", label, count);
    let len = cursor.position() as usize;
    std::str::from_utf8(&buf[..len]).unwrap_or("")
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
