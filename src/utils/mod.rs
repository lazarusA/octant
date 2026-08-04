pub mod colormap;
pub mod grid;
pub mod icechunk;
pub mod units;
pub mod zarr;
pub mod zarr_metadata;

// Re-exports for convenience
pub use grid::{check_and_orient_axes, check_and_orient_axes_with_coords};
pub use icechunk::build_sync_icechunk_store;
pub use units::{
    add_days_to_date, calculate_variable_size_bytes, format_axis_value, parse_loc,
    parse_reference_date, parse_time_unit, unit_to_milliseconds,
};
pub use zarr::{
    build_sync_store, discover_arrays_via_metadata, extract_store_variables,
    fetch_all_dimension_coordinates, fetch_slice, fetch_slice_range,
};
pub use zarr_metadata::extract_store_variables_consolidated;
