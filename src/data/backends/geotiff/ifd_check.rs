//! Reads a TIFF's IFDs, rejecting with an error the tags `async-tiff` would
//! panic on (`ImageFileDirectory::from_tags` expects them): missing required
//! tags, photometric interpretations it does not know (e.g. SGI LogL/LogLuv),
//! non-rational resolutions and malformed GeoKey directories. Unsupported
//! TIFFs then fail gracefully, on desktop and in the browser alike.

use std::collections::{HashMap, HashSet};

use async_tiff::ImageFileDirectory;
use async_tiff::TagValue;
use async_tiff::metadata::{ImageFileDirectoryReader, MetadataFetch, TiffMetadataReader};
use async_tiff::reader::Endianness;
use async_tiff::tags::{PhotometricInterpretation, Tag};

/// GeoKey ids `async-tiff` knows (its crate-private `GeoKeyTag`); it skips the
/// others without reading their values. `ifd_parity_tests` checks the list
/// against the crate.
pub(super) const KNOWN_GEO_KEYS: [u16; 45] = [
    1024, 1025, 1026, 2048, 2049, 2050, 2051, 2052, 2053, 2054, 2055, 2056, 2057, 2058, 2059, 2060,
    2061, 3072, 3073, 3074, 3075, 3076, 3077, 3078, 3079, 3080, 3081, 3082, 3083, 3084, 3085, 3086,
    3087, 3088, 3089, 3090, 3091, 3092, 3093, 3094, 3095, 4096, 4097, 4098, 4099,
];

/// Every IFD of the file, each checked before `ImageFileDirectory::from_tags`.
pub(crate) async fn read_checked_ifds<F: MetadataFetch>(
    meta: &TiffMetadataReader,
    fetch: &F,
) -> Result<Vec<ImageFileDirectory>, String> {
    let (bigtiff, endianness) = (meta.bigtiff(), meta.endianness());
    let mut ifds = Vec::new();
    let mut visited = HashSet::new();
    let mut next = meta.next_ifd_offset();
    while let Some(offset) = next {
        if !visited.insert(offset) {
            return Err(format!("IFD chain loops back to offset {offset}"));
        }
        let reader = ImageFileDirectoryReader::open(fetch, offset, bigtiff, endianness)
            .await
            .map_err(|e| e.to_string())?;
        let count = tag_count(fetch, offset, bigtiff, endianness).await?;
        let mut tags = HashMap::with_capacity(count as usize);
        for idx in 0..count {
            let (tag, value) = reader
                .read_tag(fetch, idx)
                .await
                .map_err(|e| e.to_string())?;
            tags.insert(tag, value);
        }
        check_tags(&tags).map_err(|e| format!("IFD {}: {e}", ifds.len()))?;
        ifds.push(ImageFileDirectory::from_tags(tags, endianness).map_err(|e| e.to_string())?);
        next = reader.finish(fetch).await.map_err(|e| e.to_string())?;
    }
    Ok(ifds)
}

/// Number of tags in the IFD at `offset` (its first 2, or 8 for BigTIFF, bytes).
async fn tag_count<F: MetadataFetch>(
    fetch: &F,
    offset: u64,
    bigtiff: bool,
    endianness: Endianness,
) -> Result<u64, String> {
    let width = if bigtiff { 8 } else { 2 };
    let bytes = fetch
        .fetch(offset..offset + width)
        .await
        .map_err(|e| e.to_string())?;
    let bytes = bytes.as_ref();
    let little = matches!(endianness, Endianness::LittleEndian);
    let count = match (bigtiff, bytes) {
        (true, &[a, b, c, d, e, f, g, h]) => {
            let raw = [a, b, c, d, e, f, g, h];
            if little {
                u64::from_le_bytes(raw)
            } else {
                u64::from_be_bytes(raw)
            }
        }
        (false, &[a, b]) => u64::from(if little {
            u16::from_le_bytes([a, b])
        } else {
            u16::from_be_bytes([a, b])
        }),
        _ => return Err(format!("truncated IFD at offset {offset}")),
    };
    Ok(count)
}

