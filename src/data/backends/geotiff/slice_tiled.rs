//! Tiled hyperslab reading for GeoTIFF rasters.

use async_tiff::ImageFileDirectory;
use async_tiff::decoder::DecoderRegistry;
use async_tiff::reader::AsyncFileReader;
use async_tiff::tags::{PlanarConfiguration, Predictor, SampleFormat};

use crate::data::blocks::BlockStoreError;

use super::blit::{BlitSource, ReadWindow, blit_samples_to_window};
use super::decompress::decompress_chunk;
use super::predictor::unpredict_buffer;
use super::unpack::unpack_samples_to_f32;

pub async fn read_tiled_region(
    ifd: &ImageFileDirectory,
    reader: &dyn AsyncFileReader,
    decoder_registry: &DecoderRegistry,
    win: &ReadWindow,
    target_bands: &[usize],
    out_slices: &mut [&mut [f32]],
) -> Result<(), BlockStoreError> {
    let tw = ifd.tile_width().unwrap_or(ifd.image_width()) as usize;
    let th = ifd.tile_height().unwrap_or(ifd.image_height()) as usize;

    let tile_offsets = ifd.tile_offsets().ok_or("Missing tile offsets")?;
    let tile_byte_counts = ifd.tile_byte_counts().ok_or("Missing tile byte counts")?;
    let (tiles_x, tiles_y) = ifd.tile_count().unwrap_or((1, 1));

    let tx0 = win.col_start / tw;
    let tx1 = (win.col_end.saturating_sub(1)) / tw;
    let ty0 = win.row_start / th;
    let ty1 = (win.row_end.saturating_sub(1)) / th;

    let is_planar = ifd.planar_configuration() == PlanarConfiguration::Planar;
    let samples = ifd.samples_per_pixel() as usize;
    let bits = ifd.bits_per_sample().first().copied().unwrap_or(8);
    let sample_fmt = ifd
        .sample_format()
        .first()
        .copied()
        .unwrap_or(SampleFormat::Uint);
    let predictor = ifd.predictor().unwrap_or(Predictor::None);
    let is_be = ifd.endianness() == async_tiff::reader::Endianness::BigEndian;

    for ty in ty0..=ty1.min(tiles_y.saturating_sub(1)) {
        for tx in tx0..=tx1.min(tiles_x.saturating_sub(1)) {
            let t_origin_x = tx * tw;
            let t_origin_y = ty * th;

            if is_planar {
                for (i, &band) in target_bands.iter().enumerate() {
                    let tile_idx = band * (tiles_x * tiles_y) + ty * tiles_x + tx;
                    if tile_idx >= tile_offsets.len() {
                        continue;
                    }
                    let offset = tile_offsets[tile_idx];
                    let count = tile_byte_counts[tile_idx];
                    let raw_bytes = reader
                        .get_bytes(offset..offset + count)
                        .await
                        .map_err(|e| e.to_string())?;

                    let decomp = decompress_chunk(
                        &raw_bytes,
                        ifd.compression(),
                        ifd.photometric_interpretation(),
                        ifd.jpeg_tables(),
                        1,
                        bits,
                        decoder_registry,
                    )
                    .map_err(|e| e.to_string())?;

                    let unpred = unpredict_buffer(decomp, predictor, 1, bits, tw, is_be)
                        .map_err(|e| e.to_string())?;
                    let samples_vec = unpack_samples_to_f32(
                        &unpred,
                        bits,
                        sample_fmt,
                        ifd.photometric_interpretation(),
                        tw,
                        th,
                    );

                    if let Some(out_slice) = out_slices.get_mut(i) {
                        let src = BlitSource {
                            samples: &samples_vec,
                            origin_x: t_origin_x,
                            origin_y: t_origin_y,
                            width: tw,
                            height: th,
                            samples_per_pixel: 1,
                            is_planar: true,
                        };
                        blit_samples_to_window(&src, win, &[0], &mut [out_slice]);
                    }
                }
            } else {
                let tile_idx = ty * tiles_x + tx;
                if tile_idx >= tile_offsets.len() {
                    continue;
                }
                let offset = tile_offsets[tile_idx];
                let count = tile_byte_counts[tile_idx];
                let raw_bytes = reader
                    .get_bytes(offset..offset + count)
                    .await
                    .map_err(|e| e.to_string())?;

                let decomp = decompress_chunk(
                    &raw_bytes,
                    ifd.compression(),
                    ifd.photometric_interpretation(),
                    ifd.jpeg_tables(),
                    samples as u16,
                    bits,
                    decoder_registry,
                )
                .map_err(|e| e.to_string())?;

                let unpred = unpredict_buffer(decomp, predictor, samples, bits, tw, is_be)
                    .map_err(|e| e.to_string())?;
                let samples_vec = unpack_samples_to_f32(
                    &unpred,
                    bits,
                    sample_fmt,
                    ifd.photometric_interpretation(),
                    tw * samples,
                    th,
                );

                let src = BlitSource {
                    samples: &samples_vec,
                    origin_x: t_origin_x,
                    origin_y: t_origin_y,
                    width: tw,
                    height: th,
                    samples_per_pixel: samples,
                    is_planar: false,
                };
                blit_samples_to_window(&src, win, target_bands, out_slices);
            }
        }
    }
    Ok(())
}
