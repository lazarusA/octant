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

/// What every variable of one IFD shares.
struct IfdInfo {
    prefix: String,
    width: u64,
    height: u64,
    samples: u64,
    data_type: String,
    chunk_shape: Vec<u64>,
    gdal: Vec<GdalItem>,
    attributes: HashMap<String, String>,
    geo_bounds: GeoSpatialBounds,
}

impl IfdInfo {
    fn new(ifd_idx: usize, ifd: &ImageFileDirectory) -> Self {
        let (width, height) = (ifd.image_width() as u64, ifd.image_height() as u64);
        let gdal = ifd
            .gdal_metadata()
            .map(parse_gdal_metadata)
            .unwrap_or_default();
        let mut attributes = extract_ifd_attributes(ifd);
        add_gdal_attributes(&mut attributes, &gdal, None);
        Self {
            prefix: if ifd_idx == 0 {
                String::new()
            } else {
                format!("overview_{ifd_idx}/")
            },
            width,
            height,
            samples: ifd.samples_per_pixel() as u64,
            data_type: extract_data_type_name(ifd),
            chunk_shape: extract_chunk_shape(ifd, width, height),
            gdal,
            attributes,
            geo_bounds: GeoSpatialBounds::from_ifd(ifd),
        }
    }

    fn size(&self) -> (usize, usize) {
        (self.width as usize, self.height as usize)
    }
}

fn inspect_single_ifd(
    ifd_idx: usize,
    ifd: &ImageFileDirectory,
    variables: &mut Vec<VariableInfo>,
    dimension_coordinates: &mut HashMap<String, CoordValues>,
) {
    let info = IfdInfo::new(ifd_idx, ifd);
    add_raster_variable(ifd, &info, variables, dimension_coordinates);
    let labels = band_labels(ifd, &info.gdal, info.samples as usize, false);
    add_band_variables(&info, labels.as_deref(), variables, dimension_coordinates);
}

/// The multi-band `raster` variable of an IFD with several bands (or an expanded palette).
fn add_raster_variable(
    ifd: &ImageFileDirectory,
    info: &IfdInfo,
    variables: &mut Vec<VariableInfo>,
    dimension_coordinates: &mut HashMap<String, CoordValues>,
) {
    let photometric = ifd.photometric_interpretation();
    let is_palette =
        photometric == PhotometricInterpretation::RGBPalette && ifd.colormap().is_some();
    let num_bands = if is_palette { 3 } else { info.samples };
    if num_bands <= 1 {
        return;
    }
    let raster_name = format!("{}raster", info.prefix);
    let labels = band_labels(ifd, &info.gdal, num_bands as usize, is_palette);
    let coords = &info.geo_bounds;
    coords.populate_dimension_coordinates(
        &raster_name,
        info.size(),
        labels.as_deref(),
        dimension_coordinates,
    );
    let long_name = if photometric == PhotometricInterpretation::CMYK {
        "CMYK raster"
    } else {
        "Multi-band raster"
    };
    variables.push(VariableInfo {
        name: raster_name,
        data_type: if is_palette {
            "float32".into()
        } else {
            info.data_type.clone()
        },
        shape: vec![num_bands, info.height, info.width],
        dimension_names: vec!["band".into(), "y".into(), "x".into()],
        chunk_shape: vec![1, info.chunk_shape[0], info.chunk_shape[1]],
        file_size: info.width * info.height * num_bands * 4,
        units: None,
        long_name: Some(long_name.into()),
        time_coverage_start: None,
        time_coverage_end: None,
        temporal_resolution: None,
        attributes: info.attributes.clone(),
    });
}

/// One `band_N` variable per sample of the IFD.
fn add_band_variables(
    info: &IfdInfo,
    labels: Option<&[String]>,
    variables: &mut Vec<VariableInfo>,
    dimension_coordinates: &mut HashMap<String, CoordValues>,
) {
    for band in 0..info.samples as usize {
        let var_name = format!("{}band_{}", info.prefix, band + 1);
        let coords = &info.geo_bounds;
        coords.populate_dimension_coordinates(&var_name, info.size(), None, dimension_coordinates);
        let band_long_name = labels
            .and_then(|l| l.get(band))
            .cloned()
            .unwrap_or_else(|| default_band_name(band));
        let mut band_attributes = info.attributes.clone();
        add_gdal_attributes(&mut band_attributes, &info.gdal, Some(band));

        variables.push(VariableInfo {
            name: var_name,
            data_type: info.data_type.clone(),
            shape: vec![info.height, info.width],
            dimension_names: vec!["y".into(), "x".into()],
            chunk_shape: info.chunk_shape.clone(),
            file_size: info.width * info.height * 4,
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
