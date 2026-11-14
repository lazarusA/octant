//! dynamical.org's virtual HRRR Icechunk repo, whose chunks are GRIB messages in NOAA's
//! bucket: wind through the `gribberish` codec, temperature through `scale_offset`
//! lifted into calibration (°C). Needs the network:
//! `cargo test --lib hrrr_icechunk -- --ignored --nocapture`.

use crate::data::backends::IcechunkBlockStore;
use crate::data::blocks::BlockStore;

const HRRR: &str = "https://dynamical-noaa-hrrr.s3.us-west-2.amazonaws.com/noaa-hrrr-forecast-48-hour-virtual/v0.5.0.icechunk/";

/// The finite values of the latest written `[init_time, lead_time=1, y, x]` plane of `var`.
fn latest_plane(store: &IcechunkBlockStore, var: &str) -> Vec<f32> {
    let meta = store.inspect().expect("inspect");
    let info = meta.variables.iter().find(|v| v.name == var).expect(var);
    let shape: Vec<usize> = info.shape.iter().map(|&n| n as usize).collect();
    assert_eq!(shape.len(), 4, "{var} is [init_time, lead_time, y, x]");
    // The newest init times may not be written yet: step back until one is.
    for init in (0..shape[0]).rev().take(48) {
        let request = crate::data::backends::grib::plane_request(var, &shape, &[init, 1]);
        let block = store.fetch_block(&request).expect("fetch");
        let finite: Vec<f32> = block
            .values
            .iter()
            .copied()
            .filter(|v| v.is_finite())
            .collect();
        if !finite.is_empty() {
            println!("{var}: init_time {init}, {} finite values", finite.len());
            return finite;
        }
    }
    panic!("no written {var} plane among the latest 48 init times");
}

#[test]
#[ignore = "network"]
fn hrrr_icechunk_grib_chunks_read() {
    let store = IcechunkBlockStore::open(HRRR).expect("open HRRR");

    let wind = latest_plane(&store, "wind_u_10m");
    assert!(wind.iter().all(|v| v.abs() < 120.0), "wind in m/s");

    let temperature = latest_plane(&store, "temperature_2m");
    let (min, max) =
        (temperature.iter()).fold((f32::MAX, f32::MIN), |(lo, hi), &v| (lo.min(v), hi.max(v)));
    println!("temperature_2m: {min} to {max}");
    assert!(min > -80.0 && max < 65.0, "temperature in °C, not K");
}
