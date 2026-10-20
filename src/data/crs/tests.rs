//! CRS discovery from attributes, CF grid mappings, multiscale stacks and
//! resolution to proj strings.

use std::collections::HashMap;
use std::sync::Arc;

use super::{CrsRef, Resolution, assign, attach_grid_mappings, discover, named, resolve, wkt_name};
use crate::data::metadata::VariableInfo;

/// WGS 84 / UTM zone 36N as rioxarray writes it (`spatial_ref`, WKT1).
pub(super) const UTM36N_WKT1: &str = r#"PROJCS["WGS 84 / UTM zone 36N",GEOGCS["WGS 84",DATUM["WGS_1984",SPHEROID["WGS 84",6378137,298.257223563,AUTHORITY["EPSG","7030"]],AUTHORITY["EPSG","6326"]],PRIMEM["Greenwich",0,AUTHORITY["EPSG","8901"]],UNIT["degree",0.0174532925199433,AUTHORITY["EPSG","9122"]],AUTHORITY["EPSG","4326"]],PROJECTION["Transverse_Mercator"],PARAMETER["latitude_of_origin",0],PARAMETER["central_meridian",33],PARAMETER["scale_factor",0.9996],PARAMETER["false_easting",500000],PARAMETER["false_northing",0],UNIT["metre",1,AUTHORITY["EPSG","9001"]],AXIS["Easting",EAST],AXIS["Northing",NORTH],AUTHORITY["EPSG","32636"]]"#;

/// The same CRS as CF `crs_wkt` (WKT2) writers give it.
const UTM36N_WKT2: &str = r#"PROJCRS["WGS 84 / UTM zone 36N",BASEGEOGCRS["WGS 84",DATUM["World Geodetic System 1984",ELLIPSOID["WGS 84",6378137,298.257223563,LENGTHUNIT["metre",1]]],PRIMEM["Greenwich",0,ANGLEUNIT["degree",0.0174532925199433]]],CONVERSION["UTM zone 36N",METHOD["Transverse Mercator",ID["EPSG",9807]],PARAMETER["Latitude of natural origin",0,ANGLEUNIT["degree",0.0174532925199433]],PARAMETER["Longitude of natural origin",33,ANGLEUNIT["degree",0.0174532925199433]],PARAMETER["Scale factor at natural origin",0.9996,SCALEUNIT["unity",1]],PARAMETER["False easting",500000,LENGTHUNIT["metre",1]],PARAMETER["False northing",0,LENGTHUNIT["metre",1]]],CS[Cartesian,2],AXIS["easting",east,ORDER[1],LENGTHUNIT["metre",1]],AXIS["northing",north,ORDER[2],LENGTHUNIT["metre",1]],ID["EPSG",32636]]"#;

