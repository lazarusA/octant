//! Hyperslab block slicing coordinator for TIFF/GeoTIFF datasets.

pub mod region;
pub mod request;

use std::collections::HashMap;
use std::sync::Arc;

use async_tiff::ImageFileDirectory;
use async_tiff::decoder::DecoderRegistry;
use async_tiff::reader::{AsyncFileReader, Endianness};
use async_tiff::tags::PhotometricInterpretation;

use crate::data::blocks::BlockStoreError;
use crate::data::octant_block::OctantBlock;
use crate::data::slice_request::SliceRequest;

use super::blit::ReadWindow;
use super::coords::GeoSpatialBounds;
use super::palette::apply_colormap_into;
use super::scale::BandScales;
use region::read_region;
use request::{palette_channel, parse_slice_request};

/// Fetch and decode an `OctantBlock` from an IFD (whose bounds are
/// `geo_bounds`, `GeoSpatialBounds::of_ifd_in`, and band transforms `scales`,
/// `BandScales::of_ifd_in`) for a given `SliceRequest`.
#[allow(clippy::too_many_arguments)]
pub async fn fetch_geotiff_block(
    ifd: &ImageFileDirectory,
    geo_bounds: &GeoSpatialBounds,
    scales: &BandScales,
    endianness: Endianness,
    request: &SliceRequest,
    reader: &dyn AsyncFileReader,
    decoder_registry: &DecoderRegistry,
) -> Result<OctantBlock, BlockStoreError> {
    let parsed = parse_slice_request(request, ifd);
    let out_h = parsed.row_end.saturating_sub(parsed.row_start).max(1);
    let out_w = parsed.col_end.saturating_sub(parsed.col_start).max(1);
    let plane_len = out_h
        .checked_mul(out_w)
        .ok_or_else(|| BlockStoreError::from("Hyperslab shape arithmetic overflow"))?;
    let window = ReadWindow {
        row_start: parsed.row_start,
        row_end: parsed.row_end,
        col_start: parsed.col_start,
        col_end: parsed.col_end,
        nodata_val: ifd.gdal_nodata().and_then(|s| s.trim().parse::<f64>().ok()),
    };

    let is_palette = ifd.photometric_interpretation() == PhotometricInterpretation::RGBPalette;
    let colormap = ifd.colormap();

    let (target_bands, is_single_band) = match parsed.band_idx {
        Some(b) => (vec![b], true),
        None => {
            let total = if is_palette && colormap.is_some() {
                3
            } else {
                ifd.samples_per_pixel() as usize
            };
            ((0..total).collect(), false)
        }
    };

    let total_bands = target_bands.len();
    let total_samples = total_bands
        .checked_mul(plane_len)
        .ok_or_else(|| BlockStoreError::from("Total sample buffer size overflow"))?;
    let mut buffer = vec![f32::NAN; total_samples];

    if is_palette && let Some(cmap) = colormap {
        let mut idx_buf = vec![f32::NAN; plane_len];
        read_region(
            ifd,
            endianness,
            reader,
            decoder_registry,
            &window,
            &[0],
            &mut [&mut idx_buf],
        )
        .await?;
        let bits = ifd.bits_per_sample().first().copied().unwrap_or(8);
        if is_single_band {
            let ch = palette_channel(&request.variable).unwrap_or(0);
            apply_colormap_into(&idx_buf, cmap, bits, ch, &mut buffer);
        } else {
            for ch in 0..total_bands {
                let (start, end) = (ch * plane_len, (ch + 1) * plane_len);
                apply_colormap_into(&idx_buf, cmap, bits, ch, &mut buffer[start..end]);
            }
        }
    } else {
        let (mut band_slices, bands): (Vec<&mut [f32]>, Vec<usize>) = buffer
            .chunks_mut(plane_len)
            .enumerate()
            .take(total_bands)
            .map(|(i, chunk)| (chunk, target_bands[i]))
            .unzip();
        read_region(
            ifd,
            endianness,
            reader,
            decoder_registry,
            &window,
            &bands,
            &mut band_slices,
        )
        .await?;
        for (values, &band) in buffer.chunks_mut(plane_len).zip(&target_bands) {
            if let Some(s) = scales.band(band) {
                s.apply(values);
            }
        }
    }

    let (out_shape, out_dims, origin) = if is_single_band {
        (
            vec![out_h, out_w],
            vec!["y".into(), "x".into()],
            vec![parsed.row_start, parsed.col_start],
        )
    } else {
        (
            vec![total_bands, out_h, out_w],
            vec!["band".into(), "y".into(), "x".into()],
            vec![0, parsed.row_start, parsed.col_start],
        )
    };

    let mut coords = HashMap::new();
    coords.insert(
        "x".into(),
        geo_bounds.compute_x_coords(ifd.image_width() as usize, parsed.col_start, parsed.col_end),
    );
    coords.insert(
        "y".into(),
        geo_bounds.compute_y_coords(
            ifd.image_height() as usize,
            parsed.row_start,
            parsed.row_end,
        ),
    );

    let mut attributes = HashMap::new();
    if ifd.photometric_interpretation() == PhotometricInterpretation::CMYK {
        attributes.insert("photometric".into(), "cmyk".into());
        attributes.insert("color_space".into(), "cmyk".into());
    }

    Ok(OctantBlock::new(
        request.variable.clone(),
        out_shape,
        out_dims,
        origin,
        Arc::from(buffer.into_boxed_slice()),
        coords,
        attributes,
    ))
}
