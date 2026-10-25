//! Names to what the app takes: a variable of the active dataset, a plot
//! type, a colormap. Whatever the dataset holds, its tree is read as loaded;
//! nothing here knows its format.

use super::candidates::{
    colormap_candidates, dim_candidates, group_candidates, plot_type_candidates,
    variable_candidates,
};
use super::source_names::{SourceRef, source_candidates};
use super::text::{Name, Segment, unlikeness};
use crate::data::DatasetMetadata;
use crate::plots::PlotType;

/// Names shown when nothing matches, closest first.
const CLOSEST: usize = 3;
/// How unlike a name may be and still be offered.
const OFFER_UNLIKENESS: f32 = 0.5;

#[derive(Debug, Clone, PartialEq)]
pub enum Resolution<T> {
    Resolved(T),
    /// Several match equally well: their names.
    Ambiguous(Vec<String>),
    /// None match: the closest names, if any are close.
    NotFound(Vec<String>),
}

/// The names a thing goes by, best first; `rank` orders how it matched.
pub(super) struct Candidate<T> {
    pub value: T,
    pub shown: String,
    pub names: Vec<(u8, Name)>,
}

/// Which of `names` `text` names (case, spaces and punctuation aside), or
/// the closest ones.
pub(super) fn among(names: &[&str], text: &str) -> Resolution<usize> {
    let candidates: Vec<Candidate<usize>> = names
        .iter()
        .enumerate()
        .map(|(i, name)| Candidate {
            value: i,
            shown: name.to_string(),
            names: vec![(0, Name::new(name))],
        })
        .collect();
    resolve(&candidates, text)
}

/// What names a command can find, kept until what it was built from
/// changes (`OctantApp::vocabulary`).
pub struct Vocabulary {
    variables: Vec<Candidate<usize>>,
    plot_types: Vec<Candidate<PlotType>>,
    colormaps: Vec<Candidate<u32>>,
    sources: Vec<Candidate<SourceRef>>,
    /// Every dimension name of the dataset's variables.
    dims: Vec<Candidate<()>>,
    /// Every group holding the dataset's variables.
    groups: Vec<Candidate<()>>,
    /// The wells of the open plate.
    wells: Vec<Candidate<()>>,
}

impl Vocabulary {
    /// The names in `meta` (the active dataset), plot types, colormaps and
    /// datasets to open, `open` the open ones as (store, name).
    pub fn new(meta: Option<&DatasetMetadata>, open: &[(String, String)]) -> Self {
        Self {
            variables: meta.map(variable_candidates).unwrap_or_default(),
            plot_types: plot_type_candidates(),
            colormaps: colormap_candidates(),
            sources: source_candidates(open),
            dims: meta.map(dim_candidates).unwrap_or_default(),
            groups: meta.map(group_candidates).unwrap_or_default(),
            wells: Vec::new(),
        }
    }

    pub fn source(&self, text: &str) -> Resolution<SourceRef> {
        resolve(&self.sources, text)
    }

    pub fn variable(&self, text: &str) -> Resolution<usize> {
        resolve(&self.variables, text)
    }

    pub fn plot_type(&self, text: &str) -> Resolution<PlotType> {
        resolve(&self.plot_types, text)
    }

    pub fn colormap(&self, text: &str) -> Resolution<u32> {
        resolve(&self.colormaps, text)
    }

    /// Where `segment` names a variable, plot type or colormap: the longest
    /// name found outside `taken`.
    pub(super) fn find_variable(&self, s: &Segment, taken: &[std::ops::Range<usize>]) -> Found {
        find(&self.variables, s, taken)
    }

    pub(super) fn find_plot_type(&self, s: &Segment, taken: &[std::ops::Range<usize>]) -> Found {
        find(&self.plot_types, s, taken)
    }

    /// The open plate's wells, by name (`B3`).
    pub fn with_wells(mut self, names: &[String]) -> Self {
        self.wells = names
            .iter()
            .map(|name| Candidate {
                value: (),
                shown: name.clone(),
                names: vec![(0, Name::new(name))],
            })
            .collect();
        self
    }

    pub(super) fn find_well(&self, s: &Segment, taken: &[std::ops::Range<usize>]) -> Found {
        find(&self.wells, s, taken)
    }