fn attrs(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

fn dims(names: &[&str]) -> Vec<String> {
    names.iter().map(|d| d.to_string()).collect()
}

#[test]
fn epsg_codes_come_first_in_any_form() {
    let xy = dims(&["y", "x"]);
    let both = attrs(&[("crs_wkt", UTM36N_WKT1), ("proj:code", "EPSG:32636")]);
    assert_eq!(discover(&both, &xy), Some(CrsRef::Epsg(32636)));
    let url = attrs(&[("proj:code", "http://www.opengis.net/def/crs/EPSG/0/3413")]);
    assert_eq!(discover(&url, &xy), Some(CrsRef::Epsg(3413)));
    for (key, value) in [("proj:epsg", "32636"), ("epsg_code", "EPSG:32636")] {
        assert_eq!(
            discover(&attrs(&[(key, value)]), &xy),
            Some(CrsRef::Epsg(32636))
        );
    }
}

#[test]
fn wkt_then_names_then_nothing() {
    let xy = dims(&["y", "x"]);
    // A WKT's own EPSG code (WKT1 `AUTHORITY`, WKT2 `ID`) is read as one.
    for wkt in [UTM36N_WKT1, UTM36N_WKT2] {
        let found = discover(&attrs(&[("spatial_ref", wkt)]), &xy);
        assert_eq!(found, Some(CrsRef::Epsg(32636)));
    }
    // Codes inside it (its datum's, its units') name no CRS.
    let custom = UTM36N_WKT1.replace(r#",AUTHORITY["EPSG","32636"]]"#, "]");
    let found = discover(&attrs(&[("crs_wkt", &custom)]), &xy);
    assert_eq!(found, Some(CrsRef::Wkt(custom.as_str().into())));
    // Text that is no WKT is not taken for one.
    assert_eq!(discover(&attrs(&[("spatial_ref", "utm")]), &xy), None);
    let none = HashMap::new();
    let lonlat = discover(&none, &dims(&["time", "latitude", "longitude"]));
    assert_eq!(lonlat, Some(CrsRef::Geographic));
    // x/y in metres are never assumed to be degrees.
    assert_eq!(discover(&none, &xy), None);
}

#[test]
fn names_come_from_the_wkt_or_the_citation() {
    assert_eq!(wkt_name(UTM36N_WKT1), Some("WGS 84 / UTM zone 36N"));
    assert_eq!(wkt_name(UTM36N_WKT2), Some("WGS 84 / UTM zone 36N"));
    let var = |pairs: &[(&str, &str)]| VariableInfo {
        attributes: attrs(pairs),
        ..Default::default()
    };
    assert_eq!(named(&var(&[("crs_citation", "UTM 36N")])), Some("UTM 36N"));
    let both = var(&[("crs_citation", "UTM 36N"), ("crs_wkt", UTM36N_WKT2)]);
    assert_eq!(named(&both), Some("WGS 84 / UTM zone 36N"));
}

#[test]
fn grid_mappings_lend_their_crs_keys() {
    let var = |name: &str, mapping: &str, own: &[(&str, &str)]| {
        let mut a = attrs(own);
        a.insert("grid_mapping".into(), mapping.into());
        VariableInfo {
            name: name.into(),
            attributes: a,
            ..Default::default()
        }
    };
    let mut vars = vec![
        var("r10m/b02", "spatial_ref", &[]),
        var("b03", "crs: x y", &[]),
        var("b04", "crs", &[("crs_wkt", "own")]),
    ];
    let mappings = HashMap::from([
        (
            "r10m/spatial_ref".to_string(),
            attrs(&[("crs_wkt", UTM36N_WKT1)]),
        ),
        (
            "crs".to_string(),
            attrs(&[("crs_wkt", "root"), ("epsg_code", "EPSG:4326")]),
        ),
    ]);
    attach_grid_mappings(&mut vars, |path| mappings.get(path).cloned());
    assert_eq!(
        vars[0].attributes["crs_wkt"], UTM36N_WKT1,
        "the group's mapping"
    );
    assert_eq!(vars[1].attributes["crs_wkt"], "root", "the `crs: x y` form");
    assert_eq!(vars[2].attributes["crs_wkt"], "own", "its own key kept");
    assert_eq!(vars[2].attributes["epsg_code"], "EPSG:4326");
}

#[test]
fn levels_without_a_crs_take_level_0s() {
    let (_app, mut meta) = crate::app::test_support::pyramid_app();
    let base = meta
        .variables
        .iter_mut()
        .find(|v| v.name.trim_matches('/') == "0/t2m");
    let base = base.expect("level 0");
    base.attributes
        .insert("proj:code".into(), "EPSG:32636".into());
    assign(&mut meta);
    for level in ["0/t2m", "1/t2m", "2/t2m"] {
        let var = meta
            .variables
            .iter()
            .find(|v| v.name.trim_matches('/') == level);
        assert_eq!(
            var.and_then(|v| v.crs.clone()),
            Some(CrsRef::Epsg(32636)),
            "{level}"
        );
    }
    // `sst`'s stack names none: its lat/lon dimensions make it geographic.
    let sst = meta
        .variables
        .iter()
        .find(|v| v.name.trim_matches('/') == "1/sst");
    assert_eq!(sst.and_then(|v| v.crs.clone()), Some(CrsRef::Geographic));
}

fn ready(crs: &CrsRef) -> Arc<super::Crs> {
    match resolve(crs) {
        Resolution::Ready(r) => r,
        other => panic!("{crs:?} resolved to {other:?}"),
    }
}

#[test]
fn epsg_and_wkt_resolve_to_proj_strings() {
    let utm = ready(&CrsRef::Epsg(32636));
    assert!(!utm.geographic);
    assert_eq!(utm.unit, "m");
    assert!(ready(&CrsRef::Epsg(4326)).geographic);
    assert!(ready(&CrsRef::Geographic).geographic);
    assert!(!ready(&CrsRef::Epsg(3413)).geographic);
    // WKT without an EPSG code: through `proj4wkt` (desktop only).
    let custom = UTM36N_WKT1.replace(r#",AUTHORITY["EPSG","32636"]]"#, "]");
    for wkt in [custom.as_str(), UTM36N_WKT2] {
        let from_wkt = ready(&CrsRef::Wkt(wkt.into()));
        assert!(!from_wkt.geographic, "{wkt}");
        assert_eq!(from_wkt.unit, "m");
    }
}

#[test]
fn unknown_codes_and_text_are_unsupported() {
    for crs in [
        CrsRef::Epsg(999_999),
        CrsRef::Wkt("PROJCS[\"broken\"".into()),
    ] {
        assert!(
            matches!(resolve(&crs), Resolution::Unsupported(_)),
            "{crs:?}"
        );
    }
}
