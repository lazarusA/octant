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

    /// Maps normalized `[0, 1]` viewport coordinates `(norm_x, norm_y)` to pixel cell indices `(px, py)`.
    /// Accurately accounts for irregular 1D coordinate spacings via binary search matching GPU shaders.
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
    /// Returns `None` if the ray misses a bounded regional sector.
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

                // Check bounds with slight floating-point tolerance
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
            Self::Curvilinear2D { .. } => {
                let u = ((lon_rad + std::f32::consts::PI) / (2.0 * std::f32::consts::PI))
                    .clamp(0.0, 1.0);
                let v = (0.5 - (lat_rad / std::f32::consts::PI)).clamp(0.0, 1.0);
                let px = ((u * w as f32).floor() as usize).min(w.saturating_sub(1));
                let py = ((v * h as f32).floor() as usize).min(h.saturating_sub(1));
                Some((px, py))
            }
        }
    }

    /// Computes the normalized `[0, 1]` viewport coordinates `(u_c, v_c)` corresponding to the exact center of cell `(px, py)`.
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
            Self::Curvilinear2D { .. } => {
                let u_c = (px as f32 + 0.5) / w as f32;
                let v_c = (py as f32 + 0.5) / h as f32;
                let lon_rad = (u_c - 0.5) * 2.0 * std::f32::consts::PI;
                let lat_rad = (0.5 - v_c) * std::f32::consts::PI;
                (lon_rad, lat_rad)
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
        let is_spatial_x = crate::utils::coordinates::is_spatial_x_name(x_name);
        let is_spatial_y = crate::utils::coordinates::is_spatial_y_name(y_name);

        let Some(xc) = x_coords else {
            return Self::GlobalRegular;
        };
        let Some(yc) = y_coords else {
            return Self::GlobalRegular;
        };

        if xc.is_empty() || yc.is_empty() {
            return Self::GlobalRegular;
        }

        let (x_min, x_max) = match (xc.first(), xc.last()) {
            (Some(&f), Some(&l)) => ((f.min(l)) as f32, (f.max(l)) as f32),
            _ => (0.0, width as f32),
        };

        let (y_min, y_max) = match (yc.first(), yc.last()) {
            (Some(&f), Some(&l)) => ((f.min(l)) as f32, (f.max(l)) as f32),
            _ => (0.0, height as f32),
        };

        let x_span = (x_max - x_min).abs();
        let y_span = (y_max - y_min).abs();

        // Check if spatial longitude & latitude span the full global sphere
        let is_global_extent = is_spatial_x && is_spatial_y && x_span >= 350.0 && y_span >= 160.0;

        let x_irregular = is_irregular_series(xc);
        let y_irregular = is_irregular_series(yc);

        if x_irregular || y_irregular {
            let coords_x: Arc<[f32]> = if xc.len() >= width {
                xc.iter().take(width).map(|&v| v as f32).collect()
            } else {
                (0..width)
                    .map(|i| {
                        if width <= 1 {
                            x_min
                        } else {
                            x_min + (i as f32 / (width - 1) as f32) * (x_max - x_min)
                        }
                    })
                    .collect()
            };

            let coords_y: Arc<[f32]> = if yc.len() >= height {
                yc.iter().take(height).map(|&v| v as f32).collect()
            } else {
                (0..height)
                    .map(|i| {
                        if height <= 1 {
                            y_min
                        } else {
                            y_min + (i as f32 / (height - 1) as f32) * (y_max - y_min)
                        }
                    })
                    .collect()
            };

            log::info!(
                "CoordinateGrid: Detected Irregular1D grid (x_irregular={x_irregular}, y_irregular={y_irregular}, w={width}, h={height})"
            );

            Self::Irregular1D {
                coords_x,
                coords_y,
                lon_bounds: (x_min, x_max),
                lat_bounds: (y_min, y_max),
            }
        } else if is_global_extent {
            Self::GlobalRegular
        } else if is_spatial_x || is_spatial_y {
            Self::RegionalRegular {
                lon_bounds: (x_min, x_max),
                lat_bounds: (y_min, y_max),
            }
        } else {
            Self::GlobalRegular
        }
    }
}

/// Binary searches a 1D monotonic (ascending or descending) coordinate array for the nearest cell index.
/// Matches `find_coord_cell_x` and `find_coord_cell_y` in WGSL shaders.
pub fn find_coord_cell_1d(coords: &[f32], query_val: f32) -> usize {
    let len = coords.len();
    if len <= 1 {
        return 0;
    }
    let max_idx = len - 1;
    let first = coords[0];
    let last = coords[max_idx];
    let is_descending = first > last;

    let mut low = 0;
    let mut high = max_idx.saturating_sub(1);

    if is_descending {
        while low < high {
            let mid = (low + high).div_ceil(2);
            if coords[mid] >= query_val {
                low = mid;
            } else {
                high = mid.saturating_sub(1);
            }
        }
    } else {
        while low < high {
            let mid = (low + high).div_ceil(2);
            if coords[mid] <= query_val {
                low = mid;
            } else {
                high = mid.saturating_sub(1);
            }
        }
    }

    let next = (low + 1).min(max_idx);
    if (query_val - coords[low]).abs() <= (query_val - coords[next]).abs() {
        low
    } else {
        next
    }
}

