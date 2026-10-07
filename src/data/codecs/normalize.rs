//! Codec pipeline normalization for Zarr v3 array metadata.

use serde::Deserialize;
use serde_json::Value;
use zarrs::array::DataType;
use zarrs::metadata::v3::MetadataV3;
use zarrs::metadata_ext::codec::blosc::{
    BloscCodecConfigurationNumcodecs, codec_blosc_v2_numcodecs_to_v3,
};

/// Normalizes array metadata JSON so that `zarrs` accepts it: v3 metadata with
/// non-standard or Python numcodecs codecs gets conforming codecs in a valid pipeline order,
/// and v2 unicode arrays with a null fill (as zarr-python 2 writes them) get an empty one.
pub fn normalize_v3_array_metadata(mut meta: Value) -> Value {
    fill_v2_unicode_null(&mut meta);
    let typesize = meta
        .get("data_type")
        .and_then(|d| MetadataV3::deserialize(d).ok())
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

/// zarr-python 2 and xarray write `"fill_value": null` for NumPy unicode (`<U`, `>U`)
/// arrays, which `zarrs` rejects; an empty string is the fill those writers mean.
fn fill_v2_unicode_null(meta: &mut Value) {
    let is_unicode = meta
        .get("dtype")
        .and_then(Value::as_str)
        .is_some_and(|d| d.starts_with("<U") || d.starts_with(">U"));
    if is_unicode
        && meta.get("zarr_format").and_then(Value::as_u64) == Some(2)
        && meta.get("fill_value").is_some_and(Value::is_null)
    {
        meta["fill_value"] = Value::from("");
    }
}

/// Checks if a codec name represents an array-to-bytes transformation.
#[inline]
pub fn is_array_to_bytes_codec(name: &str) -> bool {
    matches!(
        name,
        "bytes" | "vlen" | "vlen_v2" | "sharding_indexed" | "pcodec"
    )
}

/// Extracts compression level from a configuration object or defaults to 1.
fn extract_compression_level(obj: &serde_json::Map<String, Value>) -> u64 {
    obj.get("configuration")
        .and_then(|c| c.get("level"))
        .and_then(|l| l.as_u64())
        .unwrap_or(1)
}

/// Normalizes a single codec definition object into a standard Zarr v3 representation.
fn normalize_single_codec(
    obj: &serde_json::Map<String, Value>,
    name: &str,
    typesize: Option<zarrs::array::DataTypeSize>,
) -> Option<Value> {
    if name == "numcodecs.blosc" || name.ends_with(".blosc") || name == "blosc" {
        normalize_blosc_codec(obj, typesize)
    } else if name == "numcodecs.zlib" || name == "zlib" {
        let level = extract_compression_level(obj);
        Some(serde_json::json!({
            "name": "numcodecs.zlib",
            "configuration": { "level": level }
        }))
    } else if name == "numcodecs.gzip" || name == "gzip" {
        let level = extract_compression_level(obj);
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

fn normalize_blosc_codec(
    obj: &serde_json::Map<String, Value>,
    typesize: Option<zarrs::array::DataTypeSize>,
) -> Option<Value> {
    let mut new_obj = serde_json::Map::new();
    new_obj.insert("name".to_string(), serde_json::json!("blosc"));
    if let Some(config) = obj.get("configuration")
        && let Ok(blosc_numcodecs) = BloscCodecConfigurationNumcodecs::deserialize(config)
    {
        let blosc_v3 = codec_blosc_v2_numcodecs_to_v3(&blosc_numcodecs, typesize);
        if let Ok(v3_config) = serde_json::to_value(&blosc_v3) {
            new_obj.insert("configuration".to_string(), v3_config);
        }
    } else if let Some(config) = obj.get("configuration") {
        new_obj.insert("configuration".to_string(), config.clone());
    }
    Some(Value::Object(new_obj))
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
    codecs.sort_by_key(
        |c| match c.get("name").and_then(|n| n.as_str()).unwrap_or("") {
            "bitround" => 0,
            name if is_array_to_bytes_codec(name) => 1,
            "numcodecs.shuffle" | "shuffle" => 2,
            _ => 3,
        },
    );
}
