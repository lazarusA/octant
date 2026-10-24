//! Dimensions and values said in different languages, read into actions.

use super::{Parsed, Verb, parse, table};
use crate::app::actions::resolve::Vocabulary;
use crate::app::actions::{AppAction, DimRole};
use crate::app::test_support::memory_app;
use crate::i18n::Language;

fn vocab() -> Vocabulary {
    let (_, meta) = memory_app();
    Vocabulary::new(Some(&meta), &[])
}

fn set(dim: Option<&str>, value: &str) -> Parsed {
    Parsed::Action(AppAction::SetDim {
        dim: dim.map(str::to_string),
        value: value.into(),
    })
}

#[test]
fn dates_are_read_in_each_language_s_form() {
    let vocab = vocab();
    let cases = [
        (Language::English, "time 15 January 2020", Some("time")),
        (Language::English, "go to March 2010", None),
        (Language::English, "2020-01-15", None),
        (
            Language::Spanish,
            "tiempo 15 de enero de 2020",
            Some("time"),
        ),
        (Language::German, "Zeit 15. Januar 2020", Some("time")),
        (Language::Russian, "15 января 2020", None),
        (Language::ChineseSimp, "时间 2020年1月15日", Some("time")),
        (Language::Japanese, "2020年1月15日に移動", None),
        (Language::Korean, "2020년 1월 15일로 이동", None),
    ];
    for (lang, said, dim) in cases {
        let parsed = parse(said, &vocab, lang);
        let [Parsed::Action(AppAction::SetDim { dim: got, value })] = parsed.as_slice() else {
            panic!("{lang:?} '{said}': {parsed:?}");
        };
        assert_eq!(got.as_deref(), dim, "{lang:?} '{said}'");
        assert!(
            ["2020-01-15", "2010-03"].contains(&value.as_str()),
            "{lang:?} '{said}': {value}"
        );
    }
}

#[test]
fn numbers_need_a_dimension_and_two_make_a_range() {
    let vocab = vocab();
    assert_eq!(
        parse("lat 45,5", &vocab, Language::Spanish),
        [set(Some("lat"), "45.5")]
    );
    assert_eq!(
        parse("latitude -30 to 60", &vocab, Language::English),
        [Parsed::Action(AppAction::SetRange {
            dim: Some("latitude".into()),
            from: "-30".into(),
            to: "60".into(),
        })]
    );
    // The 2 of "t 2 m" is part of a name, no value.
    let plot = parse("plot t 2 m", &vocab, Language::English);
    assert!(
        plot.iter()
            .all(|p| !matches!(p, Parsed::Action(AppAction::SetDim { .. }))),
        "{plot:?}"
    );
}

#[test]
fn roles_are_said_with_a_letter_or_by_animating() {
    let vocab = vocab();
    assert_eq!(
        parse("lat as y", &vocab, Language::English),
        [Parsed::Action(AppAction::SetRole {
            dim: "lat".into(),
            role: DimRole::Y
        })]
    );
    let animate = format!(
        "{} time",
        table(Language::French)
            .verbs
            .iter()
            .find(|(v, _)| *v == Verb::Play)
            .expect("play")
            .1[0]
    );
    assert_eq!(
        parse(&animate, &vocab, Language::French),
        [
            Parsed::Action(AppAction::SetRole {
                dim: "time".into(),
                role: DimRole::Animate
            }),
            Parsed::Action(AppAction::Play),
        ]
    );
}

#[test]
fn dimensions_are_set_before_the_variable_is_plotted() {
    let vocab = vocab();
    let parsed = parse("plot t2m at time 2020-01-15", &vocab, Language::English);
    assert_eq!(
        parsed,
        [
            Parsed::Action(AppAction::SelectVariable { path: "t2m".into() }),
            set(Some("time"), "2020-01-15"),
            Parsed::Action(AppAction::Plot { plot_type: None }),
        ]
    );
}
