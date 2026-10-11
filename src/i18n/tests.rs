//! Unit tests for the Octant internationalization (i18n) subsystem.

use super::*;
use std::collections::HashSet;

const ALL_KEYS: &[Key] = &[
    Key::Dataset,
    Key::Variables,
    Key::Dimensions,
    Key::Plot,
    Key::Colormaps,
    Key::Settings,
    Key::About,
    Key::InspectMetadata,
    Key::ActiveDatasets,
    Key::NoActiveDatasets,
    Key::CacheUsage,
    Key::ClearCache,
    Key::OpenLocal,
    Key::CloudStore,
    Key::SampleDatasets,
    Key::DropFilesHere,
    Key::SourceUrlOrPath,
    Key::Inspect,
    Key::LoadingMetadata,
    Key::Group,
    Key::DataType,
    Key::Shape,
    Key::Description,
    Key::SearchVariables,
    Key::NoVariablesMatching,
    Key::ShowingItemsOfTotal,
    Key::Ready,
    Key::Loading,
    Key::Close,
    Key::Cancel,
    Key::Save,
    Key::Reset,
    Key::Error,
    Key::Play,
    Key::Pause,
    Key::StepForward,
    Key::StepBackward,
    Key::Speed,
    Key::Loop,
    Key::AboutSubtitle,
    Key::AboutDescription,
    Key::AboutHyperslabTitle,
    Key::AboutHyperslabDesc,
    Key::AboutStorageTitle,
    Key::AboutStorageDesc,
    Key::AboutShadersTitle,
    Key::AboutShadersDesc,
    Key::AboutLinksHeading,
    Key::AboutGoodFaithNotice,
    Key::AboutBrowseIcons,
];

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
fn test_all_phase1_languages_translate_every_key() {
    for &lang in Language::ALL_PHASE1 {
        for &key in ALL_KEYS {
            let translated = translate(lang, key);
            assert!(
                !translated.is_empty(),
                "Translation for {:?} in {:?} is empty",
                key,
                lang
            );
        }
    }
}

#[test]
fn test_cjk_stubs_fall_back_to_english() {
    for &lang in &[Language::ChineseSimp, Language::Japanese, Language::Korean] {
        for &key in ALL_KEYS {
            let translated = translate(lang, key);
            let english = translate(Language::English, key);
            assert_eq!(
                translated, english,
                "CJK stub for {:?} in {:?} should fall back to English",
                key, lang
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
