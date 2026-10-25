//! The names each kind of thing goes by, best first: variables of the
//! active dataset, its dimensions, plot types in every language, colormaps.

use super::resolve::Candidate;
use super::text::Name;
use crate::data::{DatasetMetadata, VariableInfo};
use crate::i18n::{Language, PlotKey, translate};
use crate::plots::PlotType;
use crate::utils::colormap::registry;

/// A variable's names, best first: its path, the path without its leading
/// groups, its listed name, its long name. Coarser levels of a multiscale
/// stack go by their stack's level 0.
pub(super) fn variable_candidates(meta: &DatasetMetadata) -> Vec<Candidate<usize>> {
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

/// The dataset's dimension names, each once.
pub(super) fn dim_candidates(meta: &DatasetMetadata) -> Vec<Candidate<()>> {
    let mut seen = std::collections::HashSet::new();
    meta.variables
        .iter()
        .flat_map(|v| &v.dimension_names)
        .filter(|name| seen.insert(name.as_str()))
        .map(|name| Candidate {
            value: (),
            shown: name.clone(),
            names: vec![(0, Name::new(name))],
        })
        .collect()
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

/// Each plot type's English id, as tools name them.
pub(super) fn plot_type_ids() -> impl Iterator<Item = &'static str> {
    PLOT_TYPES.iter().map(|(_, _, ids)| ids[0])
}

/// Plot types by their English ids and their names in every language.
pub(super) fn plot_type_candidates() -> Vec<Candidate<PlotType>> {
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

/// Colormaps by name and by `family:name` key. A name several families
/// share names the registry's first; the others go by their keys alone.
pub(super) fn colormap_candidates() -> Vec<Candidate<u32>> {
    let mut out: Vec<Candidate<u32>> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for id in 0..registry::len() as u32 {
        let Some((key, name)) = registry::with_entry(id, |e| (e.key.clone(), e.name.clone()))
        else {
            continue;
        };
        let name = Name::new(&name);
        let mut names = vec![(1, Name::new(&key))];
        if seen.insert(name.compact.clone()) {
            names.insert(0, (0, name));
        }
        out.push(Candidate {
            value: id,
            shown: key.clone(),
            names,
        });
    }
    out
}

/// The groups holding the dataset's variables (every level of each path),
/// each once, by path and by its last name.
pub(super) fn group_candidates(meta: &DatasetMetadata) -> Vec<Candidate<()>> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for path in meta.variables.iter().filter_map(VariableInfo::group_path) {
        let mut end = 0;
        for part in path.split('/') {
            end += part.len();
            let prefix = &path[..end];
            end += 1;
            if part.is_empty() || !seen.insert(prefix) {
                continue;
            }
            out.push(Candidate {
                value: (),
                shown: prefix.to_string(),
                names: vec![(0, Name::new(prefix)), (1, Name::new(part))],
            });
        }
    }
    out
}