    pub(super) fn find_group(&self, s: &Segment, taken: &[std::ops::Range<usize>]) -> Found {
        find(&self.groups, s, taken)
    }

    pub(super) fn find_dim(&self, s: &Segment, taken: &[std::ops::Range<usize>]) -> Found {
        find(&self.dims, s, taken)
    }

    pub(super) fn find_source(&self, s: &Segment, taken: &[std::ops::Range<usize>]) -> Found {
        find(&self.sources, s, taken)
    }

    pub(super) fn find_colormap(&self, s: &Segment, taken: &[std::ops::Range<usize>]) -> Found {
        find(&self.colormaps, s, taken)
    }
}

/// A name's span in a segment.
pub(super) type Found = Option<std::ops::Range<usize>>;

fn find<T>(candidates: &[Candidate<T>], s: &Segment, taken: &[std::ops::Range<usize>]) -> Found {
    let mut best: Found = None;
    for (_, name) in candidates.iter().flat_map(|c| &c.names) {
        let Some(span) = s.find(name, taken) else {
            continue;
        };
        if best.as_ref().is_none_or(|b| span.len() > b.len()) {
            best = Some(span);
        }
    }
    best
}

/// `text` against every candidate's names: the best rank that matches
/// decides; equal ranks of different things are ambiguous.
fn resolve<T: Clone>(candidates: &[Candidate<T>], text: &str) -> Resolution<T> {
    let heard = Name::new(text);
    let mut best: Option<u8> = None;
    let mut matches: Vec<&Candidate<T>> = Vec::new();
    for c in candidates {
        let Some(rank) = c
            .names
            .iter()
            .filter(|(_, n)| n.compact == heard.compact && !heard.compact.is_empty())
            .map(|(r, _)| *r)
            .min()
        else {
            continue;
        };
        if best.is_none_or(|b| rank < b) {
            best = Some(rank);
            matches.clear();
        }
        if best == Some(rank) {
            matches.push(c);
        }
    }
    match matches.as_slice() {
        [one] => Resolution::Resolved(one.value.clone()),
        [] => Resolution::NotFound(closest(candidates, text)),
        many => Resolution::Ambiguous(many.iter().map(|c| c.shown.clone()).collect()),
    }
}

/// The candidates closest to `text`, closest first.
fn closest<T>(candidates: &[Candidate<T>], text: &str) -> Vec<String> {
    let heard = Segment::new(text);
    let mut scored: Vec<(f32, &str)> = candidates
        .iter()
        .map(|c| {
            let score = c
                .names
                .iter()
                .map(|(_, n)| unlikeness(&heard, &n.compact))
                .fold(1.0, f32::min);
            (score, c.shown.as_str())
        })
        .filter(|(score, _)| *score <= OFFER_UNLIKENESS)
        .collect();
    scored.sort_by(|a, b| a.0.total_cmp(&b.0));
    scored
        .into_iter()
        .take(CLOSEST)
        .map(|(_, s)| s.to_string())
        .collect()
}

impl crate::app::OctantApp {
    /// What the names of actions can find right now: the active dataset's
    /// names and the datasets that can be opened; built again only when
    /// one of them changed.
    pub fn vocabulary(&mut self) -> &Vocabulary {
        let wells = self.plate.as_ref().map_or(0, |p| p.plate.wells.len());
        // The store too: generations can repeat once a staged selection
        // takes back the plotted one's.
        let key = (
            self.selected.store_target.clone(),
            self.selected.metadata_generation,
            self.dataset_manager.len(),
            wells,
            crate::utils::colormap::registry::generation(),
        );
        let kept = match self.commands.vocab.take() {
            Some((k, vocab)) if k == key => vocab,
            _ => self.build_vocabulary(),
        };
        &self.commands.vocab.insert((key, kept)).1
    }

    fn build_vocabulary(&self) -> Vocabulary {
        let open: Vec<(String, String)> = self
            .dataset_manager
            .iter()
            .filter_map(|d| Some((d.source.uri.clone(), d.metadata.as_ref()?.name.clone())))
            .collect();
        let wells = self
            .plate
            .as_ref()
            .map(|p| super::plate::well_names(&p.plate));
        Vocabulary::new(self.selected.metadata.as_ref(), &open)
            .with_wells(&wells.unwrap_or_default())
    }
}
