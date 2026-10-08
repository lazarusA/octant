//! Tests for volume aspect ratio stability with incoming slices and
//! uninterrupted time stepping when switching plot types during playback.

mod common;
use common::drain;
use std::time::Instant;

use octant::app::{OctantApp, StoreKind};
use octant::data::{BlockStore, backends::ProceduralBlockStore};
use octant::plots::PlotType;

fn tick(app: &mut OctantApp) {
    app.advance_playback(Instant::now());
    drain(app);
}

fn create_volume_app() -> OctantApp {
    let mut app = OctantApp::default();
    let store = ProceduralBlockStore::open("procedural://volume4d").expect("open procedural store");
    let meta = store.inspect().expect("inspect procedural store");
    app.selected.store_kind = StoreKind::ProceduralVolume4D;
    app.selected.store_target = "procedural://volume4d".to_string();
    app.load_new_metadata(meta);
    app.show_hero = false;
    app.selected.plot_type = PlotType::Volume;
    app
}

#[test]
fn test_playing_volume_then_changing_to_heatmap_continues_time_stepping() {
    let mut app = create_volume_app();
    app.plot_selection();
    drain(&mut app);

    app.is_playing = true;
    for _ in 0..3 {
        tick(&mut app);
    }
    let step_before_switch = app.current_timestep;
    assert!(step_before_switch >= 2);

    // Switch from Volume to Heatmap while playing:
    app.switch_plot_type(PlotType::Heatmap);
    drain(&mut app);

    assert_eq!(app.selected.plot_type, PlotType::Heatmap);
    assert!(app.is_playing, "playback should remain active");

    // Advance playback for several frames:
    for _ in 0..5 {
        tick(&mut app);
    }

    assert!(
        app.current_timestep > step_before_switch,
        "time stepping must continue advancing after switching to Heatmap (was {step_before_switch}, now {})",
        app.current_timestep
    );
    assert!(
        app.layers.base.data.matrix.is_some(),
        "2D matrix data must be present for heatmap"
    );
}

#[test]
fn test_volume_aspect_ratio_stable_with_incoming_slices() {
    let mut app = create_volume_app();

    // Set slider ranges and z-scale:
    app.selected.dim_ranges[1] = (0, 15); // depth: 16 slices
    app.selected.dim_ranges[2] = (0, 31); // height: 32
    app.selected.dim_ranges[3] = (0, 31); // width: 32
    app.volume_z_scale = 1.5;

    let initial_aspect = app.get_3d_aspect_ratio();

    app.plot_selection();
    drain(&mut app);

    let allocated_aspect = app.get_3d_aspect_ratio();
    assert_eq!(
        initial_aspect, allocated_aspect,
        "aspect ratio before and after initial volume allocation must match"
    );

    let allocations_before = app.layers.base.data.volume_allocations;

    // Advance playback with incoming slices:
    app.is_playing = true;
    for _ in 0..10 {
        tick(&mut app);
        let current_aspect = app.get_3d_aspect_ratio();
        assert_eq!(
            current_aspect, initial_aspect,
            "volume aspect ratio must not change as incoming slices arrive"
        );
    }

    assert_eq!(
        app.layers.base.data.volume_allocations, allocations_before,
        "volume should not be reallocated when incoming slices update the subvolume"
    );
}

#[test]
fn test_3d_time_lat_lon_defaults_and_plot_switch() {
    let mut app = OctantApp::default();
    let mut meta = octant::data::DatasetMetadata::default();
    meta.variables.push(octant::data::VariableInfo {
        name: "air_temperature_2m".to_string(),
        data_type: "float32".to_string(),
        shape: vec![10, 32, 64],
        chunk_shape: vec![1, 32, 64],
        dimension_names: vec!["time".to_string(), "lat".to_string(), "lon".to_string()],
        units: Some("K".to_string()),
        long_name: Some("Air Temperature".to_string()),
        temporal_resolution: Some("1 day".to_string()),
        time_coverage_start: None,
        time_coverage_end: None,
        file_size: 10 * 32 * 64 * 4,
        attributes: std::collections::HashMap::new(),
    });
    app.load_new_metadata(meta);

    // Initial Euclidean assignments for 3D [time, lat, lon]:
    // time: Animated AND SpatialRole::Z
    // lat: spatial: Y
    // lon: spatial: X
    assert_eq!(
        app.selected.dim_config[0].animation,
        octant::app::AnimationRole::Animated
    );
    assert_eq!(
        app.selected.dim_config[0].spatial,
        octant::app::SpatialRole::Z
    );
    assert_eq!(
        app.selected.dim_config[1].spatial,
        octant::app::SpatialRole::Y
    );
    assert_eq!(
        app.selected.dim_config[2].spatial,
        octant::app::SpatialRole::X
    );

    // Set animated range to (0, 9):
    app.selected.dim_ranges[0] = (0, 9);
    app.selected.dim_config[0].range = (0, 9);

    // Switch to Volume:
    app.switch_plot_type(PlotType::Volume);
    assert_eq!(app.selected.plot_type, PlotType::Volume);
    // time becomes spatial Z for Volume, while staying Animated:
    assert_eq!(
        app.selected.dim_config[0].spatial,
        octant::app::SpatialRole::Z
    );
    assert_eq!(
        app.selected.dim_config[0].animation,
        octant::app::AnimationRole::Animated
    );

    // Switch from Volume back to Heatmap:
    app.switch_plot_type(PlotType::Heatmap);
    assert_eq!(app.selected.plot_type, PlotType::Heatmap);
    // time must return to spatial None, active true, animation Animated:
    assert_eq!(
        app.selected.dim_config[0].spatial,
        octant::app::SpatialRole::None
    );
    assert_eq!(
        app.selected.dim_config[0].animation,
        octant::app::AnimationRole::Animated
    );
    assert!(app.selected.dim_config[0].active);
    assert_eq!(app.selected.dim_config[0].range, (0, 9));
}
