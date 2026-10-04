//! Cached filtering of the colormap registry by query, kind and family.

use crate::data::coordinates::naming::contains_ascii_case_insensitive;
use crate::utils::colormap::{ColormapKind, builtin, registry};

/// Family filter: a bundled family index, or the user's custom maps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FamilyFilter {
    Builtin(usize),
    Custom,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FilterKey {
    pub query: String,
    pub kind: Option<ColormapKind>,
    pub family: Option<FamilyFilter>,
}

/// Matching colormap ids for one `(registry generation, FilterKey)`.
#[derive(Default)]
pub struct FilterCache {
    generation: u64,
    key: Option<FilterKey>,
    ids: Vec<u32>,
}

impl FilterCache {
    /// Returns matching ids, recomputing only when the filter or registry changed.
    pub fn ids(&mut self, key: &FilterKey) -> &[u32] {
        let generation = registry::generation();
        if self.generation != generation || self.key.as_ref() != Some(key) {
            self.ids = filter_ids(key);
            self.key = Some(key.clone());
            self.generation = generation;
        }
        &self.ids
    }
}

/// Ids whose name or family name contain the query (ASCII case-insensitive) and
/// that match the kind and family filters.
pub fn filter_ids(key: &FilterKey) -> Vec<u32> {
    let families = &builtin().families;
    let query = key.query.trim();
    let total = u32::try_from(registry::len()).unwrap_or(u32::MAX);
    (0..total)
        .filter(|&id| {
            registry::with_entry(id, |e| {
                let family_name = e
                    .family
                    .and_then(|f| families.get(f))
                    .map_or("Custom", |f| f.name.as_str());
                let kind_ok = key.kind.is_none_or(|k| k == e.kind);
                let family_ok = match key.family {
                    None => true,
                    Some(FamilyFilter::Builtin(f)) => e.family == Some(f),
                    Some(FamilyFilter::Custom) => e.family.is_none(),
                };
                let text_ok = contains_ascii_case_insensitive(&e.name, query)
                    || contains_ascii_case_insensitive(family_name, query);
                kind_ok && family_ok && text_ok
            })
            .unwrap_or(false)
        })
        .collect()
}
