//! GRIB2 files, read message by message through gribberish: scanned from each message's
//! head (`scan.rs`), laid out as variables (`layout.rs`, `axes.rs`, `grid.rs`, `kind.rs`)
//! and read by the NetCDF backend through its reader seam (`reader.rs`), on the desktop
//! and in the browser's data workers, for local and remote files alike.

mod axes;
#[cfg(all(test, not(target_arch = "wasm32")))]
mod dev_files_tests;
mod entry;
mod grid;
#[cfg(test)]
mod grid_tests;
mod kind;
mod layout;
#[cfg(all(test, not(target_arch = "wasm32")))]
mod network_tests;
mod reader;
mod scan;
mod scanning;
mod sections;
#[cfg(not(target_arch = "wasm32"))]
mod sidecar;
#[cfg(all(test, not(target_arch = "wasm32")))]
mod store_tests;
#[cfg(test)]
pub mod test_files;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod time_tests;

pub use reader::GribReader;
pub use scan::{SIGNATURE_BYTES, is_grib};
#[cfg(not(target_arch = "wasm32"))]
pub use sidecar::sidecar_offsets;

/// A request for the `y`, `x` plane of `var` (shaped `shape`) at `lead` along its leading
/// dimensions, for the tests reading real stores.
#[cfg(test)]
pub(crate) fn plane_request(
    var: &str,
    shape: &[usize],
    lead: &[usize],
) -> crate::data::slice_request::SliceRequest {
    use crate::data::slice_request::{DimensionSelection, SliceRequest};
    let plane = shape
        .iter()
        .skip(lead.len())
        .map(|&n| DimensionSelection::range(0, n));
    let selections = lead
        .iter()
        .map(|&i| DimensionSelection::index(i))
        .chain(plane);
    SliceRequest::new(var, selections.collect())
}
