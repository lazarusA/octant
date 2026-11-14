//! Remote GRIB files read where NOAA publishes them, opened as the app opens them. Needs
//! the network: `cargo test --lib grib_remote -- --ignored --nocapture`.

use std::time::Instant;

use crate::data::backends::netcdf::NetCdfBlockStore;
use crate::data::blocks::BlockStore;

/// Opens `url`, then reads the first plane of `var` (`[y, x]`); prints how long each took.
fn open_and_read(url: &str, var: &str) -> Vec<f32> {
    let started = Instant::now();
    let store = NetCdfBlockStore::open_local(url).expect("open");
    let meta = store.inspect().expect("inspect");
    println!(
        "{url}: {} variables in {:?}",
        meta.variables.len(),
        started.elapsed()
    );
    let info = meta.variables.iter().find(|v| v.name == var).expect(var);
    let shape: Vec<usize> = info.shape.iter().map(|&n| n as usize).collect();
    let lead = vec![0; shape.len().saturating_sub(2)];
    let started = Instant::now();
    let request = super::plane_request(var, &shape, &lead);
    let block = store.fetch_block(&request).expect("fetch");
    println!("{var}: read in {:?}", started.elapsed());
    block.values.to_vec()
}

#[test]
#[ignore = "network"]
fn grib_remote_hrrr_surface_file() {
    let url = "s3://noaa-hrrr-bdp-pds/hrrr.20250101/conus/hrrr.t00z.wrfsfcf01.grib2";
    let values = open_and_read(url, "TMP_hag");
    let finite: Vec<f32> = values.into_iter().filter(|v| v.is_finite()).collect();
    assert!(!finite.is_empty());
    assert!(
        finite.iter().all(|t| (180.0..340.0).contains(t)),
        "2 m temperature in K"
    );
}

#[test]
#[ignore = "network"]
fn grib_remote_sidecar_lists_every_message() {
    let url =
        "https://noaa-hrrr-bdp-pds.s3.amazonaws.com/hrrr.20250101/conus/hrrr.t00z.wrfsfcf01.grib2";
    let offsets = crate::data::backends::grib::sidecar_offsets(url, u64::MAX).expect("sidecar");
    println!("{} messages in the sidecar", offsets.len());
    assert!(offsets.len() > 100 && offsets[0] == 0);
}
