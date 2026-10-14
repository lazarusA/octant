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
    repair_v2(&mut meta);
    let typesize = meta
        .get("data_type")
        .and_then(|d| MetadataV3::deserialize(d).ok())
        .and_then(|m| DataType::from_metadata(&m).ok())
        .map(|dt| dt.size());

    if let Some(codecs) = meta.get_mut("codecs").and_then(|c| c.as_array_mut()) {
        // Each codec with its pipeline stage (`codec_stage`).
        let mut staged = Vec::with_capacity(codecs.len() + 1);
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
                staged.push((1, codec.clone()));
            } else if let Some(normalized) = normalize_single_codec(obj, name, typesize) {
                staged.push((codec_stage(name).unwrap_or(3), normalized));
            } else {
                // Kept as written, so `zarrs` decodes it or reports it as
                // unsupported (never read as raw bytes); a codec it does not
                // name keeps its side of the array-to-bytes codec.
                let side = if has_array_to_bytes { 3 } else { 0 };
                staged.push((codec_stage(name).unwrap_or(side), codec.clone()));
            }
        }

        if !has_array_to_bytes {
            staged.push((1, default_bytes_codec()));
        }
        // Stable: codecs of one stage keep their order.
        staged.sort_by_key(|(stage, _)| *stage);
        *codecs = staged.into_iter().map(|(_, codec)| codec).collect();
    }

    meta
}

/// Repairs v2 metadata that early writers produce and `zarrs` rejects:
/// zarr-python 2 and xarray write `"fill_value": null` for NumPy unicode
/// (`<U`, `>U`) arrays, meaning an empty string; early zarr writers (OME-NGFF
/// 0.1 data among them) store a numeric fill as text (`"0"`) and no filters
/// as `[]`, where v2 wants a number and `null`. Float fills keep their
/// special text values (`"NaN"`, `"Infinity"`).
fn repair_v2(meta: &mut Value) {
    if meta.get("zarr_format").and_then(Value::as_u64) != Some(2) {
        return;
    }
    if meta
        .get("filters")
        .is_some_and(|f| f.as_array().is_some_and(Vec::is_empty))
    {
        meta["filters"] = Value::Null;
    }
    let dtype = meta.get("dtype").and_then(Value::as_str).unwrap_or("");
    let kind = dtype
        .trim_start_matches(['<', '>', '|', '='])
        .chars()
        .next();
    let fill = match (kind, meta.get("fill_value")) {
        (Some('U'), Some(Value::Null)) => Some(Value::from("")),
        (Some('i'), Some(Value::String(t))) => t.trim().parse::<i64>().ok().map(Value::from),
        (Some('u'), Some(Value::String(t))) => t.trim().parse::<u64>().ok().map(Value::from),
        (Some('f'), Some(Value::String(t))) => {
            let v = t.trim().parse::<f64>().ok().filter(|v| v.is_finite());
            v.map(Value::from)
        }
        _ => None,
    };
    if let Some(fill) = fill {
        meta["fill_value"] = fill;
    }
}

/// Checks if a codec name represents an array-to-bytes transformation (the
/// names `zarrs` registers, with their v2 and numcodecs spellings).
#[inline]
pub fn is_array_to_bytes_codec(name: &str) -> bool {
    matches!(
        name,
        "bytes"
            | "vlen"
            | "vlen_v2"
            | "zarrs.vlen"
            | "zarrs.vlen_v2"
            | "vlen-utf8"
            | "vlen-bytes"
            | "vlen-array"
            | "sharding_indexed"
            | "pcodec"
            | "numcodecs.pcodec"
            | "packbits"
            | "zfp"
            | "zarrs.zfp"
            | "numcodecs.zfpy"
            | "zarrs.optional"
    )
}

/// The pipeline stage of a codec `zarrs` names: 0 array-to-array, 1
/// array-to-bytes, 2 byte shuffles, 3 other bytes-to-bytes (compressors,
/// checksums); `None` for a codec it does not name.
fn codec_stage(name: &str) -> Option<u8> {
    match name {
        "bitround"
        | "numcodecs.bitround"
        | "transpose"
        | "numcodecs.fixedscaleoffset"
        | "zarrs.squeeze"
        | "reshape" => Some(0),
        name if is_array_to_bytes_codec(name) => Some(1),
        "numcodecs.shuffle" | "shuffle" => Some(2),
        "blosc" | "numcodecs.blosc" | "numcodecs.zlib" | "zlib" | "gzip" | "numcodecs.gzip"
        | "zstd" | "numcodecs.zstd" | "crc32c" | "numcodecs.crc32c" | "fletcher32" => Some(3),
        _ => None,
    }
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

/// The array-to-bytes codec of arrays that name none: little-endian bytes.
fn default_bytes_codec() -> Value {
    serde_json::json!({
        "name": "bytes",
        "configuration": { "endian": "little" }
    })
}

/// Orders codecs into canonical Zarr v3 execution order:
/// 1. Array-to-Array (e.g. `bitround`)
/// 2. Array-to-Bytes (e.g. `bytes`, `sharding_indexed`, `vlen`)
/// 3. Bytes-to-Bytes filters (e.g. `numcodecs.shuffle`)
/// 4. Bytes-to-Bytes compression/checksums (`numcodecs.zlib`, `gzip`, `zstd`, `blosc`, `crc32c`)
pub fn order_codec_pipeline(codecs: &mut [Value]) {
    codecs.sort_by_key(|c| {
        let name = c.get("name").and_then(|n| n.as_str()).unwrap_or("");
        codec_stage(name).unwrap_or(3)
    });
}
