//! Striped hyperslab reading for GeoTIFF rasters.

use async_tiff::ImageFileDirectory;
use async_tiff::decoder::DecoderRegistry;
use async_tiff::reader::AsyncFileReader;
use async_tiff::tags::{PlanarConfiguration, Predictor, SampleFormat};

use crate::data::blocks::BlockStoreError;

use super::blit::{BlitSource, ReadWindow, blit_samples_to_window};
use super::decompress::decompress_chunk;
use super::predictor::unpredict_buffer;
use super::unpack::unpack_samples_to_f32;

pub async fn read_striped_region(
    ifd: &ImageFileDirectory,
    reader: &dyn AsyncFileReader,
    decoder_registry: &DecoderRegistry,
    win: &ReadWindow,
    target_bands: &[usize],
    out_slices: &mut [&mut [f32]],
) -> Result<(), BlockStoreError> {
    let rps = ifd.rows_per_strip().unwrap_or(ifd.image_height()) as usize;
    let img_w = ifd.image_width() as usize;
    let img_h = ifd.image_height() as usize;

    let strip_offsets = ifd.strip_offsets().ok_or("Missing strip offsets")?;
    let strip_byte_counts = ifd.strip_byte_counts().ok_or("Missing strip byte counts")?;

    let is_planar = ifd.planar_configuration() == PlanarConfiguration::Planar;
    let samples = ifd.samples_per_pixel() as usize;
    let strips_per_band = img_h.div_ceil(rps);

    let strip_0 = win.row_start / rps;
    let strip_1 = (win.row_end.saturating_sub(1)) / rps;

    let bits = ifd.bits_per_sample().first().copied().unwrap_or(8);
    let sample_fmt = ifd
        .sample_format()
        .first()
        .copied()
        .unwrap_or(SampleFormat::Uint);
    let predictor = ifd.predictor().unwrap_or(Predictor::None);
    let is_be = ifd.endianness() == async_tiff::reader::Endianness::BigEndian;

    for s_idx in strip_0..=strip_1.min(strips_per_band.saturating_sub(1)) {
        let strip_rows = rps.min(img_h.saturating_sub(s_idx * rps));
        let s_origin_y = s_idx * rps;

        if is_planar {
            for (i, &band) in target_bands.iter().enumerate() {
                let global_strip_idx = band * strips_per_band + s_idx;
                if global_strip_idx >= strip_offsets.len() {
                    continue;
                }
                let offset = strip_offsets[global_strip_idx];
                let count = strip_byte_counts[global_strip_idx];
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

                let unpred = unpredict_buffer(decomp, predictor, 1, bits, img_w, is_be)
                    .map_err(|e| e.to_string())?;
                let samples_vec = unpack_samples_to_f32(
                    &unpred,
                    bits,
                    sample_fmt,
                    ifd.photometric_interpretation(),
                    img_w,
                    strip_rows,
                );

                if let Some(out_slice) = out_slices.get_mut(i) {
                    let src = BlitSource {
                        samples: &samples_vec,
                        origin_x: 0,
                        origin_y: s_origin_y,
                        width: img_w,
                        height: strip_rows,
                        samples_per_pixel: 1,
                        is_planar: true,
                    };
                    blit_samples_to_window(&src, win, &[0], &mut [out_slice]);
                }
            }
        } else {
            let global_strip_idx = s_idx;
            if global_strip_idx >= strip_offsets.len() {
                continue;
            }
            let offset = strip_offsets[global_strip_idx];
            let count = strip_byte_counts[global_strip_idx];
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

            let unpred = unpredict_buffer(decomp, predictor, samples, bits, img_w, is_be)
                .map_err(|e| e.to_string())?;
            let samples_vec = unpack_samples_to_f32(
                &unpred,
                bits,
                sample_fmt,
                ifd.photometric_interpretation(),
                img_w * samples,
                strip_rows,
            );

            let src = BlitSource {
                samples: &samples_vec,
                origin_x: 0,
                origin_y: s_origin_y,
                width: img_w,
                height: strip_rows,
                samples_per_pixel: samples,
                is_planar: false,
            };
            blit_samples_to_window(&src, win, target_bands, out_slices);
        }
    }
    Ok(())
}
