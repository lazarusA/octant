//! Core coordinate grid representations and mapping functions.

use super::detection::normalize_lon_deg;
use super::search::{find_coord_cell_1d, find_curvilinear_cell_2d};
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
        /// `true` when the i-axis increases westward (grid has reversed handedness).
        flip_i: bool,
        /// `true` when the last column wraps around to the first (periodic in i).
        is_periodic_i: bool,
    },
}

impl CoordinateGrid {
    /// Returns the coordinate mode identifier for GPU shaders:
    /// 0 = GlobalRegular, 1 = RegionalRegular, 2 = Irregular1D, 3 = Curvilinear2D.
    #[inline]
    pub fn coord_mode(&self) -> u32 {
        match self {
            Self::GlobalRegular => 0,
            Self::RegionalRegular { .. } => 1,
            Self::Irregular1D { .. } => 2,
            Self::Curvilinear2D { .. } => 3,
        }
    }

    /// Returns `[flip_i as u32, is_periodic_i as u32]` for `Curvilinear2D` grids,
    /// or `[0, 0]` for all other grid types.
    #[inline]
    pub fn curvilinear_flags(&self) -> [u32; 2] {
        match self {
            Self::Curvilinear2D { flip_i, is_periodic_i, .. } => {
                [*flip_i as u32, *is_periodic_i as u32]
            }
            _ => [0, 0],
        }
    }

    /// Returns `true` if this grid spans the full global extent (~360° lon, ~180° lat).
    #[inline]
    pub fn is_global(&self) -> bool {
        match self {
            Self::GlobalRegular => true,
            Self::RegionalRegular { .. } => false,
            Self::Irregular1D {
                lon_bounds,
                lat_bounds,
                ..
            }
            | Self::Curvilinear2D {
                lon_bounds,
                lat_bounds,
                ..
            } => {
                let x_span = (lon_bounds.1 - lon_bounds.0).abs();
                let y_span = (lat_bounds.1 - lat_bounds.0).abs();
                x_span >= 350.0 && y_span >= 160.0
            }
        }
    }

    /// Returns the longitude bounds [lon_min, lon_max] in radians.
    pub fn lon_bounds_rad(&self) -> [f32; 2] {
        match self {
            Self::GlobalRegular => [-std::f32::consts::PI, std::f32::consts::PI],
            Self::RegionalRegular { lon_bounds, .. }
            | Self::Irregular1D { lon_bounds, .. }
            | Self::Curvilinear2D { lon_bounds, .. } => {
                let min_rad = normalize_lon_deg(lon_bounds.0).to_radians();
                let max_rad = normalize_lon_deg(lon_bounds.1).to_radians();
                [min_rad, max_rad]
            }
        }
    }

    /// Returns the latitude bounds [lat_min, lat_max] in radians.
    pub fn lat_bounds_rad(&self) -> [f32; 2] {
        match self {
            Self::GlobalRegular => [-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2],
            Self::RegionalRegular { lat_bounds, .. }
            | Self::Irregular1D { lat_bounds, .. }
            | Self::Curvilinear2D { lat_bounds, .. } => {
                let min_rad = lat_bounds.0.clamp(-90.0, 90.0).to_radians();
                let max_rad = lat_bounds.1.clamp(-90.0, 90.0).to_radians();
                [min_rad, max_rad]
            }
        }
    }

    /// Returns a reference to 1D X-coordinates if this grid is Irregular1D.
    pub fn coords_x(&self) -> Option<&[f32]> {
        match self {
            Self::Irregular1D { coords_x, .. } => Some(coords_x),
            _ => None,
        }
    }

    /// Returns a reference to 1D Y-coordinates if this grid is Irregular1D.
    pub fn coords_y(&self) -> Option<&[f32]> {
        match self {
            Self::Irregular1D { coords_y, .. } => Some(coords_y),
            _ => None,
        }
    }

    /// Returns the GPU X/lon coordinate buffer slice (1D coords for Irregular1D, flattened lons for Curvilinear2D).
    pub fn gpu_coords_x(&self) -> Option<&[f32]> {
        match self {
            Self::Irregular1D { coords_x, .. } => Some(coords_x),
            Self::Curvilinear2D { lons, .. } => Some(lons),
            _ => None,
        }
    }

    /// Returns the GPU Y/lat coordinate buffer slice (1D coords for Irregular1D, flattened lats for Curvilinear2D).
    pub fn gpu_coords_y(&self) -> Option<&[f32]> {
        match self {
            Self::Irregular1D { coords_y, .. } => Some(coords_y),
            Self::Curvilinear2D { lats, .. } => Some(lats),
            _ => None,
        }
    }

    /// Returns a reference to 2D longitudes if this grid is Curvilinear2D.
    pub fn lons_2d(&self) -> Option<&[f32]> {
        match self {
            Self::Curvilinear2D { lons, .. } => Some(lons),
            _ => None,
        }
    }

