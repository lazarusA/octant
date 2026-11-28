//! A card's details, shown while it is hovered or holds keyboard focus, and
//! copied as text by its button: the whole description, who to credit, the
//! license and citation, the store, the provider's and the gallery's pages.

use crate::gallery::GalleryEntry;
use crate::i18n::{DatasetKey, tr};
use crate::ui::icons::IconTone;
use crate::ui::keyboard::focus;
use egui::{Response, RichText, Ui};
use std::fmt::Write;

/// `response` with the details of `entry` as its tooltip, on hover and
/// under keyboard focus (one tooltip: `tooltip_ui` attaches both).
pub fn attach(response: Response, entry: &GalleryEntry) -> Response {
    focus::tooltip_ui(response, |ui| show(ui, entry))
}

/// The details as plain text: title, description, then a line per field.
pub fn text(entry: &GalleryEntry) -> String {
    let mut out = format!("{}\n", entry.label);
    if !entry.subtitle.is_empty() {
        let _ = writeln!(out, "\n{}", entry.subtitle);
    }
    out.push('\n');
    let page = entry.gallery_page();
    for (name, value) in fields(entry, page.as_deref()) {
        let _ = match name {
            "" => writeln!(out, "{value}"),
            name => writeln!(out, "{name}: {value}"),
        };
    }
    out
}

/// The fields after the description, as (name, value); the credit and the
/// license's address have no name. Empty ones are left out.
fn fields<'a>(
    entry: &'a GalleryEntry,
    gallery_page: Option<&'a str>,
) -> impl Iterator<Item = (&'static str, &'a str)> {
    [
        ("", Some(entry.credit.as_str())),
        (tr(DatasetKey::GalleryLicense), Some(entry.license.as_str())),
        ("", entry.license_url.as_deref()),
        (tr(DatasetKey::GalleryCitation), entry.citation.as_deref()),
        (tr(DatasetKey::GalleryStore), Some(entry.store.trim())),
        (
            tr(DatasetKey::GallerySourcePage),
            entry.source_page.as_deref(),
        ),
        (tr(DatasetKey::GalleryPage), gallery_page),
    ]
    .into_iter()
    .filter_map(|(name, value)| Some((name, value.filter(|v| !v.is_empty())?)))
}

fn show(ui: &mut Ui, entry: &GalleryEntry) {
    ui.set_max_width(360.0);
    ui.label(RichText::new(&entry.label).strong());
    if !entry.subtitle.is_empty() {
        ui.label(&entry.subtitle);
    }
    ui.add_space(4.0);
    let page = entry.gallery_page();
    for (name, value) in fields(entry, page.as_deref()) {
        ui.horizontal_wrapped(|ui| {
            if !name.is_empty() {
                ui.label(RichText::new(name).small().strong());
            }
            ui.label(RichText::new(value).small());
        });
    }
    if cfg!(target_arch = "wasm32") && entry.desktop_only {
        let color = IconTone::Warning.color(ui.visuals());
        let hint = RichText::new(tr(DatasetKey::DesktopOnlyHint)).small();
        ui.label(hint.color(color));
    }
}

#[cfg(test)]
mod tests {
    /// The copied details hold every field the gallery gives, by name.
    #[test]
    fn the_details_copy_as_text() {
        crate::gallery::remote::install_fixture();
        let gallery = crate::gallery::current();
        let Some(seasfire) = gallery.entries().iter().find(|e| e.key == "seasfire") else {
            panic!("fixture");
        };
        let text = super::text(seasfire);
        assert!(
            text.starts_with("SeasFire Cube\n\nA global dataset"),
            "{text}"
        );
        for line in [
            "SeasFire project",
            "License: CC-BY-4.0",
            "https://creativecommons.org/licenses/by/4.0/",
            "Store: https://s3.bgc-jena.mpg.de:9000/misc/seasfire_rechunked.zarr",
            "Dataset page: https://seasfire.hua.gr/",
            "Gallery page: https://ndvis.github.io/octant-docs/gallery/seasfire",
        ] {
            assert!(text.lines().any(|l| l == line), "{line}\n{text}");
        }
    }
}
