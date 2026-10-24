use super::card::render_entry_card;
use crate::app::OctantApp;
use crate::catalog::get_catalog_entries;
use crate::i18n::{DatasetKey, tr};
use crate::ui::icons::{Icon, IconSize, IconTone, UiIconExt};
use crate::ui::keyboard::list_nav::{Above, ListNav};

/// The matching entries' cards, under the search field `above`.
pub fn render_entries_list(
    app: &mut OctantApp,
    ui: &mut egui::Ui,
    is_mobile: bool,
    should_close: &mut bool,
    above: Above,
) {
    let query = app.layout.catalog_search_query.trim();
    let entries = get_catalog_entries(app.layout.catalog_category_filter);

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
            ui.icon_toned(Icon::Info, IconSize::Md, IconTone::Muted);
            ui.add_space(6.0);
            ui.label(egui::RichText::new(tr(DatasetKey::NoCatalogMatches)).strong());
            ui.add_space(8.0);
            if ui.button(tr(DatasetKey::ClearSearch)).clicked() {
                app.layout.catalog_search_query.clear();
            }
        });
        return;
    }

    let mut to_open = None;

    let mut nav = ListNav::new(ui.ctx(), Some(above));
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for entry in entries.iter().copied().filter(is_match) {
                let trimmed_url = entry.store.trim();
                let load = render_entry_card(ui, entry, trimmed_url, is_mobile);
                nav.item(&load);
                if load.clicked() {
                    to_open = Some(entry);
                }
                ui.add_space(6.0);
            }
        });
    nav.finish(ui.ctx());

    if let Some(entry) = to_open
        && app.open_catalog_entry(entry)
    {
        *should_close = true;
    }
}

#[inline]
pub fn contains_ignore_ascii_case(haystack: &str, needle: &str) -> bool {
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
