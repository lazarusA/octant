//! Search results cache: the filtered variable tree is rebuilt only when the
//! dataset or the query changes, not on every frame the search is active.

use crate::data::{VariableInfo, VariableTreeGroup};

/// Filtered tree for one `(selected.metadata_generation, query)`; `tree` is `None`
/// when nothing matches.
pub struct SearchCache {
    generation: u64,
    query: String,
    tree: Option<VariableTreeGroup>,
}

/// Search results for `query` over `tree`, reusing `cache` while the dataset
/// (`generation`) and query are unchanged.
pub(super) fn filtered<'a>(
    cache: &'a mut Option<SearchCache>,
    generation: u64,
    query: &str,
    tree: &VariableTreeGroup,
    variables: &[VariableInfo],
) -> Option<&'a VariableTreeGroup> {
    let fresh = cache
        .as_ref()
        .is_some_and(|c| c.generation == generation && c.query == query);
    if !fresh {
        *cache = Some(SearchCache {
            generation,
            query: query.to_owned(),
            tree: tree.filter(query, variables),
        });
    }
    cache.as_ref().and_then(|c| c.tree.as_ref())
}

#[cfg(test)]
mod tests {
    use super::filtered;
    use crate::data::{VariableInfo, VariableTreeGroup};

    fn data() -> (Vec<VariableInfo>, VariableTreeGroup) {
        let variables: Vec<VariableInfo> = ["ocean/sst", "land/lai"]
            .map(|name| VariableInfo {
                name: name.into(),
                ..Default::default()
            })
            .to_vec();
        let tree = VariableTreeGroup::build_tree_from_variables(&variables);
        (variables, tree)
    }

    /// Tag the cached result, so a later call shows whether it was reused
    /// (tag kept) or recomputed (tag gone).
    fn tag(cache: &mut Option<super::SearchCache>) {
        if let Some(tree) = cache.as_mut().and_then(|c| c.tree.as_mut()) {
            tree.name = "tagged".into();
        }
    }

    fn name(result: Option<&VariableTreeGroup>) -> Option<&str> {
        result.map(|t| t.name.as_str())
    }

    #[test]
    fn reuses_results_until_query_or_dataset_changes() {
        let (vars, tree) = data();
        let mut cache = None;
        assert!(filtered(&mut cache, 1, "sst", &tree, &vars).is_some());
        tag(&mut cache);
        let again = filtered(&mut cache, 1, "sst", &tree, &vars);
        assert_eq!(
            name(again),
            Some("tagged"),
            "same query and dataset reuse it"
        );

        tag(&mut cache);
        let lai = filtered(&mut cache, 1, "lai", &tree, &vars);
        assert_ne!(name(lai), Some("tagged"), "a new query recomputes");
        assert_eq!(lai.map(|t| t.total_variable_count()), Some(1));

        tag(&mut cache);
        let reloaded = filtered(&mut cache, 2, "lai", &tree, &vars);
        assert_ne!(name(reloaded), Some("tagged"), "a new dataset recomputes");

        assert!(filtered(&mut cache, 2, "nothing", &tree, &vars).is_none());
    }
}
