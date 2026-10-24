//! Questions about the data: what a dataset or one of its groups holds,
//! which variables match a search (the Variables panel's own search), what
//! a variable is with its coordinate ranges, what is plotted. Answers are
//! data; the front ends word and show them (`answer.rs`).

use super::coord_match::Dim;
use super::resolve::{Resolution, among};
use super::{Done, Failure, NameKind, Outcome, resolved};
use crate::app::OctantApp;
use crate::data::{DatasetMetadata, VariableInfo, VariableTreeGroup};

/// Names of a group's entries an answer lists.
const ENTRIES: usize = 12;

/// A dimension: its name, size and first and last coordinates as the hover
/// writes them.
#[derive(Debug, Clone, PartialEq)]
pub struct DimSummary {
    pub name: String,
    pub size: usize,
    pub first: String,
    pub last: String,
}

/// Every group of `tree`, depth first.
fn groups(tree: &VariableTreeGroup) -> Vec<&VariableTreeGroup> {
    let mut out = vec![tree];
    for sub in &tree.subgroups {
        out.extend(groups(sub));
    }
    out
}

/// The group `name` names, by path or the name shown.
fn group_named<'t>(
    tree: &'t VariableTreeGroup,
    name: &str,
) -> Result<&'t VariableTreeGroup, Failure> {
    let all = groups(tree);
    let paths: Vec<&str> = all.iter().map(|g| g.full_path.as_str()).collect();
    let shown: Vec<&str> = all.iter().map(|g| g.display_name()).collect();
    let found = match among(&paths, name) {
        Resolution::Resolved(i) => Resolution::Resolved(i),
        _ => among(&shown, name),
    };
    resolved(found, NameKind::Group, name).map(|i| all[i])
}

fn summaries(meta: &DatasetMetadata, var: &VariableInfo, store: &str) -> Vec<DimSummary> {
    let dims = var.dimension_names.iter().zip(&var.shape);
    dims.map(|(name, &size)| {
        let len = size as usize;
        let dim = Dim {
            meta,
            var,
            store,
            name,
            len,
        };
        DimSummary {
            name: name.clone(),
            size: len,
            first: dim.text(0),
            last: dim.text(len.saturating_sub(1)),
        }
    })
    .collect()
}

impl OctantApp {
    fn variable_tree(&self, meta: &DatasetMetadata) -> VariableTreeGroup {
        self.cached_variable_tree
            .clone()
            .unwrap_or_else(|| VariableTreeGroup::build_tree_from_variables(&meta.variables))
    }

    pub(super) fn describe_dataset(&self, group: Option<&str>) -> Outcome {
        let meta = self.selected.metadata.as_ref().ok_or(Failure::NoDataset)?;
        let tree = self.variable_tree(meta);
        let shown = match group {
            Some(name) => group_named(&tree, name)?,
            None => &tree,
        };
        let subgroups = shown.subgroups.iter().map(|g| g.display_name().to_string());
        let variables = shown
            .variable_indices
            .iter()
            .filter_map(|&i| meta.variables.get(i));
        let names = variables
            .filter(|v| !v.is_coarser_level())
            .map(|v| v.display_name().to_string());
        Ok(Done::DatasetShown {
            name: group.map_or_else(|| meta.name.clone(), |_| shown.display_name().to_string()),
            variables: shown.total_variable_count(),
            entries: subgroups.chain(names).take(ENTRIES).collect(),
        })
    }

    /// Searches the Variables panel's list, as typing in its search does.
    pub(super) fn search_variables(&mut self, text: &str) -> Outcome {
        let meta = self.selected.metadata.as_ref().ok_or(Failure::NoDataset)?;
        let matches = self
            .variable_tree(meta)
            .filter(text, &meta.variables)
            .map_or(0, |found| found.total_variable_count());
        self.layout.variable_search = text.to_string();
        Ok(Done::Searching {
            text: text.to_string(),
            matches,
        })
    }

    pub(super) fn describe_variable(&mut self, path: Option<&str>) -> Outcome {
        let meta = self.selected.metadata.as_ref().ok_or(Failure::NoDataset)?;
        let idx = match path {
            Some(path) => resolved(self.vocabulary().variable(path), NameKind::Variable, path)?,
            None => self.selected.variable_idx,
        };
        let var = meta.variables.get(idx).cloned().ok_or(Failure::NoDataset)?;
        let unread = var
            .dimension_names
            .iter()
            .any(|d| meta.get_dim_coords(Some(&var.name), d).is_none());
        if unread {
            self.request_variable_coordinates(idx);
            if self.coordinate_loader.pending_count() > 0 {
                return Err(Failure::CoordinatesPending);
            }
        }
        let meta = self.selected.metadata.as_ref().ok_or(Failure::NoDataset)?;
        Ok(Done::VariableShown {
            name: var.display_name().to_string(),
            long_name: var.long_name.clone(),
            units: var.units.clone(),
            dims: summaries(meta, &var, &self.selected.store_target),
        })
    }

    /// What is plotted; with nothing plotted, what the dataset holds.
    pub(super) fn describe_view(&self) -> Outcome {
        let plotted = self.plotted();
        let (Some(meta), Some(var)) = (plotted.metadata.as_ref(), plotted.variable_info()) else {
            return self.describe_dataset(None);
        };
        let step = plotted.animated_dim.and_then(|i| {
            let name = var.dimension_names.get(i)?;
            let len = *var.shape.get(i)? as usize;
            let dim = Dim {
                meta,
                var,
                store: &plotted.store_target,
                name,
                len,
            };
            Some((
                name.clone(),
                dim.text(self.playback.current_timestep.min(len.saturating_sub(1))),
            ))
        });
        let staged = self
            .is_exploring_unplotted_variable()
            .then(|| {
                self.selected_variable_info()
                    .map(|v| v.display_name().to_string())
            })
            .flatten();
        Ok(Done::ViewShown {
            variable: var.display_name().to_string(),
            plot_type: plotted.plot_type,
            step,
            colormap: crate::utils::colormap::registry::key_of(self.layers.base.color.colormap),
            overlays: self.layers.overlays().len(),
            staged,
        })
    }
}
