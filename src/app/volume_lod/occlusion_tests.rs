//! Visibility from the plotted volume: a cube seen from the front shows
//! only its front cells when solid, every cell when nothing is drawn, and
//! a transparent one lets rays reach past its front.

use super::occlusion::{Opacity, seen_cells};
use super::select::Placement;
use crate::data::VolumeData;
use crate::data::multiscale::bricks::CellBits;
use crate::plots::camera::OrbitCamera;

/// 16^3 voxels of `value` over a 256^3 level-0 window: a 4^3 table.
fn cube(value: f32) -> VolumeData {
    VolumeData::new(
        16,
        16,
        16,
        vec![value; 16 * 16 * 16],
        value,
        value,
        String::new(),
    )
}

const PLACE: Placement = Placement {
    lo: [0.0; 3],
    hi: [256.0; 3],
    mirror: [false, true, true],
    aspect: [1.0; 3],
};

/// Looking down -z at the box's front face.
fn front() -> OrbitCamera {
    OrbitCamera {
        rotation_x: 0.0,
        ..OrbitCamera::default()
    }
}

/// Marked cells per z plane of the table (z 0 the front: stored plane 0
/// is drawn nearest the viewer).
fn planes(seen: &CellBits) -> [usize; 4] {
    std::array::from_fn(|z| {
        (z * 16..(z + 1) * 16)
            .filter(|&i| seen.get(i) == Some(true))
            .count()
    })
}

fn opacity(opaque: bool, density: f32) -> Opacity {
    let color = crate::plots::common::PlotColorParams {
        cmin: 0.0,
        cmax: 1.0,
        ..Default::default()
    };
    Opacity {
        color,
        density,
        opaque,
    }
}

fn seen(volume: &VolumeData, opacity: Opacity) -> CellBits {
    seen_cells(
        volume,
        &front(),
        &PLACE,
        [false; 3],
        [4; 3],
        [512, 512],
        opacity,
    )
}

#[test]
fn a_solid_cube_shows_its_front_cells_only() {
    let marked = planes(&seen(&cube(1.0), opacity(true, 3.0)));
    assert_eq!(marked, [16, 0, 0, 0]);
}

#[test]
fn nothing_drawn_shows_every_cell() {
    let marked = planes(&seen(&cube(-1.0), opacity(true, 3.0)));
    assert_eq!(marked, [16; 4]);
}

#[test]
fn a_faint_cube_lets_rays_through() {
    // A value low in the range at the lowest Density stays see-through.
    let marked = planes(&seen(&cube(0.05), opacity(false, 0.1)));
    assert_eq!(marked[3], 16, "{marked:?}");
}

#[test]
fn the_colormaps_opacity_lets_rays_see_further() {
    // DVR weighs Density by the colormap's opacity (`dvr_sample`).
    let dense = opacity(false, 3.0);
    let color = crate::plots::common::PlotColorParams {
        opacity: 0.02,
        ..dense.color
    };
    let faint = Opacity { color, ..dense };
    let near = planes(&seen(&cube(1.0), dense));
    let far = planes(&seen(&cube(1.0), faint));
    assert!(
        near[3] < 16,
        "the colormap's full opacity hides the back: {near:?}"
    );
    assert_eq!(far[3], 16, "{far:?}");
}
