//! Reading `gallery.json`, the chips and entry links.

use super::remote::parse;
use super::{Domain, Format, procedural_entries, shown};
use crate::app::share::parse_share;

const FIXTURE: &[u8] = include_bytes!("fixture.json");

#[test]
fn the_fixture_reads_with_formats_and_desktop_only() {
    let entries = parse(FIXTURE).expect("fixture");
    let formats: Vec<Format> = entries.iter().map(|e| e.format).collect();
    assert_eq!(
        formats,
        [
            Format::Zarr,
            Format::Icechunk,
            Format::OmeZarr,
            Format::GeoTiff
        ]
    );
    let desktop: Vec<&str> = entries
        .iter()
        .filter(|e| e.desktop_only)
        .map(|e| e.key.as_str())
        .collect();
    assert_eq!(desktop, ["copernicus-dem-30m-n50e011"]);
}

#[test]
fn another_version_or_a_bad_entry_is_refused() {
    assert!(parse(br#"{"version": 2, "entries": []}"#).is_err());
    assert!(parse(b"<html>").is_err());
    let entry = |key: &str, store: &str, format: &str| {
        format!(r#"{{"key":"{key}","title":"T","store":"{store}","format":"{format}","new":1}}"#)
    };
    let json = format!(
        r#"{{"version":1,"entries":[{},{},{},{},{{"key":"no-store"}}]}}"#,
        entry("a", "https://x/a.zarr", "zarr"),
        entry("a", "https://x/b.zarr", "zarr"),
        entry("local", "/etc/passwd", "zarr"),
        entry("future", "s3://x/c", "laz"),
    );
    let entries = parse(json.as_bytes()).expect("version 1");
    let kept: Vec<(&str, Format)> = entries.iter().map(|e| (e.key.as_str(), e.format)).collect();
    assert_eq!(kept, [("a", Format::Zarr), ("future", Format::Other)]);
}

#[test]
fn a_link_opens_the_store_then_its_view() {
    let entries = parse(FIXTURE).expect("fixture");
    let seasfire = &entries[0];
    let (Some(actions), Some(plain)) = (
        parse_share(&seasfire.link()),
        parse_share(&format!("?src={}&var=fcci_ba", seasfire.store)),
    ) else {
        panic!("{}", seasfire.link());
    };
    // Opens the store and picks the variable, then sets the rest of the view.
    assert_eq!(actions[..2], plain[..2]);
    assert!(actions.len() > plain.len());
    // No view: the store itself.
    let volume = &procedural_entries()[2];
    assert_eq!(volume.link(), "procedural://volume4d");
}

#[test]
fn chips_pick_by_domain_and_format() {
    let entries = parse(FIXTURE).expect("fixture");
    let all = || entries.iter().chain(procedural_entries());
    let domains: Vec<Domain> = entries.iter().map(|e| e.domain).collect();
    let (c, w) = (Domain::Climate, Domain::Weather);
    assert_eq!(
        domains,
        [c, w, Domain::Microscopy, Domain::EarthObservation]
    );
    assert_eq!(all().filter(|e| shown(e, None, None)).count(), 8);
    let climate = all().filter(|e| shown(e, Some(c), None)).count();
    assert_eq!(climate, 1);
    let procedural = Some(Domain::Procedural);
    let synthetic = all().filter(|e| shown(e, procedural, None)).count();
    assert_eq!(synthetic, procedural_entries().len());
    // Both chips: an entry under both.
    assert_eq!(
        all()
            .filter(|e| shown(e, Some(c), Some(Format::Icechunk)))
            .count(),
        0
    );
    assert_eq!(
        all()
            .filter(|e| shown(e, Some(w), Some(Format::Icechunk)))
            .count(),
        1
    );
}

#[test]
fn a_thumbnail_is_a_path_on_the_docs_site() {
    let entries = parse(FIXTURE).expect("fixture");
    assert_eq!(
        entries[0].thumbnail.as_deref(),
        Some("/gallery/seasfire.webp")
    );
    let json = |thumbnail: &str| {
        format!(
            r#"{{"version":1,"entries":[{{"key":"a","title":"T","store":"https://x/a","format":"zarr","thumbnail":"{thumbnail}"}}]}}"#
        )
    };
    for elsewhere in [
        "https://evil.example/x.webp",
        "//evil.example/x.webp",
        "x.webp",
    ] {
        let entries = parse(json(elsewhere).as_bytes()).expect("version 1");
        assert_eq!(entries[0].thumbnail, None, "{elsewhere}");
    }
}

#[test]
fn pages_and_licenses_are_remote_addresses() {
    let json = |page: &str| {
        format!(
            r#"{{"version":1,"entries":[{{"key":"a","title":"T","store":"https://x/a","format":"zarr","source_page":"{page}","license_url":"{page}"}}]}}"#
        )
    };
    for kept in [
        "https://x.org/a",
        "http://x.org/a",
        "gs://bucket/README",
        "s3://bucket/a",
    ] {
        let entries = parse(json(kept).as_bytes()).expect("version 1");
        assert_eq!(entries[0].source_page.as_deref(), Some(kept));
        assert_eq!(entries[0].license_url.as_deref(), Some(kept));
    }
    for dropped in [
        "javascript:alert(1)",
        "file:///etc/passwd",
        "data:text/html,x",
    ] {
        let entries = parse(json(dropped).as_bytes()).expect("version 1");
        assert_eq!(entries[0].source_page, None, "{dropped}");
        assert_eq!(entries[0].license_url, None, "{dropped}");
    }
}
