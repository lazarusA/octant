//! Inspection routines converting TIFF/GeoTIFF IFDs into Octant `DatasetMetadata`.

use std::collections::HashMap;

use async_tiff::tags::{PhotometricInterpretation, SampleFormat};
use async_tiff::{ImageFileDirectory, TIFF};

use crate::data::metadata::{DatasetMetadata, VariableInfo};

use super::coords::GeoSpatialBounds;

/// Build `DatasetMetadata` from parsed `TIFF` structure.
pub fn inspect_tiff(tiff: &TIFF, source_name: &str) -> DatasetMetadata {
    let mut variables = Vec::new();
    let mut dimension_coordinates = HashMap::new();

    for (ifd_idx, ifd) in tiff.ifds().iter().enumerate() {
        let prefix = if ifd_idx == 0 {
            String::new()
        } else {
            format!("overview_{ifd_idx}/")
        };

        let width = ifd.image_width() as u64;
        let height = ifd.image_height() as u64;
        let samples = ifd.samples_per_pixel() as u64;
        let data_type_str = extract_data_type_name(ifd);
        let attributes = extract_ifd_attributes(ifd);
        let chunk_shape = extract_chunk_shape(ifd, width, height);

        let geo_bounds = GeoSpatialBounds::from_ifd(ifd);
        let photometric = ifd.photometric_interpretation();
        let is_palette =
            photometric == PhotometricInterpretation::RGBPalette && ifd.colormap().is_some();

        let num_bands = if is_palette { 3 } else { samples };

        let mut chunk_3d = Vec::with_capacity(3);
        chunk_3d.push(1);
        chunk_3d.extend_from_slice(&chunk_shape);

        let is_cmyk = photometric == PhotometricInterpretation::CMYK;

        // Multi-band raster dataset if 2+ bands
        if num_bands > 1 {
            let raster_name = format!("{prefix}raster");
            geo_bounds.populate_dimension_coordinates(&raster_name, &mut dimension_coordinates);
            variables.push(VariableInfo {
                name: raster_name,
                data_type: if is_palette {
                    "float32".to_string()
                } else {
                    data_type_str.clone()
                },
                shape: vec![num_bands, height, width],
                dimension_names: vec!["band".to_string(), "y".to_string(), "x".to_string()],
                chunk_shape: chunk_3d,
                file_size: width
                    .saturating_mul(height)
                    .saturating_mul(num_bands)
                    .saturating_mul(4),
                units: None,
                long_name: Some(if is_cmyk {
                    "CMYK raster".to_string()
                } else {
                    "Multi-band raster".to_string()
                }),
                time_coverage_start: None,
                time_coverage_end: None,
                temporal_resolution: None,
                attributes: attributes.clone(),
            });
        }

        // Individual band variables (band_1, band_2, ...)
        for band_idx in 0..samples {
            let var_name = if samples == 1 {
                if prefix.is_empty() {
                    "band_1".to_string()
                } else {
                    format!("{prefix}band_1")
                }
            } else {
                format!("{prefix}band_{}", band_idx + 1)
            };

            geo_bounds.populate_dimension_coordinates(&var_name, &mut dimension_coordinates);

            let band_long_name = if is_cmyk {
                match band_idx {
                    0 => "Cyan (C)".to_string(),
                    1 => "Magenta (M)".to_string(),
                    2 => "Yellow (Y)".to_string(),
                    3 => "Black (K)".to_string(),
                    _ => format!("Band {}", band_idx + 1),
                }
            } else {
                format!("Band {}", band_idx + 1)
            };

            variables.push(VariableInfo {
                name: var_name,
                data_type: data_type_str.clone(),
                shape: vec![height, width],
                dimension_names: vec!["y".to_string(), "x".to_string()],
                chunk_shape: chunk_shape.clone(),
                file_size: width.saturating_mul(height).saturating_mul(4),
                units: None,
                long_name: Some(band_long_name),
                time_coverage_start: None,
                time_coverage_end: None,
                temporal_resolution: None,
                attributes: attributes.clone(),
            });
        }
    }

    DatasetMetadata {
        name: source_name.to_string(),
        store_type: "GeoTIFF".to_string(),
        variables,
        dimension_coordinates,
    }
}

fn extract_data_type_name(ifd: &ImageFileDirectory) -> String {
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

fn extract_chunk_shape(ifd: &ImageFileDirectory, width: u64, height: u64) -> Vec<u64> {
    if let (Some(tw), Some(th)) = (ifd.tile_width(), ifd.tile_height()) {
        vec![th as u64, tw as u64]
    } else if let Some(rps) = ifd.rows_per_strip() {
        vec![(rps as u64).min(height), width]
    } else {
        vec![height.min(256), width.min(256)]
    }
}

fn extract_ifd_attributes(ifd: &ImageFileDirectory) -> HashMap<String, String> {
    let mut attrs = HashMap::new();

    if let Some(nodata) = ifd.gdal_nodata() {
        attrs.insert("_FillValue".to_string(), nodata.to_string());
        attrs.insert("nodata".to_string(), nodata.to_string());
    }
    if let Some(desc) = ifd.image_description() {
        attrs.insert("description".to_string(), desc.to_string());
    }
    if let Some(soft) = ifd.software() {
        attrs.insert("software".to_string(), soft.to_string());
    }
    if let Some(dt) = ifd.date_time() {
        attrs.insert("datetime".to_string(), dt.to_string());
    }
    if let Some(geo) = ifd.geo_key_directory() {
        if let Some(ref cit) = geo.citation {
            attrs.insert("crs_citation".to_string(), cit.clone());
        }
        if let Some(ref proj_cit) = geo.proj_citation {
            attrs.insert("projection_citation".to_string(), proj_cit.clone());
        }
    }
    match ifd.photometric_interpretation() {
        PhotometricInterpretation::CMYK => {
            attrs.insert("photometric".to_string(), "cmyk".to_string());
            attrs.insert("color_space".to_string(), "cmyk".to_string());
        }
        PhotometricInterpretation::RGB => {
            attrs.insert("photometric".to_string(), "rgb".to_string());
            attrs.insert("color_space".to_string(), "rgb".to_string());
        }
        PhotometricInterpretation::RGBPalette => {
            attrs.insert("photometric".to_string(), "palette".to_string());
        }
        _ => {}
    }

    attrs
}
