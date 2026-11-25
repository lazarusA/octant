//! The visible part of a 3D sphere's plotted grid from its [`OrbitCamera`],
//! as normalized quad fractions `(u, v)` (`u` West to East, `v` North to
//! South, matching `sphere.wgsl` and the 2D heatmap quad).

use std::f64::consts::{FRAC_PI_2, PI};

use crate::data::CoordinateGrid;
use crate::plots::camera::{DEFAULT_DISTANCE, FOV_SCALE, OrbitCamera};

/// The visible `(u, v)` quad fractions of `grid` (`w` x `h` cells) on the
/// unit sphere from `camera`; `None` when the camera is at or beyond
/// [`DEFAULT_DISTANCE`] (where the base level already fits the whole globe)
/// or looking away from a regional grid.
pub(super) fn visible_uv(
    camera: &OrbitCamera,
    grid: &CoordinateGrid,
    (w, h): (usize, usize),
) -> Option<((f64, f64), (f64, f64))> {
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
    // A view holding a pole shows every longitude; else the view is widest
    // in longitude along its poleward edge.
    let polar = lat_hi >= FRAC_PI_2 - 1e-6 || lat_lo <= -FRAC_PI_2 + 1e-6;
    let edge_cos = lat_hi.abs().max(lat_lo.abs()).cos().max(1e-6);
    let dlon = if polar {
        PI
    } else {
        (gamma_x / edge_cos).min(PI)
    };

    let (u, v) = match grid.sphere_coord_mode() {
        0 => {
            let lon_lo = center_lon - dlon;
            let lon_hi = center_lon + dlon;
            let u = ((lon_lo + PI) / (2.0 * PI), (lon_hi + PI) / (2.0 * PI));
            let v = (0.5 - lat_hi / PI, 0.5 - lat_lo / PI);
            (wrapped(u, true), v)
        }
        1 => regional_uv(grid, center_lon, dlon, (lat_lo, lat_hi), (w, h)),
        _ => return None,
    };

    let overlaps = |a: (f64, f64)| a.1 > 0.0 && a.0 < 1.0 && a.1 > a.0;
    (overlaps(u) && overlaps(v)).then_some((u, v))
}

/// [`visible_uv`] on a regional grid of `w` x `h` cells: the window
/// `center_lon ± dlon` by `lat_lo..lat_hi`, as quad edge fractions.
fn regional_uv(
    grid: &CoordinateGrid,
    center_lon: f64,
    dlon: f64,
    (lat_lo, lat_hi): (f64, f64),
    (w, h): (usize, usize),
) -> ((f64, f64), (f64, f64)) {
    let [lon_min, lon_max] = grid.lon_bounds_rad().map(f64::from);
    let [lat_min, lat_max] = grid.lat_bounds_rad().map(f64::from);
    let lon_mid = 0.5 * (lon_min + lon_max);
    let center_lon = lon_mid + ((center_lon - lon_mid + PI).rem_euclid(2.0 * PI) - PI);
    let lon_span = (lon_max - lon_min).abs().max(1e-6);
    let lat_span = (lat_max - lat_min).abs().max(1e-6);
    let u = (
        (center_lon - dlon - lon_min) / lon_span,
        (center_lon + dlon - lon_min) / lon_span,
    );
    let v = ((lat_max - lat_hi) / lat_span, (lat_max - lat_lo) / lat_span);
    // Cells reach half a cell past the bounds: a whole-globe grid wraps.
    let cells = w.max(2) as f64;
    let global = lon_span * cells / (cells - 1.0) >= 2.0 * PI - 1e-3;
    let at = |c: f64, n| f64::from(edge_of_center(c as f32, n));
    let edge = |(a, b): (f64, f64), n| (at(a, n), at(b, n));
    (edge(wrapped(u, global), w), edge(v, h))
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
        let (u2, v2) =
            visible_uv(&cam_2x, &CoordinateGrid::GlobalRegular, (360, 180)).expect("visible at 2x");
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
        let (u4, v4) =
            visible_uv(&cam_4x, &CoordinateGrid::GlobalRegular, (360, 180)).expect("visible at 4x");
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
        let (u, _) = visible_uv(&seam, &grid, (360, 180)).expect("visible");
        assert_eq!(u, (0.0, 1.0), "both sides of the seam");

        let pole = (-90..=90)
            .map(|d| near(d as f32, 30.0))
            .find(|c| sub_point(c).1 > 84.0)
            .expect("a camera over the North Pole");
        let (u, v) = visible_uv(&pole, &grid, (360, 180)).expect("visible");
        assert_eq!(u, (0.0, 1.0), "every longitude around the pole");
        assert!(v.0 <= 1e-9, "up to the pole: {v:?}");
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
