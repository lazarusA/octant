//! Questions asked in every language, read into actions.

use super::tests::{action, say, vocab};
use super::{Verb, parse, table};
use crate::app::actions::AppAction;
use crate::i18n::Language;

#[test]
fn every_language_asks_questions() {
    let vocab = vocab();
    for &lang in Language::ALL {
        let t = table(lang);
        let describe = say(lang, Verb::Describe);
        let cases = [
            (
                format!("{describe} t2m"),
                AppAction::DescribeVariable {
                    path: Some("t2m".into()),
                },
            ),
            (
                format!("{describe} {}", t.dataset[0]),
                AppAction::DescribeDataset { group: None },
            ),
            (describe.to_string(), AppAction::DescribeView),
            (
                format!("{} sst", say(lang, Verb::Search)),
                AppAction::Search { text: "sst".into() },
            ),
            (
                say(lang, Verb::List).to_string(),
                AppAction::DescribeDataset { group: None },
            ),
        ];
        for (said, expected) in cases {
            assert_eq!(
                parse(&said, &vocab, lang),
                [action(expected)],
                "{lang:?} '{said}'"
            );
        }
    }
}
