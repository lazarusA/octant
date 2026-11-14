//! GRIB files read as the NetCDF backend reads any file: variables, dimensions,
//! coordinates and windows, against gribberish's own decoding.

use super::reader::GribReader;
use super::test_files::{HRRR_DPT_CONSTANT, RRFS_PEVAP, S2S_ANOMALY};
use crate::data::backends::netcdf::pure::read_ahead::{RangeSource, ReadAhead};
use crate::data::backends::netcdf::reader::{NcReader, NcVar, Raw};

/// `bytes` read through the read-ahead, as browser and remote files are.
pub(super) fn source(bytes: Vec<u8>) -> Box<dyn RangeSource + Send + Sync> {
    let len = bytes.len() as u64;
    Box::new(ReadAhead::new(len, move |offset, length| {
        let (start, end) = (offset as usize, (offset + length) as usize);
        Ok(bytes[start..end].to_vec())
    }))
}

/// A reader over `bytes`.
pub(super) fn open(bytes: Vec<u8>) -> GribReader {
    GribReader::open(source(bytes), "test.grib2").expect("open")
}

pub(super) fn data_vars(reader: &GribReader) -> Vec<NcVar> {
    (reader.variables().into_iter())
        .filter(|v| v.data_type == "float32")
        .collect()
}

pub(super) fn read_f32(
    reader: &GribReader,
    var: &NcVar,
    start: &[usize],
    count: &[usize],
) -> Vec<f32> {
    match reader.read(var, start, count).expect("read") {
        Raw::F32(values) => values,
        Raw::F64(_) => panic!("data variables read as f32"),
    }
}

/// Where section `n` of the one message `bytes` hold starts.
pub(super) fn section(bytes: &[u8], n: u8) -> usize {
    super::sections::section_at(bytes, n).expect("section").0
}

/// The message `bytes` hold, its level (product template 4.0's first surface value) and
/// forecast hour set to `level` and `hour`.
fn moved(bytes: &[u8], level: u32, hour: u32) -> Vec<u8> {
    let mut out = bytes.to_vec();
    let s4 = section(&out, 4);
    out[s4 + 18..s4 + 22].copy_from_slice(&hour.to_be_bytes());
    out[s4 + 24..s4 + 28].copy_from_slice(&level.to_be_bytes());
    out
}

#[test]
fn a_projected_message_reads_as_gribberish_decodes_it() {
    let reader = open(RRFS_PEVAP.to_vec());
    let [var] = data_vars(&reader).try_into().expect("one variable");
    let dims: Vec<&str> = var.dims.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(dims, ["y", "x"]);
    assert_eq!(
        var.attributes.get("grid_mapping").map(String::as_str),
        Some("crs")
    );
    let crs = reader.variable("crs").expect("grid mapping");
    let mapping = crs.attributes.get("grid_mapping_name").map(String::as_str);
    assert_eq!(mapping, Some("lambert_conformal_conic"));
    let proj4 = crs.attributes.get("proj4").expect("proj4");
    assert!(proj4.contains("+proj=lcc"), "{proj4}");
    assert!(
        proj4.split_whitespace().all(|t| t.starts_with('+')),
        "{proj4}"
    );

    let message = gribberish::message::read_message(RRFS_PEVAP, 0).expect("message");
    let expected: Vec<f32> = message
        .data()
        .expect("data")
        .iter()
        .map(|&v| v as f32)
        .collect();
    let all = read_f32(&reader, &var, &[0, 0], &[2, 4]);
    assert_eq!(all.iter().filter(|v| v.is_nan()).count(), 5);
    let same = |a: f32, b: f32| a == b || (a.is_nan() && b.is_nan());
    assert!(all.iter().zip(&expected).all(|(a, b)| same(*a, *b)));
    let window = read_f32(&reader, &var, &[1, 1], &[1, 2]);
    assert!(
        window
            .iter()
            .zip(&expected[5..7])
            .all(|(a, b)| same(*a, *b))
    );
}

#[test]
fn kinds_and_level_types_make_their_own_variables() {
    let reader = open(S2S_ANOMALY.to_vec());
    let vars = data_vars(&reader);
    assert_eq!(
        vars.len(),
        4,
        "{:?}",
        vars.iter().map(|v| &v.path).collect::<Vec<_>>()
    );
    for var in &vars {
        assert!(var.path.starts_with("TMP_"), "{}", var.path);
        let dims: Vec<(&str, usize)> = var.dims.iter().map(|(n, l)| (n.as_str(), *l)).collect();
        assert_eq!(dims, [("latitude", 361), ("longitude", 720)]);
    }
    let latitude = reader.variable("latitude").expect("latitude");
    assert_eq!(
        latitude.attributes.get("units").map(String::as_str),
        Some("degrees_north")
    );
    assert!(
        reader.variable("crs").is_none(),
        "lat/lon grids need no grid mapping"
    );
}

