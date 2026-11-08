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

/// Real classic and netCDF-4 files read alike, every variable whole; the files live in the
/// git-ignored `dev/land_sea_mask_grids`:
/// `cargo test --lib dev_files_read -- --ignored --nocapture`.
#[test]
#[ignore = "local files"]
fn dev_files_read_as_netcdf_c_reads_them() {
    let _lock = netcdf_lock();
    for name in [
        "2004050300_eta_211.nc",
        "tos_O1_2001-2002.nc",
        "full_healpix.nc",
        "full_gaussian.nc",
        "full_clenshaw.nc",
        "full_octahealpix.nc",
    ] {
        let path = format!(
            "{}/dev/land_sea_mask_grids/{name}",
            env!("CARGO_MANIFEST_DIR")
        );
        let store = Parity::open(&path).expect("open");
        let meta = store.inspect().expect("inspect");
        for var in meta.variables.iter().filter(|v| v.data_type != "char") {
            let shape: Vec<usize> = var.shape.iter().map(|&n| n as usize).collect();
            store
                .fetch_block(&SliceRequest::full_range(&var.name, &shape))
                .expect(&var.name);
        }
        println!("{name}: {} variables read alike", meta.variables.len());
    }
}

/// A variable naming a dimension the file lacks is refused, never indexed (a panic would
/// end the browser's data worker).
#[test]
fn a_missing_dimension_id_is_an_error() {
    let be = |n: u32| n.to_be_bytes();
    let mut bytes = b"CDF\x01".to_vec();
    bytes.extend([be(0), be(0x0A), be(1), be(1)].concat()); // numrecs, one dimension
    bytes.extend(b"x\0\0\0");
    bytes.extend(be(4)); // its length
    bytes.extend([be(0), be(0)].concat()); // no global attributes
    bytes.extend([be(0x0B), be(1), be(1)].concat()); // one variable, its name
    bytes.extend(b"v\0\0\0");
    bytes.extend([be(1), be(5)].concat()); // one dimension: id 5
    bytes.extend([be(0), be(0), be(5), be(16), be(0)].concat()); // attrs, float, vsize, begin
    let nc = TempNc::new("bad_dimid");
    std::fs::write(&nc.0, &bytes).expect("write");
    let error = super::open::open_path(nc.path()).err().expect("refused");
    assert!(error.to_string().contains("no dimension 5"), "{error}");
}

/// One record variable is stored unpadded (records 3 bytes apart here, not 4): read as
/// netCDF-C reads it.
#[test]
fn a_single_record_variable_reads_unpadded() {
    let _lock = netcdf_lock();
    let nc = TempNc::new("one_record_var");
    {
        let mut f = netcdf::create_with(&nc.0, Options::empty()).expect("create");
        f.add_unlimited_dimension("time").expect("time");
        f.add_dimension("x", 3).expect("x");
        f.add_variable::<i8>("n", &["time", "x"]).expect("n");
        f.enddef().expect("end of definitions");
        let mut n = f.variable_mut("n").expect("n");
        let values: Vec<i8> = (0..12).collect();
        n.put_values(&values, (0..4, 0..3)).expect("values");
    }
    let store = Parity::open(nc.path()).expect("open");
    for request in [
        SliceRequest::full_range("n", &[4, 3]),
        SliceRequest::new("n", vec![Sel::range(1, 4), Sel::index(2)]),
    ] {
        store.fetch_block(&request).expect("n");
    }
}

/// A header length past the end of the file is refused at once, not read again and again
/// at twice the size.
#[test]
fn a_header_length_past_the_file_is_an_error() {
    let mut bytes = b"CDF\x01".to_vec();
    bytes.extend(
        [0u32, 0x0A, 1, 0x7FFF_FFFF]
            .iter()
            .flat_map(|n| n.to_be_bytes()),
    );
    let nc = TempNc::new("bad_name_length");
    std::fs::write(&nc.0, &bytes).expect("write");
    let error = super::open::open_path(nc.path()).err().expect("refused");
    assert!(error.to_string().contains("past the end"), "{error}");
}

