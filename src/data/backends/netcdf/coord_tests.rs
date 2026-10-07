//! NetCDF dimension coordinates: `f64` numbers, string and char labels, labels replacing a
//! plain index, and coordinates resolved through groups.

use super::test_support::{TempNc, char_rows, inspect, netcdf_lock, put};
use crate::data::metadata::CoordValues;

#[test]
fn numbers_are_read_as_f64_and_kept_even_or_exact() {
    let _lock = netcdf_lock();
    let nc = TempNc::new("coord_numbers");
    let t0 = 1_700_000_000.0f64;
    let meta = inspect(&nc, |f| {
        for (dim, len) in [("time", 3), ("plev", 4), ("lon", 1440)] {
            f.add_dimension(dim, len).expect("dimension");
        }
        put(f, "time", &["time"], &[t0, t0 + 1.0, t0 + 3.0]);
        put(f, "plev", &["plev"], &[1000.0f64, 925.0, 850.0, 700.0]);
        let lon: Vec<f32> = (0..1440).map(|i| -180.0 + 0.25 * i as f32).collect();
        put(f, "lon", &["lon"], &lon);
    });
    let coords = |dim: &str| meta.get_dim_coords(None, dim).expect("coordinate");
    assert_eq!(
        coords("time").number(2),
        Some(t0 + 3.0),
        "epoch seconds stay exact"
    );
    assert_eq!(coords("plev").number(2), Some(850.0));
    assert!(matches!(coords("plev"), CoordValues::Values(_)));
    assert!(matches!(
        coords("lon"),
        CoordValues::Regular { len: 1440, .. }
    ));
}

#[test]
fn string_and_char_labels_name_their_dimension() {
    let _lock = netcdf_lock();
    let nc = TempNc::new("coord_labels");
    let meta = inspect(&nc, |f| {
        f.add_dimension("region", 3).expect("region");
        f.add_dimension("station", 2).expect("station");
        f.add_dimension("strlen", 8).expect("strlen");
        let mut region = f
            .add_string_variable("region", &["region"])
            .expect("region");
        for (i, name) in ["Europe", "Africa", "Asia"].iter().enumerate() {
            region.put_string(name, [i]).expect("region name");
        }
        // Char labels not named after their dimension count once a `coordinates`
        // attribute lists them.
        put(
            f,
            "station_name",
            &["station", "strlen"],
            &char_rows(&["Jena", "Mauna"], 8),
        );
        let mut data = f.add_variable::<f32>("co2", &["station"]).expect("co2");
        data.put_values(&[410.0f32, 415.0], ..).expect("co2 values");
        data.put_attribute("coordinates", "station_name")
            .expect("coordinates");
    });
    let label = |dim: &str, i| meta.get_dim_coords(None, dim).and_then(|c| c.label(i));
    assert_eq!(label("region", 1), Some("Africa"));
    assert_eq!(label("station", 1), Some("Mauna"), "NUL padding is trimmed");
}

#[test]
fn labels_replace_a_plain_index_but_not_real_numbers() {
    let _lock = netcdf_lock();
    let nc = TempNc::new("coord_priority");
    let meta = inspect(&nc, |f| {
        f.add_dimension("region", 2).expect("region");
        f.add_dimension("station", 2).expect("station");
        f.add_dimension("strlen", 6).expect("strlen");
        put(f, "region", &["region"], &[0i32, 1]);
        put(
            f,
            "region_name",
            &["region", "strlen"],
            &char_rows(&["North", "South"], 6),
        );
        put(f, "station", &["station"], &[101i32, 205]);
        put(
            f,
            "station_name",
            &["station", "strlen"],
            &char_rows(&["Jena", "Mauna"], 6),
        );
        let mut data = f
            .add_variable::<f32>("v", &["region", "station"])
            .expect("v");
        data.put_values(&[0.0f32; 4], ..).expect("v values");
        data.put_attribute("coordinates", "region_name station_name")
            .expect("attr");
    });
    let coords = |dim: &str| meta.get_dim_coords(None, dim).expect("coordinate");
    assert_eq!(coords("region").label(0), Some("North"));
    assert_eq!(
        coords("station").number(1),
        Some(205.0),
        "station ids are kept"
    );
}

#[test]
fn grouped_variables_resolve_coordinates_up_their_group_chain() {
    let _lock = netcdf_lock();
    let nc = TempNc::new("coord_groups");
    let meta = inspect(&nc, |f| {
        f.add_dimension("time", 2).expect("time");
        f.add_dimension("lat", 2).expect("lat");
        put(f, "time", &["time"], &[0.0f64, 6.0]);
        put(f, "lat", &["lat"], &[10.0f64, 20.0]);
        let mut g1 = f.add_group("g1").expect("g1");
        g1.add_dimension("lat", 3).expect("g1 lat");
        g1.add_group("g2").expect("g2");
        put(f, "g1/lat", &["lat"], &[30.0f64, 25.0, 10.0]);
        put(f, "g1/t2m", &["lat"], &[1.0f32, 2.0, 3.0]);
        put(f, "g1/g2/series", &["time"], &[1.0f32, 2.0]);
    });
    let grouped = meta.get_dim_coords(Some("g1/t2m"), "lat").expect("g1 lat");
    assert_eq!(grouped.len(), 3, "the group's own lat, not the root's");
    assert_eq!(grouped.number(1), Some(25.0));
    let root_time = meta
        .get_dim_coords(Some("g1/g2/series"), "time")
        .expect("time");
    assert_eq!(
        root_time.number(1),
        Some(6.0),
        "found two groups up, at the root"
    );
    let root_lat = meta.get_dim_coords(None, "lat").expect("root lat");
    assert_eq!(root_lat.len(), 2);
}
