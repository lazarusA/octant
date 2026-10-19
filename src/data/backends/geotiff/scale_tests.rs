//! Band scale and offset from GDAL metadata, applied to fetched blocks.

use super::bands::parse_gdal_metadata;
use super::scale::{BandScale, BandScales};
use super::store::GeoTiffBlockStore;
use super::test_utils::SyntheticTiffBuilder;
use crate::data::blocks::BlockStore;
use crate::data::slice_request::SliceRequest;

/// As OpenLandMap writes it: GDAL's shortest round trip of 0.1.
const PH_XML: &str = r#"<GDALMetadata>
  <Item name="OFFSET" sample="0" role="offset">0</Item>
  <Item name="SCALE" sample="0" role="scale">0.100000000000000006</Item>
</GDALMetadata>"#;

#[test]
fn scale_and_offset_are_read_per_band() {
    let xml = r#"<GDALMetadata>
  <Item name="SCALE" sample="1" role="scale">2</Item>
  <Item name="OFFSET" sample="1" role="offset">-5</Item>
  <Item name="OFFSET" sample="2" role="offset">10</Item>
  <Item name="SCALE" sample="3" role="scale">1</Item>
</GDALMetadata>"#;
    let scales = BandScales::from_items(&parse_gdal_metadata(xml), 4);
    assert_eq!(scales.band(0), None);
    let two = BandScale {
        scale: 2.0,
        offset: -5.0,
    };
    assert_eq!(scales.band(1), Some(two));
    let shifted = BandScale {
        scale: 1.0,
        offset: 10.0,
    };
    assert_eq!(scales.band(2), Some(shifted));
    assert_eq!(scales.band(3), None, "the identity is no transform");
    assert_eq!(scales.band(9), None);
}

#[test]
fn metadata_without_transforms_or_with_bad_values_has_none() {
    let none = BandScales::from_items(&parse_gdal_metadata("<GDALMetadata/>"), 3);
    assert_eq!(none, BandScales::default());
    let bad = r#"<Item name="SCALE" sample="0" role="scale">abc</Item>
<Item name="SCALE" sample="1" role="scale">0</Item>"#;
    assert_eq!(
        BandScales::from_items(&parse_gdal_metadata(bad), 2),
        BandScales::default()
    );
}

#[test]
fn applying_keeps_missing_values_missing() {
    let s = BandScale {
        scale: 0.5,
        offset: 1.0,
    };
    let mut values = [2.0, f32::NAN, -4.0];
    s.apply(&mut values);
    assert_eq!(values[0], 2.0);
    assert!(values[1].is_nan());
    assert_eq!(values[2], -1.0);
}

#[test]
fn fetched_blocks_hold_physical_values() {
    // pH 6.5 and 7.2 stored as 65 and 72; 255 is nodata.
    let raw: Vec<u8> = [65u8, 72, 255, 65].repeat(4);
    let mut builder = SyntheticTiffBuilder::new(4, 4)
        .samples(1, 8)
        .striped_data(&raw, 4);
    builder.add_ascii(42112, PH_XML);
    builder.add_ascii(42113, "255");
    let bytes = builder.build();
    let rt = crate::utils::executor::get_shared_tokio_rt();
    let store = rt
        .block_on(async move { GeoTiffBlockStore::from_bytes("ph.tif", bytes).await })
        .expect("open from bytes");

    let block = store
        .fetch_block(&SliceRequest::full_range("band_1", &[4, 4]))
        .expect("fetch band_1");
    assert!((block.values[0] - 6.5).abs() < 1e-6);
    assert!((block.values[1] - 7.2).abs() < 1e-6);
    assert!(block.values[2].is_nan(), "nodata is matched before scaling");

    let meta = store.inspect().expect("inspect");
    let band = meta.variables.iter().find(|v| v.name == "band_1");
    let scale = band.and_then(|v| v.attributes.get("SCALE"));
    assert_eq!(scale.map(String::as_str), Some("0.100000000000000006"));
}
