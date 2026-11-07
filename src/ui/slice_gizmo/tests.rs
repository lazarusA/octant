use super::drag::{apply, snap, solve};
use super::guide::{self, Guide};
use super::handles::{Handle, all};
use super::planes::{self, PlaneEdge};
use super::space::Space;
use super::*;
use crate::plots::camera::OrbitCamera;
use crate::ui::hover::camera::Camera3D;
use egui::{Rect, Vec2, pos2, vec2};

pub(super) fn rect() -> Rect {
    Rect::from_min_max(pos2(0.0, 0.0), pos2(800.0, 600.0))
}

fn camera(rotation_x: f32, rotation_y: f32) -> Camera3D {
    let orbit = OrbitCamera {
        rotation_x,
        rotation_y,
        aspect: 800.0 / 600.0,
        ..OrbitCamera::default()
    };
    Camera3D {
        rect: rect(),
        orbit,
    }
}

fn boxed(lo: [f32; 3], hi: [f32; 3]) -> Slicer {
    Slicer {
        mode: SliceMode::Box,
        lo,
        hi,
        ..Slicer::default()
    }
}

pub(super) fn screen(space: &Space, p: [f32; 3]) -> egui::Pos2 {
    space.screen(p).unwrap_or(pos2(f32::NAN, f32::NAN))
}

#[test]
fn a_box_has_8_corners_and_12_edges_a_rectangle_4_and_4() {
    for (dims, corners, edges) in [(3, 8, 12), (2, 4, 4)] {
        let (h, n) = all(dims);
        let h = &h[..n];
        assert_eq!(h.iter().filter(|h| h.is_corner(dims)).count(), corners);
        assert_eq!(h.iter().filter(|h| !h.is_corner(dims)).count(), edges);
        for (i, a) in h.iter().enumerate() {
            assert!(h[i + 1..].iter().all(|b| b != a), "{a:?} twice");
        }
    }
}

#[test]
fn faces_moving_apart_on_screen_follow_the_pointer_exactly() {
    let d = solve(
        [Some(vec2(100.0, 0.0)), Some(vec2(0.0, 100.0)), None],
        vec2(10.0, 5.0),
    );
    assert!((d[0] - 0.1).abs() < 1e-5 && (d[1] - 0.05).abs() < 1e-5 && d[2] == 0.0);
}

#[test]
fn faces_moving_alike_on_screen_move_one_at_a_time() {
    let d = solve(
        [Some(vec2(100.0, 0.0)), Some(vec2(98.0, 5.0)), None],
        vec2(10.0, 0.0),
    );
    assert_eq!(d.iter().filter(|m| **m != 0.0).count(), 1);
}

#[test]
fn a_face_seen_edge_on_is_frozen() {
    let dirs = [
        Some(vec2(100.0, 0.0)),
        Some(vec2(1.0, 0.0)),
        Some(vec2(0.0, 100.0)),
    ];
    assert_eq!(solve(dirs, vec2(10.0, 10.0))[1], 0.0);
    // Three faces, two directions: the least norm moves all that are free.
    let dirs = [vec2(100.0, 0.0), vec2(50.0, 50.0), vec2(0.0, 100.0)];
    let d = solve(dirs.map(Some), vec2(10.0, 0.0));
    let moved = dirs[0] * d[0] + dirs[1] * d[1] + dirs[2] * d[2];
    assert!((moved - vec2(10.0, 0.0)).length() < 1e-3 && d.iter().all(|m| *m != 0.0));
}

#[test]
fn a_dragged_corner_stays_under_the_pointer() {
    let space = Space::test_box(camera(0.4, 0.6));
    let mut slice = boxed([0.2; 3], [0.8; 3]);
    let corner = Handle {
        sides: [Some(true), Some(false), Some(true)],
    };
    let before = screen(&space, corner.point(&slice, 0.0));
    let motion = Vec2::new(-6.0, 4.0);
    apply(&space, &mut slice, corner, 0.0, motion);
    let after = screen(&space, corner.point(&slice, 0.0));
    assert!(
        (after - before - motion).length() < 0.5,
        "{before:?} -> {after:?}"
    );
}

