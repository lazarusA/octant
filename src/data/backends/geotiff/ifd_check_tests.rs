//! `ifd_check`: TIFFs `async-tiff` would panic on fail with an error instead.

use std::collections::HashMap;

use async_tiff::TagValue;
use async_tiff::tags::Tag;

use super::ifd_check::check_tags;
use super::store::GeoTiffBlockStore;
use super::test_utils::SyntheticTiffBuilder;

fn open(name: &str, bytes: Vec<u8>) -> Result<GeoTiffBlockStore, String> {
    let rt = crate::utils::executor::get_shared_tokio_rt();
    rt.block_on(async move { GeoTiffBlockStore::from_bytes(name, bytes).await })
        .map_err(|e| e.to_string())
}

/// A 4 x 3 striped image, one byte per pixel.
fn image() -> SyntheticTiffBuilder {
    SyntheticTiffBuilder::new(4, 3).striped_data(&[7; 12], 3)
}

/// SGI LogL/LogLuv HDR files: a photometric interpretation `async-tiff` does
/// not know, which it reports as missing by panicking.
#[test]
fn sgi_log_tiffs_fail_with_an_error() {
    for (photometric, compression, kind) in [
        (32844, 34676, "SGI LogL"),
        (32845, 34677, "SGI LogLuv"),
        (32845, 34676, "SGI LogLuv"),
    ] {
        let bytes = image()
            .photometric(photometric)
            .compression(compression)
            .build();
        let Err(err) = open("log.tif", bytes) else {
            panic!("photometric {photometric} must not open");
        };
        assert!(err.contains(kind), "{photometric}: {err}");
    }
}

#[test]
fn a_tiff_without_photometric_interpretation_fails_with_an_error() {
    let Err(err) = open("bare.tif", image().without(262).build()) else {
        panic!("a TIFF without PhotometricInterpretation must not open");
    };
    assert!(err.contains("PhotometricInterpretation"), "{err}");
}

#[test]
fn a_supported_tiff_still_opens() {
    assert!(open("gray.tif", image().build()).is_ok());
}

/// The tags every image needs, as a grayscale image.
fn minimal() -> HashMap<Tag, TagValue> {
    HashMap::from([
        (Tag::ImageWidth, TagValue::Short(4)),
        (Tag::ImageLength, TagValue::Short(3)),
        (Tag::BitsPerSample, TagValue::Short(8)),
        (Tag::SamplesPerPixel, TagValue::Short(1)),
        (Tag::PhotometricInterpretation, TagValue::Short(1)),
    ])
}

#[test]
fn minimal_tags_pass() {
    assert_eq!(check_tags(&minimal()), Ok(()));
}

#[test]
fn a_missing_required_tag_is_named() {
    let mut tags = minimal();
    tags.remove(&Tag::BitsPerSample);
    let err = check_tags(&tags).expect_err("missing tag");
    assert!(err.contains("BitsPerSample"), "{err}");
}

#[test]
fn a_non_rational_resolution_is_rejected() {
    let mut tags = minimal();
    tags.insert(Tag::XResolution, TagValue::Short(72));
    assert!(check_tags(&tags).is_err());
    tags.insert(Tag::XResolution, TagValue::Rational(72, 1));
    assert_eq!(check_tags(&tags), Ok(()));
}

fn geo_keys(entries: &[u16]) -> TagValue {
    TagValue::List(entries.iter().map(|&v| TagValue::Short(v)).collect())
}

#[test]
fn malformed_geo_key_directories_are_rejected() {
    let cases: [&[u16]; 4] = [
        &[1, 1],                          // no header
        &[2, 1, 0, 0],                    // wrong version
        &[1, 1, 0, 2, 1024, 0, 1, 2],     // fewer entries than keys
        &[1, 1, 0, 1, 3073, 34737, 9, 0], // ASCII value without params
    ];
    for entries in cases {
        let mut tags = minimal();
        tags.insert(Tag::GeoKeyDirectory, geo_keys(entries));
        assert!(check_tags(&tags).is_err(), "{entries:?}");
    }
}

#[test]
fn well_formed_geo_keys_pass_and_unknown_keys_are_skipped() {
    let mut tags = minimal();
    tags.insert(
        Tag::GeoKeyDirectory,
        geo_keys(&[
            1, 1, 0, 3, 1024, 0, 1, 2, 3073, 34737, 5, 0, 9999, 34737, 99, 0,
        ]),
    );
    tags.insert(Tag::GeoAsciiParams, TagValue::Ascii("WGS84|".into()));
    assert_eq!(check_tags(&tags), Ok(()));
}
