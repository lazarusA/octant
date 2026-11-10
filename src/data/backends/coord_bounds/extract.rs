//! Coordinate range extraction and subset retrieval from storage arrays.

use zarrs::storage::ReadableWritableListableStorage;

use super::discover::{ReadFailed, RootListing, discover_coord_array};
use crate::data::CoordValues;
use crate::data::coordinates::naming::{is_lat_name, is_lon_name};

/// The coordinate `dim_name` of `group_scope` (else of any group), `None` when the store
/// has none.
pub fn read_coord_values_scoped(
    store: ReadableWritableListableStorage,
    dim_name: &str,
    group_scope: Option<&str>,
    known_groups: &[String],
    consolidated: RootListing<'_>,
) -> Result<Option<CoordValues>, ReadFailed> {
    let candidates = coordinate_paths(dim_name, group_scope, known_groups);
    let Some(array) =
        discover_coord_array(store, consolidated, &candidates, group_scope.is_none())?
    else {
        return Ok(None);
    };
    // A 2D or empty array (curvilinear `lat`), or one neither text nor numbers (bool,
    // datetime64), is no coordinate of the dimension: remembered as absent.
    use crate::data::backends::zarr::{coords::is_text_array, slice::is_numeric};
    let readable = is_text_array(&array) || is_numeric(array.data_type());
    if !readable || !matches!(array.shape(), [len] if *len > 0) {
        return Ok(None);
    }
    crate::data::backends::zarr::read_coordinate(&array)
        .map(Some)
        .ok_or(ReadFailed)
}

/// Where the coordinate of `dim_name` may be, in the order searched: in `group_scope`,
/// at the root, then for latitude and longitude their usual names, then in each of
/// `known_groups`.
fn coordinate_paths(
    dim_name: &str,
    group_scope: Option<&str>,
    known_groups: &[String],
) -> Vec<String> {
    let clean = dim_name.trim().to_lowercase();
    let aliases: &[&str] = if is_lat_name(dim_name) {
        &["lat", "latitude", "y", "coords/lat", "coordinates/latitude"]
    } else if is_lon_name(dim_name) {
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
    let mut candidates = Vec::new();
    let mut add = |p: String| {
        let key = p.trim_start_matches('/').to_string();
        if !key.is_empty() && !candidates.contains(&key) {
            candidates.push(key);
        }
    };
    if let Some(scope) = group_scope {
        add(format!("{scope}/{clean}"));
    }
    add(clean.clone());
    for alias in aliases {
        if let Some(scope) = group_scope {
            add(format!("{scope}/{alias}"));
        }
        add(alias.to_string());
    }
    for group in known_groups {
        add(format!("{group}/{clean}"));
        for alias in aliases {
            add(format!("{group}/{alias}"));
        }
    }
    candidates
}
