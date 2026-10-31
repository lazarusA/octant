//! Coordinate range extraction and subset retrieval from storage arrays.

use zarrs::storage::ReadableWritableListableStorage;

use super::discover::discover_coord_array;
use crate::data::CoordValues;
use zarrs::metadata_ext::group::consolidated_metadata::ConsolidatedMetadataMetadata;

pub fn read_coord_values_scoped(
    store: ReadableWritableListableStorage,
    dim_name: &str,
    group_scope: Option<&str>,
    known_groups: &[String],
    consolidated: Option<&ConsolidatedMetadataMetadata>,
) -> Option<CoordValues> {
    let clean = dim_name.trim().to_lowercase();
    let mut candidates = Vec::new();
    let mut add_candidate = |p: String| {
        let clean_key = p.trim_start_matches('/').to_string();
        if !clean_key.is_empty() && !candidates.contains(&clean_key) {
            candidates.push(clean_key);
        }
    };

    if let Some(scope) = group_scope {
        add_candidate(format!("{}/{}", scope, clean));
    }
    add_candidate(clean.clone());

    let is_lat = clean.contains("lat") || clean == "y";
    let is_lon = clean.contains("lon") || clean == "x";

    let aliases: &[&str] = if is_lat {
        &["lat", "latitude", "y", "coords/lat", "coordinates/latitude"]
    } else if is_lon {
        &[
            "lon",
            "longitude",
            "x",
            "coords/lon",
            "coordinates/longitude",
        ]
    } else {
        &[]
    };

    for alias in aliases {
        if let Some(scope) = group_scope {
            add_candidate(format!("{}/{}", scope, alias));
        }
        add_candidate(alias.to_string());
    }

    for group in known_groups {
        add_candidate(format!("{}/{}", group, clean));
        for alias in aliases {
            add_candidate(format!("{}/{}", group, alias));
        }
    }

    let array = discover_coord_array(store, consolidated, &candidates, group_scope.is_none())?;
    crate::data::backends::zarr::read_coordinate(&array)
}