/// Rejects the tags `ImageFileDirectory::from_tags` would panic on.
pub(crate) fn check_tags(tags: &HashMap<Tag, TagValue>) -> Result<(), String> {
    for (tag, name) in [
        (Tag::ImageWidth, "ImageWidth"),
        (Tag::ImageLength, "ImageLength"),
        (Tag::BitsPerSample, "BitsPerSample"),
        (Tag::SamplesPerPixel, "SamplesPerPixel"),
        (Tag::PhotometricInterpretation, "PhotometricInterpretation"),
    ] {
        if !tags.contains_key(&tag) {
            return Err(format!("unsupported TIFF: missing required tag {name}"));
        }
    }
    if let Some(Ok(code)) = tags
        .get(&Tag::PhotometricInterpretation)
        .map(|v| v.clone().into_u16())
        && PhotometricInterpretation::from_u16(code).is_none()
    {
        return Err(format!(
            "unsupported TIFF: {} photometric interpretation",
            photometric_name(code)
        ));
    }
    for (tag, name) in [
        (Tag::XResolution, "XResolution"),
        (Tag::YResolution, "YResolution"),
    ] {
        if tags
            .get(&tag)
            .is_some_and(|v| !matches!(v, TagValue::Rational(..)))
        {
            return Err(format!("unsupported TIFF: {name} is not a rational"));
        }
    }
    check_geo_keys(tags)
}

/// Human name of a photometric interpretation `async-tiff` cannot read.
fn photometric_name(code: u16) -> String {
    match code {
        9 => "ICC Lab".into(),
        10 => "ITU Lab".into(),
        32803 => "CFA (raw sensor)".into(),
        32844 => "SGI LogL (HDR)".into(),
        32845 => "SGI LogLuv (HDR)".into(),
        34892 => "LinearRaw".into(),
        other => format!("code {other}"),
    }
}

/// The GeoKey directory: version 1.1 header, one 4-value entry per key, and
/// every known key's ASCII or double value inside its params tag.
fn check_geo_keys(tags: &HashMap<Tag, TagValue>) -> Result<(), String> {
    let Some(Ok(dir)) = tags
        .get(&Tag::GeoKeyDirectory)
        .map(|v| v.clone().into_u16_vec())
    else {
        return Ok(());
    };
    let bad = |what: &str| Err(format!("unsupported GeoTIFF: {what}"));
    let [version, revision, _, keys, ..] = dir[..] else {
        return bad("GeoKey directory without a header");
    };
    if (version, revision) != (1, 1) {
        return bad("GeoKey directory version is not 1.1");
    }
    if dir.len() < 4 * (usize::from(keys) + 1) {
        return bad("GeoKey directory shorter than its key count");
    }
    let ascii = tags
        .get(&Tag::GeoAsciiParams)
        .and_then(|v| v.clone().into_string().ok());
    let doubles = tags
        .get(&Tag::GeoDoubleParams)
        .and_then(|v| v.clone().into_f64_vec().ok());
    for &[id, location, count, offset] in dir[4..].as_chunks::<4>().0.iter().take(usize::from(keys))
    {
        if !KNOWN_GEO_KEYS.contains(&id) {
            continue;
        }
        let range = usize::from(offset)..usize::from(offset) + usize::from(count);
        let fits = match Tag::from_u16_exhaustive(location) {
            Tag::GeoAsciiParams => ascii.as_deref().is_some_and(|s| s.get(range).is_some()),
            Tag::GeoDoubleParams => doubles.as_deref().is_some_and(|d| {
                // One value is indexed, several are sliced.
                if count == 1 {
                    d.get(range.start).is_some()
                } else {
                    d.get(range).is_some()
                }
            }),
            _ => true,
        };
        if !fits {
            return bad("GeoKey value outside its params tag");
        }
    }
    Ok(())
}
