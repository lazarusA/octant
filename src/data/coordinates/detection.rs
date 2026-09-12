//! Grid auto-detection and dimension classification heuristics.

use super::types::CoordinateGrid;
use std::sync::Arc;

/// Normalizes longitude degree values to [-180, 180].
#[inline]
pub fn normalize_lon_deg(lon: f32) -> f32 {
    if lon > 180.0 { lon - 360.0 } else { lon }
}

/// Checks if a 1D sequence of coordinates has non-uniform spacing (> 0.05% relative delta variation).
pub fn is_irregular_series(coords: &[f64]) -> bool {
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

/// Automatically classifies and constructs a `CoordinateGrid` from dimension coordinate arrays.
pub fn detect_grid(
    x_name: &str,
    y_name: &str,
    x_coords: Option<&[f64]>,
    y_coords: Option<&[f64]>,
    width: usize,
    height: usize,
) -> CoordinateGrid {
    let is_healpix_x = crate::utils::coordinates::is_healpix_dim_name(x_name);
    let is_healpix_y = crate::utils::coordinates::is_healpix_dim_name(y_name);

    if is_healpix_x || is_healpix_y {
        let npix = if height == 1 { width } else { width * height };
        if let Some(nside) = super::healpix::npix_to_nside(npix) {
            log::info!("CoordinateGrid: Detected HEALPix grid (nside={nside}, npix={npix})");
            return CoordinateGrid::Healpix {
                nside,
                ordering: super::healpix::HealpixOrder::Ring,
                npix,
                coords_lon: None,
                coords_lat: None,
            };
        }
    }

    let is_spatial_x = crate::utils::coordinates::is_spatial_x_name(x_name);
    let is_spatial_y = crate::utils::coordinates::is_spatial_y_name(y_name);

    let Some(xc) = x_coords else {
        return CoordinateGrid::GlobalRegular;
    };
    let Some(yc) = y_coords else {
        return CoordinateGrid::GlobalRegular;
    };

    if xc.is_empty() || yc.is_empty() {
        return CoordinateGrid::GlobalRegular;
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

        CoordinateGrid::Irregular1D {
            coords_x,
            coords_y,
            lon_bounds: (x_min, x_max),
            lat_bounds: (y_min, y_max),
        }
    } else if is_global_extent {
        // Standard [-180, 180] origin: collapse to GlobalRegular (no bounds needed).
        // [0, 360]-origin grids (lon_min ≥ -5°): preserve the actual bounds in
        // RegionalRegular so downstream systems (e.g. the coastline overlay) can
        // project into the correct lon domain.  The heatmap shader uses the same
        // code path for coord_mode 0 and 1, so this is a safe change.
        if x_min >= -5.0 {
            log::info!(
                "CoordinateGrid: Global extent with [0,360] origin (lon_min={x_min:.2}); \
                 using RegionalRegular to preserve bounds."
            );
            CoordinateGrid::RegionalRegular {
                lon_bounds: (x_min, x_max),
                lat_bounds: (y_min, y_max),
            }
        } else {
            CoordinateGrid::GlobalRegular
        }
    } else if is_spatial_x || is_spatial_y {
        CoordinateGrid::RegionalRegular {
            lon_bounds: (x_min, x_max),
            lat_bounds: (y_min, y_max),
        }
    } else {
        CoordinateGrid::GlobalRegular
    }
}
