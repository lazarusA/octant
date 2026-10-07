//! Coordinates read from chunks the browser preloads into memory: a chunk that failed to
//! download reads as the fill value there, so such coordinates are settled afterwards.

use std::collections::HashMap;

use crate::data::CoordValues;

/// How completely a coordinate's chunks were fetched into memory. A missing chunk (HTTP
/// 404) counts as fetched: it legitimately reads as the fill value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoordPreload {
    Complete,
    /// The first and last chunks arrived, some middle one did not.
    Partial,
    /// The first or last chunk did not arrive.
    Failed,
}

/// Settles the coordinates of incompletely fetched coordinate arrays (`(path, state)`,
/// e.g. `("ocean/depth", Partial)`): every key ending in the array's name keeps only its
/// endpoints when the middle is missing, and is dropped when an endpoint is missing.
pub fn settle_preloaded_coordinates(
    coords: &mut HashMap<String, CoordValues>,
    preloads: &[(String, CoordPreload)],
) {
    for (path, state) in preloads {
        let base = path.rsplit('/').next().unwrap_or(path).trim();
        let named = |key: &str| {
            key.rsplit('/')
                .next()
                .is_some_and(|k| k.eq_ignore_ascii_case(base))
        };
        match state {
            CoordPreload::Complete => {}
            CoordPreload::Failed => coords.retain(|key, _| !named(key)),
            CoordPreload::Partial => {
                for (key, values) in coords.iter_mut() {
                    if named(key)
                        && let Some(ends) = values.to_endpoints()
                    {
                        *values = ends;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn incomplete_coordinates_keep_their_ends_or_are_dropped() {
        let depth = CoordValues::from_values(vec![0.0, 10.0, 50.0], false).expect("depth");
        let lat = CoordValues::from_values(vec![-1.0, 1.0], false).expect("lat");
        let mut coords = HashMap::from([
            ("depth".to_string(), depth.clone()),
            ("ocean/temp/depth".to_string(), depth),
            ("lat".to_string(), lat.clone()),
            ("time".to_string(), lat),
        ]);
        settle_preloaded_coordinates(
            &mut coords,
            &[
                ("ocean/depth".into(), CoordPreload::Partial),
                ("lat".into(), CoordPreload::Failed),
                ("time".into(), CoordPreload::Complete),
            ],
        );
        let ends = CoordValues::Endpoints {
            first: 0.0,
            last: 50.0,
            len: 3,
        };
        assert_eq!(coords.get("depth"), Some(&ends));
        assert_eq!(coords.get("ocean/temp/depth"), Some(&ends));
        assert!(!coords.contains_key("lat"));
        assert!(coords.get("time").is_some_and(CoordValues::is_exact));
    }
}
