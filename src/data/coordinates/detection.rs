//! Grid auto-detection and dimension classification heuristics.

use super::types::CoordinateGrid;
use std::sync::Arc;

/// Normalizes longitude degree values to [-180, 180].
#[inline]
pub fn normalize_lon_deg(lon: f32) -> f32 {
    if lon > 180.0 { lon - 360.0 } else { lon }
}

/// Converts a lon/lat pair in degrees to a unit vector on the unit sphere.
#[inline]
pub fn lonlat_deg_to_unit_vec(lon_deg: f32, lat_deg: f32) -> [f32; 3] {
    let lon = lon_deg.to_radians();
    let lat = lat_deg.to_radians();
    let cos_lat = lat.cos();
    [
        cos_lat * lon.cos(),
        lat.sin(),
        cos_lat * lon.sin(),
    ]
}

/// Computes the spherical mean of multiple lon/lat points in degrees.
/// This is numerically stable near the poles, where simple longitude averaging is undefined.
#[inline]
pub fn spherical_mean_lonlat_deg(points: &[(f32, f32)]) -> (f32, f32) {
    let mut sum = [0.0f32; 3];
    for &(lon_deg, lat_deg) in points {
        let v = lonlat_deg_to_unit_vec(lon_deg, lat_deg);
        sum[0] += v[0];
        sum[1] += v[1];
        sum[2] += v[2];
    }

    let norm = (sum[0] * sum[0] + sum[1] * sum[1] + sum[2] * sum[2]).sqrt();
    if norm < 1e-6 {
        return (0.0, 0.0);
    }

    let x = sum[0] / norm;
    let y = sum[1] / norm;
    let z = sum[2] / norm;
    let lon = z.atan2(x).to_degrees();
    let lat = y.atan2((x * x + z * z).sqrt()).to_degrees();

    (lon, lat)
}

/// Detects whether a curvilinear grid has reversed i-axis handedness (i increases westward).
///
/// Uses a cross-product sign check on the i/j tangent vectors for the first valid interior cell.
/// Matches the TypeScript `detectCurvilinearLongitudeFlip` algorithm.
pub fn detect_curvilinear_longitude_flip(
    longitudes: &[f32],
    latitudes: &[f32],
    ni: usize,
    nj: usize,
) -> bool {
    for row in 0..nj.saturating_sub(1) {
        let col_limit = (ni - 1).min(10);
        for col in 0..col_limit {
            let tl = row * ni + col;
            let tr = tl + 1;
            let bl = (row + 1) * ni + col;
            let br = bl + 1;
            // Skip any cells with non-finite coordinates
            if [longitudes[tl], longitudes[tr], longitudes[bl], longitudes[br],
                latitudes[tl],  latitudes[tr],  latitudes[bl],  latitudes[br]]
                .iter()
                .any(|v| !v.is_finite())
            {
                continue;
            }
            let dlon_i = longitudes[tr] - longitudes[tl];
            let dlat_i = latitudes[tr]  - latitudes[tl];
            let dlon_j = longitudes[bl] - longitudes[tl];
            let dlat_j = latitudes[bl]  - latitudes[tl];
            // Cross-product z-component; negative means right-handed (westward i)
            return dlon_i * dlat_j - dlat_i * dlon_j < 0.0;
        }
    }
    false
}

/// Detects whether a curvilinear grid is periodic in the i-direction (columns wrap around).
///
/// Compares the gap between the last and first longitudes in a sample row against the
/// mean inter-cell spacing (×4 tolerance). Matches the TypeScript
/// `detectCurvilinearColumnPeriodicity` algorithm.
pub fn detect_curvilinear_column_periodicity(
    longitudes: &[f32],
    ni: usize,
    nj: usize,
) -> bool {
    if ni < 3 {
        return false;
    }
    let sample_row = nj / 2;
    let sample_count = (ni - 1).min(10);
    let mut spacing_sum = 0.0f64;
    let mut valid = 0usize;
    for col in 0..sample_count {
        let a = longitudes[sample_row * ni + col] as f64;
        let b = longitudes[sample_row * ni + col + 1] as f64;
        let gap = ((b - a + 540.0) % 360.0) - 180.0;
        let spacing = gap.abs();
        if spacing > 0.0 {
            spacing_sum += spacing;
            valid += 1;
        }
    }
    if valid == 0 {
        return false;
    }
    let mean_spacing = spacing_sum / valid as f64;
    let first = longitudes[sample_row * ni] as f64;
    let last  = longitudes[sample_row * ni + ni - 1] as f64;
    let wrap_gap = (((first - last + 540.0) % 360.0) - 180.0).abs();
    wrap_gap < mean_spacing * 4.0
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
        CoordinateGrid::GlobalRegular
    } else if is_spatial_x || is_spatial_y {
        CoordinateGrid::RegionalRegular {
            lon_bounds: (x_min, x_max),
            lat_bounds: (y_min, y_max),
        }
    } else {
        CoordinateGrid::GlobalRegular
    }
}

