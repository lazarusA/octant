//! Decoded sizes are capped: a chunk never decodes past the size its codec
//! chain declares, and a capped read fails instead of growing.

use zarrs::array::BytesRepresentation;
use zarrs_codec::{BytesToBytesCodecTraits, CodecOptions};

use super::blusc_plugin::BluscCodec;
use super::capped::{MAX_DECODED_BYTES, UP_FRONT_BYTES, cap_for, read_capped, reserve_for};
use super::zstd_plugin::RuzstdCodec;

/// 10 000 bytes that compress well.
fn data() -> Vec<u8> {
    (0..10_000u32).map(|i| (i % 7) as u8).collect()
}

#[test]
fn a_capped_read_fails_past_its_cap() {
    let bytes = [1u8; 100];
    assert_eq!(read_capped(&bytes[..], 100, 0).expect("at the cap"), bytes);
    assert!(read_capped(&bytes[..], 99, 0).is_err());
}

#[test]
fn caps_follow_the_declared_size() {
    assert_eq!(cap_for(&BytesRepresentation::FixedSize(1000)), 1000);
    assert_eq!(cap_for(&BytesRepresentation::BoundedSize(1000)), 1000);
    assert_eq!(
        cap_for(&BytesRepresentation::UnboundedSize),
        MAX_DECODED_BYTES
    );
    // A fixed size is reserved whole; anything else starts small.
    assert_eq!(reserve_for(&BytesRepresentation::FixedSize(1000)), 1000);
    assert_eq!(
        reserve_for(&BytesRepresentation::BoundedSize(1000)),
        UP_FRONT_BYTES
    );
    // Metadata claiming a huge chunk caps at the limit, not at its claim.
    assert_eq!(
        cap_for(&BytesRepresentation::FixedSize(u64::MAX)),
        MAX_DECODED_BYTES
    );
}

#[test]
fn zstd_output_past_the_declared_size_is_an_error() {
    let original = data();
    let compressed = ruzstd::encoding::compress_to_vec(
        &original[..],
        ruzstd::encoding::CompressionLevel::Fastest,
    );
    let (codec, options) = (RuzstdCodec::new(), CodecOptions::default());
    let fixed = |n| BytesRepresentation::FixedSize(n);

    let decoded = codec
        .decode(compressed.clone().into(), &fixed(10_000), &options)
        .expect("a chunk of its declared size decodes");
    assert_eq!(decoded.as_ref(), original);
    assert!(
        codec
            .decode(compressed.into(), &fixed(1_000), &options)
            .is_err(),
        "a chunk expanding past its declared size"
    );
}

#[test]
fn blosc_header_claiming_more_than_the_chunk_is_an_error() {
    let original = data();
    let (codec, options) = (BluscCodec::new(), CodecOptions::default());
    let compressed = codec
        .encode(original.clone().into(), &options)
        .expect("compress");
    let fixed = |n| BytesRepresentation::FixedSize(n);

    let decoded = codec
        .decode(compressed.clone(), &fixed(10_000), &options)
        .expect("a chunk of its declared size decodes");
    assert_eq!(decoded.as_ref(), original);
    let err = codec
        .decode(compressed, &fixed(100), &options)
        .expect_err("a header claiming more than the chunk holds");
    assert!(err.to_string().contains("claims 10000 bytes"), "{err}");
}
