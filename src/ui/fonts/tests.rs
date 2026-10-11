//! The bundled CJK subsets cover every character their translations use; a
//! failure means `cargo run --features font-tools --bin subset_fonts` must be
//! run again after a translation change.

use super::CjkFont;
use crate::i18n::Language;
use skrifa::MetadataProvider;

fn read(file: &str) -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets/fonts")
        .join(file);
    std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// Characters of `chars` that `font` has no outline for.
fn missing(font: &[u8], chars: impl IntoIterator<Item = char>) -> String {
    let font = skrifa::FontRef::new(font).expect("bundled font parses");
    let charmap = font.charmap();
    let outlines = font.outline_glyphs();
    chars
        .into_iter()
        .filter(|&c| charmap.map(c).and_then(|g| outlines.get(g)).is_none())
        .collect()
}

#[test]
fn ui_subsets_cover_their_translations() {
    for font in CjkFont::ALL {
        let gaps = missing(&read(font.ui_file()), font.required_chars());
        assert!(
            gaps.is_empty(),
            "{} lacks '{gaps}': run `cargo run --features font-tools --bin subset_fonts`",
            font.ui_file()
        );
    }
}

#[test]
fn name_subsets_cover_their_language_names() {
    for font in CjkFont::ALL {
        let name = font.language().display_name();
        let gaps = missing(&read(font.names_file()), name.chars());
        assert!(gaps.is_empty(), "{} lacks '{gaps}'", font.names_file());
    }
}

#[test]
fn bundled_desktop_fonts_are_the_ui_subsets() {
    for font in CjkFont::ALL {
        assert_eq!(super::store::bundled(font), read(font.ui_file()).as_slice());
    }
}

#[test]
fn the_ui_language_font_comes_first() {
    assert_eq!(
        CjkFont::fallback_order(Language::Korean),
        [CjkFont::Kr, CjkFont::Sc, CjkFont::Jp]
    );
    assert_eq!(
        CjkFont::fallback_order(Language::Japanese),
        [CjkFont::Jp, CjkFont::Sc, CjkFont::Kr]
    );
    assert_eq!(CjkFont::fallback_order(Language::German), CjkFont::ALL);
    for &lang in Language::ALL {
        let font = CjkFont::for_language(lang);
        assert_eq!(font.map(CjkFont::language), font.map(|_| lang));
    }
}