    /// Returns a reference to 2D latitudes if this grid is Curvilinear2D.
    pub fn lats_2d(&self) -> Option<&[f32]> {
        match self {
            Self::Curvilinear2D { lats, .. } => Some(lats),
            _ => None,
        }
    }

    /// Maps normalized `[0, 1]` viewport coordinates `(norm_x, norm_y)` to pixel cell indices `(px, py)`.
    pub fn find_cell_from_norm(
        &self,
        norm_x: f32,
        norm_y: f32,
        width: usize,
        height: usize,
    ) -> (usize, usize) {
        let w = width.max(1);
        let h = height.max(1);
        let nx = norm_x.clamp(0.0, 1.0);
        let ny = norm_y.clamp(0.0, 1.0);

        match self {
            Self::Irregular1D {
                coords_x, coords_y, ..
            } => {
                let px = if coords_x.len() >= 2 {
                    let first_x = coords_x[0];
                    let last_x = coords_x[coords_x.len() - 1];
                    let target_x = first_x + nx * (last_x - first_x);
                    find_coord_cell_1d(coords_x, target_x)
                } else {
                    ((nx * w as f32).floor() as usize).min(w.saturating_sub(1))
                };

                let py = if coords_y.len() >= 2 {
                    let first_y = coords_y[0];
                    let last_y = coords_y[coords_y.len() - 1];
                    let target_y = first_y + ny * (last_y - first_y);
                    find_coord_cell_1d(coords_y, target_y)
                } else {
                    ((ny * h as f32).floor() as usize).min(h.saturating_sub(1))
                };

                (px.min(w.saturating_sub(1)), py.min(h.saturating_sub(1)))
            }
            _ => {
                let px = ((nx * w as f32).floor() as usize).min(w.saturating_sub(1));
                let py = ((ny * h as f32).floor() as usize).min(h.saturating_sub(1));
                (px, py)
            }
        }
    }

    /// Maps geographic spherical coordinates `(lon_rad, lat_rad)` to cell indices `(px, py)`.
    pub fn find_cell_from_lon_lat_rad(
        &self,
        lon_rad: f32,
        lat_rad: f32,
        width: usize,
        height: usize,
    ) -> Option<(usize, usize)> {
        let w = width.max(1);
        let h = height.max(1);

        match self {
            Self::GlobalRegular => {
                let u = ((lon_rad + std::f32::consts::PI) / (2.0 * std::f32::consts::PI))
                    .clamp(0.0, 1.0);
                let v = (0.5 - (lat_rad / std::f32::consts::PI)).clamp(0.0, 1.0);
                let px = ((u * w as f32).floor() as usize).min(w.saturating_sub(1));
                let py = ((v * h as f32).floor() as usize).min(h.saturating_sub(1));
                Some((px, py))
            }
            Self::RegionalRegular { .. } => {
                let [lon_min, lon_max] = self.lon_bounds_rad();
                let [lat_min, lat_max] = self.lat_bounds_rad();

                if lon_rad < lon_min - 0.05
                    || lon_rad > lon_max + 0.05
                    || lat_rad < lat_min - 0.05
                    || lat_rad > lat_max + 0.05
                {
                    return None;
                }

                let span_lon = (lon_max - lon_min).abs().max(1e-6);
                let span_lat = (lat_max - lat_min).abs().max(1e-6);

                let u = ((lon_rad - lon_min) / span_lon).clamp(0.0, 1.0);
                let v = ((lat_max - lat_rad) / span_lat).clamp(0.0, 1.0);

                let px = ((u * w as f32).floor() as usize).min(w.saturating_sub(1));
                let py = ((v * h as f32).floor() as usize).min(h.saturating_sub(1));
                Some((px, py))
            }
            Self::Irregular1D {
                coords_x, coords_y, ..
            } => {
                let lon_deg = lon_rad.to_degrees();
                let lat_deg = lat_rad.to_degrees();

                let px = if coords_x.len() >= 2 {
                    find_coord_cell_1d(coords_x, lon_deg)
                } else {
                    let u = ((lon_rad + std::f32::consts::PI) / (2.0 * std::f32::consts::PI))
                        .clamp(0.0, 1.0);
                    ((u * w as f32).floor() as usize).min(w.saturating_sub(1))
                };

                let py = if coords_y.len() >= 2 {
                    find_coord_cell_1d(coords_y, lat_deg)
                } else {
                    let v = (0.5 - (lat_rad / std::f32::consts::PI)).clamp(0.0, 1.0);
                    ((v * h as f32).floor() as usize).min(h.saturating_sub(1))
                };

                Some((px.min(w.saturating_sub(1)), py.min(h.saturating_sub(1))))
            }
            Self::Curvilinear2D { lons, lats, .. } => {
                let (px, py) = find_curvilinear_cell_2d(lons, lats, lon_rad, lat_rad, w, h);
                Some((px.min(w.saturating_sub(1)), py.min(h.saturating_sub(1))))
            }
        }
    }

