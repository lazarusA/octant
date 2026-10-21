//! The GeoTIFF tile decoders: each round-trips, and none decodes past its cap.

use std::io::Write;

use super::decoders::{deflate, lzw, packbits, zstd};

/// 10 000 bytes that compress well.
fn data() -> Vec<u8> {
    (0..10_000u32).map(|i| (i / 50 % 7) as u8).collect()
}

/// PackBits: runs of 50 equal bytes, each as a repeat (n = -49).
fn packbits_of(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    for run in data.chunks(50) {
        out.extend([(-(run.len() as i8 - 1)) as u8, run[0]]);
    }
    // A no-op byte (-128) is skipped.
    out.push(0x80);
    out
}

fn lzw_of(data: &[u8]) -> Vec<u8> {
    weezl::encode::Encoder::with_tiff_size_switch(weezl::BitOrder::Msb, 8)
        .encode(data)
        .expect("lzw")
}

fn deflate_of(data: &[u8]) -> Vec<u8> {
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::fast());
    encoder.write_all(data).expect("deflate");
    encoder.finish().expect("deflate")
}

/// Two frames, as some writers split a tile.
fn zstd_of(data: &[u8]) -> Vec<u8> {
    let (a, b) = data.split_at(data.len() / 2);
    let mut out = ::zstd::encode_all(a, 1).expect("zstd");
    out.extend(::zstd::encode_all(b, 1).expect("zstd"));
    out
}

type Decode = fn(&[u8], usize) -> async_tiff::error::AsyncTiffResult<Vec<u8>>;

#[test]
fn tiles_decode_up_to_their_cap_and_never_past_it() {
    let original = data();
    let cases: [(&str, Decode, Vec<u8>); 4] = [
        ("PackBits", packbits, packbits_of(&original)),
        ("LZW", lzw, lzw_of(&original)),
        ("Deflate", deflate, deflate_of(&original)),
        ("Zstd", zstd, zstd_of(&original)),
    ];
    for (name, decode, encoded) in cases {
        let decoded = decode(&encoded, original.len()).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(decoded, original, "{name} round trip");
        assert!(
            decode(&encoded, original.len() - 1).is_err(),
            "{name} past its cap"
        );
    }
}
