//! Cell index and spherical mapping routines for CoordinateGrid.

use crate::data::coordinates::topologies::{
    CartesianTopology, CurvilinearTopology, HealpixTopology, Irregular1DTopology,
};
use crate::data::coordinates::topology::GridTopology;
use crate::data::coordinates::types::CoordinateGrid;

pub fn surface_uv_to_cell(
    grid: &CoordinateGrid,
    u: f32,
    v: f32,
    width: usize,
    height: usize,
) -> (usize, usize) {
    match grid {
        CoordinateGrid::GlobalRegular
        | CoordinateGrid::RegionalRegular { .. }
        | CoordinateGrid::Curvilinear2D { .. } => {
            let px = ((u.clamp(0.0, 1.0) * width.max(1) as f32).floor() as usize)
                .min(width.max(1).saturating_sub(1));
            let py = ((v.clamp(0.0, 1.0) * height.max(1) as f32).floor() as usize)
                .min(height.max(1).saturating_sub(1));
            (px, py)
        }
        CoordinateGrid::Irregular1D {
            coords_x, coords_y, ..
        } => Irregular1DTopology::cell_from_norm_slices(
            coords_x,
            coords_y,
            u.clamp(0.0, 1.0),
            v.clamp(0.0, 1.0),
            width,
            height,
        ),
        CoordinateGrid::Healpix {
            nside,
            ordering,
            npix,
            ..
        } => {
            let lon_rad = u.clamp(0.0, 1.0) * 2.0 * std::f32::consts::PI;
            let lat_rad = (0.5 - v.clamp(0.0, 1.0)) * std::f32::consts::PI;
            HealpixTopology::lon_lat_to_cell_calc(*nside, *ordering, *npix, lon_rad, lat_rad)
        }
    }
}

pub fn norm_to_cell(
    grid: &CoordinateGrid,
    norm_x: f32,
    norm_y: f32,
    width: usize,
    height: usize,
) -> (usize, usize) {
    match grid {
        CoordinateGrid::GlobalRegular
        | CoordinateGrid::RegionalRegular { .. }
        | CoordinateGrid::Curvilinear2D { .. } => {
            let px = ((norm_x.clamp(0.0, 1.0) * width.max(1) as f32).floor() as usize)
                .min(width.max(1).saturating_sub(1));
            let py = ((norm_y.clamp(0.0, 1.0) * height.max(1) as f32).floor() as usize)
                .min(height.max(1).saturating_sub(1));
            (px, py)
        }
        CoordinateGrid::Irregular1D {
            coords_x, coords_y, ..
        } => Irregular1DTopology::cell_from_norm_slices(
            coords_x,
            coords_y,
            norm_x.clamp(0.0, 1.0),
            norm_y.clamp(0.0, 1.0),
            width,
            height,
        ),
        CoordinateGrid::Healpix {
            nside,
            ordering,
            npix,
            ..
        } => {
            let lon_rad = (norm_x.clamp(0.0, 1.0) - 0.5) * 2.0 * std::f32::consts::PI;
            let lat_rad = (0.5 - norm_y.clamp(0.0, 1.0)) * std::f32::consts::PI;
            HealpixTopology::lon_lat_to_cell_calc(*nside, *ordering, *npix, lon_rad, lat_rad)
        }
    }
}

