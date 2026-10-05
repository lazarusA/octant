use super::*;
use crate::app::OctantApp;
use crate::data::VolumeData;
use crate::plots::PlotType;
use egui::{Pos2, Rect, pos2};

fn create_test_app_with_volume() -> OctantApp {
    OctantApp {
        active_plot_type: PlotType::Volume,
        matrix_data: None,
        volume_data: Some(VolumeData {
            dataset_name: "test_volume".to_string(),
            values: vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0],
            width: 2,
            height: 2,
            depth: 2,
            min_val: 10.0,
            max_val: 80.0,
        }),
        color_range_min: 10.0,
        color_range_max: 80.0,
        show_hover_card: true,
        ..Default::default()
    }
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

    let (norm_x, norm_y, is_valid_hit, geo_coords, point_3d_hit) = resolve_hit_coordinates(
        &app,
        None,
        &camera,
        sampler.as_ref(),
        &transform_2d,
        rect,
        hover_pos,
    );

    assert!(is_valid_hit, "center of volume must be a valid raycast hit");
    assert!(geo_coords.is_none());
    assert!(point_3d_hit.is_some());
    let (hit_x, hit_y, hit_z, hit_val) = point_3d_hit.unwrap_or((0, 0, 0, 0.0));
    assert!(hit_x < 2);
    assert!(hit_y < 2);
    assert!(hit_z < 2);
    assert!((10.0..=80.0).contains(&hit_val));
    assert!(norm_x > 0.0 && norm_x < 1.0);
    assert!(norm_y > 0.0 && norm_y < 1.0);
}

#[test]
fn test_point_cloud_hover_hit_coordinates_resolved() {
    let mut app = create_test_app_with_volume();
    app.active_plot_type = PlotType::PointCloud;
    let rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(800.0, 600.0));
    let hover_pos = pos2(400.0, 300.0);
    let sampler = VolumeSampler::from_app(&app, None);
    let camera = Camera3D::from_app(&app, rect);
    let transform_2d = Transform2D::from_app(&app, rect);

    let (_, _, is_valid_hit, _, point_3d_hit) = resolve_hit_coordinates(
        &app,
        None,
        &camera,
        sampler.as_ref(),
        &transform_2d,
        rect,
        hover_pos,
    );

    assert!(
        is_valid_hit,
        "center of point cloud must be a valid raycast hit"
    );
    assert!(point_3d_hit.is_some());
}

#[test]
fn test_volume_target_pos_and_entries_without_matrix_data() {
    let app = create_test_app_with_volume();
    let rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(800.0, 600.0));
    let hover_pos = pos2(400.0, 300.0);
    let sampler = VolumeSampler::from_app(&app, None);
    let camera = Camera3D::from_app(&app, rect);
    let transform_2d = Transform2D::from_app(&app, rect);

    let (norm_x, norm_y, _, geo_coords, point_3d_hit) = resolve_hit_coordinates(
        &app,
        None,
        &camera,
        sampler.as_ref(),
        &transform_2d,
        rect,
        hover_pos,
    );

    let (raw_val, entries, px, py) = resolve_cell_value_and_dim_entries(
        &app,
        None,
        None,
        None,
        sampler.as_ref(),
        norm_x,
        norm_y,
        geo_coords,
        point_3d_hit,
    );

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
        raw_val,
        point_3d_hit,
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