/// Computes the optimal lookup table (LUT) size for a 1D coordinate axis.
/// Scales dynamically as clamp(dim_len * 2, 4096, 65536) to prevent cell undersampling
/// while providing sub-pixel precision on 4K displays.
#[inline]
pub fn compute_coord_lut_size(dim_len: usize) -> usize {
    (dim_len.saturating_mul(2)).clamp(4096, 65536)
}

/// Builds a 1D coordinate lookup table mapping normalized [0, 1] positions to cell indices.
/// Returns a `Vec<f32>` where each element is the cell index (stored as f32 for GPU storage buffers).
pub fn build_1d_coord_lut(coords: &[f32], lut_size: usize) -> Vec<f32> {
    if coords.is_empty() {
        return Vec::new();
    }
    let m = lut_size.max(1);
    if coords.len() == 1 {
        return vec![0.0; m];
    }
    let first = coords[0];
    let last = coords[coords.len() - 1];
    let span = last - first;
    let mut lut = Vec::with_capacity(m);
    let m_denom = (m - 1).max(1) as f32;
    for k in 0..m {
        let u = k as f32 / m_denom;
        let target = first + u * span;
        let cell_idx = find_coord_cell_1d(coords, target) as f32;
        lut.push(cell_idx);
    }
    lut
}

/// Normalizes longitude degree values to [-180, 180].
#[inline]
fn normalize_lon_deg(lon: f32) -> f32 {
    if lon > 180.0 { lon - 360.0 } else { lon }
}

