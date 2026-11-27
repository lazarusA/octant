//! A 3D plot's camera through a link (`camera=pitch,yaw,distance[,tx,ty,tz]`):
//! written for every 3D plot, the default view too, replayed as
//! `SetCamera`, clamped as the orbit controller clamps.

use super::parse_share;
use super::tests::link_of;
use crate::app::OctantApp;
use crate::app::actions::{AppAction, Failure};
use crate::app::test_support::{poll_until, pyramid_app, selection_of};
use crate::plots::PlotType;
use crate::plots::camera::{MAX_DISTANCE, PITCH_LIMIT, TARGET_LIMIT, min_distance};

/// The pyramid's `0/t2m` plotted, then drawn as `plot_type` (links read only
/// the plot type of what is plotted).
fn plotted_as(plot_type: PlotType) -> OctantApp {
    let (mut app, meta) = pyramid_app();
    app.layout.canvas_px = [20, 20];
    app.selected = selection_of(&mut app, &meta, "0/t2m");
    app.plot_selection();
    poll_until(&mut app, |app| app.layers.base.data.matrix.is_some());
    app.layers.base.selection_mut().plot_type = plot_type;
    app
}

/// The value of `key` in `link`.
fn value<'a>(link: &'a str, key: &str) -> Option<&'a str> {
    let query = link.split_once('?').map_or(link, |(_, q)| q);
    query
        .split('&')
        .find_map(|p| p.strip_prefix(key)?.strip_prefix('='))
}

fn camera(pitch: f32, yaw: f32, distance: f32, target: Option<[f32; 3]>) -> AppAction {
    AppAction::SetCamera {
        pitch,
        yaw,
        distance,
        target,
    }
}

#[test]
fn camera_reads_into_set_camera_after_the_view() {
    let read = |value: &str| {
        let link = format!("?src=https://x.org/a&var=t&camera={value}&view=1..2,3..4");
        let actions = parse_share(&link).unwrap_or_default();
        let at = actions
            .iter()
            .position(|a| matches!(a, AppAction::SetCamera { .. }));
        let view = actions
            .iter()
            .position(|a| matches!(a, AppAction::SetView { .. }));
        assert!(at.is_none() || at > view, "after the view: {actions:?}");
        at.map(|i| actions[i].clone())
    };
    assert_eq!(read("14.3,-30,2.6"), Some(camera(14.3, -30.0, 2.6, None)));
    let with_target = Some(camera(20.0, 45.0, 1.8, Some([0.1, 0.0, -0.2])));
    assert_eq!(read("20,45,1.8,0.1,0,-0.2"), with_target);
    for bad in ["14,30", "14,30,2,1", "a,b,c", "nan,0,2", "0,inf,2"] {
        assert_eq!(read(bad), None, "{bad}");
    }
}

#[test]
fn set_camera_clamps_as_dragging_and_zooming_do() {
    let mut app = plotted_as(PlotType::Volume);
    let done = app.dispatch(camera(120.0, 45.0, 100.0, Some([9.0, -9.0, 0.5])));
    assert!(done.is_ok(), "{done:?}");
    let nav = &app.nav;
    assert_eq!(nav.sphere_rotation_x, PITCH_LIMIT);
    assert!((nav.sphere_rotation_y - 45f32.to_radians()).abs() < 1e-6);
    assert_eq!(nav.sphere_zoom, MAX_DISTANCE);
    assert_eq!(nav.orbit_target, [TARGET_LIMIT, -TARGET_LIMIT, 0.5]);
    assert!(!nav.sphere_auto_rotate);

    let done = app.dispatch(camera(-120.0, 0.0, 0.0, None));
    assert!(done.is_ok(), "{done:?}");
    assert_eq!(app.nav.sphere_rotation_x, -PITCH_LIMIT);
    assert_eq!(app.nav.sphere_zoom, min_distance(PlotType::Volume));
    assert_eq!(app.nav.orbit_target, [0.0; 3]);

    // A huge yaw is wrapped, leaving dragging f32 steps to turn by.
    let done = app.dispatch(camera(0.0, 1e9 + 45.0, 2.0, None));
    assert!(done.is_ok(), "{done:?}");
    assert!((0.0..std::f32::consts::TAU).contains(&app.nav.sphere_rotation_y));

    // A tool call is not parsed as a link is: non-finite values are refused.
    let before = app.nav.sphere_zoom;
    let done = app.dispatch(camera(f32::NAN, 0.0, 2.0, None));
    assert_eq!(done, Err(Failure::Refused));
    assert_eq!(app.nav.sphere_zoom, before);
}

