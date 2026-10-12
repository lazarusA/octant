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
        CjkFont::fallback_order(Language::Korean, Some(CjkFont::Jp)),
        [CjkFont::Kr, CjkFont::Sc, CjkFont::Jp]
    );
    assert_eq!(
        CjkFont::fallback_order(Language::Japanese, None),
        [CjkFont::Jp, CjkFont::Sc, CjkFont::Kr]
    );
    assert_eq!(
        CjkFont::fallback_order(Language::German, None),
        CjkFont::ALL
    );
    assert_eq!(
        CjkFont::fallback_order(Language::German, Some(CjkFont::Kr)),
        [CjkFont::Kr, CjkFont::Sc, CjkFont::Jp]
    );
    for &lang in Language::ALL {
        let font = CjkFont::for_language(lang);
        assert_eq!(font.map(CjkFont::language), font.map(|_| lang));
    }
}

#[test]
fn dataset_text_scripts_are_detected() {
    use super::Scripts;
    let scripts = |text: &str| {
        let mut s = Scripts::default();
        s.add(text);
        s
    };
    assert!(!scripts("precipitation (mm/day)").any());
    assert_eq!(
        scripts("\u{964D}\u{6C34}\u{91CF}"),
        Scripts {
            han: true,
            ..Default::default()
        }
    );
    let japanese = scripts("\u{6C17}\u{6E29}\u{30C7}\u{30FC}\u{30BF}");
    assert!(japanese.han && japanese.kana && !japanese.hangul);
    assert!(scripts("\u{AC15}\u{C218}\u{B7C9}").hangul);

    let mut meta = crate::data::DatasetMetadata {
        name: "store".into(),
        store_type: "zarr".into(),
        variables: Vec::new(),
        dimension_coordinates: Default::default(),
    };
    meta.dimension_coordinates.insert(
        "station".into(),
        crate::data::CoordValues::Labels(vec!["\u{6771}\u{4EAC}".to_string()].into()),
    );
    assert!(Scripts::of(&meta).han);
}

#[test]
fn han_text_uses_the_ui_then_system_then_dataset_font() {
    use super::Scripts;
    let han_only = Scripts {
        han: true,
        ..Default::default()
    };
    let with_kana = Scripts {
        kana: true,
        ..han_only
    };
    assert_eq!(
        han_only.han_font(Language::Korean, Some(Language::Japanese)),
        CjkFont::Kr
    );
    assert_eq!(
        han_only.han_font(Language::English, Some(Language::Japanese)),
        CjkFont::Jp
    );
    assert_eq!(with_kana.han_font(Language::English, None), CjkFont::Jp);
    assert_eq!(han_only.han_font(Language::German, None), CjkFont::Sc);
    // Kana needs the Japanese font even when Han text goes to Chinese.
    assert_eq!(with_kana.needed(CjkFont::Sc), [true, true, false]);
    // Han text in the Korean font also loads the Chinese one for what it lacks.
    assert_eq!(han_only.needed(CjkFont::Kr), [true, false, true]);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn system_fonts_draw_their_script_when_installed() {
    for font in CjkFont::ALL {
        assert!(!super::system::families(font).is_empty());
        // Only machines with a CJK font installed check one.
        let Some(data) = super::system::load(font) else {
            continue;
        };
        let face = skrifa::FontRef::from_index(&data.font, data.index).expect("system font parses");
        let sample = match font {
            CjkFont::Kr => '\u{D55C}',
            _ => '\u{4E2D}',
        };
        assert!(
            face.charmap().map(sample).is_some(),
            "{font:?} lacks {sample}"
        );
    }
}

#[test]
fn dataset_characters_go_to_the_font_that_draws_them() {
    let meta = crate::data::DatasetMetadata {
        name: "\u{6771}\u{4EAC}\u{306E}\u{6C17}\u{6E29} \u{AE30}\u{C628} caf\u{E9}\u{3002}".into(),
        ..Default::default()
    };
    let [sc, jp, kr] = super::dataset_text::DatasetText::of(&meta).by_font(CjkFont::Sc);
    // Han and CJK punctuation go to the Han font, kana and Hangul to theirs.
    assert_eq!(
        sc.into_iter().collect::<String>(),
        "\u{3002}\u{4EAC}\u{6771}\u{6C17}\u{6E29}"
    );
    assert_eq!(jp.into_iter().collect::<String>(), "\u{306E}");
    assert_eq!(kr.into_iter().collect::<String>(), "\u{AE30}\u{C628}");
}

#[test]
fn dataset_text_reads_new_labels_only_once() {
    use super::dataset_text::DatasetText;
    let mut meta = crate::data::DatasetMetadata {
        name: "plain".into(),
        ..Default::default()
    };
    let mut text = DatasetText::default();
    text.update(&meta, 1);
    assert!(!text.scripts.any());
    // Labels arriving later (same dataset) are read.
    meta.dimension_coordinates.insert(
        "station".into(),
        crate::data::CoordValues::Labels(vec!["\u{D55C}".to_string()].into()),
    );
    text.update(&meta, 1);
    assert!(text.scripts.hangul);
    // A new dataset starts over.
    let other = crate::data::DatasetMetadata::default();
    text.update(&other, 2);
    assert!(!text.scripts.any());
}

#[test]
fn fonts_loaded_off_frame_are_installed_by_the_next_frame() {
    use std::sync::atomic::Ordering;
    let ctx = egui::Context::default();
    super::fonts_arrived(&ctx);
    assert!(super::FONTS_ARRIVED.load(Ordering::Acquire));
    super::FontEngine::watch_dataset(&ctx, &crate::app::OctantApp::default());
    assert!(!super::FONTS_ARRIVED.load(Ordering::Acquire));
}
