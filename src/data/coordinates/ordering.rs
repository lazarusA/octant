//! Grid indexing and space-filling curve ordering detection (HEALPix Ring vs Nested).

use std::collections::HashMap;

use super::dggs::DggsMetadata;
use super::healpix::HealpixOrder;
use super::naming::contains_ascii_case_insensitive;

/// Detects HEALPix ordering scheme with zero heap allocations, reusing already-parsed DGGS metadata if present.
#[inline]
pub fn detect_healpix_ordering_with_dggs(
    attributes: &HashMap<String, String>,
    dggs: Option<&DggsMetadata>,
) -> HealpixOrder {
    if let Some(dggs) = dggs
        && dggs.is_healpix()
    {
        return dggs.healpix_ordering();
    }

    let is_nested = attributes
        .get("healpix_nest")
        .is_some_and(|v| v.eq_ignore_ascii_case("true") || v == "1")
        || attributes
            .get("healpix_order")
            .is_some_and(|v| v.eq_ignore_ascii_case("nested"))
        || attributes
            .get("ordering")
            .is_some_and(|v| v.eq_ignore_ascii_case("nested"))
        || attributes
            .get("grid_type")
            .is_some_and(|v| contains_ascii_case_insensitive(v, "nested"));

    if is_nested {
        HealpixOrder::Nested
    } else {
        HealpixOrder::Ring
    }
}