pub fn lon_lat_to_cell(
    grid: &CoordinateGrid,
    lon_rad: f32,
    lat_rad: f32,
    width: usize,
    height: usize,
) -> Option<(usize, usize)> {
    match grid {
        CoordinateGrid::GlobalRegular => {
            CartesianTopology::Global.lon_lat_to_cell(lon_rad, lat_rad, width, height)
        }
        CoordinateGrid::RegionalRegular {
            lon_bounds,
            lat_bounds,
        } => CartesianTopology::Regional {
            lon_bounds: *lon_bounds,
            lat_bounds: *lat_bounds,
        }
        .lon_lat_to_cell(lon_rad, lat_rad, width, height),
        CoordinateGrid::Irregular1D {
            coords_x, coords_y, ..
        } => Irregular1DTopology::lon_lat_to_cell_slices(
            coords_x, coords_y, lon_rad, lat_rad, width, height,
        ),
        CoordinateGrid::Curvilinear2D { .. } => {
            let u =
                ((lon_rad + std::f32::consts::PI) / (2.0 * std::f32::consts::PI)).clamp(0.0, 1.0);
            let v = (0.5 - (lat_rad / std::f32::consts::PI)).clamp(0.0, 1.0);
            let px =
                ((u * width.max(1) as f32).floor() as usize).min(width.max(1).saturating_sub(1));
            let py =
                ((v * height.max(1) as f32).floor() as usize).min(height.max(1).saturating_sub(1));
            Some((px, py))
        }
        CoordinateGrid::Healpix {
            nside,
            ordering,
            npix,
            ..
        } => Some(HealpixTopology::lon_lat_to_cell_calc(
            *nside, *ordering, *npix, lon_rad, lat_rad,
        )),
    }
}

pub fn cell_center_norm(
    grid: &CoordinateGrid,
    px: usize,
    py: usize,
    width: usize,
    height: usize,
) -> (f32, f32) {
    match grid {
        CoordinateGrid::GlobalRegular
        | CoordinateGrid::RegionalRegular { .. }
        | CoordinateGrid::Curvilinear2D { .. } => (
            (px as f32 + 0.5) / width.max(1) as f32,
            (py as f32 + 0.5) / height.max(1) as f32,
        ),
        CoordinateGrid::Irregular1D {
            coords_x, coords_y, ..
        } => {
            Irregular1DTopology::cell_center_norm_slices(coords_x, coords_y, px, py, width, height)
        }
        CoordinateGrid::Healpix {
            nside, ordering, ..
        } => HealpixTopology::cell_center_norm_calc(*nside, *ordering, px),
    }
}

pub fn cell_center_surface_xz(
    grid: &CoordinateGrid,
    px: usize,
    py: usize,
    width: usize,
    height: usize,
    aspect: f32,
) -> (f32, f32) {
    match grid {
        CoordinateGrid::Healpix {
            nside, ordering, ..
        } => HealpixTopology::cell_center_surface_xz_calc(*nside, *ordering, px, aspect),
        _ => {
            let (u_c, v_c) = cell_center_norm(grid, px, py, width, height);
            ((2.0 * u_c - 1.0) * aspect, 2.0 * v_c - 1.0)
        }
    }
}

pub fn cell_center_lon_lat_rad(
    grid: &CoordinateGrid,
    px: usize,
    py: usize,
    width: usize,
    height: usize,
) -> (f32, f32) {
    match grid {
        CoordinateGrid::GlobalRegular => {
            let (u_c, v_c) = cell_center_norm(grid, px, py, width, height);
            (
                (u_c - 0.5) * 2.0 * std::f32::consts::PI,
                (0.5 - v_c) * std::f32::consts::PI,
            )
        }
        CoordinateGrid::RegionalRegular {
            lon_bounds,
            lat_bounds,
        } => {
            // The bounds are the first and last cell centers, as the sphere draws them
            // and `lon_lat_to_cell` inverts them.
            CartesianTopology::Regional {
                lon_bounds: *lon_bounds,
                lat_bounds: *lat_bounds,
            }
            .cell_center_lon_lat_rad(px, py, width, height)
        }
        CoordinateGrid::Irregular1D {
            coords_x, coords_y, ..
        } => Irregular1DTopology::cell_center_lon_lat_rad_slices(coords_x, coords_y, px, py),
        CoordinateGrid::Curvilinear2D { lons, lats, .. } => {
            CurvilinearTopology::cell_center_lon_lat_rad_slices(lons, lats, px, py, width, height)
        }
        CoordinateGrid::Healpix {
            nside,
            ordering,
            coords_lon,
            coords_lat,
            ..
        } => HealpixTopology::cell_center_lon_lat_rad_calc(
            *nside,
            *ordering,
            coords_lon.as_deref(),
            coords_lat.as_deref(),
            px,
        ),
    }
}
