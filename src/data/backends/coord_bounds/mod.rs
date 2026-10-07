//! Zarr storage coordinate bounds retrieval and caching.

pub mod cache;
pub mod candidates;
pub mod discover;
pub mod extract;
pub mod preload;
#[cfg(test)]
mod tests;
pub mod variables;

pub use cache::{
    evict_coord_values, get_cached_coord_bounds, get_cached_coord_bounds_scoped,
    get_cached_coord_bounds_with_rank, get_cached_coord_values_scoped,
    get_cached_coord_values_with_rank,
};
pub use candidates::collect_coordinate_candidates;
pub use extract::{
    fetch_all_dimension_coordinates, read_coord_bounds, read_coord_bounds_scoped,
    read_coord_bounds_with_rank, read_coord_values_scoped,
};
pub use preload::{CoordPreload, settle_preloaded_coordinates};
pub use variables::fetch_all_dimension_coordinates_for_variables;
