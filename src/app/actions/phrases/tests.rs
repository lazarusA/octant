//! Utterances in every language read into actions over the in-memory
//! dataset's names.

use super::{Parsed, Verb, parse, table};
use crate::app::actions::AppAction;
use crate::app::actions::resolve::{Resolution, Vocabulary};
use crate::app::test_support::memory_app;
use crate::i18n::{Language, PlotKey, translate};
use crate::plots::PlotType;

const VERBS: [Verb; 13] = [
    Verb::Open,
    Verb::Goto,
    Verb::Describe,
    Verb::Search,
    Verb::List,
    Verb::Select,
    Verb::Plot,
    Verb::Play,
    Verb::Pause,
    Verb::Next,
    Verb::Previous,
    Verb::Colormap,
    Verb::ResetView,
];

pub(super) fn vocab() -> Vocabulary {
    let (_, meta) = memory_app();
    Vocabulary::new(Some(&meta), &[])
}

/// `lang`'s first phrasing of `verb`.
pub(super) fn say(lang: Language, verb: Verb) -> &'static str {
    let (_, says) = table(lang)
        .verbs
        .iter()
        .find(|(v, _)| *v == verb)
        .expect("verb");
    says[0]
}

fn select(path: &str) -> Parsed {
    Parsed::Action(AppAction::SelectVariable { path: path.into() })
}

pub(super) fn action(action: AppAction) -> Parsed {
    Parsed::Action(action)
}

#[test]
fn every_language_says_every_verb_and_then() {
    for &lang in Language::ALL {
        let phrases = table(lang);
        for verb in VERBS {
            let says = phrases.verbs.iter().find(|(v, _)| *v == verb);
            assert!(
                says.is_some_and(|(_, s)| !s.is_empty()),
                "{lang:?} lacks {verb:?}"
            );
        }
        assert!(!phrases.then.is_empty(), "{lang:?} has no 'then'");
        assert!(!phrases.catalog.is_empty(), "{lang:?} has no catalog");
    }
}

#[test]
fn every_language_plots_a_variable_as_a_plot_type() {
    let vocab = vocab();
    for &lang in Language::ALL {
        let volume = translate(lang, PlotKey::Volume);
        let utterance = format!("{} t2m {volume}", say(lang, Verb::Plot));
        let parsed = parse(&utterance, &vocab, lang);
        let [
            first,
            Parsed::Action(AppAction::Plot {
                plot_type: Some(said),
            }),
        ] = parsed.as_slice()
        else {
            panic!("{lang:?} '{utterance}': {parsed:?}");
        };
        assert_eq!(*first, select("t2m"), "{lang:?}");
        assert_eq!(
            vocab.plot_type(said),
            Resolution::Resolved(PlotType::Volume),
            "{lang:?}"
        );
    }
}

#[test]
fn every_language_chains_commands_with_then() {
    let vocab = vocab();
    for &lang in Language::ALL {
        let t = table(lang);
        let utterance = format!(
            "{} t2m {} {}",
            say(lang, Verb::Plot),
            t.then[0],
            say(lang, Verb::Play)
        );
        let parsed = parse(&utterance, &vocab, lang);
        let expected = [
            select("t2m"),
            action(AppAction::Plot { plot_type: None }),
            action(AppAction::Play),
        ];
        assert_eq!(parsed, expected, "{lang:?} '{utterance}'");
    }
}

#[test]
fn a_language_other_than_the_ui_one_is_understood() {
    let vocab = vocab();
    assert_eq!(
        parse("Reproduce.", &vocab, Language::English),
        [action(AppAction::Play)]
    );
    assert_eq!(
        parse("t2mを表示して", &vocab, Language::German),
        [select("t2m"), action(AppAction::Plot { plot_type: None })]
    );
}

#[test]
fn spoken_forms_of_names_are_found() {
    let vocab = vocab();
    let parsed = parse(
        "Plot T 2 M as a volume, then play.",
        &vocab,
        Language::English,
    );
    let [
        Parsed::Action(AppAction::SelectVariable { path }),
        Parsed::Action(AppAction::Plot { plot_type: Some(_) }),
        _,
    ] = parsed.as_slice()
    else {
        panic!("{parsed:?}");
    };
    assert_eq!(vocab.variable(path), vocab.variable("t2m"));
    assert_eq!(
        parse("next step", &vocab, Language::English),
        [action(AppAction::Step { by: 1 })]
    );
    assert_eq!(
        parse("色图 viridis", &vocab, Language::ChineseSimp),
        [action(AppAction::SetColormap {
            name: "viridis".into(),
            layer: None,
        })]
    );
}

#[test]
fn filler_is_ignored_and_near_names_are_kept_for_an_answer() {
    let vocab = vocab();
    assert_eq!(
        parse("plot it as a volume", &vocab, Language::English),
        [action(AppAction::Plot {
            plot_type: Some("volume".into())
        })]
    );
    assert_eq!(
        parse("plot sstt", &vocab, Language::English),
        [select("sstt"), action(AppAction::Plot { plot_type: None })]
    );
}

#[test]
fn names_alone_select_or_plot() {
    let vocab = vocab();
    assert_eq!(parse("sst", &vocab, Language::English), [select("sst")]);
    assert_eq!(
        parse("Volume.", &vocab, Language::English),
        [action(AppAction::Plot {
            plot_type: Some("volume".into())
        })]
    );
    assert_eq!(
        parse("well, hmm", &vocab, Language::English),
        [
            Parsed::NotUnderstood("well".into()),
            Parsed::NotUnderstood("hmm".into())
        ]
    );
}

#[test]
fn a_named_dataset_or_variable_alone_opens_or_selects() {
    let vocab = vocab();
    assert_eq!(
        parse("SeasFire Cube", &vocab, Language::English),
        [action(AppAction::OpenSource {
            source: "seasfire cube".into()
        })]
    );
    assert_eq!(
        parse("open t2m", &vocab, Language::English),
        [select("t2m")]
    );
    assert_eq!(
        parse("show the catalog", &vocab, Language::English),
        [action(AppAction::OpenCatalog)]
    );
}

#[test]
fn the_language_covering_most_of_the_utterance_reads_it() {
    let vocab = vocab();
    let play = action(AppAction::Play);
    for (lang, said) in [
        (Language::English, "abre sst depois reproduz"),
        (Language::English, "sstを描画してそれから再生して"),
        (Language::Russian, "покажи sst потім відтвори"),
    ] {
        let parsed = parse(said, &vocab, lang);
        assert_eq!(parsed.last(), Some(&play), "{said}: {parsed:?}");
    }
}