    /// Computes the normalized `[0, 1]` viewport coordinates `(u_c, v_c)` for the exact center of cell `(px, py)`.
    pub fn cell_center_norm(
        &self,
        px: usize,
        py: usize,
        width: usize,
        height: usize,
    ) -> (f32, f32) {
        let w = width.max(1);
        let h = height.max(1);

        match self {
            Self::Irregular1D {
                coords_x, coords_y, ..
            } => {
                let u_c = if coords_x.len() >= 2 {
                    let first_x = coords_x[0];
                    let last_x = coords_x[coords_x.len() - 1];
                    let span_x = if (last_x - first_x).abs() > 1e-6 {
                        last_x - first_x
                    } else {
                        1.0
                    };
                    let cur_x = coords_x.get(px).copied().unwrap_or(first_x);
                    ((cur_x - first_x) / span_x).clamp(0.0, 1.0)
                } else {
                    (px as f32 + 0.5) / w as f32
                };

                let v_c = if coords_y.len() >= 2 {
                    let first_y = coords_y[0];
                    let last_y = coords_y[coords_y.len() - 1];
                    let span_y = if (last_y - first_y).abs() > 1e-6 {
                        last_y - first_y
                    } else {
                        1.0
                    };
                    let cur_y = coords_y.get(py).copied().unwrap_or(first_y);
                    ((cur_y - first_y) / span_y).clamp(0.0, 1.0)
                } else {
                    (py as f32 + 0.5) / h as f32
                };

                (u_c, v_c)
            }
            _ => {
                let u_c = (px as f32 + 0.5) / w as f32;
                let v_c = (py as f32 + 0.5) / h as f32;
                (u_c, v_c)
            }
        }
    }

    /// Computes the exact geographic longitude and latitude `(lon_rad, lat_rad)` in radians for the center of cell `(px, py)`.
    pub fn cell_center_lon_lat_rad(
        &self,
        px: usize,
        py: usize,
        width: usize,
        height: usize,
    ) -> (f32, f32) {
        let w = width.max(1);
        let h = height.max(1);

        match self {
            Self::GlobalRegular => {
                let u_c = (px as f32 + 0.5) / w as f32;
                let v_c = (py as f32 + 0.5) / h as f32;
                let lon_rad = (u_c - 0.5) * 2.0 * std::f32::consts::PI;
                let lat_rad = (0.5 - v_c) * std::f32::consts::PI;
                (lon_rad, lat_rad)
            }
            Self::RegionalRegular { .. } => {
                let [lon_min, lon_max] = self.lon_bounds_rad();
                let [lat_min, lat_max] = self.lat_bounds_rad();
                let u_c = (px as f32 + 0.5) / w as f32;
                let v_c = (py as f32 + 0.5) / h as f32;
                let lon_rad = lon_min + u_c * (lon_max - lon_min);
                let lat_rad = lat_max - v_c * (lat_max - lat_min);
                (lon_rad, lat_rad)
            }
            Self::Irregular1D {
                coords_x, coords_y, ..
            } => {
                let deg_lon = coords_x.get(px).copied().unwrap_or(0.0);
                let deg_lat = coords_y.get(py).copied().unwrap_or(0.0);
                (deg_lon.to_radians(), deg_lat.to_radians())
            }
            Self::Curvilinear2D { lons, lats, .. } => {
                let idx = py * w + px;
                let deg_lon = lons.get(idx).copied().unwrap_or(0.0);
                let deg_lat = lats.get(idx).copied().unwrap_or(0.0);
                (deg_lon.to_radians(), deg_lat.to_radians())
            }
        }
    }

    /// Computes the 3D surface model space `(world_x, world_z)` coordinates for the center of cell `(px, py)` matching `surface.wgsl`.
    pub fn cell_center_surface_xz(
        &self,
        px: usize,
        py: usize,
        width: usize,
        height: usize,
        data_aspect: f32,
    ) -> (f32, f32) {
        let (u_c, v_c) = self.cell_center_norm(px, py, width, height);
        let world_x = (2.0 * u_c - 1.0) * data_aspect;
        let world_z = 2.0 * v_c - 1.0;
        (world_x, world_z)
    }

    /// Automatically classifies and constructs a `CoordinateGrid` from dimension coordinate arrays.
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

    /// Automatically classifies and constructs a `CoordinateGrid`, prioritizing 2D curvilinear coordinates.
    pub fn detect_curvilinear_grid(
        x_name: &str,
        y_name: &str,
        x_coords: Option<&[f64]>,
        y_coords: Option<&[f64]>,
        curvilinear_coords: &std::collections::HashMap<String, crate::data::CurvilinearCoord2D>,
        width: usize,
        height: usize,
    ) -> Self {
        super::detection::detect_curvilinear_grid(
            x_name,
            y_name,
            x_coords,
            y_coords,
            curvilinear_coords,
            width,
            height,
        )
    }
}