/// Runs at least a read-ahead block apart are read value by value, not a block each: the
/// time coordinate of a file with large records, and a point over a fixed dimension, cost
/// bytes, not blocks.
#[test]
fn records_far_apart_read_only_their_bytes() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::pure::read_ahead::ReadAhead;

    let _lock = netcdf_lock();
    let nc = TempNc::new("far_records");
    {
        let mut f = netcdf::create_with(&nc.0, Options::empty()).expect("create");
        f.add_unlimited_dimension("time").expect("time");
        f.add_dimension("x", 70_000).expect("x");
        f.add_dimension("step", 4).expect("step");
        f.add_variable::<f64>("time", &["time"]).expect("time");
        f.add_variable::<f32>("v", &["time", "x"]).expect("v");
        f.add_variable::<f32>("w", &["step", "x"]).expect("w");
        // Rows of 4 KB, steps of 400 KB: a point's rows are close, its steps far apart.
        f.add_dimension("row", 100).expect("row");
        f.add_dimension("col", 1000).expect("col");
        f.add_variable::<f32>("u", &["step", "row", "col"])
            .expect("u");
        f.enddef().expect("end of definitions");
        let w: Vec<f32> = (0..4 * 70_000).map(|i| i as f32).collect();
        let mut fixed = f.variable_mut("w").expect("w");
        fixed.put_values(&w, (0..4, 0..70_000)).expect("w values");
        let mut time = f.variable_mut("time").expect("time");
        time.put_values(&[0.0f64, 1.0, 2.0, 3.0], 0..4)
            .expect("times");
    }
    let bytes = Arc::new(std::fs::read(&nc.0).expect("read"));
    let fetched = Arc::new(AtomicU64::new(0));
    let (file, count) = (bytes.clone(), fetched.clone());
    let source = ReadAhead::new(bytes.len() as u64, move |offset, length| {
        count.fetch_add(length, Ordering::SeqCst);
        let range = offset as usize..(offset + length) as usize;
        Ok(file.get(range).unwrap_or_default().to_vec())
    });
    let reader = super::open::open_source(source, "far_records").expect("open");
    let time = reader.variable("time").expect("time");
    let before = fetched.load(Ordering::SeqCst);
    let Ok(super::reader::Raw::F64(values)) = reader.read(&time, &[0], &[4]) else {
        panic!("time read as f64");
    };
    assert_eq!(values, [0.0, 1.0, 2.0, 3.0]);
    assert_eq!(
        fetched.load(Ordering::SeqCst) - before,
        4 * 8,
        "8 bytes per record"
    );
    let w = reader.variable("w").expect("w");
    let before = fetched.load(Ordering::SeqCst);
    let Ok(super::reader::Raw::F32(values)) = reader.read(&w, &[0, 5], &[4, 1]) else {
        panic!("w read as f32");
    };
    assert_eq!(values, [5.0, 70_005.0, 140_005.0, 210_005.0]);
    assert_eq!(
        fetched.load(Ordering::SeqCst) - before,
        4 * 4,
        "4 bytes per step"
    );
    // A point of a 3D variable: its rows are close, its steps far apart.
    let u = reader.variable("u").expect("u");
    let before = fetched.load(Ordering::SeqCst);
    reader.read(&u, &[0, 1, 5], &[4, 1, 1]).expect("u");
    assert_eq!(
        fetched.load(Ordering::SeqCst) - before,
        4 * 4,
        "4 bytes per step of a 3D point"
    );
}

/// A file a writer is still streaming records into says so with all-ones `numrecs`: the
/// record count then comes from the file's size.
#[test]
fn a_streaming_record_count_comes_from_the_file_size() {
    let _lock = netcdf_lock();
    let nc = TempNc::new("streaming");
    write(&nc, Options::empty(), false);
    let mut bytes = std::fs::read(&nc.0).expect("read");
    bytes[4..8].copy_from_slice(&[0xFF; 4]);
    std::fs::write(&nc.0, &bytes).expect("write");
    let reader = super::open::open_path(nc.path()).expect("open");
    let t = reader.variable("t").expect("t");
    assert_eq!(t.dims[0], ("time".to_owned(), 3), "three records");
    let Ok(super::reader::Raw::F64(values)) = reader.read(&t, &[2, 2, 3], &[1, 1, 1]) else {
        panic!("t read as f64");
    };
    assert_eq!(values, [35.0], "the last record's last value");
}
