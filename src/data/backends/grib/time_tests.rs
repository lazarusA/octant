//! Messages of several reference times: stacked by reference time and step (hours from it,
//! as cfgrib does), never by reference and valid time, whose grid would hold only its
//! diagonal for an hourly reanalysis.

use super::test_files::HRRR_DPT_CONSTANT;
use super::tests::{data_vars, open, read_f32, section};
use crate::data::backends::netcdf::reader::{NcReader, Raw};

/// The HRRR message run at `hour` (section 1's reference hour), `step` hours ahead.
fn run(hour: u8, step: u32) -> Vec<u8> {
    let mut out = HRRR_DPT_CONSTANT.to_vec();
    let (s1, s4) = (section(&out, 1), section(&out, 4));
    out[s1 + 16] = hour;
    out[s4 + 18..s4 + 22].copy_from_slice(&step.to_be_bytes());
    out
}

fn dims(reader: &super::reader::GribReader) -> Vec<(String, usize)> {
    let [var] = data_vars(reader).try_into().expect("one variable");
    var.dims
}

#[test]
fn analyses_at_several_times_stack_by_reference_time_only() {
    let mut bytes = run(20, 0);
    bytes.extend(run(21, 0));
    let reader = open(bytes);
    let dims = dims(&reader);
    assert_eq!(
        (dims.len(), dims[0].0.as_str(), dims[0].1),
        (3, "reference_time", 2)
    );
    let [var] = data_vars(&reader).try_into().expect("one variable");
    assert_eq!(
        var.attributes.get("step").map(String::as_str),
        Some("0 hours")
    );
}

#[test]
fn runs_and_steps_make_a_dense_grid() {
    let mut bytes = Vec::new();
    for (hour, step) in [(20, 0), (21, 0), (20, 1), (21, 1)] {
        bytes.extend(run(hour, step));
    }
    let reader = open(bytes);
    let names: Vec<(String, usize)> = dims(&reader).into_iter().take(2).collect();
    let expected = [("reference_time".to_owned(), 2), ("step".to_owned(), 2)];
    assert_eq!(names, expected);
    match reader.read(&reader.variable("step").expect("step"), &[0], &[2]) {
        Ok(Raw::F64(steps)) => assert_eq!(steps, [0.0, 1.0]),
        _ => panic!("steps"),
    }
    let [var] = data_vars(&reader).try_into().expect("one variable");
    let points = read_f32(&reader, &var, &[0, 0, 10, 10], &[2, 2, 1, 1]);
    assert!(
        points.iter().all(|v| v.is_finite()),
        "every slot filled: {points:?}"
    );
}
