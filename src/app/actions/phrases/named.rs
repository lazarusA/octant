//! What one command names: its verbs, then a plot type, a colormap (only
//! after a colormap verb), the catalog or the dataset, its dimensions and
//! values, a variable, a dataset to open, a group, each outside the others.

use std::ops::Range;

use super::layer_words::{LayerWords, layer_words};
use super::{Phrases, Verb, verbs_in, word_of};
use crate::app::actions::AppAction;
use crate::app::actions::resolve::Vocabulary;
use crate::app::actions::text::{Segment, address_in, overlaps};

/// A verb and where it was said.
type SaidVerb = (Range<usize>, Verb);

/// The verbs of `segment` and where it names a plot type: where a verb and
/// a plot type name overlap ("color map" and "map", 画 and 热力图), the
/// longer is meant.
fn verbs_and_plot_type(
    segment: &Segment,
    phrases: &Phrases,
    vocab: &Vocabulary,
) -> (Vec<SaidVerb>, Option<Range<usize>>) {
    let mut found = verbs_in(phrases, segment);
    let mut plot_span = vocab.find_plot_type(segment, &[]);
    if let Some(p) = &plot_span {
        let overlapping = |r: &Range<usize>| overlaps(r, p);
        if found
            .iter()
            .any(|(r, _)| overlapping(r) && r.len() >= p.len())
        {
            plot_span = vocab.find_plot_type(segment, &spans(&found));
        } else {
            found.retain(|(r, _)| !overlapping(r));
        }
    }
    (found, plot_span)
}

fn spans(found: &[(Range<usize>, Verb)]) -> Vec<Range<usize>> {
    found.iter().map(|(r, _)| r.clone()).collect()
}

/// What one command names.
pub(super) struct Named {
    pub verbs: Vec<Verb>,
    pub colormap: Option<String>,
    pub plot_type: Option<String>,
    pub variable: Option<String>,
    /// A dataset to open: a landing sample, catalog entry or open dataset.
    pub source: Option<String>,
    /// Whether it names the dataset catalog.
    pub catalog: bool,
    /// Whether it names the dataset itself.
    pub dataset: bool,
    /// A group of the dataset's variables.
    pub group: Option<String>,
    /// Everything but its verbs: what a search looks for.
    pub query: String,
    /// What it says of dimensions (`dims::dim_actions`).
    pub dims: Vec<AppAction>,
    /// What it says of layers, the export and a plate.
    pub layers: LayerWords,
    /// An address typed as written (`https://x/Data.zarr`).
    pub address: Option<String>,
    /// The words left over: a name not found, or filler.
    pub rest: String,
}

pub(super) fn named(segment: &Segment, phrases: &Phrases, vocab: &Vocabulary) -> Named {
    let (found, plot_span) = verbs_and_plot_type(segment, phrases, vocab);
    let mut taken = spans(&found);
    let query = segment.rest(&taken);
    let (address, at) = address_in(&segment.text).unzip();
    taken.extend(at);
    let verbs: Vec<Verb> = found.into_iter().map(|(_, v)| v).collect();
    let take = |span: Option<Range<usize>>, taken: &mut Vec<Range<usize>>| {
        let text = span.as_ref().map(|r| segment.text[r.clone()].to_string());
        taken.extend(span);
        text
    };
    let plot_type = take(plot_span, &mut taken);
    let colormap = if verbs.contains(&Verb::Colormap) {
        take(vocab.find_colormap(segment, &taken), &mut taken)
    } else {
        None
    };
    let catalog = take(word_of(segment, phrases.catalog, &taken), &mut taken).is_some();
    let dataset = take(word_of(segment, phrases.dataset, &taken), &mut taken).is_some();
    let layers = layer_words(segment, phrases, vocab, &verbs, &mut taken);
    let dims = super::dims::dim_actions(segment, phrases, vocab, &verbs, &mut taken);
    // A variable and a dataset may share a name; the verb decides.
    let names_from = taken.clone();
    let variable = take(vocab.find_variable(segment, &names_from), &mut taken);
    let source = take(vocab.find_source(segment, &names_from), &mut taken);
    let group = take(vocab.find_group(segment, &names_from), &mut taken);
    Named {
        rest: segment.rest(&taken),
        verbs,
        colormap,
        plot_type,
        variable,
        source,
        catalog,
        dataset,
        group,
        query,
        dims,
        layers,
        address,
    }
}
