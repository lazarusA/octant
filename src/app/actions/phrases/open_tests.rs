//! Datasets, the catalog and typed addresses opened in every language.

use super::tests::{action, say, vocab};
use super::{Verb, commands, parse, table};
use crate::app::actions::AppAction;
use crate::i18n::Language;

#[test]
fn every_language_opens_the_catalog_and_a_dataset_by_name() {
    let vocab = vocab();
    for &lang in Language::ALL {
        let open = say(lang, Verb::Open);
        let catalog = format!("{open} {}", table(lang).catalog[0]);
        assert_eq!(
            parse(&catalog, &vocab, lang),
            [action(AppAction::OpenCatalog)],
            "{lang:?} '{catalog}'"
        );
        let entry = format!("{open} seasfire");
        assert_eq!(
            parse(&entry, &vocab, lang),
            [action(AppAction::OpenSource {
                source: "seasfire".into()
            })],
            "{lang:?} '{entry}'"
        );
    }
}

#[test]
fn what_follows_open_is_a_command_of_its_own() {
    let cut = commands("open seasfire plot t2m as a volume", Language::English);
    let texts: Vec<&str> = cut.iter().map(|(_, s)| s.text.trim()).collect();
    assert_eq!(texts, ["open seasfire", "plot t2m as a volume"]);
}

#[test]
fn typed_addresses_open_as_written() {
    let vocab = vocab();
    let parsed = parse(
        "Open s3://Bucket/My.zarr. Then plot",
        &vocab,
        Language::English,
    );
    assert_eq!(
        parsed,
        [
            action(AppAction::OpenSource {
                source: "s3://Bucket/My.zarr".into()
            }),
            action(AppAction::Plot { plot_type: None }),
        ]
    );
}

#[test]
fn words_inside_an_address_are_no_commands() {
    let vocab = vocab();
    let url = "https://data.example/play/then/Show.zarr";
    assert_eq!(
        parse(&format!("open {url}"), &vocab, Language::English),
        [action(AppAction::OpenSource { source: url.into() })]
    );
}

#[test]
fn addresses_touching_cjk_text_and_windows_paths_are_kept_whole() {
    let vocab = vocab();
    let open = |source: &str| {
        action(AppAction::OpenSource {
            source: source.into(),
        })
    };
    assert_eq!(
        parse("打开https://x/Data.zarr", &vocab, Language::ChineseSimp),
        [open("https://x/Data.zarr")]
    );
    assert_eq!(
        parse(
            "打开 https://x/Data.zarr然后播放",
            &vocab,
            Language::ChineseSimp
        ),
        [open("https://x/Data.zarr"), action(AppAction::Play)]
    );
    // A full stop after the address still ends the command.
    assert_eq!(
        parse(
            "打开https://x/Data.zarr。播放",
            &vocab,
            Language::ChineseSimp
        ),
        [open("https://x/Data.zarr"), action(AppAction::Play)]
    );
    // A folder's own CJK name stays.
    assert_eq!(
        parse("打开 /数据/温度数据", &vocab, Language::ChineseSimp),
        [open("/数据/温度数据")]
    );
    assert_eq!(
        parse("open C:\\Data\\x.nc", &vocab, Language::English),
        [open("C:\\Data\\x.nc")]
    );
}

#[test]
fn a_query_string_stays_in_its_address() {
    let vocab = vocab();
    let url = "https://data.example/a.zarr?X-Amz-Signature=AbC;x=1";
    assert_eq!(
        parse(&format!("open {url} then plot"), &vocab, Language::English),
        [
            action(AppAction::OpenSource { source: url.into() }),
            action(AppAction::Plot { plot_type: None }),
        ]
    );
}
