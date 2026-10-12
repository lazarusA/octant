//! Decimal separators and dates per language, and typed numbers in either style.

use super::Language;
use super::number::{decimal_separator, localize_bytes_in, localizing_in, parse_in};
use std::fmt::Write;

#[test]
fn cjk_and_english_keep_the_point_others_use_a_comma() {
    for lang in [
        Language::English,
        Language::ChineseSimp,
        Language::Japanese,
        Language::Korean,
    ] {
        assert_eq!(decimal_separator(lang), '.', "{lang:?}");
    }
    for &lang in Language::ALL {
        if !matches!(
            lang,
            Language::English | Language::ChineseSimp | Language::Japanese | Language::Korean
        ) {
            assert_eq!(decimal_separator(lang), ',', "{lang:?}");
        }
    }
}

#[test]
fn numbers_are_written_with_the_separator() {
    let mut out = String::new();
    let _ = write!(
        localizing_in(Language::German, &mut out),
        "{:.2}|{:.2e}|{}",
        1.23456,
        -0.00012,
        7
    );
    assert_eq!(out, "1,23|-1,20e-4|7");
    let mut out = String::new();
    let _ = write!(
        localizing_in(Language::Japanese, &mut out),
        "{:.2}",
        1.23456
    );
    assert_eq!(out, "1.23");

    let mut bytes = *b"-12.50";
    localize_bytes_in(Language::French, &mut bytes);
    assert_eq!(&bytes, b"-12,50");
}

#[test]
fn typed_numbers_parse_in_either_style() {
    assert_eq!(parse_in(Language::German, "0,5"), Some(0.5));
    assert_eq!(parse_in(Language::German, "0.5"), Some(0.5));
    assert_eq!(parse_in(Language::Finnish, "\u{2212}1,25 "), Some(-1.25));
    assert_eq!(parse_in(Language::French, "1,5e-3"), Some(0.0015));
    assert_eq!(parse_in(Language::English, "1,000.5"), Some(1000.5));
    assert_eq!(parse_in(Language::Korean, "2.5"), Some(2.5));
    assert_eq!(parse_in(Language::German, "abc"), None);
}

#[test]
fn dates_are_native_in_cjk_and_iso_elsewhere() {
    use super::date::Date;
    let day = Date {
        year: 2020,
        month: 1,
        day: 5,
        hour: None,
    };
    let with_hour = Date {
        hour: Some(6),
        ..day
    };
    let text = |date: Date, lang| date.in_language(lang).to_string();
    assert_eq!(text(day, Language::English), "2020-01-05");
    assert_eq!(text(with_hour, Language::German), "2020-01-05 06:00");
    assert_eq!(
        text(day, Language::ChineseSimp),
        "2020\u{5E74}1\u{6708}5\u{65E5}"
    );
    assert_eq!(
        text(with_hour, Language::Japanese),
        "2020\u{5E74}1\u{6708}5\u{65E5} 06:00"
    );
    assert_eq!(
        text(day, Language::Korean),
        "2020\u{B144} 1\u{C6D4} 5\u{C77C}"
    );
}
