//! The visible part of a 3D sphere's plotted grid from its [`OrbitCamera`],
//! as normalized quad fractions `(u, v)` (`u` West to East, `v` North to
//! South, matching `sphere.wgsl` and the 2D heatmap quad).

use std::f64::consts::{FRAC_PI_2, PI};

use crate::data::CoordinateGrid;
use crate::plots::camera::{DEFAULT_DISTANCE, FOV_SCALE, OrbitCamera};

/// What a 3D sphere view shows of its plotted grid, as quad fractions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct SphereWindow {
    /// The screen's longitudes along its equatorward edge, where cells look
    /// widest (the level is picked to fit them), and its latitudes.
    pub u: (f64, f64),
    pub v: (f64, f64),
    /// The longitudes the tile reads: the view's widest span (its poleward
    /// edge), every one around a pole or across a wrapping grid's seam.
    pub read_u: (f64, f64),
}

/// What `camera` shows of `grid` (`w` x `h` cells) on the unit sphere;
/// `None` when the camera is at or beyond [`DEFAULT_DISTANCE`] (where the
/// base level already fits the whole globe) or looking away from a regional
/// grid.
pub(super) fn visible_uv(
    camera: &OrbitCamera,
    grid: &CoordinateGrid,
    size: (usize, usize),
) -> Option<SphereWindow> {
    if camera.distance >= DEFAULT_DISTANCE - 1e-4 {
        return None;
    }
    let eye = camera.eye();
    let r = (eye[0] * eye[0] + eye[1] * eye[1] + eye[2] * eye[2])
        .sqrt()
        .max(1e-6);
    let center_lon = f64::from(eye[0].atan2(eye[2]));
    let center_lat = f64::from((eye[1] / r).clamp(-1.0, 1.0).asin());

    let d = f64::from(camera.distance);
    let fov = f64::from(FOV_SCALE);
    let gamma_y = cap_angle(d, 1.0 / fov);
    let gamma_x = cap_angle(d, f64::from(camera.aspect) / fov);

    let lat_lo = (center_lat - gamma_y).max(-FRAC_PI_2);
    let lat_hi = (center_lat + gamma_y).min(FRAC_PI_2);
    let polar = lat_hi >= FRAC_PI_2 - 1e-6 || lat_lo <= -FRAC_PI_2 + 1e-6;
    let dlon_at = |lat: f64| (gamma_x / lat.cos().max(1e-6)).min(PI);
    let crosses_equator = lat_lo <= 0.0 && lat_hi >= 0.0;
    let equatorward = match crosses_equator {
        true => 0.0,
        false => lat_lo.abs().min(lat_hi.abs()),
    };
    let poleward = match polar {
        true => PI,
        false => dlon_at(lat_lo.abs().max(lat_hi.abs())),
    };

    let (center_u, per_rad, wraps, v) = lon_frame(grid, center_lon, (lat_lo, lat_hi), size)?;
    let span = |dlon: f64| (center_u - per_rad * dlon, center_u + per_rad * dlon);
    let read_u = wrapped(span(poleward), wraps);
    let overlaps = |a: (f64, f64)| a.1 > 0.0 && a.0 < 1.0 && a.1 > a.0;
    (overlaps(read_u) && overlaps(v)).then_some(SphereWindow {
        u: span(dlon_at(equatorward)),
        v,
        read_u,
    })
}

