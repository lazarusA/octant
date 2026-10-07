//! Candidate coordinate array and dimension name discovery heuristics.

use crate::data::VariableInfo;
use crate::data::slice_request::DimensionSelection;

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

/// The coordinate arrays one variable's dimensions may be read from, spatial ones first so
/// map axes arrive before a long time axis: its dimension names, its DGGS cell coordinate,
/// and the spatial aliases (`lat`, `longitude`, ...) when a dimension is spatial or generic
/// (`dim_N`, resolved through them).
pub fn variable_coordinate_candidates(variable: &VariableInfo) -> Vec<String> {
    coordinate_candidates(&variable.dimension_names, variable)
}

/// The coordinate arrays a block of `variable` reads: those of the dimensions selected as a
/// range (fixed indices need none in the block), as [`variable_coordinate_candidates`].
pub fn block_coordinate_candidates(
    variable: &VariableInfo,
    selections: &[DimensionSelection],
) -> Vec<String> {
    let ranged: Vec<String> = variable
        .dimension_names
        .iter()
        .zip(selections)
        .filter(|(_, sel)| matches!(sel, DimensionSelection::Range { .. }))
        .map(|(name, _)| name.clone())
        .collect();
    coordinate_candidates(&ranged, variable)
}

fn coordinate_candidates(dims: &[String], variable: &VariableInfo) -> Vec<String> {
    use crate::utils::grid_flips::{is_lat_name, is_lon_name};
    let spatial = |name: &str| is_lat_name(name) || is_lon_name(name);
    let mut names: Vec<String> = Vec::new();
    let mut push = |name: &str| {
        let clean = name.trim().trim_start_matches('/');
        if !clean.is_empty() && !names.iter().any(|n| n == clean) {
            names.push(clean.to_string());
        }
    };
    for dim in dims {
        push(dim);
    }
    if let Some(dggs) =
        crate::data::coordinates::dggs::DggsMetadata::from_attributes(&variable.attributes)
        && let Some(coord) = dggs.coordinate.as_deref()
    {
        push(coord);
    }
    if dims.iter().any(|d| spatial(d) || d.starts_with("dim_")) {
        for alias in ["lat", "latitude", "y", "lon", "longitude", "x"] {
            push(alias);
        }
    }
    // Stable: spatial names keep their order ahead of the rest.
    names.sort_by_key(|n| !spatial(n));
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_variable_reads_its_own_dimensions_spatial_first() {
        let var = VariableInfo {
            name: "t2m".into(),
            dimension_names: ["time", "latitude", "longitude"].map(String::from).to_vec(),
            ..Default::default()
        };
        let names = variable_coordinate_candidates(&var);
        assert_eq!(names[..2], ["latitude", "longitude"]);
        assert_eq!(names.last().map(String::as_str), Some("time"));
        assert!(names.iter().any(|n| n == "lat"), "aliases of spatial axes");
        assert!(
            !names.iter().any(|n| n == "depth"),
            "no other variable's axes"
        );

        let station = VariableInfo {
            dimension_names: vec!["station".into()],
            ..Default::default()
        };
        assert_eq!(variable_coordinate_candidates(&station), ["station"]);

        // A block at one time step reads no time coordinate.
        let sels = [
            DimensionSelection::index(4),
            DimensionSelection::range(0, 10),
            DimensionSelection::range(0, 20),
        ];
        assert!(
            !block_coordinate_candidates(&var, &sels)
                .iter()
                .any(|n| n == "time")
        );
    }
}
