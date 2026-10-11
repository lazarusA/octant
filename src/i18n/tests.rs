//! Unit tests for the Octant internationalization (i18n) subsystem.

use super::template::PLACEHOLDER;
use super::*;
use std::collections::HashSet;

#[test]
fn test_languages_unique_codes_and_names() {
    let mut codes = HashSet::new();
    let mut names = HashSet::new();

    for &lang in Language::ALL {
        let code = lang.code();
        let name = lang.display_name();

        assert!(!code.is_empty(), "Language code cannot be empty");
        assert!(!name.is_empty(), "Language name cannot be empty");
        assert!(codes.insert(code), "Duplicate language code: {code}");
        assert!(names.insert(name), "Duplicate language name: {name}");
    }
}

#[test]
fn test_language_index_roundtrip() {
    for &lang in Language::ALL {
        assert_eq!(Language::from_index(lang as u8), lang);
    }
    assert_eq!(Language::from_index(u8::MAX), Language::English);
}

#[test]
fn test_all_phase1_languages_translate_every_key() {
    for &lang in Language::ALL_PHASE1 {
        for key in Key::all() {
            let translated = translations::exact(lang, key);
            assert!(
                translated.is_some_and(|t| !t.is_empty()),
                "{key:?} has no {lang:?} translation"
            );
        }
    }
}

#[test]
fn test_templates_keep_their_placeholders() {
    for key in Key::all() {
        let expected = translate(Language::English, key)
            .matches(PLACEHOLDER)
            .count();
        for &lang in Language::ALL {
            let found = translate(lang, key).matches(PLACEHOLDER).count();
            assert_eq!(
                found, expected,
                "{key:?} in {lang:?} has {found} placeholders, English has {expected}"
            );
        }
    }
}

#[test]
fn test_overlay_limit_message_names_the_limit() {
    let limit = crate::app::overlays::MAX_OVERLAYS.to_string();
    for &lang in Language::ALL {
        let text = translate(lang, PlotKey::OverlayStackFull);
        assert!(
            text.contains(&limit),
            "{lang:?}: '{text}' does not name {limit}"
        );
    }
}

#[test]
fn test_fill_places_values_in_order() {
    let mut buf = [0; 64];
    assert_eq!(
        fill(&mut buf, "Octant {} ({})", &[&"VII", &"-, -, -"]),
        "Octant VII (-, -, -)"
    );
    let mut buf = [0; 64];
    assert_eq!(fill(&mut buf, "'{}' {}", &[&42]), "'42' ");
    let mut buf = [0; 8];
    assert_eq!(fill(&mut buf, "no values", &[]), "no value");
}

#[test]
fn test_cjk_stubs_fall_back_to_english() {
    for &lang in &[Language::ChineseSimp, Language::Japanese, Language::Korean] {
        for key in Key::all() {
            let translated = translate(lang, key);
            let english = translate(Language::English, key);
            assert_eq!(
                translated, english,
                "CJK stub for {key:?} in {lang:?} should fall back to English"
            );
        }
    }
}

#[test]
fn test_language_serde_roundtrip() {
    for &lang in Language::ALL {
        let json = serde_json::to_string(&lang).expect("serialize language");
        let decoded: Language = serde_json::from_str(&json).expect("deserialize language");
        assert_eq!(lang, decoded);
    }
}

#[test]
fn test_translations_are_plain_text() {
    // Letters of any script, digits, ASCII punctuation, Spanish opening marks,
    // spaces, line breaks and the degree sign only: no typographic quotes, ellipsis characters,
    // arrows or emoji (AGENTS.md).
    for &lang in Language::ALL {
        for key in Key::all() {
            let text = translate(lang, key);
            let odd = text.chars().find(|&c| {
                !(c.is_alphanumeric()
                    || matches!(c, ' ' | '\n' | '°' | '¿' | '¡')
                    || c.is_ascii_punctuation())
            });
            assert!(odd.is_none(), "{key:?} in {lang:?} has {odd:?}: '{text}'");
        }
    }
}