/// Where longitude `center_lon` sits across `grid`'s quad (`w` x `h`
/// cells), the quad fraction per radian of longitude, whether the grid wraps
/// around the globe, and latitudes `lat_lo..lat_hi` down the quad.
fn lon_frame(
    grid: &CoordinateGrid,
    center_lon: f64,
    (lat_lo, lat_hi): (f64, f64),
    (w, h): (usize, usize),
) -> Option<(f64, f64, bool, (f64, f64))> {
    match grid.sphere_coord_mode() {
        0 => {
            let v = (0.5 - lat_hi / PI, 0.5 - lat_lo / PI);
            Some(((center_lon + PI) / (2.0 * PI), 1.0 / (2.0 * PI), true, v))
        }
        1 => {
            let [lon_min, lon_max] = grid.lon_bounds_rad().map(f64::from);
            let [lat_min, lat_max] = grid.lat_bounds_rad().map(f64::from);
            let lon_mid = 0.5 * (lon_min + lon_max);
            let center = lon_mid + ((center_lon - lon_mid + PI).rem_euclid(2.0 * PI) - PI);
            let lon_span = (lon_max - lon_min).abs().max(1e-6);
            let lat_span = (lat_max - lat_min).abs().max(1e-6);
            let edge = |c: f64, n| f64::from(edge_of_center(c as f32, n));
            let v = (
                edge((lat_max - lat_hi) / lat_span, h),
                edge((lat_max - lat_lo) / lat_span, h),
            );
            // Cells reach half a cell past the bounds: a whole-globe grid wraps.
            let cells = w.max(2) as f64;
            let wraps = lon_span * cells / (cells - 1.0) >= 2.0 * PI - 1e-3;
            let per_rad = (edge(1.0, w) - edge(0.0, w)) / lon_span;
            Some((edge((center - lon_min) / lon_span, w), per_rad, wraps, v))
        }
        _ => None,
    }
}

/// `u` on a grid around the globe (`wraps`): the whole range when it runs
/// past either seam, where `Axis::visible` would clamp away the far side.
fn wrapped(u: (f64, f64), wraps: bool) -> (f64, f64) {
    match wraps && (u.0 < 0.0 || u.1 > 1.0) {
        true => (0.0, 1.0),
        false => u,
    }
}

/// Regional grids put their first and last cell centers on their bounds
/// (`get_lon_lat`): fraction `c` of the bounds is quad edge fraction
/// `(c (n - 1) + 0.5) / n` of `n` cells, as tiles' `uv` and `slice_at` read.
pub(crate) fn edge_of_center(c: f32, n: usize) -> f32 {
    if n <= 1 {
        return c;
    }
    let n = n as f32;
    (c * (n - 1.0) + 0.5) / n
}

/// The inverse of [`edge_of_center`].
pub(crate) fn center_of_edge(p: f32, n: usize) -> f32 {
    if n <= 1 {
        return p;
    }
    let n = n as f32;
    (p * n - 0.5) / (n - 1.0)
}

