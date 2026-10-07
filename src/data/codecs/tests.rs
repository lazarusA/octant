use super::*;
use zarrs::array::ArrayMetadata;
use zarrs::array::BytesRepresentation;
use zarrs_codec::{BytesToBytesCodecTraits, CodecOptions};

#[test]
fn test_normalize_v3_zlib_and_shuffle() {
    let raw = serde_json::json!({
        "zarr_format": 3,
        "node_type": "array",
        "shape": [5, 3600, 7200],
        "data_type": "int16",
        "chunk_grid": {
            "name": "regular",
            "configuration": {
                "chunk_shape": [1, 1200, 2400]
            }
        },
        "chunk_key_encoding": {
            "name": "default",
            "configuration": { "separator": "/" }
        },
        "fill_value": -9999,
        "codecs": [
            { "name": "numcodecs.shuffle", "configuration": { "elementsize": 2 } },
            { "name": "numcodecs.zlib", "configuration": { "level": 1 } }
        ]
    });

    let norm = normalize_v3_array_metadata(raw);
    let codecs = norm.get("codecs").and_then(|c| c.as_array()).unwrap();
    assert_eq!(codecs.len(), 3);
    assert_eq!(codecs[0]["name"], "bytes");
    assert_eq!(codecs[1]["name"], "numcodecs.shuffle");
    assert_eq!(codecs[2]["name"], "numcodecs.zlib");

    let array_meta: Result<ArrayMetadata, _> = serde_json::from_value(norm);
    assert!(array_meta.is_ok());
}

#[test]
fn test_normalize_v3_blosc() {
    let raw = serde_json::json!({
        "zarr_format": 3,
        "node_type": "array",
        "shape": [100, 100],
        "data_type": "float32",
        "chunk_grid": {
            "name": "regular",
            "configuration": { "chunk_shape": [10, 10] }
        },
        "chunk_key_encoding": {
            "name": "default",
            "configuration": { "separator": "/" }
        },
        "fill_value": 0.0,
        "codecs": [
            {
                "name": "numcodecs.blosc",
                "configuration": {
                    "cname": "zstd",
                    "clevel": 5,
                    "shuffle": 1,
                    "blocksize": 0
                }
            }
        ]
    });

    let norm = normalize_v3_array_metadata(raw);
    let array_meta: Result<ArrayMetadata, _> = serde_json::from_value(norm);
    assert!(array_meta.is_ok());
}

#[test]
fn test_blusc_codec_round_trip() {
    let codec = BluscCodec::new();
    let options = CodecOptions::default();
    let rep = BytesRepresentation::UnboundedSize;

    let original_data: Vec<f32> = (0..256).map(|x| (x as f32) * 0.5).collect();
    let bytes: &[u8] = bytemuck::cast_slice(&original_data);

    let compressed = codec
        .encode(bytes.into(), &options)
        .expect("compression should succeed");
    assert!(!compressed.is_empty());

    let decompressed = codec
        .decode(compressed, &rep, &options)
        .expect("decompression should succeed");
    assert_eq!(decompressed.as_ref(), bytes);
}

#[test]
fn test_blusc_codec_int16_round_trip() {
    let codec = BluscCodec::new();
    let options = CodecOptions::default();
    let rep = BytesRepresentation::UnboundedSize;

    let original_data: Vec<i16> = (0..1024).map(|x| (x % 500) as i16).collect();
    let bytes: &[u8] = bytemuck::cast_slice(&original_data);

    let compressed = codec
        .encode(bytes.into(), &options)
        .expect("compression should succeed");
    assert!(!compressed.is_empty());

    let decompressed = codec
        .decode(compressed, &rep, &options)
        .expect("decompression should succeed");
    assert_eq!(decompressed.as_ref(), bytes);
}

#[test]
fn test_blusc_decompress_various_compressors() {
    use blusc::api::*;
    use blusc::*;

    let original_data: Vec<f32> = (0..2048)
        .map(|i| if i % 17 == 0 { 0.0 } else { (i as f32) * 1.25 })
        .collect();
    let bytes: &[u8] = bytemuck::cast_slice(&original_data);
    let codec = BluscCodec::new();
    let options = CodecOptions::default();
    let rep = BytesRepresentation::UnboundedSize;

    let compressors = [
        (BLOSC_BLOSCLZ, "blosclz"),
        (BLOSC_LZ4, "lz4"),
        (BLOSC_SNAPPY, "snappy"),
        (BLOSC_ZLIB, "zlib"),
        (BLOSC_ZSTD, "zstd"),
    ];

    for (compcode, compname) in compressors {
        for shuffle in [BLOSC_NOSHUFFLE, BLOSC_SHUFFLE, BLOSC_BITSHUFFLE] {
            let mut cparams = BLOSC2_CPARAMS_DEFAULTS;
            cparams.compcode = compcode;
            cparams.typesize = 4;
            cparams.filters[5] = shuffle;
            cparams.clevel = 5;

            let cctx = blosc2_create_cctx(cparams);
            let mut compressed = vec![0u8; bytes.len() + BLOSC2_MAX_OVERHEAD];
            let csize = blosc2_compress_ctx(&cctx, bytes, &mut compressed);
            assert!(
                csize > 0,
                "Compression failed for {compname} with shuffle {shuffle}"
            );
            compressed.truncate(csize as usize);

            let decompressed = codec
                .decode(compressed.into(), &rep, &options)
                .unwrap_or_else(|e| {
                    panic!("Decompression failed for {compname} with shuffle {shuffle}: {e:?}")
                });

            let recovered: &[f32] = bytemuck::cast_slice(decompressed.as_ref());
            assert_eq!(
                recovered,
                original_data.as_slice(),
                "Mismatch for {compname} with shuffle {shuffle}"
            );
        }
    }
}

#[test]
fn test_ruzstd_codec_decompress() {
    let codec = RuzstdCodec::new();
    let options = CodecOptions::default();
    let rep = BytesRepresentation::UnboundedSize;

    // A valid Zstandard compressed frame for string "Hello Zstd from Octant WASM!"
    // Generated with zstd level 3
    let original = b"Hello Zstd from Octant WASM!";
    let compressed_bytes = vec![
        0x28, 0xb5, 0x2f, 0xfd, 0x24, 0x1c, 0xe1, 0x00, 0x00, 0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20,
        0x5a, 0x73, 0x74, 0x64, 0x20, 0x66, 0x72, 0x6f, 0x6d, 0x20, 0x4f, 0x63, 0x74, 0x61, 0x6e,
        0x74, 0x20, 0x57, 0x41, 0x53, 0x4d, 0x21, 0x68, 0x02, 0xa7, 0x3d,
    ];

    let decompressed = codec
        .decode(compressed_bytes.into(), &rep, &options)
        .expect("RuzstdCodec decompression should succeed");
    assert_eq!(decompressed.as_ref(), original);
}

#[test]
fn v2_unicode_null_fill_becomes_empty_string() {
    let v2 = |dtype: &str, fill: serde_json::Value| {
        let meta = serde_json::json!({"zarr_format": 2, "dtype": dtype, "fill_value": fill});
        normalize_v3_array_metadata(meta)["fill_value"].clone()
    };
    assert_eq!(v2("<U6", serde_json::Value::Null), "");
    assert_eq!(v2(">U2", serde_json::Value::Null), "");
    assert_eq!(v2("<U6", "x".into()), "x");
    // Numeric arrays keep their missing fill value.
    assert!(v2("<f4", serde_json::Value::Null).is_null());
}
