use super::slicer::{SliceAxis, SliceMode, Slicer};
use crate::plots::PlotType;
use crate::plots::slice::SliceUniform;
use crate::utils::height::HeightScale;

fn boxed(lo: [f32; 3], hi: [f32; 3]) -> Slicer {
    Slicer {
        mode: SliceMode::Box,
        lo,
        hi,
        ..Slicer::default()
    }
}

#[test]
fn hidden_guides_still_cut() {
    let mut s = boxed([0.2; 3], [0.8; 3]);
    s.guides = false;
    assert!(s.cutting() && !s.guides_shown());
    assert_eq!(s.uniform().lo[3], 1.0);
}

#[test]
fn uniforms_flip_axes_and_take_heights() {
    let s = boxed([0.1, 0.2, 0.3], [0.6, 0.7, 0.8]);
    let u = s.uniform_in([false, true, true], None);
    let near = |a: [f32; 4], b: [f32; 4]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-6);
    assert!(near(u.lo, [0.1, 0.3, 0.2, 1.0]) && near(u.hi, [0.6, 0.8, 0.7, 0.0]));
    assert_eq!(s.uniform_in([false; 3], Some((-1.0, 2.0))).hi[2], 2.0);
    let mut p = Slicer::default();
    p.set_mode(SliceMode::Planes);
    p.planes = [Some(0.25), None, Some(0.5)];
    let u = p.uniform_in([false, true, true], None);
    assert_eq!(u.lo, [0.25, -1.0, 0.5, 2.0]);
    assert_eq!(Slicer::default().uniform(), SliceUniform::OFF);
}

#[test]
fn a_plot_type_without_the_mode_turns_the_slicer_off() {
    let mut s = Slicer::default();
    s.set_mode(SliceMode::Planes);
    s.guides = false;
    s.adapt_to(PlotType::Heatmap);
    assert_eq!(s.mode, SliceMode::Off);
    let mut s = boxed([0.2; 3], [0.8; 3]);
    s.adapt_to(PlotType::Surface);
    assert_eq!((s.mode, s.lo, s.hi), (SliceMode::Box, [0.0; 3], [1.0; 3]));
    assert_eq!(SliceAxis::of(PlotType::Surface)[2], SliceAxis::Height);
}

#[test]
fn cells_are_kept_by_their_centers_in_drawn_or_stored_order() {
    // 10 cells; [0.2, 0.55] holds the centers 0.25 .. 0.55 of cells 2..=5.
    let s = boxed([0.2, 0.0, 0.0], [0.55, 1.0, 1.0]);
    let r = s.index_ranges([10, 4, 1], [false; 3]);
    assert_eq!(r, [2..6, 0..4, 0..1]);
    let r = s.index_ranges([10, 4, 1], [true, false, false]);
    assert_eq!(r[0], 4..8);
    // A box between two centers keeps nothing; one on the edge keeps a cell.
    assert_eq!(
        boxed([0.41, 0.0, 0.0], [0.44, 1.0, 1.0]).index_ranges([10, 1, 1], [false; 3])[0].len(),
        0
    );
    assert_eq!(
        Slicer::default().index_ranges([3, 3, 3], [false; 3])[0],
        0..3
    );
}

#[test]
fn planes_name_the_cell_they_cross() {
    let mut s = Slicer::default();
    s.set_mode(SliceMode::Planes);
    s.planes = [Some(0.95), Some(0.0), None];
    assert_eq!(
        s.plane_indices([10, 4, 6], [false; 3]),
        [Some(9), Some(0), None]
    );
    assert_eq!(
        s.plane_indices([10, 4, 6], [true, false, false])[0],
        Some(0)
    );
}

#[test]
fn height_cuts_read_back_as_data_values() {
    let scale = HeightScale {
        cmin: 200.0,
        cmax: 300.0,
        composite: false,
        flip: false,
    };
    let s = boxed([0.0; 3], [1.0, 1.0, 0.5]);
    assert_eq!(s.height_range(&scale, true), (f32::MIN, 0.5));
    assert_eq!(
        s.height_range(&scale, false),
        (f32::MIN, f32::MAX),
        "flat: never cut"
    );
    assert_eq!(s.height_values(&scale), Some((200.0, 250.0)));
    let flipped = HeightScale {
        flip: true,
        ..scale
    };
    assert_eq!(s.height_values(&flipped), Some((250.0, 300.0)));
    for v in [200.0, 237.5, 300.0] {
        assert_eq!(scale.value_at(scale.normalized(v)), Some(v));
    }
}

#[test]
fn coastlines_are_cut_only_when_asked_and_keep_the_choice() {
    let mut s = boxed([0.2; 3], [0.8; 3]);
    assert_eq!(
        s.for_coastlines(s.uniform()),
        SliceUniform::OFF,
        "off by default"
    );
    s.cut_coastlines = true;
    assert_eq!(s.for_coastlines(s.uniform()), s.uniform());
    s.set_mode(SliceMode::Planes);
    assert!(s.cut_coastlines, "kept across modes");
}
