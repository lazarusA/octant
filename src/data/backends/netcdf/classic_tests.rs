//! Classic files (CDF-1, CDF-2, CDF-5) written by netCDF-C read the same through
//! `classic/` as through netCDF-C (`test_support::Parity`).

use netcdf::Options;

use super::test_support::{NcChar, Parity, TempNc, char_rows, netcdf_lock};
use crate::data::blocks::BlockStore;
use crate::data::slice_request::{DimensionSelection as Sel, SliceRequest};

/// Writes the test file; classic files take every definition before any data.
fn write(nc: &TempNc, options: Options, cdf5: bool) {
    let mut f = netcdf::create_with(&nc.0, options).expect("create");
    f.add_unlimited_dimension("time").expect("time");
    f.add_dimension("lat", 3).expect("lat");
    f.add_dimension("lon", 4).expect("lon");
    f.add_dimension("strlen", 6).expect("strlen");
    f.add_attribute("title", "classic").expect("title");
    f.add_variable::<f32>("lat", &["lat"]).expect("lat");
    f.add_variable::<f64>("lon", &["lon"]).expect("lon");
    f.add_variable::<NcChar>("lat_name", &["lat", "strlen"])
        .expect("lat_name");
    let mut t = f
        .add_variable::<i16>("t", &["time", "lat", "lon"])
        .expect("t");
    t.put_attribute("scale_factor", 0.5f32).expect("scale");
    t.put_attribute("add_offset", 100.0f64).expect("offset");
    t.put_attribute("_FillValue", -1i16).expect("fill");
    // A second record variable, so records interleave.
    f.add_variable::<i8>("n", &["time"]).expect("n");
    if cdf5 {
        f.add_variable::<i64>("big", &["lon"]).expect("big");
    }
    f.enddef().expect("end of definitions");
    let put = |name: &str, f: &mut netcdf::FileMut, write: &dyn Fn(&mut netcdf::VariableMut)| {
        write(&mut f.variable_mut(name).expect(name));
    };
    put("lat", &mut f, &|v| {
        v.put_values(&[10.0f32, 20.0, 30.0], ..).expect("lat")
    });
    put("lon", &mut f, &|v| {
        v.put_values(&[0.5f64, 1.5, 2.5, 3.5], ..).expect("lon")
    });
    let names = char_rows(&["a", "bb", "ccc"], 6);
    put("lat_name", &mut f, &|v| {
        v.put_values(&names, ..).expect("names")
    });
    let values: Vec<i16> = (0..36).map(|i| if i == 7 { -1 } else { i }).collect();
    put("t", &mut f, &|v| {
        v.put_values(&values, (0..3, 0..3, 0..4)).expect("t")
    });
    put("n", &mut f, &|v| {
        v.put_values(&[1i8, -2, 3], 0..3).expect("n")
    });
    if cdf5 {
        put("big", &mut f, &|v| {
            v.put_values(&[1i64 << 40, -3, 0, 7], ..).expect("big")
        });
    }
}

#[test]
fn classic_files_read_as_netcdf_c_reads_them() {
    let _lock = netcdf_lock();
    for (name, options) in [
        ("cdf1", Options::empty()),
        ("cdf2", Options::_64BIT_OFFSET),
        ("cdf5", Options::_64BIT_DATA),
    ] {
        let nc = TempNc::new(name);
        write(&nc, options, name == "cdf5");
        let store = Parity::open(nc.path()).expect("open");
        let meta = store.inspect().expect("inspect");
        assert_eq!(meta.name, "classic");
        let windows = [
            SliceRequest::full_range("t", &[3, 3, 4]),
            SliceRequest::new("t", vec![Sel::range(1, 3), Sel::index(2), Sel::range(1, 3)]),
            SliceRequest::new("n", vec![Sel::range(1, 3)]),
            SliceRequest::new("lon", vec![Sel::range(1, 4)]),
        ];
        for request in windows {
            store.fetch_block(&request).expect(name);
        }
    }
}

/// Real classic files read alike, every variable's first window included; the files live
/// in the git-ignored `dev/land_sea_mask_grids`:
/// `cargo test --lib real_classic_files -- --ignored --nocapture`.
#[test]
#[ignore = "local files"]
fn real_classic_files_read_as_netcdf_c_reads_them() {
    let _lock = netcdf_lock();
    for name in ["2004050300_eta_211.nc", "tos_O1_2001-2002.nc"] {
        let path = format!(
            "{}/dev/land_sea_mask_grids/{name}",
            env!("CARGO_MANIFEST_DIR")
        );
        let store = Parity::open(&path).expect("open");
        let meta = store.inspect().expect("inspect");
        for var in meta.variables.iter().filter(|v| v.data_type != "char") {
            let shape: Vec<usize> = var.shape.iter().map(|&n| (n as usize).min(8)).collect();
            store
                .fetch_block(&SliceRequest::full_range(&var.name, &shape))
                .expect(&var.name);
        }
        println!("{name}: {} variables read alike", meta.variables.len());
    }
}
