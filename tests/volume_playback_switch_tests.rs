//! Regression tests for playing an animated volume (slice of N time steps),
//! switching to 2D/Heatmap, advancing playback past the initial selection,
//! and switching back to Volume.

use std::time::{Duration, Instant};

use octant::app::{OctantApp, StoreKind};
use octant::data::{BlockStore, OctantBlock, backends::ProceduralBlockStore};
use octant::plots::PlotType;

fn drain(app: &mut OctantApp) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        app.poll_block_prefetch_results();
        if app.block_prefetcher.pending_count() == 0 {
            break;
        }
        if Instant::now() >= deadline {
            panic!("prefetcher did not finish");
        }
        std::thread::sleep(Duration::from_millis(2));
    }
}

fn tick(app: &mut OctantApp) {
    app.advance_playback(Instant::now());
    drain(app);
}

fn create_volume_app() -> OctantApp {
    let mut app = OctantApp::default();
    let store = ProceduralBlockStore::open("procedural://volume4d").expect("open procedural store");
    let meta = store.inspect().expect("inspect procedural store");
    app.selected_store_kind = StoreKind::ProceduralVolume4D;
    app.store_target_input = "procedural://volume4d".to_string();
    app.load_new_metadata(meta);
    app.show_hero = false;
    app.active_plot_type = PlotType::Volume;
    app
}

#[test]
fn test_playing_volume_switch_to_heatmap_play_past_selection_and_switch_back_to_volume() {
    let mut app = create_volume_app();
    app.selected_dim_ranges[0] = (0, 3);
    app.dim_config[0].range = (0, 3);
    app.plot_selection();
    drain(&mut app);

    assert!(
        app.volume_data.is_some(),
        "volume data should be present initially"
    );

    app.is_playing = true;
    for _ in 0..2 {
        tick(&mut app);
    }
    assert!(app.current_timestep <= 3);

    app.switch_plot_type(PlotType::Heatmap);
    drain(&mut app);
    assert_eq!(app.active_plot_type, PlotType::Heatmap);

    for _ in 0..8 {
        tick(&mut app);
    }
    assert!(
        app.current_timestep > 3,
        "timestep must be beyond initial selection (now {})",
        app.current_timestep
    );
    assert!(
        app.matrix_data.is_some(),
        "2D matrix data should be present for Heatmap"
    );

    app.switch_plot_type(PlotType::Volume);
    drain(&mut app);
    assert_eq!(app.active_plot_type, PlotType::Volume);
    assert!(app.volume_data.is_some());
}

#[test]
fn test_3d_time_series_volume_playback_switch_to_2d_and_back() {
    let mut app = OctantApp::default();
    let mut meta = octant::data::DatasetMetadata::default();
    meta.variables.push(octant::data::VariableInfo {
        name: "temp_3d".to_string(),
        data_type: "float32".to_string(),
        shape: vec![20, 32, 32],
        chunk_shape: vec![1, 32, 32],
        dimension_names: vec!["time".to_string(), "lat".to_string(), "lon".to_string()],
        units: Some("K".to_string()),
        long_name: Some("3D Temperature".to_string()),
        temporal_resolution: Some("1 day".to_string()),
        time_coverage_start: None,
        time_coverage_end: None,
        file_size: 20 * 32 * 32 * 4,
        attributes: std::collections::HashMap::new(),
    });
    app.load_new_metadata(meta);

    app.active_plot_type = PlotType::Volume;
    app.dim_config[0].spatial = octant::app::SpatialRole::Z;
    app.dim_config[0].animation = octant::app::AnimationRole::Animated;
    app.dim_config[0].range = (0, 3);
    app.selected_dim_ranges[0] = (0, 3);

    let initial_values = vec![100.0f32; 4 * 32 * 32];
    let initial_block = OctantBlock::new(
        "temp_3d".to_string(),
        vec![4, 32, 32],
        vec!["time".to_string(), "lat".to_string(), "lon".to_string()],
        vec![0, 0, 0],
        initial_values,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );
    app.plot_selection();
    app.apply_block_projection(&initial_block);

    assert!(app.volume_data.is_some(), "volume data should be present");
    assert_eq!(app.volume_data.as_ref().unwrap().depth, 4);

    app.switch_plot_type(PlotType::Heatmap);
    assert_eq!(app.active_plot_type, PlotType::Heatmap);

    app.current_timestep = 10;
    let step10_values = vec![200.0f32; 32 * 32];
    let step10_block = OctantBlock::new(
        "temp_3d".to_string(),
        vec![1, 32, 32],
        vec!["time".to_string(), "lat".to_string(), "lon".to_string()],
        vec![10, 0, 0],
        step10_values,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );
    app.apply_block_projection(&step10_block);
    assert!(
        app.matrix_data.is_some(),
        "matrix data should be present for Heatmap"
    );

    app.switch_plot_type(PlotType::Volume);
    assert_eq!(app.active_plot_type, PlotType::Volume);

    app.apply_block_projection(&step10_block);

    let vdata = app
        .volume_data
        .as_ref()
        .expect("volume data must be present when returning to Volume");
    assert_eq!(vdata.depth, 4);
    let non_nan_count = vdata.values.iter().filter(|v| !v.is_nan()).count();
    assert!(
        non_nan_count > 0,
        "volume data must contain valid non-NaN values for current timestep when returning to Volume"
    );
}
