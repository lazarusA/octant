//! What an IFD's tags say about its variables: data type, chunk shape and
//! attributes.

use std::collections::HashMap;

use async_tiff::ImageFileDirectory;
use async_tiff::tags::{ExtraSamples, PhotometricInterpretation, SampleFormat};

pub(super) fn extract_data_type_name(ifd: &ImageFileDirectory) -> String {
    let fmt = ifd
        .sample_format()
        .first()
        .copied()
        .unwrap_or(SampleFormat::Uint);
    let bits = ifd.bits_per_sample().first().copied().unwrap_or(8);
    match (fmt, bits) {
        (SampleFormat::Float, 16) => "float16",
        (SampleFormat::Float, 32) => "float32",
        (SampleFormat::Float, 64) => "float64",
        (SampleFormat::Uint, 1) => "bool",
        (SampleFormat::Uint, 4) => "uint4",
        (SampleFormat::Uint, 8) => "uint8",
        (SampleFormat::Uint, 12) => "uint12",
        (SampleFormat::Uint, 16) => "uint16",
        (SampleFormat::Uint, 32) => "uint32",
        (SampleFormat::Uint, 64) => "uint64",
        (SampleFormat::Int, 8) => "int8",
        (SampleFormat::Int, 16) => "int16",
        (SampleFormat::Int, 32) => "int32",
        (SampleFormat::Int, 64) => "int64",
        _ => "float32",
    }
    .to_string()
}

pub(super) fn extract_chunk_shape(ifd: &ImageFileDirectory, width: u64, height: u64) -> Vec<u64> {
    if let (Some(tw), Some(th)) = (ifd.tile_width(), ifd.tile_height()) {
        vec![th as u64, tw as u64]
    } else if let Some(rps) = ifd.rows_per_strip() {
        vec![(rps as u64).min(height), width]
    } else {
        vec![height.min(256), width.min(256)]
    }
}

pub(super) fn extract_ifd_attributes(ifd: &ImageFileDirectory) -> HashMap<String, String> {
    let mut attrs = HashMap::new();
    if let Some(nodata) = ifd.gdal_nodata() {
        attrs.insert("_FillValue".into(), nodata.into());
        attrs.insert("nodata".into(), nodata.into());
    }
    for (k, v) in [
        ("description", ifd.image_description()),
        ("software", ifd.software()),
        ("datetime", ifd.date_time()),
    ] {
        if let Some(val) = v {
            attrs.insert(k.into(), val.into());
        }
    }
    if let Some(geo) = ifd.geo_key_directory() {
        // Projected, else geographic; read by `data::crs::discover`.
        if let Some(code) = geo.epsg_code() {
            attrs.insert("proj:code".into(), format!("EPSG:{code}"));
        }
        if let Some(ref cit) = geo.citation {
            attrs.insert("crs_citation".into(), cit.clone());
        }
        if let Some(ref cit) = geo.proj_citation {
            attrs.insert("projection_citation".into(), cit.clone());
        }
    }
    match ifd.photometric_interpretation() {
        PhotometricInterpretation::CMYK => {
            attrs.insert("photometric".into(), "cmyk".into());
            attrs.insert("color_space".into(), "cmyk".into());
        }
        PhotometricInterpretation::RGB => {
            attrs.insert("photometric".into(), "rgb".into());
            attrs.insert("color_space".into(), "rgb".into());
        }
        PhotometricInterpretation::RGBPalette => {
            attrs.insert("photometric".into(), "palette".into());
        }
        _ => {}
    }
    // An RGBA raster opens in color (`composite::opens_in_color`).
    let last_extra = ifd.extra_samples().and_then(|e| e.last());
    if matches!(
        last_extra,
        Some(ExtraSamples::AssociatedAlpha | ExtraSamples::UnassociatedAlpha)
    ) {
        attrs.insert("alpha_band".into(), "last".into());
    }
    attrs
}
