pub mod colormap;
pub mod diagnostics;
pub mod error;
pub mod executor;
#[cfg(test)]
mod executor_tests;
pub mod grid;
pub mod grid_flips;
pub mod math;
pub mod metadata;
pub mod path;
pub mod remote;
pub mod stack_str;
pub mod units;

pub use path::{expand_tilde, expand_tilde_str, infer_store_kind_from_target};
pub use remote::{ParsedStorageUrl, parse_remote_storage_url};
pub use stack_str::stack_str;

// Format-agnostic & domain re-exports
pub use crate::data::backends::coord_bounds::{
    fetch_all_dimension_coordinates, fetch_all_dimension_coordinates_for_variables,
};
#[cfg(not(target_arch = "wasm32"))]
pub use crate::data::backends::icechunk::build_sync_icechunk_store;
pub use crate::data::backends::zarr::build_sync_store;
pub use error::OctantError;
pub use executor::TaskExecutor;
#[cfg(not(target_arch = "wasm32"))]
pub use executor::{TokioBlockOn, get_shared_tokio_rt};
pub use grid::check_and_orient_axes_with_coords;
pub use math::{
    apply_zoom_pan_at_point, calculate_3d_depth, compute_finite_min_max, ease_in_out_cubic, lerp3,
    xorshift64_f32,
};
pub use metadata::{
    default_dimension_names_for_rank, discover_arrays_via_http_metadata, extract_store_variables,
    resolve_array_dimension_names, variable_info_from_array, variable_info_from_node_metadata,
};

pub use units::{
    ByteSize, add_days_to_date, calculate_variable_size_bytes, data_type_bytes, format_axis_value,
    format_byte_size, format_count_metric, parse_loc, parse_reference_date, parse_time_unit,
    unit_to_milliseconds,
};

/// Convenience alias for format-agnostic metadata extraction
pub use metadata::extract_store_variables as extract_store_variables_consolidated;
