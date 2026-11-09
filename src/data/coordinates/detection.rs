//! Grid auto-detection and dimension classification heuristics.

use super::ordering::detect_healpix_ordering_with_dggs;
use super::types::CoordinateGrid;
use std::sync::Arc;

/// Checks if a 1D sequence of coordinates has non-uniform spacing (> 0.05% relative delta variation).
pub fn is_irregular_series(coords: &[f64]) -> bool {
    if coords.len() < 3 {
        return false;
    }

    let mut min_delta = f64::MAX;
    let mut max_delta = f64::MIN;
    let mut sum_delta = 0.0;
    let mut valid_count = 0usize;

    for w in coords.windows(2) {
        let delta = (w[1] - w[0]).abs();
        if delta < 1e-7 {
            continue; // Skip identical values
        }
        min_delta = min_delta.min(delta);
        max_delta = max_delta.max(delta);
        sum_delta += delta;
        valid_count += 1;
    }

    if min_delta == f64::MAX || valid_count == 0 {
        return false;
    }

    let mean_delta = sum_delta / valid_count as f64;
    if mean_delta < 1e-7 {
        return false;
    }

    let delta_variation = (max_delta - min_delta) / mean_delta;
    delta_variation > 0.0005 // > 0.05% variation is considered irregular (e.g. Gaussian grids, Clenshaw-Curtis)
}

/// Automatically classifies and constructs a `CoordinateGrid` from dimension coordinate arrays and OctantBlock metadata.
pub fn detect_grid_from_block(
    block: &crate::data::OctantBlock,
    x_name: &str,
    y_name: &str,
    x_coords: Option<&[f64]>,
    y_coords: Option<&[f64]>,
    width: usize,
    height: usize,
) -> CoordinateGrid {
    let dggs_opt = super::dggs::DggsMetadata::from_attributes(&block.attributes);
    let is_dggs_healpix = dggs_opt.as_ref().is_some_and(|d| d.is_healpix());

    let is_healpix_x = super::naming::is_healpix_dim_name(x_name)
        || dggs_opt.as_ref().is_some_and(|d| d.matches_dim(x_name));
    let is_healpix_y = super::naming::is_healpix_dim_name(y_name)
        || dggs_opt.as_ref().is_some_and(|d| d.matches_dim(y_name));

    let is_healpix_attr = is_dggs_healpix
        || block.attributes.contains_key("healpix_zoom")
        || block.attributes.contains_key("healpix_nest")
        || block.attributes.contains_key("healpix_order")
        || block
            .attributes
            .get("grid_type")
            .is_some_and(|g| super::naming::contains_ascii_case_insensitive(g, "healpix"))
        || block
            .attributes
            .get("ordering")
            .is_some_and(|g| super::naming::contains_ascii_case_insensitive(g, "nested"));

    let npix = if height == 1 {
        width
    } else {
        width.saturating_mul(height)
    };
    let nside_opt = dggs_opt
        .as_ref()
        .and_then(|d| d.healpix_nside(npix))
        .or_else(|| super::healpix::npix_to_nside(npix));

    if (is_healpix_x || is_healpix_y || is_healpix_attr)
        && let Some(nside) = nside_opt
    {
        let ordering = detect_healpix_ordering_with_dggs(&block.attributes, dggs_opt.as_ref());

        let coords_lon = block
            .coordinates
            .get("lon")
            .map(|l| l.iter().map(|&v| v as f32).collect::<Arc<[f32]>>());
        let coords_lat = block
            .coordinates
            .get("lat")
            .map(|l| l.iter().map(|&v| v as f32).collect::<Arc<[f32]>>());

        log::info!(
            "CoordinateGrid: Detected HEALPix grid from block (nside={nside}, ordering={:?}, npix={npix})",
            ordering
        );
        return CoordinateGrid::Healpix {
            nside,
            ordering,
            npix,
            coords_lon,
            coords_lat,
        };
    }

    detect_grid(x_name, y_name, x_coords, y_coords, width, height)
}

#[inline]
fn build_coordinate_slice(
    coords: &[f64],
    target_len: usize,
    min_val: f32,
    max_val: f32,
) -> Arc<[f32]> {
    if coords.len() >= target_len {
        coords.iter().take(target_len).map(|&v| v as f32).collect()
    } else {
        (0..target_len)
            .map(|i| {
                if target_len <= 1 {
                    min_val
                } else {
                    min_val + (i as f32 / (target_len - 1) as f32) * (max_val - min_val)
                }
            })
            .collect()
    }
}

/// Automatically classifies and constructs a `CoordinateGrid` from dimension coordinate arrays.
/// HEALPix grids are recognized before this, by `detect_grid_from_block`.
pub fn detect_grid(
    x_name: &str,
    y_name: &str,
    x_coords: Option<&[f64]>,
    y_coords: Option<&[f64]>,
    width: usize,
    height: usize,
) -> CoordinateGrid {
    let is_spatial_x = super::naming::is_spatial_x_name(x_name);
    let is_spatial_y = super::naming::is_spatial_y_name(y_name);

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
        let coords_x = build_coordinate_slice(xc, width, x_min, x_max);
        let coords_y = build_coordinate_slice(yc, height, y_min, y_max);

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
