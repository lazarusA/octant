use super::card::render_entry_card;
use crate::app::{OctantApp, StoreKind};
use crate::catalog::get_catalog_entries;
use crate::ui::icons::{Icon, UiIconExt};

pub fn render_entries_list(
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
