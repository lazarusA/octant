use octant::utils::units::{parse_loc, parse_reference_date, parse_time_unit};

#[test]
fn test_parse_time_unit_hour_aliases_with_since() {
    let (scale1, offset1) = parse_time_unit(Some("hours since 2024-01-01"));
    let (scale2, offset2) = parse_time_unit(Some("h since 2024-01-01"));
    let (scale3, offset3) = parse_time_unit(Some("hr since 2024-01-01"));
    let (scale4, offset4) = parse_time_unit(Some("hrs since 2024-01-01"));

    assert_eq!(scale1, 3600000);
    assert_eq!(scale2, 3600000);
    assert_eq!(scale3, 3600000);
    assert_eq!(scale4, 3600000);

    assert_eq!(offset1, offset2);
    assert_eq!(offset2, offset3);
    assert_eq!(offset3, offset4);
}

#[test]
fn test_parse_time_unit_minute_second_day_ms_aliases_with_since() {
    assert_eq!(parse_time_unit(Some("min since 2024-01-01")).0, 60000);
    assert_eq!(parse_time_unit(Some("mins since 2024-01-01")).0, 60000);
    assert_eq!(parse_time_unit(Some("s since 2024-01-01")).0, 1000);
    assert_eq!(parse_time_unit(Some("sec since 2024-01-01")).0, 1000);
    assert_eq!(parse_time_unit(Some("secs since 2024-01-01")).0, 1000);
    assert_eq!(parse_time_unit(Some("d since 2024-01-01")).0, 86400000);
    assert_eq!(parse_time_unit(Some("ms since 2024-01-01")).0, 1);
}

#[test]
fn test_parse_time_unit_bare_duration_units() {
    assert_eq!(parse_time_unit(Some("hours")).0, 3600000);
    assert_eq!(parse_time_unit(Some("hour")).0, 3600000);
    assert_eq!(parse_time_unit(Some("h")).0, 3600000);
    assert_eq!(parse_time_unit(Some("hr")).0, 3600000);
    assert_eq!(parse_time_unit(Some("hrs")).0, 3600000);
    assert_eq!(parse_time_unit(Some("days")).0, 86400000);
    assert_eq!(parse_time_unit(Some("minutes")).0, 60000);
}

#[test]
fn test_parse_loc_bare_hour_duration_values() {
    assert_eq!(parse_loc(Some(12.0), "hours"), Some("12 h".to_string()));
    assert_eq!(parse_loc(Some(24.0), "h"), Some("24 h".to_string()));
    assert_eq!(parse_loc(Some(6.0), "hr"), Some("6 h".to_string()));
    assert_eq!(parse_loc(Some(48.0), "hrs"), Some("48 h".to_string()));
    assert_eq!(parse_loc(Some(0.0), "hour"), Some("0 h".to_string()));
    assert_eq!(parse_loc(Some(12.5), "hours"), Some("12.50 h".to_string()));
}

#[test]
fn test_parse_loc_bare_duration_coarsest_unit() {
    assert_eq!(parse_loc(Some(0.0), "seconds"), Some("0 h".to_string()));
    assert_eq!(parse_loc(Some(3600.0), "seconds"), Some("1 h".to_string()));
    assert_eq!(parse_loc(Some(7200.0), "s"), Some("2 h".to_string()));
    assert_eq!(parse_loc(Some(10800.0), "sec"), Some("3 h".to_string()));
    assert_eq!(parse_loc(Some(30.0), "seconds"), Some("30 s".to_string()));
    assert_eq!(
        parse_loc(Some(1800.0), "seconds"),
        Some("30 min".to_string())
    );
    assert_eq!(parse_loc(Some(5.0), "d"), Some("5 d".to_string()));
    assert_eq!(parse_loc(Some(500.0), "ms"), Some("500 ms".to_string()));
}

#[test]
fn test_parse_loc_cf_absolute_datetime() {
    assert_eq!(
        parse_loc(Some(12.0), "hours since 2024-01-01"),
        Some("01-01-2024 12:00".to_string())
    );
    assert_eq!(
        parse_loc(Some(12.0), "h since 2024-01-01"),
        Some("01-01-2024 12:00".to_string())
    );
    assert_eq!(
        parse_loc(Some(12.0), "hrs since 2024-01-01"),
        Some("01-01-2024 12:00".to_string())
    );
}

#[test]
fn test_parse_loc_degrees() {
    assert_eq!(
        parse_loc(Some(-120.5), "degrees_east"),
        Some("-120.50°".to_string())
    );
    assert_eq!(parse_loc(Some(45.0), "deg"), Some("45.00°".to_string()));
}

#[test]
fn test_parse_loc_fallback_and_none() {
    assert_eq!(parse_loc(Some(100.0), "hPa"), Some("100.00".to_string()));
    assert_eq!(parse_loc(None, "hours"), None);
}

#[test]
fn test_seasfire_reference_date_parsing() {
    let (year, month, day, step) = parse_reference_date(
        None,
        None,
        None,
        Some("https://s3.bgc-jena.mpg.de:9000/misc/seasfire_rechunked.zarr"),
    );

    assert_eq!(year, 2001);
    assert_eq!(month, 1);
    assert_eq!(day, 1);
    assert_eq!(step, 8);
}
