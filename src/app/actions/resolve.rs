//! Names to what the app takes: a variable of the active dataset, a plot
//! type, a colormap. Whatever the dataset holds, its tree is read as loaded;
//! nothing here knows its format.

use super::source_names::{SourceRef, source_candidates};
use super::text::{Name, Segment, unlikeness};
use crate::data::DatasetMetadata;
use crate::i18n::{Language, PlotKey, translate};
use crate::plots::PlotType;
use crate::utils::colormap::registry;

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

/// A variable's names, best first: its path, the path without its leading
/// groups, its listed name, its long name. Coarser levels of a multiscale
/// stack go by their stack's level 0.
fn variable_candidates(meta: &DatasetMetadata) -> Vec<Candidate<usize>> {
    let mut out = Vec::new();
    for (idx, var) in meta.variables.iter().enumerate() {
        if var.is_coarser_level() {
            continue;
        }
        let path = var.name.trim_matches('/');
        let mut names = vec![(0, Name::new(path))];
        let mut rest = path;
        while let Some((_, tail)) = rest.split_once('/') {
            names.push((1, Name::new(tail)));
            rest = tail;
        }
        names.push((2, Name::new(var.display_name())));
        if let Some(long) = var.long_name.as_deref() {
            names.push((3, Name::new(long)));
        }
        out.push(Candidate {
            value: idx,
            shown: path.to_string(),
            names,
        });
    }
    out
}

const PLOT_TYPES: [(PlotType, PlotKey, &[&str]); 6] = [
    (
        PlotType::Heatmap,
        PlotKey::Heatmap,
        &["heatmap", "heat map", "map"],
    ),
    (PlotType::Line, PlotKey::Line, &["line", "line plot"]),
    (PlotType::Surface, PlotKey::Surface, &["surface"]),
    (PlotType::Volume, PlotKey::Volume, &["volume"]),
    (PlotType::Sphere, PlotKey::Sphere, &["sphere", "globe"]),
    (
        PlotType::PointCloud,
        PlotKey::PointCloud,
        &["point cloud", "points"],
    ),
];

/// Plot types by their English ids and their names in every language.
fn plot_type_candidates() -> Vec<Candidate<PlotType>> {
    PLOT_TYPES
        .iter()
        .map(|&(plot_type, key, ids)| {
            let mut names: Vec<(u8, Name)> = ids.iter().map(|id| (0, Name::new(id))).collect();
            for &lang in Language::ALL {
                names.push((0, Name::new(translate(lang, key))));
            }
            Candidate {
                value: plot_type,
                shown: plot_type.display_name().to_string(),
                names,
            }
        })
        .collect()
}

/// Colormaps by name and by `family:name` key. Maps of one name in several
/// families are one choice, the registry's first.
fn colormap_candidates() -> Vec<Candidate<u32>> {
    let mut out: Vec<Candidate<u32>> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for id in 0..registry::len() as u32 {
        let Some((key, name)) = registry::with_entry(id, |e| (e.key.clone(), e.name.clone()))
        else {
            continue;
        };
        let name = Name::new(&name);
        if !seen.insert(name.compact.clone()) {
            continue;
        }
        out.push(Candidate {
            value: id,
            shown: key.clone(),
            names: vec![(0, name), (1, Name::new(&key))],
        });
    }
    out
}

/// What names a command can find: built once per utterance.
pub struct Vocabulary {
    variables: Vec<Candidate<usize>>,
    plot_types: Vec<Candidate<PlotType>>,
    colormaps: Vec<Candidate<u32>>,
    sources: Vec<Candidate<SourceRef>>,
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
        let Some(span) = s.find(name) else {
            continue;
        };
        if taken.iter().any(|t| super::text::overlaps(t, &span)) {
            continue;
        }
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
