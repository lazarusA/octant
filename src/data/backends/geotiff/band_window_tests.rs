//! A multi-band raster reads only the bands its request selects, and an
//! alpha last band is tagged for the composite default.

use super::test_utils::SyntheticTiffBuilder;
use super::tests::open_store_from_bytes;
use crate::data::blocks::BlockStore;
use crate::data::slice_request::{DimensionSelection, SliceRequest};

/// A 4x4 chunky 8-bit raster of `bands` bands whose sample `i` holds `i`.
fn raster(bands: u16) -> Vec<u8> {
    let raw: Vec<u8> = (0..16 * bands).map(|i| i as u8).collect();
    SyntheticTiffBuilder::new(4, 4)
        .samples(bands, 8)
        .striped_data(&raw, 4)
        .build()
}

fn request(band: DimensionSelection) -> SliceRequest {
    SliceRequest {
        variable: "raster".into(),
        selections: vec![
            band,
            DimensionSelection::Range { start: 0, end: 4 },
            DimensionSelection::Range { start: 0, end: 4 },
        ],
    }
}

#[test]
fn a_raster_reads_only_the_selected_bands() {
    let store = open_store_from_bytes("bands.tif", raster(5));
    let block = store
        .fetch_block(&request(DimensionSelection::Range { start: 1, end: 4 }))
        .expect("bands 1-3");
    assert_eq!(block.shape, [3, 4, 4]);
    assert_eq!(block.origin[0], 1);
    // Pixel 0 of bands 1 and 3, pixel 1 of band 1 (interleaved by 5).
    assert_eq!(
        (block.values[0], block.values[32], block.values[1]),
        (1.0, 3.0, 6.0)
    );

    let one = store
        .fetch_block(&request(DimensionSelection::Index(4)))
        .expect("band 4");
    assert_eq!((one.shape.as_slice(), one.origin[0]), (&[1, 4, 4][..], 4));
    assert_eq!(one.values[0], 4.0);
}

#[test]
fn an_expanded_palette_reads_only_the_selected_channels() {
    let indices: Vec<u8> = (0..16).collect();
    let ramp: Vec<u16> = (0..256).map(|i| (i * 100) as u16).collect();
    let bytes = SyntheticTiffBuilder::new(4, 4)
        .samples(1, 8)
        .photometric(3)
        .colormap(&ramp, &ramp, &ramp)
        .striped_data(&indices, 4)
        .build();
    let store = open_store_from_bytes("palette.tif", bytes);
    let all = store
        .fetch_block(&SliceRequest::full_range("raster", &[3, 4, 4]))
        .expect("all");
    let blue = store
        .fetch_block(&request(DimensionSelection::Index(2)))
        .expect("blue");
    assert_eq!(blue.origin[0], 2);
    assert_eq!(&blue.values[..], &all.values[32..48]);
}

#[test]
fn an_alpha_last_band_is_tagged() {
    let alpha_of = |extra: Option<u16>| {
        let raw = vec![0u8; 16 * 4];
        let mut builder = SyntheticTiffBuilder::new(4, 4)
            .samples(4, 8)
            .photometric(2)
            .striped_data(&raw, 4);
        if let Some(kind) = extra {
            builder.add_short_vec(338, &[kind]); // ExtraSamples
        }
        let store = open_store_from_bytes("rgba.tif", builder.build());
        let meta = store.inspect().expect("inspect");
        let raster = meta.variables.iter().find(|v| v.name == "raster");
        raster.and_then(|v| v.attributes.get("alpha_band").cloned())
    };
    assert_eq!(alpha_of(Some(2)).as_deref(), Some("last"), "unassociated");
    assert_eq!(alpha_of(Some(1)).as_deref(), Some("last"), "associated");
    assert_eq!(alpha_of(Some(0)), None, "unspecified extra sample");
    assert_eq!(alpha_of(None), None);
}
