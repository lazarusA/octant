//! What a command says of layers, the export and a plate: that a plot is
//! an overlay, an export format, a color range's two numbers, a well and
//! its field number.

use std::ops::Range;

use super::values::numbers;
use super::{Phrases, Verb, word_of};
use crate::app::actions::AppAction;
use crate::app::actions::resolve::Vocabulary;
use crate::app::actions::text::Segment;
use crate::export::ExportFormat;

/// Export formats as said, alike in every language: each one's extension
/// (what `Export` takes), and `jpeg` for JPEG's `jpg`.
fn said_formats() -> impl Iterator<Item = (&'static str, &'static str)> {
    let extensions = ExportFormat::ALL
        .iter()
        .map(|f| (f.extension(), f.extension()));
    extensions.chain([("jpeg", ExportFormat::Jpeg.extension())])
}

pub(super) struct LayerWords {
    /// Whether a plot is said to be an overlay.
    pub overlay: bool,
    pub format: Option<String>,
    /// A color range's ends, after a range verb.
    pub range: Option<(f32, f32)>,
    /// A well picked or a field of it opened.
    pub plate: Option<AppAction>,
}

/// The layer, export and plate words of a command holding `verbs`; the
/// words they use join `taken`.
pub(super) fn layer_words(
    segment: &Segment,
    phrases: &Phrases,
    vocab: &Vocabulary,
    verbs: &[Verb],
    taken: &mut Vec<Range<usize>>,
) -> LayerWords {
    let overlay_word = word_of(segment, phrases.overlay, taken);
    let overlay = overlay_word.is_some();
    taken.extend(overlay_word);
    let format = said_formats().find_map(|(word, format)| {
        let span = word_of(segment, &[word], taken)?;
        taken.push(span);
        Some(format.to_string())
    });
    let range = verbs.contains(&Verb::Range).then(|| {
        let said = numbers(segment, taken);
        let [(a, sa), (b, sb), ..] = said.as_slice() else {
            return None;
        };
        taken.extend([sa.clone(), sb.clone()]);
        Some((a.parse().ok()?, b.parse().ok()?))
    });
    LayerWords {
        overlay,
        format,
        range: range.flatten(),
        plate: plate_action(segment, phrases, vocab, taken),
    }
}

/// A well of the open plate named, and the number said after "field".
fn plate_action(
    segment: &Segment,
    phrases: &Phrases,
    vocab: &Vocabulary,
    taken: &mut Vec<Range<usize>>,
) -> Option<AppAction> {
    let well = vocab.find_well(segment, taken);
    taken.extend(well.clone());
    let field = word_of(segment, phrases.field, taken).and_then(|word| {
        let (number, span) = numbers(segment, taken)
            .into_iter()
            .find(|(_, span)| span.start >= word.end)?;
        taken.extend([word, span]);
        number.parse().ok()
    });
    taken.extend(word_of(segment, phrases.well, taken));
    let well = well.map(|span| segment.text[span].to_string());
    (well.is_some() || field.is_some()).then_some(AppAction::OpenPlateWell { well, field })
}
