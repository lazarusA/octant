//! GeoTIFF x/y coordinates: georeferenced pixel centers, or pixel indices.

use super::store::GeoTiffBlockStore;
use super::test_utils::SyntheticTiffBuilder;
use crate::data::DatasetMetadata;
use crate::data::blocks::BlockStore;

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