/// Central angle on the unit sphere from the sub-camera point to the ray at
/// `tan_fov` from the view axis, clamped to the horizon when the ray misses
/// the globe.
fn cap_angle(d: f64, tan_fov: f64) -> f64 {
    let theta = tan_fov.atan();
    let d_sin = d * theta.sin();
    if d_sin < 1.0 {
        d_sin.asin() - theta
    } else {
        (1.0 / d.max(1.0)).acos()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_or_farther_distance_asks_for_no_tile() {
        let cam = OrbitCamera::default();
        assert_eq!(
            visible_uv(&cam, &CoordinateGrid::GlobalRegular, (360, 180)),
            None
        );
    }

    #[test]
    fn zooming_in_shrinks_the_visible_window_around_the_sub_camera_point() {
        let cam_2x = OrbitCamera {
            rotation_x: 0.0,
            rotation_y: 0.0,
            distance: 1.75,
            ..OrbitCamera::default()
        };
        let w2 =
            visible_uv(&cam_2x, &CoordinateGrid::GlobalRegular, (360, 180)).expect("visible at 2x");
        let (u2, v2) = (w2.read_u, w2.v);
        assert!(
            (u2.0 + u2.1 - 1.0).abs() < 1e-6,
            "centered at lon 0: {u2:?}"
        );
        assert!(
            (v2.0 + v2.1 - 1.0).abs() < 1e-6,
            "centered at lat 0: {v2:?}"
        );
        assert!(u2.1 - u2.0 < 0.5 && v2.1 - v2.0 < 0.5);

        let cam_4x = OrbitCamera {
            distance: 1.375,
            ..cam_2x
        };
        let w4 =
            visible_uv(&cam_4x, &CoordinateGrid::GlobalRegular, (360, 180)).expect("visible at 4x");
        let (u4, v4) = (w4.read_u, w4.v);
        assert!(u4.1 - u4.0 < 0.25 && v4.1 - v4.0 < 0.25);
    }

    #[test]
    fn regional_grid_on_back_of_globe_asks_for_no_tile() {
        let grid = CoordinateGrid::RegionalRegular {
            lon_bounds: (10.0, 20.0),
            lat_bounds: (45.0, 55.0),
        };
        let front = OrbitCamera {
            rotation_x: 50.0_f32.to_radians(),
            rotation_y: -15.0_f32.to_radians(),
            distance: 1.2,
            ..OrbitCamera::default()
        };
        assert!(visible_uv(&front, &grid, (40, 40)).is_some());

        let back = OrbitCamera {
            rotation_y: 165.0_f32.to_radians(),
            ..front
        };
        assert_eq!(visible_uv(&back, &grid, (40, 40)), None);
    }

    /// The camera's sub-point (longitude, latitude) in degrees.
    fn sub_point(cam: &OrbitCamera) -> (f32, f32) {
        let e = cam.eye();
        let r = (e[0] * e[0] + e[1] * e[1] + e[2] * e[2]).sqrt();
        (
            e[0].atan2(e[2]).to_degrees(),
            (e[1] / r).asin().to_degrees(),
        )
    }

    #[test]
    fn a_view_over_the_antimeridian_or_a_pole_asks_for_every_longitude() {
        let near = |rx: f32, ry: f32| OrbitCamera {
            rotation_x: rx.to_radians(),
            rotation_y: ry.to_radians(),
            distance: 1.375,
            ..OrbitCamera::default()
        };
        let grid = CoordinateGrid::GlobalRegular;
        let seam = (0..360)
            .map(|d| near(0.0, d as f32))
            .find(|c| (sub_point(c).0.abs() - 175.0).abs() < 1.0 && sub_point(c).1.abs() < 1.0)
            .expect("a camera over lon 175");
        let w = visible_uv(&seam, &grid, (360, 180)).expect("visible");
        assert_eq!(w.read_u, (0.0, 1.0), "both sides of the seam");
        // The level still fits the screen's own span, not the whole globe.
        assert!(w.u.1 - w.u.0 < 0.25, "the screen's span: {:?}", w.u);
        let px = super::super::geometry::pixels_over(100, w.u, w.read_u);
        assert!(px > 400, "pixels over the whole band: {px}");

        let pole = (-90..=90)
            .map(|d| near(d as f32, 30.0))
            .find(|c| sub_point(c).1 > 84.0)
            .expect("a camera over the North Pole");
        let w = visible_uv(&pole, &grid, (360, 180)).expect("visible");
        assert_eq!(w.read_u, (0.0, 1.0), "every longitude around the pole");
        assert!(w.u.1 - w.u.0 < 1.0, "denser than one level over the globe");
        assert!(w.v.0 <= 1e-9, "up to the pole: {:?}", w.v);
    }

    #[test]
    fn regional_edge_and_center_fractions_invert() {
        // 10 cells: the first center is the bounds' 0 and half a cell in.
        assert!((edge_of_center(0.0, 10) - 0.05).abs() < 1e-6);
        assert!((edge_of_center(1.0, 10) - 0.95).abs() < 1e-6);
        for p in [0.0_f32, 0.05, 0.3, 0.95, 1.0] {
            assert!((edge_of_center(center_of_edge(p, 10), 10) - p).abs() < 1e-6);
        }
    }
}
