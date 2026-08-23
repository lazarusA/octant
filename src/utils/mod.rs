pub mod colormap;
pub mod coordinates;
pub mod diagnostics;
pub mod error;
pub mod executor;
pub mod grid;
pub mod math;
pub mod metadata;
pub mod units;

// Format-agnostic & domain re-exports
pub use crate::data::backends::icechunk_storage::build_sync_icechunk_store;
pub use crate::data::backends::zarr_storage::build_sync_store;
pub use coordinates::fetch_all_dimension_coordinates;
pub use error::OctantError;
pub use executor::{TaskExecutor, TokioBlockOn, get_shared_tokio_rt};
pub use grid::check_and_orient_axes_with_coords;
pub use math::{compute_finite_min_max, ease_in_out_cubic, lerp3, xorshift64_f32};
pub use metadata::{
    default_dimension_names_for_rank, discover_arrays_via_http_metadata, extract_store_variables,
    variable_info_from_array,
};
pub use units::{
    add_days_to_date, calculate_variable_size_bytes, data_type_bytes, format_axis_value,
    format_byte_size, format_count_metric, parse_loc, parse_reference_date, parse_time_unit,
    unit_to_milliseconds,
};

/// Convenience alias for format-agnostic metadata extraction
pub use metadata::extract_store_variables as extract_store_variables_consolidated;
