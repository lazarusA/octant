//! Hyperslab reading coordinator for GeoTIFF rasters.

use std::collections::HashMap;
use std::sync::Arc;

use async_tiff::ImageFileDirectory;
use async_tiff::decoder::DecoderRegistry;
use async_tiff::reader::AsyncFileReader;
use async_tiff::tags::PhotometricInterpretation;

use crate::data::blocks::BlockStoreError;
use crate::data::octant_block::OctantBlock;
use crate::data::slice_request::SliceRequest;

use super::blit::ReadWindow;
use super::coords::GeoSpatialBounds;
use super::palette::apply_colormap;
use super::slice_striped::read_striped_region;
use super::slice_tiled::read_tiled_region;

/// Fetch and decode an `OctantBlock` from an IFD for a given `SliceRequest`.
pub async fn fetch_geotiff_block(
    ifd: &ImageFileDirectory,
    request: &SliceRequest,
    reader: &dyn AsyncFileReader,
    decoder_registry: &DecoderRegistry,
) -> Result<OctantBlock, BlockStoreError> {
    let (band_opt, row_range, col_range) = parse_slice_request(request, ifd);
    let (row_start, row_end) = row_range;
    let (col_start, col_end) = col_range;

    let out_height = row_end.saturating_sub(row_start).max(1);
    let out_width = col_end.saturating_sub(col_start).max(1);
    let nodata_val = ifd.gdal_nodata().and_then(|s| s.trim().parse::<f64>().ok());

    let is_palette = ifd.photometric_interpretation() == PhotometricInterpretation::RGBPalette;
    let colormap = ifd.colormap();

    let (values, out_shape, out_dims) = if let Some(band_idx) = band_opt {
        // Single 2D band extraction
        let mut buffer = vec![f32::NAN; out_height * out_width];
        let window = ReadWindow {
            row_start,
            row_end,
            col_start,
            col_end,
            nodata_val,
        };
        read_region(
            ifd,
            reader,
            decoder_registry,
            &window,
            &[band_idx],
            &mut [&mut buffer],
        )
        .await?;

        if is_palette
            && let Some(cmap) = colormap
            && (request.variable.starts_with("red")
                || request.variable.starts_with("green")
                || request.variable.starts_with("blue"))
        {
            let ch = if request.variable.starts_with("red") {
                0
            } else if request.variable.starts_with("green") {
                1
            } else {
                2
            };
            let bits = ifd.bits_per_sample().first().copied().unwrap_or(8);
            buffer = apply_colormap(&buffer, cmap, bits, ch);
        }

        (
            Arc::from(buffer.into_boxed_slice()),
            vec![out_height, out_width],
            vec!["y".to_string(), "x".to_string()],
        )
    } else {
        // Multi-band 3D cube extraction
        let total_bands = if is_palette && colormap.is_some() {
            3
        } else {
            ifd.samples_per_pixel() as usize
        };
        let mut buffer = vec![f32::NAN; total_bands * out_height * out_width];
        let plane_len = out_height * out_width;

        let window = ReadWindow {
            row_start,
            row_end,
            col_start,
            col_end,
            nodata_val,
        };

        if is_palette && colormap.is_some() {
            // Read palette indices into band 0, then map across all channels
            let mut idx_buf = vec![f32::NAN; plane_len];
            read_region(
                ifd,
                reader,
                decoder_registry,
                &window,
                &[0],
                &mut [&mut idx_buf],
            )
            .await?;

            if let Some(cmap) = colormap {
                let bits = ifd.bits_per_sample().first().copied().unwrap_or(8);
                for b in 0..3 {
                    let slice_start = b * plane_len;
                    let slice_end = slice_start + plane_len;
                    let mapped = apply_colormap(&idx_buf, cmap, bits, b);
                    buffer[slice_start..slice_end].copy_from_slice(&mapped);
                }
            }
        } else {
            // Single-pass multi-band extraction for chunky or planar datasets
            let target_bands: Vec<usize> = (0..total_bands).collect();
            let mut slices: Vec<&mut [f32]> = buffer.chunks_exact_mut(plane_len).collect();
            read_region(
                ifd,
                reader,
                decoder_registry,
                &window,
                &target_bands,
                &mut slices,
            )
            .await?;
        }

        (
            Arc::from(buffer.into_boxed_slice()),
            vec![total_bands, out_height, out_width],
            vec!["band".to_string(), "y".to_string(), "x".to_string()],
        )
    };

    let geo_bounds = GeoSpatialBounds::from_ifd(ifd);
    let mut coords = HashMap::new();
    coords.insert(
        "x".to_string(),
        geo_bounds.compute_x_coords(ifd.image_width() as usize, col_start, col_end),
    );
    coords.insert(
        "y".to_string(),
        geo_bounds.compute_y_coords(ifd.image_height() as usize, row_start, row_end),
    );

    let origin = if band_opt.is_some() {
        vec![row_start, col_start]
    } else {
        vec![0, row_start, col_start]
    };

    let mut attributes = HashMap::new();
    if ifd.photometric_interpretation() == PhotometricInterpretation::CMYK {
        attributes.insert("photometric".to_string(), "cmyk".to_string());
        attributes.insert("color_space".to_string(), "cmyk".to_string());
    }

    Ok(OctantBlock::new(
        request.variable.clone(),
        out_shape,
        out_dims,
        origin,
        values,
        coords,
        attributes,
    ))
}

async fn read_region(
    ifd: &ImageFileDirectory,
    reader: &dyn AsyncFileReader,
    decoder_registry: &DecoderRegistry,
    win: &ReadWindow,
    target_bands: &[usize],
    out_slices: &mut [&mut [f32]],
) -> Result<(), BlockStoreError> {
    if ifd.tile_width().is_some() {
        read_tiled_region(ifd, reader, decoder_registry, win, target_bands, out_slices).await
    } else {
        read_striped_region(ifd, reader, decoder_registry, win, target_bands, out_slices).await
    }
}

fn parse_slice_request(
    request: &SliceRequest,
    ifd: &ImageFileDirectory,
) -> (Option<usize>, (usize, usize), (usize, usize)) {
    let img_w = ifd.image_width() as usize;
    let img_h = ifd.image_height() as usize;
    let var = request.variable.trim();

    let band_idx = if var.contains("band_") {
        var.rsplit("band_")
            .next()
            .and_then(|s| s.parse::<usize>().ok())
            .map(|idx| idx.saturating_sub(1))
    } else if (ifd.samples_per_pixel() == 1
        && ifd.photometric_interpretation() != PhotometricInterpretation::RGBPalette)
        || !var.contains("raster")
    {
        Some(0)
    } else {
        None
    };

    let (row_sel, col_sel) = if band_idx.is_some() || request.selections.len() <= 2 {
        let r = request
            .selections
            .first()
            .map(|s| s.bounds())
            .unwrap_or((0, img_h));
        let c = request
            .selections
            .get(1)
            .map(|s| s.bounds())
            .unwrap_or((0, img_w));
        (r, c)
    } else {
        let r = request
            .selections
            .get(1)
            .map(|s| s.bounds())
            .unwrap_or((0, img_h));
        let c = request
            .selections
            .get(2)
            .map(|s| s.bounds())
            .unwrap_or((0, img_w));
        (r, c)
    };

    let row_start = row_sel.0.min(img_h);
    let row_end = row_sel.1.max(row_start + 1).min(img_h);
    let col_start = col_sel.0.min(img_w);
    let col_end = col_sel.1.max(col_start + 1).min(img_w);

    (band_idx, (row_start, row_end), (col_start, col_end))
}
