//! Band names and metadata from the GDAL metadata tag, the photometric layout and extra samples.

use std::collections::HashMap;

use async_tiff::ImageFileDirectory;
use async_tiff::tags::{Compression, ExtraSamples, PhotometricInterpretation};

const RGB_NAMES: [&str; 3] = ["Red", "Green", "Blue"];
const CMYK_NAMES: [&str; 4] = ["Cyan", "Magenta", "Yellow", "Black"];

/// One `<Item>` of a GDAL metadata block (TIFF tag 42112).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GdalItem {
    pub name: String,
    /// Zero-based band the item belongs to; `None` for dataset-level items.
    pub sample: Option<usize>,
    pub role: Option<String>,
    pub value: String,
}

impl GdalItem {
    fn is_description(&self) -> bool {
        self.role.as_deref() == Some("description")
    }
}

/// Reads the `<Item>` entries of a GDAL metadata block, skipping malformed ones.
pub fn parse_gdal_metadata(xml: &str) -> Vec<GdalItem> {
    let mut items = Vec::new();
    let mut rest = xml;
    while let Some(start) = rest.find("<Item") {
        let after = &rest[start + "<Item".len()..];
        let Some(tag_end) = after.find('>') else {
            break;
        };
        let open = &after[..tag_end];
        let body = &after[tag_end + 1..];
        let (raw_value, next) = if open.ends_with('/') {
            ("", body)
        } else {
            let Some(close) = body.find("</Item>") else {
                break;
            };
            (&body[..close], &body[close + "</Item>".len()..])
        };
        if let Some(name) = attr_value(open, "name") {
            items.push(GdalItem {
                name: unescape(name),
                sample: attr_value(open, "sample").and_then(|s| s.trim().parse().ok()),
                role: attr_value(open, "role").map(unescape),
                value: unescape(raw_value.trim()),
            });
        }
        rest = next;
    }
    items
}

/// Names for the `count` bands of `ifd`, or `None` when no band has one. `expanded_palette`
/// names the three bands a color palette expands into.
pub fn band_labels(
    ifd: &ImageFileDirectory,
    items: &[GdalItem],
    count: usize,
    expanded_palette: bool,
) -> Option<Vec<String>> {
    let layout = layout_names(
        ifd.photometric_interpretation(),
        ifd.compression(),
        count,
        expanded_palette,
    );
    // Descriptions and extra samples describe the stored index sample, not the bands a
    // palette expands into.
    let (items, extra) = match ifd.extra_samples() {
        _ if expanded_palette => (&[][..], &[][..]),
        Some(extra) => (items, extra),
        None => (items, &[][..]),
    };
    resolve_labels(items, layout, extra, count)
}

/// The channel names the photometric layout gives `count` bands: RGB for RGB rasters,
/// expanded palettes and JPEG-compressed YCbCr (decoded to RGB), CMYK inks for CMYK.
pub fn layout_names(
    photometric: PhotometricInterpretation,
    compression: Compression,
    count: usize,
    expanded_palette: bool,
) -> &'static [&'static str] {
    let jpeg = matches!(compression, Compression::JPEG | Compression::ModernJPEG);
    match photometric {
        PhotometricInterpretation::RGBPalette if expanded_palette => &RGB_NAMES,
        PhotometricInterpretation::RGB if count >= RGB_NAMES.len() => &RGB_NAMES,
        PhotometricInterpretation::YCbCr if jpeg && count >= RGB_NAMES.len() => &RGB_NAMES,
        PhotometricInterpretation::CMYK if count >= CMYK_NAMES.len() => &CMYK_NAMES,
        _ => &[],
    }
}

/// Per band, the first name found: GDAL description, photometric layout, then alpha.
pub fn resolve_labels(
    items: &[GdalItem],
    layout: &[&str],
    extra: &[ExtraSamples],
    count: usize,
) -> Option<Vec<String>> {
    let extra_start = count.saturating_sub(extra.len());
    let mut named = false;
    let labels = (0..count)
        .map(|i| {
            let alpha = i
                .checked_sub(extra_start)
                .and_then(|k| extra.get(k))
                .is_some_and(|e| {
                    matches!(
                        e,
                        ExtraSamples::AssociatedAlpha | ExtraSamples::UnassociatedAlpha
                    )
                });
            let name = description(items, i)
                .or_else(|| layout.get(i).copied())
                .or_else(|| alpha.then_some("Alpha"));
            named |= name.is_some();
            name.map_or_else(|| default_band_name(i), str::to_string)
        })
        .collect();
    named.then_some(labels)
}

/// The fallback name of band `i`.
pub fn default_band_name(i: usize) -> String {
    format!("Band {}", i + 1)
}

/// Adds the non-description items of band `sample` (or dataset-level items for `None`),
/// keeping attributes that are already set.
pub fn add_gdal_attributes(
    attrs: &mut HashMap<String, String>,
    items: &[GdalItem],
    sample: Option<usize>,
) {
    for item in items
        .iter()
        .filter(|it| it.sample == sample && !it.is_description())
    {
        attrs
            .entry(item.name.clone())
            .or_insert_with(|| item.value.clone());
    }
}

fn description(items: &[GdalItem], sample: usize) -> Option<&str> {
    items
        .iter()
        .find(|it| it.sample == Some(sample) && it.is_description() && !it.value.is_empty())
        .map(|it| it.value.as_str())
}

/// The quoted value of attribute `key` inside an opening tag's text.
fn attr_value<'a>(tag: &'a str, key: &str) -> Option<&'a str> {
    let mut rest = tag;
    loop {
        let eq = rest.find('=')?;
        let name = rest[..eq].trim();
        let after = rest[eq + 1..].trim_start();
        let quote = after.chars().next().filter(|c| *c == '"' || *c == '\'')?;
        let body = &after[1..];
        let end = body.find(quote)?;
        if name == key {
            return Some(&body[..end]);
        }
        rest = &body[end + 1..];
    }
}

/// Decodes XML entities; unknown ones are kept as written.
fn unescape(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        let tail = &rest[amp..];
        let decoded = tail
            .find(';')
            .filter(|&semi| semi <= 10)
            .and_then(|semi| decode_entity(&tail[1..semi]).map(|c| (c, semi)));
        match decoded {
            Some((c, semi)) => {
                out.push(c);
                rest = &tail[semi + 1..];
            }
            None => {
                out.push('&');
                rest = &tail[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

fn decode_entity(entity: &str) -> Option<char> {
    match entity {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        _ => {
            let num = entity.strip_prefix('#')?;
            let code = match num.strip_prefix(['x', 'X']) {
                Some(hex) => u32::from_str_radix(hex, 16).ok()?,
                None => num.parse().ok()?,
            };
            char::from_u32(code)
        }
    }
}
