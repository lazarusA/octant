//! Band name resolution and GDAL metadata parsing tests.

use std::collections::HashMap;

use async_tiff::tags::ExtraSamples;

use super::bands::{GdalItem, add_gdal_attributes, parse_gdal_metadata, resolve_labels};
use super::store::GeoTiffBlockStore;
use super::test_utils::SyntheticTiffBuilder;
use crate::data::blocks::BlockStore;
use crate::data::metadata::DatasetMetadata;
use crate::ui::hover::field::HoverField;
use crate::ui::hover::format::format_dimension_coord;

const GDAL_METADATA_TAG: u16 = 42112;
const EXTRA_SAMPLES_TAG: u16 = 338;

const SENTINEL_XML: &str = r#"<GDALMetadata>
  <Item name="AREA_OR_POINT">Area</Item>
  <Item name="DESCRIPTION" sample="0" role="description">Red</Item>
  <Item name="DESCRIPTION" sample="3" role="description">NIR &amp; edge</Item>
  <Item name="wavelength" sample="3">842</Item>
</GDALMetadata>"#;

fn inspect(bytes: Vec<u8>) -> DatasetMetadata {
    let rt = crate::utils::executor::get_shared_tokio_rt();
    let store = rt
        .block_on(async move { GeoTiffBlockStore::from_bytes("bands.tif", bytes).await })
        .expect("open from bytes");
    store.inspect().expect("inspect")
}

fn four_band_tiff(xml: Option<&str>) -> Vec<u8> {
    let mut builder = SyntheticTiffBuilder::new(2, 2)
        .samples(4, 8)
        .striped_data(&[0u8; 16], 2);
    if let Some(xml) = xml {
        builder.add_ascii(GDAL_METADATA_TAG, xml);
    }
    builder.build()
}

fn labels(values: &[&str]) -> Option<Vec<String>> {
    Some(values.iter().map(|s| s.to_string()).collect())
}

#[test]
fn gdal_items_keep_sample_role_and_unescaped_text() {
    let items = parse_gdal_metadata(SENTINEL_XML);
    assert_eq!(items.len(), 4);
    assert_eq!(
        items[0],
        GdalItem {
            name: "AREA_OR_POINT".into(),
            sample: None,
            role: None,
            value: "Area".into(),
        }
    );
    assert_eq!(items[2].sample, Some(3));
    assert_eq!(items[2].value, "NIR & edge");
    assert_eq!(items[3].role, None);
}

