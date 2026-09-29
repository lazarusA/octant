//! Candidate coordinate array and dimension name discovery heuristics.

use crate::data::VariableInfo;

/// Collects candidate 1D coordinate array names from dataset variables, dimension names, and standard aliases.
pub fn collect_coordinate_candidates(variables: &[VariableInfo]) -> Vec<String> {
    let mut coord_candidates = Vec::new();

    // 1. All 1D variables in the store
    for var in variables {
        if var.shape.len() == 1 && var.shape.first().copied().unwrap_or(0) > 0 {
            let clean = var.name.trim().trim_start_matches('/').to_string();
            if !coord_candidates.contains(&clean) {
                coord_candidates.push(clean);
            }
        }
    }

    // 2. All dimension names declared in multidimensional variables
    for var in variables {
        for dim in &var.dimension_names {
            let clean = dim.trim().trim_start_matches('/').to_string();
            if !clean.is_empty() && !coord_candidates.contains(&clean) {
                coord_candidates.push(clean);
            }
        }
        if let Some(dggs) =
            crate::data::coordinates::dggs::DggsMetadata::from_attributes(&var.attributes)
            && let Some(ref coord_name) = dggs.coordinate
        {
            let clean = coord_name.trim().trim_start_matches('/').to_string();
            if !clean.is_empty() && !coord_candidates.contains(&clean) {
                coord_candidates.push(clean);
            }
        }
    }

    // 3. Standard fallback spatial & temporal coordinate aliases
    for fallback in &[
        "lat",
        "latitude",
        "y",
        "lon",
        "longitude",
        "x",
        "time",
        "ti",
        "date",
        "datetime",
        "time_counter",
        "valid_time",
        "leadtime",
        "step",
        "depth",
        "lev",
        "level",
        "height",
    ] {
        let s = fallback.to_string();
        if !coord_candidates.contains(&s) {
            coord_candidates.push(s);
        }
    }

    coord_candidates
}