/// Automatically classifies and constructs a `CoordinateGrid`, checking 2D curvilinear coordinates first.
#[inline]
pub fn is_curvilinear_center_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    if lower.ends_with("_bnds")
        || lower.ends_with("bnds")
        || lower.contains("_bounds")
        || lower.contains("bounds")
        || lower.contains("vertex")
    {
        return false;
    }
    true
}

pub fn detect_curvilinear_grid(
    x_name: &str,
    y_name: &str,
    x_coords: Option<&[f64]>,
    y_coords: Option<&[f64]>,
    curvilinear_coords: &std::collections::HashMap<String, crate::data::CurvilinearCoord2D>,
    width: usize,
    height: usize,
) -> CoordinateGrid {
    if !curvilinear_coords.is_empty() {
        let lon_candidates = ["nav_lon", "lon", "longitude", "lons", "x_lon"];
        let lat_candidates = ["nav_lat", "lat", "latitude", "lats", "y_lat"];

        let find_coord = |candidates: &[&str]| -> Option<&crate::data::CurvilinearCoord2D> {
            for cand in candidates {
                if is_curvilinear_center_name(cand) {
                    if let Some(c) = curvilinear_coords.get(*cand)
                        && c.width == width
                        && c.height == height
                    {
                        return Some(c);
                    }
                }
                for (k, v) in curvilinear_coords {
                    let clean = k.to_ascii_lowercase();
                    if is_curvilinear_center_name(k)
                        && clean.contains(cand)
                        && v.width == width
                        && v.height == height
                    {
                        return Some(v);
                    }
                }
            }
            None
        };

        if let (Some(lon_c), Some(lat_c)) =
            (find_coord(&lon_candidates), find_coord(&lat_candidates))
        {
            let (lon_min, lon_max) = crate::utils::compute_finite_min_max(&lon_c.values);
            let (lat_min, lat_max) = crate::utils::compute_finite_min_max(&lat_c.values);

            let flip_i =
                detect_curvilinear_longitude_flip(&lon_c.values, &lat_c.values, width, height);
            let is_periodic_i =
                detect_curvilinear_column_periodicity(&lon_c.values, width, height);

            log::info!(
                "CoordinateGrid: Detected Curvilinear2D grid \
                 (w={width}, h={height}, \
                 lon_bounds=[{lon_min:.2}, {lon_max:.2}], \
                 lat_bounds=[{lat_min:.2}, {lat_max:.2}], \
                 flip_i={flip_i}, is_periodic_i={is_periodic_i})"
            );

            return CoordinateGrid::Curvilinear2D {
                lons: lon_c.values.clone(),
                lats: lat_c.values.clone(),
                lon_bounds: (lon_min, lon_max),
                lat_bounds: (lat_min, lat_max),
                flip_i,
                is_periodic_i,
            };
        }
    }

    detect_grid(x_name, y_name, x_coords, y_coords, width, height)
}

#[cfg(test)]
mod tests {
    use super::{lonlat_deg_to_unit_vec, spherical_mean_lonlat_deg};

    #[test]
    fn spherical_mean_lonlat_deg_is_stable_at_the_north_pole() {
        let corners = [
            (0.0, 89.0),
            (90.0, 88.0),
            (180.0, 89.0),
            (270.0, 87.0),
        ];

        let (lon, lat) = spherical_mean_lonlat_deg(&corners);
        let pole_vec = lonlat_deg_to_unit_vec(0.0, 90.0);
        let mean_vec = lonlat_deg_to_unit_vec(lon, lat);
        let dot = pole_vec[0] * mean_vec[0] + pole_vec[1] * mean_vec[1] + pole_vec[2] * mean_vec[2];

        assert!((lat - 89.0).abs() < 2.0, "lat={lat}");
        assert!(dot > 0.98, "mean should stay near the north pole; lon={lon}, lat={lat}, dot={dot}");
    }
}
