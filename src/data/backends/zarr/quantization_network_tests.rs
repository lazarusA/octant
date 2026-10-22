//! Real embedding stores read dequantised, against values read with Python
//! (`cargo test --lib reads_dequantised -- --ignored`).

use crate::data::blocks::BlockStore;
use crate::data::{DimensionSelection, SliceRequest};

fn assert_close(values: &[f32], expected: &[f32]) {
    for (v, e) in values.iter().zip(expected) {
        assert!((v - e).abs() < 1e-6, "{values:?}");
    }
}

/// `cargo test --lib tessera_pixel_reads_dequantised -- --ignored`: one
/// TESSERA pixel (zone 01N, 2017) against the values the icechunk Python
/// library gives (codes `[2, 21, 18, 25]` times scale `0.032480314`).
#[test]
#[ignore = "network"]
fn tessera_pixel_reads_dequantised() {
    let url = "https://tessera-embeddings.s3.amazonaws.com/v1.1/dclimate.icechunk/";
    let store = crate::data::backends::IcechunkBlockStore::open(url).expect("open");
    let request = SliceRequest {
        variable: "01N/embeddings".into(),
        selections: vec![
            DimensionSelection::Index(0),
            DimensionSelection::Index(137_216),
            DimensionSelection::Index(22_528),
            DimensionSelection::Range { start: 0, end: 4 },
        ],
    };
    let block = store.fetch_block(&request).expect("block");
    assert_close(
        &block.values,
        &[0.064_960_63, 0.682_086_6, 0.584_645_6, 0.812_007_84],
    );
}

/// One AlphaEarth pixel in Berlin (2024), through the `s3://` URL of a bucket
/// with dots in its name: the codes `[-27, -79, -10, -65]` of Google's own
/// COG (33N tile `xvs7w6wxoaq660epm-0000008192-0000000000`) as
/// `(x / 127.5)² · sign(x)`.
#[test]
#[ignore = "network"]
fn alphaearth_pixel_reads_dequantised() {
    let url = "s3://us-west-2.opendata.source.coop/tge-labs/aef-mosaic/";
    let store = super::store::ZarrBlockStore::open_remote(url).expect("open");
    let request = SliceRequest {
        variable: "embeddings".into(),
        selections: vec![
            DimensionSelection::Index(7),
            DimensionSelection::Range { start: 0, end: 4 },
            DimensionSelection::Index(346_936),
            DimensionSelection::Index(2_152_983),
        ],
    };
    let block = store.fetch_block(&request).expect("block");
    assert_close(
        &block.values,
        &[-0.044_844_29, -0.383_913_88, -0.006_151_48, -0.259_900_04],
    );
}