/// Checks if a 1D sequence of coordinates has non-uniform spacing (> 0.05% relative delta variation).
fn is_irregular_series(coords: &[f64]) -> bool {
    if coords.len() < 3 {
        return false;
    }

    let mut min_delta = f64::MAX;
    let mut max_delta = f64::MIN;
    let mut sum_delta = 0.0;
    let count = coords.len() - 1;

    for i in 0..count {
        let delta = (coords[i + 1] - coords[i]).abs();
        if delta < 1e-7 {
            continue; // Skip identical values
        }
        min_delta = min_delta.min(delta);
        max_delta = max_delta.max(delta);
        sum_delta += delta;
    }

    if min_delta == f64::MAX || count == 0 {
        return false;
    }

    let mean_delta = sum_delta / count as f64;
    if mean_delta < 1e-7 {
        return false;
    }

    let delta_variation = (max_delta - min_delta) / mean_delta;
    delta_variation > 0.0005 // > 0.05% variation is considered irregular (e.g. Gaussian grids, Clenshaw-Curtis)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_coord_cell_1d_ascending() {
        let coords = [10.0, 20.0, 40.0, 80.0];
        assert_eq!(find_coord_cell_1d(&coords, 5.0), 0);
        assert_eq!(find_coord_cell_1d(&coords, 10.0), 0);
        assert_eq!(find_coord_cell_1d(&coords, 14.0), 0);
        assert_eq!(find_coord_cell_1d(&coords, 16.0), 1);
        assert_eq!(find_coord_cell_1d(&coords, 35.0), 2);
        assert_eq!(find_coord_cell_1d(&coords, 70.0), 3);
        assert_eq!(find_coord_cell_1d(&coords, 100.0), 3);
    }

    #[test]
    fn test_find_coord_cell_1d_descending() {
        let coords = [80.0, 40.0, 20.0, 10.0];
        assert_eq!(find_coord_cell_1d(&coords, 100.0), 0);
        assert_eq!(find_coord_cell_1d(&coords, 70.0), 0);
        assert_eq!(find_coord_cell_1d(&coords, 35.0), 1);
        assert_eq!(find_coord_cell_1d(&coords, 16.0), 2);
        assert_eq!(find_coord_cell_1d(&coords, 5.0), 3);
    }

    #[test]
    fn test_cell_center_norm_irregular() {
        let coords_x: Arc<[f32]> = Arc::new([0.0, 10.0, 30.0, 100.0]);
        let coords_y: Arc<[f32]> = Arc::new([0.0, 50.0, 100.0]);
        let grid = CoordinateGrid::Irregular1D {
            coords_x,
            coords_y,
            lon_bounds: (0.0, 100.0),
            lat_bounds: (0.0, 100.0),
        };

        // For cell px=1 (x=10.0), center in norm should be (10.0 - 0.0)/100.0 = 0.1
        let (u_c, v_c) = grid.cell_center_norm(1, 1, 4, 3);
        assert!((u_c - 0.1).abs() < 1e-5);
        assert!((v_c - 0.5).abs() < 1e-5);

        // find_cell_from_norm with norm_x=0.08 should return px=1 (closest to 10.0)
        let (px, py) = grid.find_cell_from_norm(0.08, 0.5, 4, 3);
        assert_eq!(px, 1);
        assert_eq!(py, 1);
    }

    #[test]
    fn detects_regular_global_grid() {
        let lons: Vec<f64> = (0..360).map(|i| -180.0 + i as f64).collect();
        let lats: Vec<f64> = (0..181).map(|i| -90.0 + i as f64).collect();

        let grid = CoordinateGrid::detect_grid("lon", "lat", Some(&lons), Some(&lats), 360, 181);
        assert_eq!(grid, CoordinateGrid::GlobalRegular);
        assert_eq!(grid.coord_mode(), 0);
    }

    #[test]
    fn detects_regular_regional_grid() {
        let lons: Vec<f64> = (0..50).map(|i| 10.0 + i as f64 * 0.5).collect();
        let lats: Vec<f64> = (0..40).map(|i| 35.0 + i as f64 * 0.5).collect();

        let grid = CoordinateGrid::detect_grid("lon", "lat", Some(&lons), Some(&lats), 50, 40);
        match grid {
            CoordinateGrid::RegionalRegular {
                lon_bounds,
                lat_bounds,
            } => {
                assert!((lon_bounds.0 - 10.0).abs() < 1e-3);
                assert!((lat_bounds.0 - 35.0).abs() < 1e-3);
            }
            other => panic!("Expected RegionalRegular, got {:?}", other),
        }
        assert_eq!(grid.coord_mode(), 1);
    }

    #[test]
    fn detects_irregular_1d_grid() {
        // Regional Gaussian-like irregular spacing
        let lons: Vec<f64> = (0..50).map(|i| 10.0 + i as f64 * 0.5).collect();
        let mut lats = Vec::new();
        let mut curr = 30.0;
        for i in 0..40 {
            lats.push(curr);
            curr += 0.5 + (i as f64 * 0.05);
        }

        let grid = CoordinateGrid::detect_grid("lon", "lat", Some(&lons), Some(&lats), 50, 40);
        assert_eq!(grid.coord_mode(), 2);
        assert!(matches!(grid, CoordinateGrid::Irregular1D { .. }));
        assert!(!grid.is_global()); // Regional irregular
    }

    #[test]
    fn detects_global_irregular_1d_grid() {
        // Full-globe Gaussian latitude grid (-180..180 lon, -90..90 non-linear lat)
        let lons: Vec<f64> = (0..360).map(|i| -180.0 + i as f64).collect();
        let mut lats = Vec::new();
        let mut curr = -90.0;
        for _ in 0..180 {
            lats.push(curr);
            let lat_rad = (curr as f32).to_radians();
            // Gaussian latitude-like spacing: wider at equator, narrower at poles
            let step = 1.0 + (lat_rad.cos() as f64) * 0.5;
            curr += step;
        }

        let grid = CoordinateGrid::detect_grid("lon", "lat", Some(&lons), Some(&lats), 360, 180);
        assert_eq!(grid.coord_mode(), 2);
        assert!(matches!(grid, CoordinateGrid::Irregular1D { .. }));
        assert!(grid.is_global()); // Global irregular
    }

    #[test]
    fn test_coord_lut_matches_binary_search() {
        // Stretched exponential grid
        let w = 256;
        let coords: Vec<f32> = (0..w)
            .map(|i| {
                let t = i as f32 / (w - 1) as f32;
                t * t * 100.0
            })
            .collect();

        let lut_size = compute_coord_lut_size(w);
        assert_eq!(lut_size, 4096);
        let lut = build_1d_coord_lut(&coords, lut_size);
        assert_eq!(lut.len(), 4096);

        // Verify across fine sample points that LUT matches direct binary search within +/- 1 cell boundary
        for sample_k in 0..1000 {
            let u = sample_k as f32 / 999.0;
            let target = coords[0] + u * (coords[coords.len() - 1] - coords[0]);
            let exact_cell = find_coord_cell_1d(&coords, target);

            let lut_idx = ((u * (lut_size - 1) as f32) + 0.5) as usize;
            let lut_cell = lut[lut_idx.min(lut_size - 1)] as usize;

            let diff = (exact_cell as isize - lut_cell as isize).abs();
            assert!(
                diff <= 1,
                "LUT cell {lut_cell} deviated from exact cell {exact_cell} at u={u}"
            );
        }
    }

    #[test]
    fn test_coord_lut_descending() {
        let coords = [100.0, 75.0, 30.0, 10.0, 0.0];
        let lut = build_1d_coord_lut(&coords, 4096);
        assert_eq!(lut[0], 0.0);
        assert_eq!(lut[4095], 4.0);
    }
}
