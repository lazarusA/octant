//! Geographic and Cartesian coordinate mapping system.

pub mod affine;
#[cfg(test)]
mod affine_tests;
pub mod detection;
pub mod dggs;
pub mod healpix;
mod impl_topology;
pub mod lut;
pub mod naming;
pub mod ordering;
pub mod same_geometry;
pub mod search;
#[cfg(test)]
mod tests;
pub mod topologies;
pub mod topology;
pub mod types;

pub use detection::{detect_grid, detect_grid_from_block};
pub use dggs::{DggsEllipsoid, DggsMetadata};
pub use healpix::{
    HealpixOrder, ang2pix_ring, npix_to_nside, nside_to_npix, pix_boundaries, pix2ang_ring,
    pix2ring,
};
pub use lut::{build_1d_coord_lut, compute_coord_lut_size};
pub use naming::{
    contains_ascii_case_insensitive, format_dimension_axis_title, is_animated_time_name,
    is_healpix_dim_name, is_spatial_x_name, is_spatial_y_name, is_spatial_z_name,
};
pub use ordering::{detect_healpix_ordering, detect_healpix_ordering_with_dggs};
pub use same_geometry::is_same_geometry;
pub use search::find_coord_cell_1d;
pub use topologies::{
    CartesianTopology, CurvilinearTopology, HealpixTopology, Irregular1DTopology,
};
pub use topology::GridTopology;
pub use types::CoordinateGrid;
