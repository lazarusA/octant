//! Codec pipeline normalization and abstraction for Zarr v3 arrays.
//!
//! Handles translation of legacy and Python numcodecs codec specifications
//! into standard Zarr v3 codec configurations (`blosc`, `numcodecs.zlib`,
//! `numcodecs.shuffle`, `zstd`, `gzip`, `crc32c`).

use serde_json::Value;
use zarrs::array::DataType;
use zarrs::metadata::v3::MetadataV3;
use zarrs::metadata_ext::codec::blosc::{
    BloscCodecConfigurationNumcodecs, codec_blosc_v2_numcodecs_to_v3,
};

/// Normalizes Zarr v3 array metadata JSON so that non-standard and Python numcodecs
/// representations conform to `zarrs` codec plugins with a valid pipeline order.
pub fn normalize_v3_array_metadata(mut meta: Value) -> Value {
    let typesize = meta
        .get("data_type")
        .and_then(|d| serde_json::from_value::<MetadataV3>(d.clone()).ok())
        .and_then(|m| DataType::from_metadata(&m).ok())
        .map(|dt| dt.size());

    if let Some(codecs) = meta.get_mut("codecs").and_then(|c| c.as_array_mut()) {
        let mut normalized_codecs = Vec::with_capacity(codecs.len() + 1);
        let mut has_array_to_bytes = false;

        for codec in codecs.iter() {
            let Some(obj) = codec.as_object() else {
                continue;
            };
            let Some(name) = obj.get("name").and_then(|n| n.as_str()) else {
                continue;
            };

            if is_array_to_bytes_codec(name) {
                has_array_to_bytes = true;
                normalized_codecs.push(codec.clone());
            } else if let Some(normalized) = normalize_single_codec(obj, name, typesize) {
                normalized_codecs.push(normalized);
            }
        }

        ensure_array_to_bytes_codec(&mut normalized_codecs, has_array_to_bytes);
        order_codec_pipeline(&mut normalized_codecs);

        *codecs = normalized_codecs;
    }

    meta
}

/// Checks if a codec name represents an array-to-bytes transformation.
#[inline]
pub fn is_array_to_bytes_codec(name: &str) -> bool {
    matches!(
        name,
        "bytes" | "vlen" | "vlen_v2" | "sharding_indexed" | "pcodec"
    )
}

/// Normalizes a single codec definition object into a standard Zarr v3 representation.
fn normalize_single_codec(
    obj: &serde_json::Map<String, Value>,
    name: &str,
    typesize: Option<zarrs::array::DataTypeSize>,
) -> Option<Value> {
    if name == "numcodecs.blosc" || name.ends_with(".blosc") || name == "blosc" {
        if let Some(config) = obj.get("configuration").cloned()
            && let Ok(blosc_numcodecs) =
                serde_json::from_value::<BloscCodecConfigurationNumcodecs>(config)
        {
            let blosc_v3 = codec_blosc_v2_numcodecs_to_v3(&blosc_numcodecs, typesize);
            let mut new_obj = serde_json::Map::new();
            new_obj.insert("name".to_string(), serde_json::json!("blosc"));
            if let Ok(v3_config) = serde_json::to_value(&blosc_v3) {
                new_obj.insert("configuration".to_string(), v3_config);
            }
            Some(Value::Object(new_obj))
        } else {
            let mut new_obj = serde_json::Map::new();
            new_obj.insert("name".to_string(), serde_json::json!("blosc"));
            if let Some(config) = obj.get("configuration") {
                new_obj.insert("configuration".to_string(), config.clone());
            }
            Some(Value::Object(new_obj))
        }
    } else if name == "numcodecs.zlib" || name == "zlib" {
        let level = obj
            .get("configuration")
            .and_then(|c| c.get("level"))
            .and_then(|l| l.as_u64())
            .unwrap_or(1);
        Some(serde_json::json!({
            "name": "numcodecs.zlib",
            "configuration": { "level": level }
        }))
    } else if name == "numcodecs.gzip" || name == "gzip" {
        let level = obj
            .get("configuration")
            .and_then(|c| c.get("level"))
            .and_then(|l| l.as_u64())
            .unwrap_or(1);
        Some(serde_json::json!({
            "name": "gzip",
            "configuration": { "level": level }
        }))
    } else if name == "numcodecs.shuffle" || name == "shuffle" {
        let fixed_typesize = typesize.and_then(|ts| match ts {
            zarrs::array::DataTypeSize::Fixed(s) => Some(s),
            _ => None,
        });
        let elementsize = obj
            .get("configuration")
            .and_then(|c| c.get("elementsize"))
            .and_then(|e| e.as_u64())
            .map(|e| e as usize)
            .or(fixed_typesize)
            .unwrap_or(1);
        Some(serde_json::json!({
            "name": "numcodecs.shuffle",
            "configuration": { "elementsize": elementsize }
        }))
    } else if name == "numcodecs.zstd" || name == "zstd" {
        let level = obj
            .get("configuration")
            .and_then(|c| c.get("level"))
            .and_then(|l| l.as_i64())
            .unwrap_or(0);
        let checksum = obj
            .get("configuration")
            .and_then(|c| c.get("checksum"))
            .and_then(|c| c.as_bool())
            .unwrap_or(false);
        Some(serde_json::json!({
            "name": "zstd",
            "configuration": { "level": level, "checksum": checksum }
        }))
    } else if name == "numcodecs.crc32c" || name == "crc32c" {
        Some(serde_json::json!({ "name": "crc32c" }))
    } else if name == "fletcher32" || name == "bitround" {
        Some(Value::Object(obj.clone()))
    } else {
        None
    }
}

/// Ensures that an array-to-bytes codec (defaulting to little-endian bytes) is present.
fn ensure_array_to_bytes_codec(codecs: &mut Vec<Value>, has_array_to_bytes: bool) {
    if !has_array_to_bytes {
        codecs.insert(
            0,
            serde_json::json!({
                "name": "bytes",
                "configuration": {
                    "endian": "little"
                }
            }),
        );
    }
}

/// Orders codecs into canonical Zarr v3 execution order:
/// 1. Array-to-Array (e.g. `bitround`)
/// 2. Array-to-Bytes (e.g. `bytes`, `sharding_indexed`, `vlen`)
/// 3. Bytes-to-Bytes filters (e.g. `numcodecs.shuffle`)
/// 4. Bytes-to-Bytes compression/checksums (`numcodecs.zlib`, `gzip`, `zstd`, `blosc`, `crc32c`)
pub fn order_codec_pipeline(codecs: &mut [Value]) {
    codecs.sort_by_key(|c| {
        let name = c.get("name").and_then(|n| n.as_str()).unwrap_or("");
        if name == "bitround" {
            0
        } else if is_array_to_bytes_codec(name) {
            1
        } else if name == "numcodecs.shuffle" || name == "shuffle" {
            2
        } else {
            3
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use zarrs::array::ArrayMetadata;

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
}
