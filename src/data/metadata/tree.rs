//! Variable hierarchy tree and search filtering algorithms.

use super::variable::VariableInfo;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VariableTreeGroup {
    /// Segment name of this group (e.g. "atmosphere", "forecast", or "Root").
    pub name: String,
    /// Full path of this group (e.g. "atmosphere/forecast" or "" for root).
    pub full_path: String,
    /// A name to show instead of `name` (an OME-Zarr series' name).
    pub label: Option<String>,
    /// Indices of variables directly belonging to this group (into `DatasetMetadata::variables`).
    pub variable_indices: Vec<usize>,
    /// Child subgroups.
    pub subgroups: Vec<VariableTreeGroup>,
    /// Variables in this group and all descendants, computed when the group is
    /// built so drawing a folder never walks its subtree.
    total_count: usize,
}

impl VariableTreeGroup {
    /// A group whose total count is computed from its (already built) parts.
    pub fn new(
        name: String,
        full_path: String,
        variable_indices: Vec<usize>,
        subgroups: Vec<VariableTreeGroup>,
    ) -> Self {
        let nested: usize = subgroups.iter().map(|g| g.total_count).sum();
        Self {
            total_count: variable_indices.len() + nested,
            name,
            full_path,
            label: None,
            variable_indices,
            subgroups,
        }
    }

    /// Returns the total number of variables in this group and all its descendant subgroups.
    pub fn total_variable_count(&self) -> usize {
        self.total_count
    }

    /// Recompute every total count bottom-up after building the tree in place.
    fn recount(&mut self) -> usize {
        let nested: usize = self.subgroups.iter_mut().map(Self::recount).sum();
        self.total_count = self.variable_indices.len() + nested;
        self.total_count
    }

    /// Recursively filters the tree according to a search query string.
    /// Returns `Some(filtered_group)` if this group, any of its subgroups, or any of its variables match.
    pub fn filter(&self, query: &str, variables: &[VariableInfo]) -> Option<VariableTreeGroup> {
        let query_lower = query.trim().to_lowercase();
        if query_lower.is_empty() {
            return Some(self.clone());
        }
        self.filter_lowercased(&query_lower, variables)
    }

    fn filter_lowercased(
        &self,
        query_lower: &str,
        variables: &[VariableInfo],
    ) -> Option<VariableTreeGroup> {
        let group_matches = self.name.to_lowercase().contains(query_lower)
            || self.full_path.to_lowercase().contains(query_lower)
            || self
                .label
                .as_deref()
                .is_some_and(|l| l.to_lowercase().contains(query_lower));

        let mut filtered_vars = Vec::new();
        for &idx in &self.variable_indices {
            if let Some(var) = variables.get(idx)
                && (group_matches || var_matches(var, query_lower))
            {
                filtered_vars.push(idx);
            }
        }

        let mut filtered_subgroups = Vec::new();
        for sub in &self.subgroups {
            if let Some(filtered_sub) = sub.filter_lowercased(query_lower, variables) {
                filtered_subgroups.push(filtered_sub);
            }
        }

        if !filtered_vars.is_empty() || !filtered_subgroups.is_empty() {
            let mut group = VariableTreeGroup::new(
                self.name.clone(),
                self.full_path.clone(),
                filtered_vars,
                filtered_subgroups,
            );
            group.label = self.label.clone();
            Some(group)
        } else {
            None
        }
    }

    /// The name a folder shows: its label, else its path segment.
    pub fn display_name(&self) -> &str {
        self.label.as_deref().unwrap_or(&self.name)
    }

    /// Labels every group `label_of` names, by full path.
    pub fn apply_labels(&mut self, label_of: &impl Fn(&str) -> Option<String>) {
        self.label = label_of(&self.full_path);
        for sub in &mut self.subgroups {
            sub.apply_labels(label_of);
        }
    }

    /// The subgroup at `path` (`a/b/c`), created where missing; `self` for
    /// an empty path.
    fn group_at(&mut self, path: &str) -> &mut Self {
        let mut group = self;
        let mut full_path = String::new();
        for seg in path.split('/').filter(|s| !s.is_empty()) {
            if !full_path.is_empty() {
                full_path.push('/');
            }
            full_path.push_str(seg);
            let i = match group.subgroups.iter().position(|g| g.name == seg) {
                Some(i) => i,
                None => {
                    group.subgroups.push(VariableTreeGroup {
                        name: seg.to_string(),
                        full_path: full_path.clone(),
                        ..Default::default()
                    });
                    group.subgroups.len() - 1
                }
            };
            group = &mut group.subgroups[i];
        }
        group
    }

    /// Builds a hierarchical variable tree from a slice of VariableInfo.
    pub fn build_tree_from_variables(variables: &[VariableInfo]) -> VariableTreeGroup {
        let mut root = VariableTreeGroup {
            name: "Root".to_string(),
            ..Default::default()
        };

        // Coarser levels of a multiscale stack are listed under its level 0,
        // which sits in the group its levels share.
        for (idx, var) in variables.iter().enumerate() {
            if !var.is_coarser_level() {
                root.group_at(var.display_group())
                    .variable_indices
                    .push(idx);
            }
        }

        root.recount();
        root
    }
}

/// Whether `var`'s name, listed name, long name or units contain
/// `query_lower` (lowercase).
fn var_matches(var: &VariableInfo, query_lower: &str) -> bool {
    let has = |text: &str| text.to_lowercase().contains(query_lower);
    has(&var.name)
        || has(var.display_name())
        || var.long_name.as_deref().is_some_and(has)
        || var.units.as_deref().is_some_and(has)
}
