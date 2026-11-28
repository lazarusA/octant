use super::card::{GAP, card, grid};
use crate::app::OctantApp;
use crate::data::coordinates::naming::contains_ascii_case_insensitive as contains;
use crate::gallery::{Gallery, GalleryEntry, all_entries, shown};
use crate::i18n::{CommonKey, DatasetKey, tr};
use crate::ui::icons::{Icon, IconSize, IconTone, UiIconExt};
use crate::ui::keyboard::list_nav::{Above, ListNav};

/// The gallery's state, then a grid of the matching entries' cards, under
/// the search field `above`.
pub fn render_entries_list(
    app: &mut OctantApp,
    ui: &mut egui::Ui,
    gallery: &Gallery,
    is_mobile: bool,
    should_close: &mut bool,
    above: Above,
) {
    gallery_status(ui, gallery);
    let query = app.layout.gallery_search_query.trim();
    let (domain, format) = (app.layout.gallery_domain, app.layout.gallery_format);
    let is_match = |e: &&GalleryEntry| -> bool {
        shown(e, domain, format)
            && (query.is_empty()
                || contains(&e.key, query)
                || contains(&e.label, query)
                || contains(&e.subtitle, query)
                || contains(&e.store, query)
                || e.tags.iter().any(|t| contains(t, query)))
    };

    if !all_entries(gallery).any(|e| is_match(&e)) {
        no_matches(app, ui);
        return;
    }

    let mut nav = ListNav::new(ui.ctx(), Some(above));
    let to_open = cards(
        ui,
        all_entries(gallery).filter(is_match),
        is_mobile,
        &mut nav,
    );
    nav.finish(ui.ctx());

    if let Some(entry) = to_open
        && app.open_gallery_entry(entry)
    {
        *should_close = true;
    }
}

/// The `entries`' cards in rows of the grid's columns; the one clicked.
fn cards<'a>(
    ui: &mut egui::Ui,
    entries: impl Iterator<Item = &'a GalleryEntry>,
    is_mobile: bool,
    nav: &mut ListNav,
) -> Option<&'a GalleryEntry> {
    let mut to_open = None;
    let mut entries = entries.peekable();
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // Clear of the scroll bar, which floats over the content.
            let scroll = &ui.spacing().scroll;
            let room = ui.available_width() - scroll.bar_width - scroll.bar_inner_margin;
            let (columns, width) = grid(room, is_mobile);
            ui.spacing_mut().item_spacing.y = GAP;
            // One row of `columns` cards at a time, so every row lines up.
            while entries.peek().is_some() {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = GAP;
                    for entry in entries.by_ref().take(columns) {
                        let response = card(ui, entry, width);
                        nav.item(&response);
                        if response.clicked() {
                            to_open = Some(entry);
                        }
                    }
                });
            }
        });
    to_open
}

/// Nothing matches: say so, with a button clearing the search and chips.
fn no_matches(app: &mut OctantApp, ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(32.0);
        ui.icon_toned(Icon::Info, IconSize::Md, IconTone::Muted);
        ui.add_space(6.0);
        ui.label(egui::RichText::new(tr(DatasetKey::NoGalleryMatches)).strong());
        ui.add_space(8.0);
        if ui.button(tr(DatasetKey::ClearSearch)).clicked() {
            app.layout.gallery_search_query.clear();
            app.layout.gallery_domain = None;
            app.layout.gallery_format = None;
        }
    });
}

/// While the gallery loads, a spinner; after a failure (offline, say), why,
/// and a button that fetches it again, with the thumbnails that failed.
fn gallery_status(ui: &mut egui::Ui, gallery: &Gallery) {
    match gallery {
        Gallery::Ready(_) => return,
        Gallery::Loading => {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(egui::RichText::new(tr(CommonKey::Loading)).small());
            });
        }
        Gallery::Failed(why) => {
            ui.horizontal_wrapped(|ui| {
                ui.icon_toned(Icon::Warning, IconSize::Sm, IconTone::Warning);
                ui.label(egui::RichText::new(tr(DatasetKey::GalleryUnavailable)).small());
                let retry = egui::Button::new(tr(DatasetKey::Reload));
                let clicked = ui.push_id("gallery_retry", |ui| ui.add(retry)).inner;
                if clicked.on_hover_text(&**why).clicked() {
                    crate::gallery::remote::retry(ui.ctx());
                    super::thumbs::forget_failed(ui.ctx());
                }
            });
        }
    }
    ui.add_space(4.0);
}
