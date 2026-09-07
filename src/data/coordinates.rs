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
}
