//! A link's step outside the selected time window moves the window.

use super::tests::{app_of, apply};

/// A link's step past the default time window moves the window to hold it
/// (keeping its width), so plotting does not clamp the step away.
#[test]
fn a_step_past_the_time_window_moves_the_window() {
    use crate::app::test_support::{write_array, write_coordinate};
    use serde_json::json;
    let store = std::sync::Arc::new(zarrs::storage::store::MemoryStore::new());
    let steps = 400;
    write_array(
        &store,
        "tas",
        &["time", "lat", "lon"],
        &[steps, 5, 4],
        &[1, 5, 4],
    );
    let days: Vec<f32> = (0..steps).map(|d| d as f32).collect();
    let units = json!({ "units": "days since 2020-01-01" });
    write_coordinate(&store, "time", &days, units);
    write_coordinate(&store, "lat", &[10.0, 20.0, 30.0, 40.0, 50.0], json!({}));
    write_coordinate(&store, "lon", &[0.0, 90.0, 180.0, 270.0], json!({}));
    for (date, step) in [("2021-02-03", 399), ("2020-07-19", 200)] {
        let mut app = app_of(store.clone(), "https://share.test/long.zarr");
        let src = "?src=https://share.test/long.zarr&var=tas&anim=time";
        apply(&mut app, &format!("{src}&time={date}"));
        assert_eq!(app.playback.current_timestep, step, "{date}");
        let (start, end) = app.plotted().dim_config[0].range;
        assert!((start..=end).contains(&step), "{date}: {start}..{end}");
        assert!(end < steps as usize, "{date}: {start}..{end}");
    }
}
