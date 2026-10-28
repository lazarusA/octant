//! Coordinates placed by grid affines.

use std::collections::HashMap;
use std::sync::Arc;

use zarrs::array::{ArrayBuilder, ArraySubset, DataType, FillValue};
use zarrs::storage::store::MemoryStore;

use super::affine::GridAffine;
use crate::data::backends::coord_bounds::fetch_all_dimension_coordinates_for_variables;
use crate::data::multiscale::geozarr_layout::array_affine;
use crate::data::{CoordValues, DatasetMetadata, VariableInfo};

fn dims(names: &[&str]) -> Vec<String> {
    names.iter().map(|d| d.to_string()).collect()
}

/// TESSERA's global level 0: 0.0001 degree cells from 180 W, 90 N.
const GLOBAL: [f64; 6] = [0.0001, 0.0, -180.0, 0.0, -0.0001, 90.0];

#[test]
fn pixel_registration_places_cell_centres() {
    let a = GridAffine {
        transform: GLOBAL,
        x: 1,
        y: 0,
        node: false,
    };
    let names = dims(&["lat", "lon", "band"]);
    let coords = a.coordinates(&names, &[1_800_000, 3_600_000, 4]);
    let [
        (
            x,
            CoordValues::Regular {
                start: x0,
                step: dx,
                len: nx,
            },
        ),
        (
            y,
            CoordValues::Regular {
                start: y0,
                step: dy,
                len: ny,
            },
        ),
    ] = coords.as_slice()
    else {
        panic!("x and y regular: {coords:?}");
    };
    assert_eq!((*x, *y), ("lon", "lat"));
    assert!((x0 + 179.99995).abs() < 1e-9 && (y0 - 89.99995).abs() < 1e-9);
    assert_eq!(
        (*dx, *dy, *nx, *ny),
        (0.0001, -0.0001, 3_600_000, 1_800_000)
    );
}

#[test]
fn node_registration_starts_at_the_origin() {
    let a = GridAffine {
        transform: [2.0, 0.0, 10.0, 0.0, -2.0, 50.0],
        x: 1,
        y: 0,
        node: true,
    };
    let names = dims(&["y", "x"]);
    let coords = a.coordinates(&names, &[4, 4]);
    assert_eq!(coords[0].1.first_number(), Some(10.0));
    assert_eq!(coords[1].1.first_number(), Some(50.0));
}

#[test]
fn rotated_or_degenerate_affines_place_nothing() {
    let names = dims(&["y", "x"]);
    for transform in [
        [1.0, 0.5, 0.0, 0.0, -1.0, 0.0],
        [1.0, 0.0, 0.0, f64::NAN, -1.0, 0.0],
    ] {
        let a = GridAffine {
            transform,
            x: 1,
            y: 0,
            node: false,
        };
        assert!(a.coordinates(&names, &[4, 4]).is_empty(), "{transform:?}");
    }
    let flat = GridAffine {
        transform: [0.0, 0.0, 0.0, 0.0, -1.0, 0.0],
        x: 1,
        y: 0,
        node: false,
    };
    let coords = flat.coordinates(&names, &[4, 4]);
    assert_eq!(coords.len(), 1, "a zero x step places only y");
    assert_eq!(coords[0].0, "y");
}

#[test]
fn array_attributes_give_an_affine() {
    let attrs = HashMap::from([
        ("spatial:transform".to_string(), format!("{GLOBAL:?}")),
        (
            "spatial:dimensions".to_string(),
            r#"["lat","lon"]"#.to_string(),
        ),
    ]);
    let a = array_affine(&attrs, &dims(&["band", "lat", "lon"])).expect("affine");
    assert_eq!((a.x, a.y, a.node), (2, 1, false));
    assert_eq!(array_affine(&HashMap::new(), &dims(&["y", "x"])), None);
}

#[test]
fn affine_variables_still_read_other_coordinate_arrays() {
    let store = Arc::new(MemoryStore::new());
    let dt =
        DataType::from_metadata(&zarrs::metadata::v3::MetadataV3::new("float64")).expect("float64");
    let band = ArrayBuilder::new(vec![4], vec![4], dt, FillValue::from(f64::NAN))
        .build(store.clone(), "/g/0/band")
        .expect("band");
    band.store_metadata().expect("band metadata");
    band.store_array_subset(&ArraySubset::new_with_shape(vec![4]), &[1.0, 2.0, 3.0, 4.0])
        .expect("band values");
    let variables = vec![VariableInfo {
        name: "g/0/rgb".into(),
        shape: vec![8, 16, 4],
        dimension_names: dims(&["lat", "lon", "band"]),
        affine: Some(GridAffine {
            transform: [22.5, 0.0, -180.0, 0.0, -22.5, 90.0],
            x: 1,
            y: 0,
            node: false,
        }),
        ..Default::default()
    }];
    let meta = DatasetMetadata {
        dimension_coordinates: fetch_all_dimension_coordinates_for_variables(
            store,
            &variables,
            Some("affine_store"),
        ),
        variables,
        ..Default::default()
    };
    let first = |dim| meta.get_dim_coords(Some("g/0/rgb"), dim)?.first_number();
    assert_eq!(first("lon"), Some(-168.75));
    assert_eq!(first("lat"), Some(78.75));
    assert_eq!(first("band"), Some(1.0), "read from its array");
}
