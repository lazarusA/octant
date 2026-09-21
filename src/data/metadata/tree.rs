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
}

impl VariableTreeGroup {
    /// Returns the total number of variables in this group and all its descendant subgroups.
    pub fn total_variable_count(&self) -> usize {
        let direct = self.variable_indices.len();
        let nested: usize = self
            .subgroups
            .iter()
            .map(|g| g.total_variable_count())
            .sum();
        direct + nested
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
            Some(VariableTreeGroup {
                name: self.name.clone(),
                full_path: self.full_path.clone(),
                variable_indices: filtered_vars,
                subgroups: filtered_subgroups,
            })
        } else {
            None
        }
    }

    /// Builds a hierarchical variable tree from a slice of VariableInfo.
    pub fn build_tree_from_variables(variables: &[VariableInfo]) -> VariableTreeGroup {
        let mut root = VariableTreeGroup {
            name: "Root".to_string(),
            full_path: String::new(),
            variable_indices: Vec::new(),
            subgroups: Vec::new(),
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
                                variable_indices: Vec::new(),
                                subgroups: Vec::new(),
                            });
                            current_group.subgroups.len() - 1
                        }
                    };

                    current_group = &mut current_group.subgroups[sub_idx];
                }

                current_group.variable_indices.push(idx);
            }
        }

        root
    }
}
