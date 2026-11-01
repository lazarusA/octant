use super::*;
use crate::app::OctantApp;
use crate::app::VariableSelection;
use crate::data::VolumeData;
use crate::plots::PlotType;
use egui::{Pos2, Rect, pos2};

fn create_test_app_with_volume() -> OctantApp {
    let mut app = OctantApp {
        selected: VariableSelection {
            plot_type: PlotType::Volume,
            ..Default::default()
        },
        ..Default::default()
    };
    app.layout.show_hover_card = true;
    app.layers.base.color.range_min = 10.0;
    app.layers.base.color.range_max = 80.0;
    app.layers.base.data.volume = Some(VolumeData {
        dataset_name: "test_volume".to_string(),
        values: vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0],
        width: 2,
        height: 2,
        depth: 2,
        min_val: 10.0,
        max_val: 80.0,
    });
    app
}

#[test]
fn test_volume_sampler_builds_without_matrix_data() {
    let app = create_test_app_with_volume();
    let sampler = VolumeSampler::from_app(&app, None);
    assert!(sampler.is_some(), "sampler must construct from volume_data");
    let sampler = sampler.unwrap_or_else(|| panic!("expected sampler"));
    assert_eq!(sampler.width, 2);
    assert_eq!(sampler.height, 2);
    assert_eq!(sampler.depth, 2);
    assert_eq!(sampler.sample_cell(0, 0, 0), 10.0);
    assert_eq!(sampler.sample_cell(1, 1, 1), 80.0);
}

#[test]
fn test_volume_hover_hit_coordinates_resolved() {
    let app = create_test_app_with_volume();
    let rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(800.0, 600.0));
    let hover_pos = pos2(400.0, 300.0); // Center of canvas looking straight at volume
    let sampler = VolumeSampler::from_app(&app, None);
    let camera = Camera3D::from_app(&app, rect);
    let transform_2d = Transform2D::from_app(&app, rect);

    let hit = resolve_hit_coordinates(
        &app,
        None,
        &camera,
        sampler.as_ref(),
        &transform_2d,
        rect,
        hover_pos,
    );

    assert!(hit.is_valid, "center of volume must be a valid raycast hit");
    assert!(hit.geo_coords.is_none());
    assert!(hit.point_3d.is_some());
    let (hit_x, hit_y, hit_z, hit_val) = hit.point_3d.unwrap_or((0, 0, 0, 0.0));
    assert!(hit_x < 2);
    assert!(hit_y < 2);
    assert!(hit_z < 2);
    assert!((10.0..=80.0).contains(&hit_val));
    assert!(hit.norm_x > 0.0 && hit.norm_x < 1.0);
    assert!(hit.norm_y > 0.0 && hit.norm_y < 1.0);
}

#[test]
fn test_point_cloud_hover_hit_coordinates_resolved() {
    let mut app = create_test_app_with_volume();
    app.selected.plot_type = PlotType::PointCloud;
    let rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(800.0, 600.0));
    let hover_pos = pos2(400.0, 300.0);
    let sampler = VolumeSampler::from_app(&app, None);
    let camera = Camera3D::from_app(&app, rect);
    let transform_2d = Transform2D::from_app(&app, rect);

    let hit = resolve_hit_coordinates(
        &app,
        None,
        &camera,
        sampler.as_ref(),
        &transform_2d,
        rect,
        hover_pos,
    );

    assert!(
        hit.is_valid,
        "center of point cloud must be a valid raycast hit"
    );
    assert!(hit.point_3d.is_some());
}

#[test]
fn test_volume_target_pos_and_entries_without_matrix_data() {
    let app = create_test_app_with_volume();
    let rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(800.0, 600.0));
    let hover_pos = pos2(400.0, 300.0);
    let sampler = VolumeSampler::from_app(&app, None);
    let camera = Camera3D::from_app(&app, rect);
    let transform_2d = Transform2D::from_app(&app, rect);

    let hit = resolve_hit_coordinates(
        &app,
        None,
        &camera,
        sampler.as_ref(),
        &transform_2d,
        rect,
        hover_pos,
    );

    let (raw_val, entries, px, py) =
        resolve_cell_value_and_dim_entries(&app, None, None, None, sampler.as_ref(), &hit);

    assert!((10.0..=80.0).contains(&raw_val));
    assert_eq!(
        entries.len(),
        3,
        "3D volume must have 3 spatial dimension entries (z, y, x)"
    );

    let target_pos = resolve_target_screen_pos(
        &app,
        None,
        &camera,
        sampler.as_ref(),
        &transform_2d,
        px,
        py,
        hit.point_3d,
    );

    assert!(
        target_pos.is_some(),
        "3D hit must project to screen coordinates for leader line"
    );
    let target = target_pos.unwrap_or(Pos2::ZERO);
    assert!(
        rect.contains(target),
        "target pos {target:?} must lie inside canvas {rect:?}"
    );
}

#[test]
fn a_thinned_mesh_reads_the_cell_it_draws() {
    // 1001 columns thinned by 2: 501 drawn, not a whole multiple.
    let (w, h) = (1001, 3);
    let values = (0..w * h).map(|i| i as f32).collect();
    let full = crate::data::MatrixData::new(w, h, values, 0.0, 1.0, "m".into(), 1);
    let thinned = full.decimated(2).expect("thinned");
    let mut app = OctantApp::default();
    app.layers.base.data.mesh = Some(crate::app::layers::MeshGrid {
        matrix: thinned,
        stride: 2,
    });
    app.layers.base.data.matrix = Some(full);
    // The centre of drawn column 400 of 501, row 0 of 2.
    let cell = super::hit::drawn_cell(&app, 400.5 / 501.0, 0.25);
    // Column 400 of 501 is drawn over 800.2 (1001 / 501 cells each); row 0
    // of 2 over cell 0.75 of 3.
    assert_eq!(
        cell,
        Some((800, 0)),
        "the cell under each drawn cell's middle"
    );
    let (_, _, px, py) = super::entries_2d::resolve_2d_plot_entries(
        &app,
        app.layers.base.data.matrix.as_ref().expect("matrix"),
        None,
        None,
        (400.5 / 501.0, 0.25),
        cell,
        None,
    );
    assert_eq!((px, py), (800, 0));
}
