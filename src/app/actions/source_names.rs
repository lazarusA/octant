//! The datasets a command can open by name: the landing samples, the
//! catalog's entries and the datasets already open. One store is one
//! choice, whatever names it goes by (the SeasFire sample and catalog entry
//! are one).

use super::resolve::Candidate;
use super::text::Name;
use crate::catalog::{CatalogCategoryFilter, CatalogEntry, get_catalog_entries};
use crate::ui::hero::chips::SAMPLES;

/// Where a named dataset is opened from.
#[derive(Debug, Clone, PartialEq)]
pub enum SourceRef {
    /// A landing sample's store, opened as its pill does.
    Sample(&'static str),
    /// A catalog entry, opened as its card does.
    Entry(&'static CatalogEntry),
    /// A dataset already open (or any address), by its store.
    Store(String),
}

impl SourceRef {
    fn store(&self) -> &str {
        match self {
            SourceRef::Sample(uri) => uri,
            SourceRef::Entry(entry) => entry.store.trim(),
            SourceRef::Store(uri) => uri,
        }
    }
}

/// Every dataset by name; `open` lists the open ones as (store, name).
pub(super) fn source_candidates(open: &[(String, String)]) -> Vec<Candidate<SourceRef>> {
    let mut out: Vec<Candidate<SourceRef>> = Vec::new();
    for entry in get_catalog_entries(CatalogCategoryFilter::All) {
        out.push(Candidate {
            value: SourceRef::Entry(entry),
            shown: entry.label.to_string(),
            names: vec![(0, Name::new(entry.label)), (0, Name::new(entry.key))],
        });
    }
    for (label, uri, _) in SAMPLES {
        if uri == crate::app::sources::CATALOG_SAMPLE {
            continue;
        }
        let label = label.trim_start_matches('/');
        add_name(&mut out, uri, label, || SourceRef::Sample(uri));
    }
    for (uri, name) in open {
        add_name(&mut out, uri, name, || SourceRef::Store(uri.clone()));
    }
    out
}

/// Adds `name` to the candidate of `store`, or a new candidate.
fn add_name(
    out: &mut Vec<Candidate<SourceRef>>,
    store: &str,
    name: &str,
    value: impl FnOnce() -> SourceRef,
) {
    let store = store.trim().trim_end_matches('/');
    let same = |c: &&mut Candidate<SourceRef>| c.value.store().trim_end_matches('/') == store;
    match out.iter_mut().find(same) {
        Some(c) => c.names.push((0, Name::new(name))),
        None => out.push(Candidate {
            value: value(),
            shown: name.to_string(),
            names: vec![(0, Name::new(name))],
        }),
    }
}