#[test]
fn malformed_gdal_metadata_yields_the_complete_items_only() {
    assert!(parse_gdal_metadata("not xml").is_empty());
    assert!(parse_gdal_metadata(r#"<Item name="DESCRIPTION" sample="0">Red"#).is_empty());
    let items =
        parse_gdal_metadata(r#"<Item name="a" sample='1'>x &#x41;&#66; &bogus;</Item><Item"#);
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].sample, Some(1));
    assert_eq!(items[0].value, "x AB &bogus;");
}

#[test]
fn descriptions_win_over_layout_and_unnamed_bands_get_defaults() {
    let items = parse_gdal_metadata(SENTINEL_XML);
    assert_eq!(
        resolve_labels(&items, &[], &[], 4),
        labels(&["Red", "Band 2", "Band 3", "NIR & edge"])
    );
    let rgb = ["Red", "Green", "Blue"];
    let alpha = [ExtraSamples::UnassociatedAlpha];
    assert_eq!(
        resolve_labels(&items, &rgb, &alpha, 4),
        labels(&["Red", "Green", "Blue", "NIR & edge"])
    );
    assert_eq!(
        resolve_labels(&[], &rgb, &alpha, 4),
        labels(&["Red", "Green", "Blue", "Alpha"])
    );
    assert_eq!(
        resolve_labels(&[], &[], &[ExtraSamples::Unspecified], 4),
        None
    );
}

#[test]
fn gdal_items_become_dataset_and_band_attributes() {
    let items = parse_gdal_metadata(SENTINEL_XML);
    let mut attrs = HashMap::from([("AREA_OR_POINT".to_string(), "Point".to_string())]);
    add_gdal_attributes(&mut attrs, &items, None);
    assert_eq!(
        attrs.get("AREA_OR_POINT").map(String::as_str),
        Some("Point")
    );
    add_gdal_attributes(&mut attrs, &items, Some(3));
    assert_eq!(attrs.get("wavelength").map(String::as_str), Some("842"));
    assert!(!attrs.contains_key("DESCRIPTION"));
}

#[test]
fn labelled_multiband_file_names_its_bands() {
    let meta = inspect(four_band_tiff(Some(SENTINEL_XML)));
    let expected = labels(&["Red", "Band 2", "Band 3", "NIR & edge"]);
    let band = meta
        .get_dim_coords(Some("raster"), "band")
        .and_then(|c| c.labels())
        .map(<[String]>::to_vec);
    assert_eq!(band, expected);

    let nir = meta.variables.iter().find(|v| v.name == "band_4");
    let nir = nir.expect("band_4 variable");
    assert_eq!(nir.long_name.as_deref(), Some("NIR & edge"));
    assert_eq!(
        nir.attributes.get("wavelength").map(String::as_str),
        Some("842")
    );

    let raster = meta.variables.iter().find(|v| v.name == "raster");
    assert_eq!(
        format_dimension_coord(Some(&meta), raster, None, "band", 3, 4, None),
        HoverField::new("band", "NIR & edge")
    );
}

#[test]
fn unlabelled_bands_keep_the_index_and_rgba_bands_are_named() {
    let meta = inspect(four_band_tiff(None));
    assert_eq!(meta.get_dim_coords(Some("raster"), "band"), None);
    let raster = meta.variables.iter().find(|v| v.name == "raster");
    assert_eq!(
        format_dimension_coord(Some(&meta), raster, None, "band", 1, 4, None),
        HoverField::new("band", "2 / 4")
    );

    let mut rgba = SyntheticTiffBuilder::new(2, 2)
        .samples(4, 8)
        .photometric(2)
        .striped_data(&[0u8; 16], 2);
    rgba.add_short(EXTRA_SAMPLES_TAG, 2);
    let meta = inspect(rgba.build());
    let band = meta
        .get_dim_coords(Some("raster"), "band")
        .and_then(|c| c.labels())
        .map(<[String]>::to_vec);
    assert_eq!(band, labels(&["Red", "Green", "Blue", "Alpha"]));
}

#[test]
fn cmyk_bands_keep_their_ink_names() {
    let cmyk = SyntheticTiffBuilder::new(2, 2)
        .samples(4, 8)
        .photometric(5)
        .striped_data(&[0u8; 16], 2)
        .build();
    let meta = inspect(cmyk);
    let names: Vec<_> = (1..=4)
        .filter_map(|i| {
            meta.variables
                .iter()
                .find(|v| v.name == format!("band_{i}"))
        })
        .filter_map(|v| v.long_name.as_deref())
        .collect();
    assert_eq!(names, ["Cyan", "Magenta", "Yellow", "Black"]);
}

#[test]
fn expanded_palette_bands_are_rgb_but_the_index_band_is_not() {
    let ramp = vec![0u16; 256];
    let palette = SyntheticTiffBuilder::new(2, 2)
        .samples(1, 8)
        .photometric(3)
        .colormap(&ramp, &ramp, &ramp)
        .striped_data(&[0u8; 4], 2)
        .build();
    let meta = inspect(palette);
    let band = meta
        .get_dim_coords(Some("raster"), "band")
        .and_then(|c| c.labels())
        .map(<[String]>::to_vec);
    assert_eq!(band, labels(&["Red", "Green", "Blue"]));
    let index = meta.variables.iter().find(|v| v.name == "band_1");
    assert_eq!(index.and_then(|v| v.long_name.as_deref()), Some("Band 1"));
}
