pub mod colormap;
/// Development-only log helpers, called by hand while debugging; kept though nothing calls them.
pub mod diagnostics;
/// Future work: the typed error meant to replace `BlockStoreError` (`Box<dyn Error>`) and
/// `Result<_, String>`; not used yet.
pub mod error;
pub mod executor;
#[cfg(test)]
mod executor_tests;
pub mod grid;
pub mod grid_flips;
pub mod height;
pub mod math;
pub mod metadata;
pub mod path;
pub mod remote;
pub mod stack_str;
pub mod units;

pub use path::{expand_tilde, expand_tilde_str, infer_store_kind_from_target};
pub use stack_str::stack_str;

pub use math::{
    apply_zoom_pan_at_point, compute_finite_min_max, ease_in_out_cubic, lerp3, scroll_zoom_factor,
    xorshift64_f32,
};
pub use metadata::resolve_array_dimension_names;

pub use units::{
    ByteSize, calculate_variable_size_bytes, data_type_bytes, format_byte_size, format_count_metric,
};

/// Convenience alias for format-agnostic metadata extraction
pub use metadata::extract_store_variables as extract_store_variables_consolidated;
