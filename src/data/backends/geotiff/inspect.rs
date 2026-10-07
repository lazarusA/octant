//! Inspection routines converting TIFF/GeoTIFF IFDs into Octant `DatasetMetadata`.

use std::collections::HashMap;

use async_tiff::tags::{PhotometricInterpretation, SampleFormat};
use async_tiff::{ImageFileDirectory, TIFF};

use crate::data::metadata::{CoordValues, DatasetMetadata, VariableInfo};

use super::bands::{
    GdalItem, add_gdal_attributes, band_labels, default_band_name, parse_gdal_metadata,
};
use super::coords::GeoSpatialBounds;

/// Build `DatasetMetadata` from parsed `TIFF` structure.
pub fn inspect_tiff(tiff: &TIFF, source_name: &str) -> DatasetMetadata {
    let mut variables = Vec::new();
    let mut dimension_coordinates = HashMap::new();

    for (ifd_idx, ifd) in tiff.ifds().iter().enumerate() {
        inspect_single_ifd(ifd_idx, ifd, &mut variables, &mut dimension_coordinates);
    }

    DatasetMetadata {
        name: source_name.to_string(),
        store_type: "GeoTIFF".to_string(),
        variables,
        dimension_coordinates,
    }
}

fn inspect_single_ifd(
    ifd_idx: usize,
    ifd: &ImageFileDirectory,
    variables: &mut Vec<VariableInfo>,
    dimension_coordinates: &mut HashMap<String, CoordValues>,
) {
    let prefix = if ifd_idx == 0 {
        String::new()
    } else {
        format!("overview_{ifd_idx}/")
    };
    let (width, height, samples) = (
        ifd.image_width() as u64,
        ifd.image_height() as u64,
        ifd.samples_per_pixel() as u64,
    );
    let data_type_str = extract_data_type_name(ifd);
    let gdal = ifd
        .gdal_metadata()
        .map(parse_gdal_metadata)
        .unwrap_or_default();
    let mut attributes = extract_ifd_attributes(ifd);
    add_gdal_attributes(&mut attributes, &gdal, None);
    let chunk_shape = extract_chunk_shape(ifd, width, height);
    let geo_bounds = GeoSpatialBounds::from_ifd(ifd);
    let is_palette = ifd.photometric_interpretation() == PhotometricInterpretation::RGBPalette
        && ifd.colormap().is_some();
    let num_bands = if is_palette { 3 } else { samples };
    let is_cmyk = ifd.photometric_interpretation() == PhotometricInterpretation::CMYK;

    if num_bands > 1 {
        let raster_name = format!("{prefix}raster");
        let labels = band_labels(ifd, &gdal, num_bands as usize, is_palette);
        geo_bounds.populate_dimension_coordinates(
            &raster_name,
            (width as usize, height as usize),
            labels.as_deref(),
            dimension_coordinates,
        );
        variables.push(VariableInfo {
            name: raster_name,
            data_type: if is_palette {
                "float32".into()
            } else {
                data_type_str.clone()
            },
            shape: vec![num_bands, height, width],
            dimension_names: vec!["band".into(), "y".into(), "x".into()],
            chunk_shape: vec![1, chunk_shape[0], chunk_shape[1]],
            file_size: width * height * num_bands * 4,
            units: None,
            long_name: Some(if is_cmyk {
                "CMYK raster".into()
            } else {
                "Multi-band raster".into()
            }),
            time_coverage_start: None,
            time_coverage_end: None,
            temporal_resolution: None,
            attributes: attributes.clone(),
        });
    }

    let sample_labels = band_labels(ifd, &gdal, samples as usize, false);
    add_band_variables(
        &prefix,
        samples,
        width,
        height,
        &data_type_str,
        &chunk_shape,
        sample_labels.as_deref(),
        &attributes,
        &gdal,
        &geo_bounds,
        variables,
        dimension_coordinates,
    );
}

#[allow(clippy::too_many_arguments)]
fn add_band_variables(
    prefix: &str,
    samples: u64,
    width: u64,
    height: u64,
    data_type_str: &str,
    chunk_shape: &[u64],
    labels: Option<&[String]>,
    attributes: &HashMap<String, String>,
    gdal: &[GdalItem],
    geo_bounds: &GeoSpatialBounds,
    variables: &mut Vec<VariableInfo>,
    dimension_coordinates: &mut HashMap<String, CoordValues>,
) {
    for band_idx in 0..samples {
        let var_name = if samples == 1 {
            if prefix.is_empty() {
                "band_1".into()
            } else {
                format!("{prefix}band_1")
            }
        } else {
            format!("{prefix}band_{}", band_idx + 1)
        };

        let size = (width as usize, height as usize);
        geo_bounds.populate_dimension_coordinates(&var_name, size, None, dimension_coordinates);
        let band = band_idx as usize;
        let band_long_name = labels
            .and_then(|l| l.get(band))
            .cloned()
            .unwrap_or_else(|| default_band_name(band));
        let mut band_attributes = attributes.clone();
        add_gdal_attributes(&mut band_attributes, gdal, Some(band));

        variables.push(VariableInfo {
            name: var_name,
            data_type: data_type_str.to_string(),
            shape: vec![height, width],
            dimension_names: vec!["y".into(), "x".into()],
            chunk_shape: chunk_shape.to_vec(),
            file_size: width * height * 4,
            units: None,
            long_name: Some(band_long_name),
            time_coverage_start: None,
            time_coverage_end: None,
            temporal_resolution: None,
            attributes: band_attributes,
        });
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
    attrs
}
