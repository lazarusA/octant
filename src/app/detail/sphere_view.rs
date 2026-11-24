//! The visible part of a 3D sphere's plotted grid from its [`OrbitCamera`],
//! as normalized quad fractions `(u, v)` (`u` West to East, `v` North to
//! South, matching `sphere.wgsl` and the 2D heatmap quad).

use std::f64::consts::{FRAC_PI_2, PI};

use crate::data::CoordinateGrid;
use crate::plots::camera::{DEFAULT_DISTANCE, FOV_SCALE, OrbitCamera};

/// The visible `(u, v)` fractions of `grid` on the unit sphere from `camera`;
/// `None` when the camera is at or beyond [`DEFAULT_DISTANCE`] (where the
/// base level already fits the whole globe) or looking away from a regional
/// grid.
pub(super) fn visible_uv(
    camera: &OrbitCamera,
    grid: &CoordinateGrid,
) -> Option<((f64, f64), (f64, f64))> {
    if camera.distance >= DEFAULT_DISTANCE - 1e-4 {
        return None;
    }
    let eye = camera.eye();
    let r = (eye[0] * eye[0] + eye[1] * eye[1] + eye[2] * eye[2])
        .sqrt()
        .max(1e-6);
    let mut center_lon = f64::from(eye[0].atan2(eye[2]));
    let center_lat = f64::from((eye[1] / r).clamp(-1.0, 1.0).asin());

    let d = f64::from(camera.distance);
    let fov = f64::from(FOV_SCALE);
    let gamma_y = cap_angle(d, 1.0 / fov);
    let gamma_x = cap_angle(d, f64::from(camera.aspect) / fov);

    let lat_lo = (center_lat - gamma_y).max(-FRAC_PI_2);
    let lat_hi = (center_lat + gamma_y).min(FRAC_PI_2);
    let cos_lat = center_lat.cos().max(0.15);
    let dlon = (gamma_x / cos_lat).min(PI);

    let (u, v) = match grid.sphere_coord_mode() {
        0 => {
            let lon_lo = center_lon - dlon;
            let lon_hi = center_lon + dlon;
            let u = ((lon_lo + PI) / (2.0 * PI), (lon_hi + PI) / (2.0 * PI));
            let v = (0.5 - lat_hi / PI, 0.5 - lat_lo / PI);
            (u, v)
        }
        1 => {
            let [lon_min, lon_max] = grid.lon_bounds_rad().map(f64::from);
            let [lat_min, lat_max] = grid.lat_bounds_rad().map(f64::from);
            if lon_max > PI && center_lon < 0.0 {
                center_lon += 2.0 * PI;
            }
            let lon_lo = center_lon - dlon;
            let lon_hi = center_lon + dlon;
            let lon_span = (lon_max - lon_min).abs().max(1e-6);
            let lat_span = (lat_max - lat_min).abs().max(1e-6);
            let u = ((lon_lo - lon_min) / lon_span, (lon_hi - lon_min) / lon_span);
            let v = ((lat_max - lat_hi) / lat_span, (lat_max - lat_lo) / lat_span);
            (u, v)
        }
        _ => return None,
    };

    let overlaps = |a: (f64, f64)| a.1 > 0.0 && a.0 < 1.0 && a.1 > a.0;
    (overlaps(u) && overlaps(v)).then_some((u, v))
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
        assert_eq!(visible_uv(&cam, &CoordinateGrid::GlobalRegular), None);
    }

    #[test]
    fn zooming_in_shrinks_the_visible_window_around_the_sub_camera_point() {
        let cam_2x = OrbitCamera {
            rotation_x: 0.0,
            rotation_y: 0.0,
            distance: 1.75,
            ..OrbitCamera::default()
        };
        let (u2, v2) = visible_uv(&cam_2x, &CoordinateGrid::GlobalRegular).expect("visible at 2x");
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
        let (u4, v4) = visible_uv(&cam_4x, &CoordinateGrid::GlobalRegular).expect("visible at 4x");
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
        assert!(visible_uv(&front, &grid).is_some());

        let back = OrbitCamera {
            rotation_y: 165.0_f32.to_radians(),
            ..front
        };
        assert_eq!(visible_uv(&back, &grid), None);
    }
}