#[test]
fn faces_held_at_the_plot_edge_leave_the_motion_to_the_others() {
    // The whole box: the edge's x face cannot grow past 1, so dragging
    // right and down moves only the y face (down) instead of nothing.
    let space = Space::test_box(camera(0.0, 0.0));
    let mut slice = boxed([0.0; 3], [1.0; 3]);
    let edge = Handle {
        sides: [Some(true), Some(false), None],
    };
    apply(&space, &mut slice, edge, 0.0, vec2(10.0, 10.0));
    assert_eq!(slice.hi[0], 1.0);
    assert!((slice.lo[1] - 10.0 / 240.0).abs() < 2e-3, "{slice:?}");
}

#[test]
fn an_edge_seen_end_on_moves_only_the_face_across_the_view() {
    // Looking down z: the edge along x on the y and z faces; dragging down
    // moves the y face, the z face (toward the viewer) stays.
    let space = Space::test_box(camera(0.0, 0.0));
    let mut slice = boxed([0.0; 3], [1.0; 3]);
    let edge = Handle {
        sides: [None, Some(false), Some(false)],
    };
    apply(&space, &mut slice, edge, 0.5, vec2(0.0, 20.0));
    assert!(slice.lo[1] > 0.0);
    assert_eq!((slice.lo[2], slice.hi[2]), (0.0, 1.0));
}

#[test]
fn cuts_close_to_one_cell_snap_to_cells_and_reset_by_their_faces() {
    // 10 cells a side: a cut keeps at least one, and lands on cell edges.
    let space = Space::test_box(camera(0.4, 0.6));
    let mut slice = boxed([0.0; 3], [1.0; 3]);
    let corner = Handle {
        sides: [Some(true), Some(true), Some(true)],
    };
    apply(&space, &mut slice, corner, 0.0, vec2(-5000.0, 5000.0));
    snap(&space, &mut slice, corner);
    for k in 0..3 {
        let width = slice.hi[k] - slice.lo[k];
        assert!(width >= 0.1 - 1e-6, "{slice:?}");
        assert!(((slice.hi[k] * 10.0).round() - slice.hi[k] * 10.0).abs() < 1e-4);
    }
    let edge = Handle {
        sides: [None, Some(true), Some(true)],
    };
    reset(&mut slice, Grip::Box(edge), 3);
    assert_eq!(
        (slice.lo[1], slice.hi[1], slice.lo[2], slice.hi[2]),
        (0.0, 1.0, 0.0, 1.0)
    );
}

#[test]
fn a_dragged_plane_moves_along_its_axis_only_and_snaps_to_a_cell() {
    let space = Space::test_box(camera(0.4, 0.6));
    let mut slice = Slicer::default();
    slice.set_mode(SliceMode::Planes);
    assert_eq!(slice.planes, [Some(0.5), Some(0.5), None]);
    let (edges, n) = planes::edges(&slice.planes);
    assert_eq!(n, 8);
    // The x plane's edge on the top face (v = 0), grabbed at its middle.
    let edge = PlaneEdge {
        plane: 0,
        side: 1,
        hi: false,
    };
    let at = screen(&space, edge.point(&slice, 0.5));
    let grabbed = guide::hit(&space, &slice, &edges[..n], at, GRAB_RADIUS);
    assert_eq!(grabbed.map(|(e, _)| e.plane), Some(0));
    planes::apply(&space, &mut slice, edge, 0.5, vec2(30.0, 0.0));
    let moved = slice.planes[0].unwrap_or(0.5);
    assert!(moved > 0.5 && moved <= 1.0, "{moved}");
    assert_eq!(slice.planes[1..], [Some(0.5), None]);
    let snapped = space.snap_center(0, moved);
    assert!(((snapped * 10.0 - 0.5).fract()).abs() < 1e-4, "{snapped}");
    reset(&mut slice, Grip::Plane(edge), 3);
    assert_eq!(slice.planes[0], Some(0.5));
}

#[test]
fn resize_cursors_point_the_way_a_grip_moves() {
    use super::cursor::resize_icon;
    use egui::CursorIcon as C;
    assert_eq!(resize_icon(vec2(1.0, 0.1)), C::ResizeHorizontal);
    assert_eq!(resize_icon(vec2(-1.0, 0.0)), C::ResizeHorizontal);
    assert_eq!(resize_icon(vec2(0.1, 1.0)), C::ResizeVertical);
    // Screen y grows down: down-right is the NW-SE diagonal.
    assert_eq!(resize_icon(vec2(1.0, 1.0)), C::ResizeNwSe);
    assert_eq!(resize_icon(vec2(1.0, -1.0)), C::ResizeNeSw);
    assert_eq!(resize_icon(Vec2::ZERO), C::Grab);
}
