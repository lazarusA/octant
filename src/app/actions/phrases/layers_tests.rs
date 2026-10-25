//! Layers, colors and the export asked in every language.

use super::tests::{action, say, vocab};
use super::{Parsed, Verb, parse, table};
use crate::app::actions::AppAction;
use crate::i18n::Language;

#[test]
fn every_language_handles_layers_colors_and_export() {
    let vocab = vocab();
    for &lang in Language::ALL {
        let overlay = table(lang).overlay[0];
        let cases: [(String, Vec<Parsed>); 5] = [
            (
                format!("{} sst {overlay}", say(lang, Verb::Plot)),
                vec![
                    action(AppAction::SelectVariable { path: "sst".into() }),
                    action(AppAction::AddOverlay),
                ],
            ),
            (
                format!("{} sst", say(lang, Verb::Hide)),
                vec![action(AppAction::SetLayerVisible {
                    layer: "sst".into(),
                    visible: false,
                })],
            ),
            (
                format!("{} sst", say(lang, Verb::Remove)),
                vec![action(AppAction::RemoveOverlay {
                    layer: "sst".into(),
                })],
            ),
            (
                format!("{} 250 300", say(lang, Verb::Range)),
                vec![action(AppAction::SetColorRange {
                    min: 250.0,
                    max: 300.0,
                    layer: None,
                })],
            ),
            (
                format!("{} pdf", say(lang, Verb::Export)),
                vec![action(AppAction::Export {
                    format: Some("pdf".into()),
                })],
            ),
        ];
        for (said, expected) in cases {
            assert_eq!(parse(&said, &vocab, lang), expected, "{lang:?} '{said}'");
        }
    }
}
