//! Variable hierarchy tree and search filtering algorithms.

use super::variable::VariableInfo;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VariableTreeGroup {
    /// Segment name of this group (e.g. "atmosphere", "forecast", or "Root").
    pub name: String,
    /// Full path of this group (e.g. "atmosphere/forecast" or "" for root).
    pub full_path: String,
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
            || self.full_path.to_lowercase().contains(query_lower);

        let mut filtered_vars = Vec::new();
        for &idx in &self.variable_indices {
            if let Some(var) = variables.get(idx)
                && (group_matches
                    || var.name.to_lowercase().contains(query_lower)
                    || var
                        .long_name
                        .as_deref()
                        .is_some_and(|l| l.to_lowercase().contains(query_lower))
                    || var
                        .units
                        .as_deref()
                        .is_some_and(|u| u.to_lowercase().contains(query_lower)))
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
            Some(VariableTreeGroup::new(
                self.name.clone(),
                self.full_path.clone(),
                filtered_vars,
                filtered_subgroups,
            ))
        } else {
            None
        }
    }

    /// Builds a hierarchical variable tree from a slice of VariableInfo.
    pub fn build_tree_from_variables(variables: &[VariableInfo]) -> VariableTreeGroup {
        let mut root = VariableTreeGroup {
            name: "Root".to_string(),
            ..Default::default()
        };

        for (idx, var) in variables.iter().enumerate() {
            let clean_name = var.name.trim_start_matches('/').trim_end_matches('/');
            let segments: Vec<&str> = clean_name.split('/').filter(|s| !s.is_empty()).collect();

            if segments.len() <= 1 {
                // Root-level variable
                root.variable_indices.push(idx);
            } else {
                // Nested variable: traverse / insert subgroups
                let mut current_group = &mut root;
                let mut current_path = String::new();

                for &seg in &segments[..segments.len() - 1] {
                    if !current_path.is_empty() {
                        current_path.push('/');
                    }
                    current_path.push_str(seg);

                    let pos = current_group.subgroups.iter().position(|g| g.name == seg);

                    let sub_idx = match pos {
                        Some(p) => p,
                        None => {
                            current_group.subgroups.push(VariableTreeGroup {
                                name: seg.to_string(),
                                full_path: current_path.clone(),
                                ..Default::default()
                            });
                            current_group.subgroups.len() - 1
                        }
                    };

                    current_group = &mut current_group.subgroups[sub_idx];
                }

                current_group.variable_indices.push(idx);
            }
        }

        root.recount();
        root
    }
}
