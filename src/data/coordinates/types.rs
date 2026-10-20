//! Core coordinate grid representations and mapping functions.

use super::healpix::HealpixOrder;
use super::topology::GridTopology;
use crate::plots::PlotType;
use std::sync::Arc;

/// Represents the coordinate grid configuration for a 2D scalar field slice.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum CoordinateGrid {
    /// Full global sphere: longitude spans ~360° and latitude spans ~180°.
    #[default]
    GlobalRegular,

    /// Regular regional bounding box with uniform step size.
    RegionalRegular {
        /// (lon_min, lon_max) in degrees
        lon_bounds: (f32, f32),
        /// (lat_min, lat_max) in degrees
        lat_bounds: (f32, f32),
    },

    /// 1D non-uniform coordinate arrays along X and Y axes (e.g. Gaussian latitude grids, stretched grids).
    Irregular1D {
        /// Non-uniform coordinates along X (e.g. longitude in degrees, length = width)
        coords_x: Arc<[f32]>,
        /// Non-uniform coordinates along Y (e.g. latitude in degrees, length = height)
        coords_y: Arc<[f32]>,
        lon_bounds: (f32, f32),
        lat_bounds: (f32, f32),
    },

    /// 2D curvilinear coordinates (lon(y,x), lat(y,x)) of shape (height * width).
    Curvilinear2D {
        lons: Arc<[f32]>,
        lats: Arc<[f32]>,
        lon_bounds: (f32, f32),
        lat_bounds: (f32, f32),
    },

    /// HEALPix (Hierarchical Equal Area isoLatitude Pixelation) discrete global grid.
    Healpix {
        nside: usize,
        ordering: HealpixOrder,
        npix: usize,
        coords_lon: Option<Arc<[f32]>>,
        coords_lat: Option<Arc<[f32]>>,
    },
}

impl CoordinateGrid {
    #[inline]
    pub fn name(&self) -> &'static str {
        GridTopology::name(self)
    }

    #[inline]
    pub fn render_coord_mode(&self) -> u32 {
        self.shader_coord_mode()
    }

    /// The coordinate mode the sphere draws with: regional regular grids
    /// whose bounds are no degrees (projected coordinates, in metres) wrap
    /// the whole globe as global ones do instead of a sliver past the poles.
    pub fn sphere_coord_mode(&self) -> u32 {
        let (lon, lat) = (self.lon_bounds_deg(), self.lat_bounds_deg());
        let degrees = [lon.0, lon.1].iter().all(|v| (-360.0..=360.0).contains(v))
            && [lat.0, lat.1].iter().all(|v| (-90.0..=90.0).contains(v));
        match self.render_coord_mode() {
            1 if !degrees => 0,
            mode => mode,
        }
    }

    #[inline]
    pub fn is_healpix(&self) -> bool {
        matches!(self, Self::Healpix { .. })
    }

    #[inline]
    pub fn has_1d_coords(&self) -> bool {
        matches!(self, Self::Irregular1D { .. })
    }

    #[inline]
    pub fn requires_geo_coords(&self) -> bool {
        GridTopology::requires_geo_coords(self)
    }

    #[inline]
    pub fn lon_bounds_rad(&self) -> [f32; 2] {
        GridTopology::lon_bounds_rad(self)
    }

    #[inline]
    pub fn lat_bounds_rad(&self) -> [f32; 2] {
        GridTopology::lat_bounds_rad(self)
    }

    #[inline]
    pub fn lon_bounds_deg(&self) -> (f32, f32) {
        GridTopology::lon_bounds_deg(self)
    }

    #[inline]
    pub fn lat_bounds_deg(&self) -> (f32, f32) {
        GridTopology::lat_bounds_deg(self)
    }

    #[inline]
    pub fn is_global(&self) -> bool {
        GridTopology::is_global(self)
    }

    #[inline]
    pub fn is_global_extent(&self) -> bool {
        GridTopology::is_global_extent(self)
    }

    #[inline]
    pub fn cell_center_surface_xz(
        &self,
        px: usize,
        py: usize,
        width: usize,
        height: usize,
        aspect: f32,
    ) -> (f32, f32) {
        GridTopology::cell_center_surface_xz(self, px, py, width, height, aspect)
    }

    #[inline]
    pub fn cell_center_lon_lat_rad(
        &self,
        px: usize,
        py: usize,
        width: usize,
        height: usize,
    ) -> (f32, f32) {
        GridTopology::cell_center_lon_lat_rad(self, px, py, width, height)
    }

    #[inline]
    pub fn cell_center_norm(
        &self,
        px: usize,
        py: usize,
        width: usize,
        height: usize,
    ) -> (f32, f32) {
        GridTopology::cell_center_norm(self, px, py, width, height)
    }

    #[inline]
    pub fn spatial_rank(&self) -> usize {
        GridTopology::spatial_rank(self)
    }

    #[inline]
    pub fn supported_plot_types(&self) -> &'static [PlotType] {
        GridTopology::supported_plot_types(self)
    }

    #[inline]
    pub fn data_aspect_ratio(&self, width: usize, height: usize) -> f32 {
        GridTopology::data_aspect_ratio(self, width, height)
    }

    #[inline]
    pub fn same_geometry(&self, other: &Self) -> bool {
        super::same_geometry::is_same_geometry(self, other)
    }

    pub fn coords_x(&self) -> Option<&[f32]> {
        match self {
            Self::Irregular1D { coords_x, .. } => Some(coords_x),
            _ => None,
        }
    }

    pub fn coords_y(&self) -> Option<&[f32]> {
        match self {
            Self::Irregular1D { coords_y, .. } => Some(coords_y),
            _ => None,
        }
    }

    pub fn find_cell_from_norm(
        &self,
        norm_x: f32,
        norm_y: f32,
        width: usize,
        height: usize,
    ) -> (usize, usize) {
        self.norm_to_cell(norm_x, norm_y, width, height)
    }

    pub fn find_cell_from_surface_uv(
        &self,
        u: f32,
        v: f32,
        width: usize,
        height: usize,
    ) -> (usize, usize) {
        self.surface_uv_to_cell(u, v, width, height)
    }

    pub fn find_cell_from_lon_lat_rad(
        &self,
        lon_rad: f32,
        lat_rad: f32,
        width: usize,
        height: usize,
    ) -> Option<(usize, usize)> {
        self.lon_lat_to_cell(lon_rad, lat_rad, width, height)
    }

    pub fn detect_grid(
        x_name: &str,
        y_name: &str,
        x_coords: Option<&[f64]>,
        y_coords: Option<&[f64]>,
        width: usize,
        height: usize,
    ) -> Self {
        super::detection::detect_grid(x_name, y_name, x_coords, y_coords, width, height)
    }

    pub fn detect_grid_from_block(
        block: &crate::data::OctantBlock,
        x_name: &str,
        y_name: &str,
        x_coords: Option<&[f64]>,
        y_coords: Option<&[f64]>,
        width: usize,
        height: usize,
    ) -> Self {
        super::detection::detect_grid_from_block(
            block, x_name, y_name, x_coords, y_coords, width, height,
        )
    }
}
