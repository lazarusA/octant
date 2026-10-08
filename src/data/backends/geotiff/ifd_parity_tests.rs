//! `ifd_check` copies two pieces of `async-tiff` it keeps private: the GeoKey
//! ids it knows and the IFD reading loop. These tests compare them with the
//! crate itself, so an `async-tiff` update that changes either fails here.

use std::collections::HashMap;

use async_tiff::metadata::TiffMetadataReader;
use async_tiff::reader::Endianness;
use async_tiff::tags::Tag;
use async_tiff::{ImageFileDirectory, TagValue};

use super::ifd_check::{KNOWN_GEO_KEYS, read_checked_ifds};
use super::reader::MemoryTiffReader;
use super::test_utils::SyntheticTiffBuilder;

/// A grayscale image whose GeoKey directory holds `keys`.
fn with_geo_keys(keys: &[u16]) -> HashMap<Tag, TagValue> {
    HashMap::from([
        (Tag::ImageWidth, TagValue::Short(4)),
        (Tag::ImageLength, TagValue::Short(3)),
        (Tag::BitsPerSample, TagValue::Short(8)),
        (Tag::SamplesPerPixel, TagValue::Short(1)),
        (Tag::PhotometricInterpretation, TagValue::Short(1)),
        (
            Tag::GeoKeyDirectory,
            TagValue::List(keys.iter().map(|&v| TagValue::Short(v)).collect()),
        ),
    ])
}

/// Whether `async-tiff` reads GeoKey `id` instead of skipping it: a key it
/// knows, given inline (location 0), either lands in the directory or fails
/// to convert; a key it skips leaves the directory empty.
fn crate_knows_geo_key(id: u16, empty: &ImageFileDirectory) -> bool {
    let tags = with_geo_keys(&[1, 1, 0, 1, id, 0, 1, 1]);
    match ImageFileDirectory::from_tags(tags, Endianness::LittleEndian) {
        Ok(ifd) => ifd.geo_key_directory() != empty.geo_key_directory(),
        Err(_) => true,
    }
}

#[test]
fn known_geo_keys_match_async_tiff() {
    let empty =
        ImageFileDirectory::from_tags(with_geo_keys(&[1, 1, 0, 0]), Endianness::LittleEndian)
            .expect("an empty GeoKey directory parses");
    let known: Vec<u16> = (0..=u16::MAX)
        .filter(|&id| crate_knows_geo_key(id, &empty))
        .collect();
    assert_eq!(known, KNOWN_GEO_KEYS, "update ifd_check::KNOWN_GEO_KEYS");
}

/// Our IFD loop and the crate's `read_all_ifds` on the same bytes.
fn both_readers(bytes: Vec<u8>) -> (Vec<ImageFileDirectory>, Vec<ImageFileDirectory>) {
    let rt = crate::utils::executor::get_shared_tokio_rt();
    rt.block_on(async move {
        let reader = MemoryTiffReader::new(bytes);
        let mut meta = TiffMetadataReader::try_open(&reader).await.expect("header");
        let ours = read_checked_ifds(&meta, &reader)
            .await
            .expect("checked IFDs");
        let theirs = meta.read_all_ifds(&reader).await.expect("crate IFDs");
        (ours, theirs)
    })
}

#[test]
fn checked_ifds_match_the_crate_reader() {
    let tile: Vec<u8> = (0..16 * 16 * 3 * 4).map(|i| i as u8).collect();
    let files = [
        SyntheticTiffBuilder::new(4, 3)
            .striped_data(&[7; 12], 3)
            .build(),
        SyntheticTiffBuilder::new(32, 32)
            .samples(3, 8)
            .photometric(2)
            .tiled(16, 16, &tile, 4)
            .build(),
        SyntheticTiffBuilder::new(4, 3)
            .striped_data(&[7; 12], 3)
            .geo_keys(
                [0.1, 0.1, 0.0],
                [0.0, 0.0, 0.0, -120.0, 35.0, 0.0],
                &[1, 1, 0, 2, 1024, 0, 1, 1, 3073, 34737, 6, 0],
                Some("WGS 84|"),
            )
            .build(),
    ];
    for (i, bytes) in files.into_iter().enumerate() {
        let (ours, theirs) = both_readers(bytes);
        assert!(!ours.is_empty(), "file {i}: no IFDs");
        assert_eq!(ours, theirs, "file {i}");
    }
}
