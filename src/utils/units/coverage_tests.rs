use super::coverage::{date_in_units, stated_coverage, stated_span, step_inside};
use crate::data::{CoordValues, VariableInfo};

/// ARCO-ERA5's time axis: hourly from 1900, 1323648 steps.
const ERA5_UNITS: &str = "hours since 1900-01-01 00:00:00";
const ERA5_STEPS: usize = 1_323_648;
/// 2026-11-17T23:00, the last hour of `valid_time_stop_era5t`'s day.
const ERA5_LAST: usize = 1_112_207;

fn era5_axis() -> CoordValues {
    CoordValues::Regular {
        start: 0.0,
        step: 1.0,
        len: ERA5_STEPS,
    }
}

fn era5_var() -> VariableInfo {
    let mut var = VariableInfo::default();
    for (k, v) in [
        ("valid_time_start", "1940-01-01"),
        ("valid_time_stop", "2026-06-30"),
        ("valid_time_stop_era5t", "2026-11-17"),
    ] {
        var.attributes.insert(k.into(), v.into());
    }
    var
}

#[test]
fn era5_root_attributes_state_its_coverage() {
    assert_eq!(
        stated_coverage(&era5_var()),
        (Some("1940-01-01"), Some("2026-11-17"))
    );
}

#[test]
fn acdd_coverage_moves_nothing() {
    // Growing stores often keep a stale `time_coverage_end`.
    let mut var = VariableInfo {
        time_coverage_start: Some("2000-01-01".into()),
        time_coverage_end: Some("2001-01-01".into()),
        ..VariableInfo::default()
    };
    var.attributes
        .insert("time_coverage_end".into(), "2001-01-01".into());
    assert_eq!(stated_coverage(&var), (None, None));
}

#[test]
fn a_bare_end_date_holds_its_whole_day() {
    let (start, end) = stated_span(&era5_var(), ERA5_UNITS);
    assert_eq!(start, Some(350_616.0));
    let end = end.unwrap_or_default();
    assert!(end >= ERA5_LAST as f64 && end < (ERA5_LAST + 1) as f64);
}

#[test]
fn dates_become_values_in_cf_units() {
    assert_eq!(date_in_units("1940-01-01", ERA5_UNITS), Some(350_616.0));
    assert_eq!(
        date_in_units("1970-01-02", "days since 1970-01-01"),
        Some(1.0)
    );
    // Times of day count, in the date and in the reference.
    assert_eq!(
        date_in_units("2020-12-31", "hours since 2020-12-30 12:00"),
        Some(12.0)
    );
    assert_eq!(
        date_in_units("2020-01-01T06:30:00Z", "minutes since 2020-01-01T00:00:00Z"),
        Some(390.0)
    );
    assert_eq!(date_in_units("1940-01-01", "hours"), None);
    assert_eq!(date_in_units("soon", ERA5_UNITS), None);
}

#[test]
fn zones_after_the_time_are_ignored() {
    let days = |units: &str| date_in_units("1970-01-02", units);
    assert_eq!(days("days since 1970-01-01 00:00:00 UTC"), Some(1.0));
    assert_eq!(days("days since 1970-01-01T00:00:00-05:00"), Some(1.0));
    assert_eq!(days("days since 1970-01-01T00:00:00+01:00"), Some(1.0));
    assert_eq!(days("days since 1970-01-01 00:00:00.0"), Some(1.0));
}

#[test]
fn a_step_outside_the_data_moves_to_its_last_step() {
    let (start, end) = stated_span(&era5_var(), ERA5_UNITS);
    let axis = era5_axis();
    // The default one-chunk range at 1900 holds no data: the record's end.
    let step = step_inside(&axis, ERA5_STEPS, 0, (0, 0), (start, end));
    assert_eq!(step, Some(ERA5_LAST));
    // A range over the whole axis keeps the step nearest it: 1940.
    let whole = step_inside(&axis, ERA5_STEPS, 0, (0, ERA5_STEPS - 1), (start, end));
    assert_eq!(whole, Some(350_616));
    // Past the data too.
    let past = step_inside(
        &axis,
        ERA5_STEPS,
        ERA5_STEPS - 1,
        (0, ERA5_STEPS - 1),
        (start, end),
    );
    assert_eq!(past, Some(ERA5_LAST));
    // Inside it, the last day's hours included, the step stays.
    assert_eq!(
        step_inside(
            &axis,
            ERA5_STEPS,
            400_000,
            (0, ERA5_STEPS - 1),
            (start, end)
        ),
        None
    );
    assert_eq!(
        step_inside(
            &axis,
            ERA5_STEPS,
            ERA5_LAST - 11,
            (0, ERA5_STEPS - 1),
            (start, end)
        ),
        None
    );
}

#[test]
fn only_a_start_moves_to_the_first_step_after_it() {
    let axis = CoordValues::Values(vec![0.0, 10.0, 20.0, 30.0].into());
    assert_eq!(
        step_inside(&axis, 4, 0, (0, 3), (Some(15.0), None)),
        Some(2)
    );
    assert_eq!(step_inside(&axis, 4, 3, (0, 3), (Some(15.0), None)), None);
    // No step inside: nothing to move to.
    assert_eq!(step_inside(&axis, 4, 0, (0, 3), (Some(40.0), None)), None);
    assert_eq!(
        step_inside(&axis, 4, 0, (0, 3), (Some(12.0), Some(18.0))),
        None
    );
}

#[test]
fn descending_or_estimated_axes_move_nothing() {
    let descending = CoordValues::Values(vec![30.0, 20.0, 10.0, 0.0].into());
    assert_eq!(
        step_inside(&descending, 4, 0, (0, 3), (Some(5.0), Some(15.0))),
        None
    );
    let ends = CoordValues::Endpoints {
        first: 0.0,
        last: 30.0,
        len: 4,
    };
    assert_eq!(step_inside(&ends, 4, 0, (0, 3), (Some(15.0), None)), None);
}

#[test]
fn labels_have_no_step_inside() {
    let axis = CoordValues::Labels(vec!["a".to_string(), "b".to_string()].into());
    assert_eq!(step_inside(&axis, 2, 0, (0, 1), (Some(1.0), None)), None);
}

#[test]
fn a_range_reaching_into_the_data_keeps_its_covered_part() {
    let (start, end) = stated_span(&era5_var(), ERA5_UNITS);
    let axis = era5_axis();
    // 1935-01-01 to 1945-12-31, hourly since 1900; data from 1940.
    let range = (306_792, 403_223);
    let step = step_inside(&axis, ERA5_STEPS, range.0, range, (start, end));
    assert_eq!(step, Some(350_616));
    // A range wholly before the data falls back to its last step.
    let early = (0, 1000);
    let step = step_inside(&axis, ERA5_STEPS, 0, early, (start, end));
    assert_eq!(step, Some(ERA5_LAST));
}