#[test]
fn padded_messages_on_two_grids_are_all_found() {
    let mut bytes = RRFS_PEVAP.to_vec();
    bytes.extend([0; 8]);
    bytes.extend(HRRR_DPT_CONSTANT);
    let reader = open(bytes);
    let vars = data_vars(&reader);
    let names: Vec<&str> = vars.iter().map(|v| v.path.as_str()).collect();
    assert_eq!(names.len(), 2, "{names:?}");
    let dpt = vars.iter().find(|v| v.path == "DPT").expect("DPT");
    let dims: Vec<&str> = dpt.dims.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(dims, ["y_1", "x_1"]);
    let value = read_f32(&reader, dpt, &[500, 900], &[1, 1]);
    assert!((value[0] - 192.124_79).abs() < 1e-3, "{value:?}");
}

#[test]
fn messages_varying_in_level_and_time_stack_with_gaps_as_nan() {
    // Levels 100 and 500 hPa at hours 0 and 1, the 500 hPa hour 1 message missing.
    let mut bytes = HRRR_DPT_CONSTANT.to_vec();
    bytes.extend(moved(HRRR_DPT_CONSTANT, 50000, 0));
    bytes.extend(moved(HRRR_DPT_CONSTANT, 10000, 1));
    let reader = open(bytes);
    let [var] = data_vars(&reader).try_into().expect("one variable");
    let dims: Vec<(&str, usize)> = var.dims.iter().map(|(n, l)| (n.as_str(), *l)).collect();
    assert_eq!(dims[..2], [("time", 2), ("isobar", 2)], "{dims:?}");
    let time = reader.variable("time").expect("time");
    assert!(
        time.attributes
            .get("units")
            .is_some_and(|u| u.starts_with("seconds since"))
    );
    let level = match reader.read(&reader.variable("isobar").expect("level"), &[0], &[2]) {
        Ok(Raw::F64(values)) => values,
        _ => panic!("levels"),
    };
    assert_eq!(level, [10000.0, 50000.0]);
    let points = read_f32(&reader, &var, &[0, 0, 10, 10], &[2, 2, 1, 1]);
    assert!(
        points[..3].iter().all(|v| (v - 192.124_79).abs() < 1e-3),
        "{points:?}"
    );
    assert!(points[3].is_nan(), "no message at 500 hPa, hour 1");
}

#[test]
fn grib1_is_refused_with_its_reason() {
    let mut bytes = b"GRIB\0\0\0\x01".to_vec();
    bytes.extend([0; 24]);
    let error = GribReader::open(source(bytes), "old.grb")
        .err()
        .expect("refused");
    assert!(error.to_string().contains("GRIB1"), "{error}");
}

/// Offsets from a sidecar index read the same variables as a scan through the file.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn indexed_offsets_read_as_a_scan_does() {
    let mut bytes = RRFS_PEVAP.to_vec();
    bytes.extend([0; 8]);
    let second = bytes.len() as u64;
    bytes.extend(HRRR_DPT_CONSTANT);
    let scanned = open(bytes.clone());
    let indexed = GribReader::open_at(source(bytes), "indexed.grib2", &[0, second]).expect("open");
    let paths = |r: &GribReader| {
        r.variables()
            .into_iter()
            .map(|v| v.path)
            .collect::<Vec<_>>()
    };
    assert_eq!(paths(&indexed), paths(&scanned));
}

/// The HRRR message as a layer from 0 m to `top` m above ground (product template 4.0's
/// first and second fixed surfaces).
fn layer(top: u32) -> Vec<u8> {
    let mut out = HRRR_DPT_CONSTANT.to_vec();
    let s4 = section(&out, 4);
    out[s4 + 22..s4 + 28].copy_from_slice(&[103, 0, 0, 0, 0, 0]);
    out[s4 + 28..s4 + 30].copy_from_slice(&[103, 0]);
    out[s4 + 30..s4 + 34].copy_from_slice(&top.to_be_bytes());
    out
}

/// Layers sharing their bottom (HRRR's 0-1000 m and 0-6000 m wind shear) stack by their
/// tops; a message at an index another already holds is left out, not lost unseen.
#[test]
fn layers_sharing_a_bottom_stack_by_their_tops() {
    let mut bytes = layer(1000);
    bytes.extend(layer(6000));
    let reader = open(bytes.clone());
    let [var] = data_vars(&reader).try_into().expect("one variable");
    assert_eq!((var.dims[0].0.as_str(), var.dims[0].1), ("hag", 2));
    match reader.read(&reader.variable("hag").expect("level"), &[0], &[2]) {
        Ok(Raw::F64(tops)) => assert_eq!(tops, [1000.0, 6000.0]),
        _ => panic!("levels"),
    }

    bytes.extend(layer(6000));
    let found = super::scan::scan(source(bytes).as_ref(), "twice.grib2").expect("scan");
    let (_, left_out) = super::layout::layout(&found.fields).expect("layout");
    assert_eq!(left_out, [found.fields[2].offset]);
}
