//! GeoTIFF x/y coordinates: georeferenced pixel centers, or pixel indices.

use super::store::GeoTiffBlockStore;
use super::test_utils::SyntheticTiffBuilder;
use crate::data::DatasetMetadata;
use crate::data::blocks::BlockStore;
use crate::data::slice_request::{DimensionSelection, SliceRequest};

fn inspect(bytes: Vec<u8>) -> DatasetMetadata {
    let rt = crate::utils::executor::get_shared_tokio_rt();
    let store = rt
        .block_on(async move { GeoTiffBlockStore::from_bytes("axes.tif", bytes).await })
        .expect("open from bytes");
    store.inspect().expect("inspect")
}

#[test]
fn axes_are_pixel_centers_or_pixel_indices() {
    let tiff = |geo: bool| {
        let builder = SyntheticTiffBuilder::new(10, 10).striped_data(&[0u8; 100], 10);
        let keys = [1, 1, 0, 1, 1024, 0, 1, 1];
        let builder = if geo {
            builder.geo_keys(
                [0.1, 0.1, 0.0],
                [0.0, 0.0, 0.0, -120.0, 35.0, 0.0],
                &keys,
                None,
            )
        } else {
            builder
        };
        inspect(builder.build())
    };
    let close = |a: f64, b: f64| (a - b).abs() < 1e-9;

    let geo = tiff(true);
    let x = geo.get_dim_coords(Some("band_1"), "x").expect("x");
    let y = geo.get_dim_coords(Some("band_1"), "y").expect("y");
    assert!(x.matches(10) && y.matches(10));
    assert!(close(x.number(0).unwrap_or(f64::NAN), -119.95));
    assert!(close(x.number(9).unwrap_or(f64::NAN), -119.05));
    assert!(
        close(y.number(0).unwrap_or(f64::NAN), 34.95),
        "rows run north to south"
    );
    assert!(close(y.number(9).unwrap_or(f64::NAN), 34.05));

    let plain = tiff(false);
    let x = plain.get_dim_coords(Some("band_1"), "x").expect("x");
    assert_eq!((x.number(0), x.number(9)), (Some(0.0), Some(9.0)));
}

fn close(a: Option<f64>, b: f64) -> bool {
    a.is_some_and(|a| (a - b).abs() < 1e-9)
}

/// A 10x10 raster whose model transformation is `m` (row-major 4x4), with `keys`.
fn transformed(m: [f64; 16], keys: &[u16]) -> Vec<u8> {
    let mut builder = SyntheticTiffBuilder::new(10, 10).striped_data(&[0u8; 100], 10);
    builder.add_double_vec(34264, &m);
    builder.add_short_vec(34735, keys);
    builder.build()
}

const PROJECTED: [u16; 8] = [1, 1, 0, 1, 1024, 0, 1, 1];

#[test]
fn block_coordinates_are_the_metadata_pixel_centers() {
    let bytes = SyntheticTiffBuilder::new(10, 10)
        .striped_data(&[0u8; 100], 10)
        .geo_keys(
            [0.1, 0.1, 0.0],
            [0.0, 0.0, 0.0, -120.0, 35.0, 0.0],
            &PROJECTED,
            None,
        )
        .build();
    let rt = crate::utils::executor::get_shared_tokio_rt();
    let store = rt
        .block_on(async move { GeoTiffBlockStore::from_bytes("centers.tif", bytes).await })
        .expect("open from bytes");
    let meta = store.inspect().expect("inspect");
    let request = SliceRequest::new(
        "band_1",
        vec![
            DimensionSelection::range(2, 5),
            DimensionSelection::range(3, 6),
        ],
    );
    let block = store.fetch_block(&request).expect("fetch block");
    let x = meta.get_dim_coords(Some("band_1"), "x").expect("x");
    let y = meta.get_dim_coords(Some("band_1"), "y").expect("y");
    let expected = |c: &crate::data::CoordValues, range: std::ops::Range<usize>| -> Vec<f64> {
        range.filter_map(|i| c.number(i)).collect()
    };
    assert_eq!(block.coordinates["x"], expected(x, 3..6));
    assert_eq!(block.coordinates["y"], expected(y, 2..5));
}

#[test]
fn south_up_rasters_label_rows_from_the_south() {
    let m = [
        0.1, 0.0, 0.0, -120.0, //
        0.0, 0.1, 0.0, 34.0, //
        0.0, 0.0, 0.0, 0.0, //
        0.0, 0.0, 0.0, 1.0,
    ];
    let meta = inspect(transformed(m, &PROJECTED));
    let y = meta.get_dim_coords(Some("band_1"), "y").expect("y");
    assert!(close(y.number(0), 34.05), "row 0 is the southernmost");
    assert!(close(y.number(9), 34.95));
}

#[test]
fn rotated_rasters_fall_back_to_pixel_indices() {
    let m = [
        0.1, 0.02, 0.0, -120.0, //
        0.02, -0.1, 0.0, 35.0, //
        0.0, 0.0, 0.0, 0.0, //
        0.0, 0.0, 0.0, 1.0,
    ];
    let meta = inspect(transformed(m, &PROJECTED));
    let x = meta.get_dim_coords(Some("band_1"), "x").expect("x");
    assert_eq!((x.number(0), x.number(9)), (Some(0.0), Some(9.0)));
}

#[test]
fn pixel_is_point_tie_points_mark_pixel_centers() {
    // GTRasterTypeGeoKey (1025) = RasterPixelIsPoint: the tie point is pixel (0, 0)'s center.
    let keys = [1, 1, 0, 2, 1024, 0, 1, 1, 1025, 0, 1, 2];
    let bytes = SyntheticTiffBuilder::new(10, 10)
        .striped_data(&[0u8; 100], 10)
        .geo_keys(
            [0.1, 0.1, 0.0],
            [0.0, 0.0, 0.0, -120.0, 35.0, 0.0],
            &keys,
            None,
        )
        .build();
    let meta = inspect(bytes);
    let x = meta.get_dim_coords(Some("band_1"), "x").expect("x");
    let y = meta.get_dim_coords(Some("band_1"), "y").expect("y");
    assert!(close(x.number(0), -120.0));
    assert!(close(y.number(0), 35.0));
}