#[test]
fn a_3d_plot_s_camera_is_written_even_at_the_default_view() {
    // A heatmap writes no camera; a 3D plot at the default view does, so the
    // receiver's camera moves back from wherever it was.
    let mut heatmap = plotted_as(PlotType::Heatmap);
    heatmap.nav.sphere_zoom = 1.5;
    assert_eq!(value(&link_of(&mut heatmap), "camera"), None);
    let mut volume = plotted_as(PlotType::Volume);
    assert_eq!(value(&link_of(&mut volume), "camera"), Some("14.3,0,2.5"));

    // Yaw within ±180 degrees, rounded; the target when off center.
    volume.nav.sphere_rotation_y = 7.0; // 401.07 degrees
    volume.nav.sphere_rotation_x = -0.5;
    volume.nav.sphere_zoom = 1.234;
    volume.nav.orbit_target = [0.1, 0.0, -0.25];
    let link = link_of(&mut volume);
    assert_eq!(
        value(&link, "camera"),
        Some("-28.6,41.1,1.234,0.1,0,-0.25"),
        "{link}"
    );

    // The sphere looks at its center: no target.
    let mut sphere = plotted_as(PlotType::Sphere);
    sphere.nav.orbit_target = [0.3, 0.0, 0.0];
    sphere.nav.sphere_zoom = 1.8;
    assert_eq!(value(&link_of(&mut sphere), "camera"), Some("14.3,0,1.8"));
    // Close to the globe the distance keeps its third decimal.
    sphere.nav.sphere_zoom = 1.004;
    assert_eq!(value(&link_of(&mut sphere), "camera"), Some("14.3,0,1.004"));

    // A target that rounds to the center is not written.
    volume.nav.orbit_target = [0.001, 0.0, -0.004];
    let link = link_of(&mut volume);
    assert_eq!(value(&link, "camera"), Some("-28.6,41.1,1.234"), "{link}");
}

#[test]
fn a_3d_plot_s_link_turns_the_receiver_s_camera_the_same() {
    let mut sender = plotted_as(PlotType::Volume);
    sender.nav.sphere_rotation_x = 0.6;
    sender.nav.sphere_rotation_y = -2.0;
    sender.nav.sphere_zoom = 3.4;
    sender.nav.orbit_target = [0.2, -0.1, 0.0];
    let link = link_of(&mut sender);

    let mut receiver = plotted_as(PlotType::Volume);
    let actions = parse_share(&link).unwrap_or_default();
    let set = actions
        .iter()
        .find(|a| matches!(a, AppAction::SetCamera { .. }));
    let done = receiver.dispatch(set.cloned().unwrap_or(AppAction::ResetView));
    assert!(done.is_ok(), "{done:?}");
    assert_eq!(link_of(&mut receiver), link);
    let (a, b) = (&sender.nav, &receiver.nav);
    assert!((a.sphere_rotation_x - b.sphere_rotation_x).abs() < 0.001);
    let turn = |yaw: f32| yaw.rem_euclid(std::f32::consts::TAU);
    assert!((turn(a.sphere_rotation_y) - turn(b.sphere_rotation_y)).abs() < 0.001);
    assert!((a.sphere_zoom - b.sphere_zoom).abs() < 0.01);
}
