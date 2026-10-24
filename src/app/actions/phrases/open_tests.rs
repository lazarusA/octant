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
