pub mod colormap;
pub mod coordinates;
pub mod error;
pub mod executor;
pub mod grid;
pub mod metadata;
pub mod units;

// Format-agnostic & domain re-exports
pub use crate::data::backends::icechunk_storage::build_sync_icechunk_store;
pub use crate::data::backends::zarr_storage::build_sync_store;
pub use coordinates::fetch_all_dimension_coordinates;
pub use error::OctantError;
pub use executor::TaskExecutor;
#[cfg(not(target_arch = "wasm32"))]
pub use executor::{TokioBlockOn, get_shared_tokio_rt};
pub use grid::check_and_orient_axes_with_coords;
pub use metadata::{
    discover_arrays_via_http_metadata, extract_store_variables, variable_info_from_array,
};
pub use units::{
    add_days_to_date, calculate_variable_size_bytes, format_axis_value, parse_loc,
    parse_reference_date, parse_time_unit, unit_to_milliseconds,
};

/// Convenience alias for format-agnostic metadata extraction
pub use metadata::extract_store_variables as extract_store_variables_consolidated;
