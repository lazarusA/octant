use crate::utils::units::*;

#[test]
fn test_parse_time_unit_hour_aliases() {
    let (s1, _) = parse_time_unit(Some("hours since 2024-01-01"));
    let (s2, _) = parse_time_unit(Some("h since 2024-01-01"));
    let (s3, _) = parse_time_unit(Some("hr since 2024-01-01"));
    let (s4, _) = parse_time_unit(Some("hrs since 2024-01-01"));

    assert_eq!(s1, 3600000);
    assert_eq!(s2, 3600000);
    assert_eq!(s3, 3600000);
    assert_eq!(s4, 3600000);
}

#[test]
fn test_parse_time_unit_other_aliases() {
    assert_eq!(parse_time_unit(Some("min since 2024-01-01")).0, 60000);
    assert_eq!(parse_time_unit(Some("mins since 2024-01-01")).0, 60000);
    assert_eq!(parse_time_unit(Some("s since 2024-01-01")).0, 1000);
    assert_eq!(parse_time_unit(Some("sec since 2024-01-01")).0, 1000);
    assert_eq!(parse_time_unit(Some("secs since 2024-01-01")).0, 1000);
    assert_eq!(parse_time_unit(Some("d since 2024-01-01")).0, 86400000);
    assert_eq!(parse_time_unit(Some("ms since 2024-01-01")).0, 1);
}

#[test]
fn test_parse_time_unit_bare_duration() {
    assert_eq!(parse_time_unit(Some("hours")).0, 3600000);
    assert_eq!(parse_time_unit(Some("hour")).0, 3600000);
    assert_eq!(parse_time_unit(Some("h")).0, 3600000);
    assert_eq!(parse_time_unit(Some("hr")).0, 3600000);
    assert_eq!(parse_time_unit(Some("hrs")).0, 3600000);
    assert_eq!(parse_time_unit(Some("days")).0, 86400000);
    assert_eq!(parse_time_unit(Some("minutes")).0, 60000);
}

#[test]
fn test_parse_loc_durations() {
    assert_eq!(parse_loc(Some(12.0), "hours"), Some("12 h".to_string()));
    assert_eq!(parse_loc(Some(24.0), "h"), Some("24 h".to_string()));
    assert_eq!(parse_loc(Some(6.0), "hr"), Some("6 h".to_string()));
    assert_eq!(parse_loc(Some(48.0), "hrs"), Some("48 h".to_string()));
    assert_eq!(parse_loc(Some(0.0), "hour"), Some("0 h".to_string()));
    assert_eq!(parse_loc(Some(12.5), "hours"), Some("12.50 h".to_string()));

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
fn test_parse_loc_datetime() {
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
fn test_parse_loc_degrees_and_fallback() {
    assert_eq!(
        parse_loc(Some(-120.5), "degrees_east"),
        Some("-120.50°".to_string())
    );
    assert_eq!(parse_loc(Some(45.0), "deg"), Some("45.00°".to_string()));
    assert_eq!(parse_loc(Some(100.0), "hPa"), Some("100.00".to_string()));
    assert_eq!(parse_loc(None, "hours"), None);
}

#[test]
fn test_add_days_to_date_basic_and_negative() {
    assert_eq!(add_days_to_date(2024, 1, 1, 0), (2024, 1, 1));
    assert_eq!(add_days_to_date(2024, 1, 1, 10), (2024, 1, 11));
    assert_eq!(add_days_to_date(2024, 1, 1, 31), (2024, 2, 1));
    assert_eq!(add_days_to_date(2024, 1, 1, -1), (2023, 12, 31));
    assert_eq!(add_days_to_date(2024, 3, 1, -1), (2024, 2, 29));
    assert_eq!(add_days_to_date(2023, 3, 1, -1), (2023, 2, 28));
}

#[test]
fn test_add_days_to_date_leap_century_and_large_offset() {
    assert_eq!(add_days_to_date(2000, 2, 28, 1), (2000, 2, 29));
    assert_eq!(add_days_to_date(1900, 2, 28, 1), (1900, 3, 1));
    let (y, m, d) = add_days_to_date(1970, 1, 1, 1_000_000_000);
    assert!(y > 2000000);
    assert!((1..=12).contains(&m));
    assert!((1..=31).contains(&d));
}

#[test]
fn test_days_civil_roundtrip() {
    let cases = [
        (1970, 1, 1),
        (2024, 2, 29),
        (2000, 12, 31),
        (1600, 3, 1),
        (2400, 7, 15),
    ];
    for &(y, m, d) in &cases {
        let days = days_from_civil(y as i64, m as u32, d as u32);
        let res = civil_from_days(days);
        assert_eq!(res, (y, m, d));
    }
}
