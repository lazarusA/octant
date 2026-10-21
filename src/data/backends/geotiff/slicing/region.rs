//! Spatial tile and strip region reading and decoding routines.

use async_tiff::ImageFileDirectory;
use async_tiff::decoder::DecoderRegistry;
use async_tiff::reader::{AsyncFileReader, Endianness};
use async_tiff::tags::{PhotometricInterpretation, PlanarConfiguration, Predictor, SampleFormat};

use crate::data::blocks::BlockStoreError;

use super::super::blit::{ReadWindow, blit_chunk_to_window};
use super::super::decode::unpredict_buffer;

struct GridContext<'a> {
    reader: &'a dyn AsyncFileReader,
    decoder: &'a dyn async_tiff::decoder::Decoder,
    ifd: &'a ImageFileDirectory,
    offsets: &'a [u64],
    byte_counts: &'a [u64],
    endianness: Endianness,
    bits: u16,
    sample_fmt: SampleFormat,
    is_white_zero: bool,
    win: &'a ReadWindow,
}

/// Read a spatial hyperslab region across all tiles/strips covering the window.
pub async fn read_region(
    ifd: &ImageFileDirectory,
    endianness: Endianness,
    reader: &dyn AsyncFileReader,
    decoder_registry: &DecoderRegistry,
    win: &ReadWindow,
    target_bands: &[usize],
    out_slices: &mut [&mut [f32]],
) -> Result<(), BlockStoreError> {
    let (img_w, img_h) = (ifd.image_width() as usize, ifd.image_height() as usize);
    let is_planar = ifd.planar_configuration() == PlanarConfiguration::Planar;
    let samples = ifd.samples_per_pixel() as usize;
    let bits = ifd.bits_per_sample().first().copied().unwrap_or(8);
    let sample_fmt = ifd
        .sample_format()
        .first()
        .copied()
        .unwrap_or(SampleFormat::Uint);
    let is_white_zero = ifd.photometric_interpretation() == PhotometricInterpretation::WhiteIsZero;

    let decoder = decoder_registry
        .as_ref()
        .get(&ifd.compression())
        .ok_or_else(|| format!("Unsupported compression: {:?}", ifd.compression()))?;

    let is_tiled = ifd.tile_width().is_some();
    let tw = ifd.tile_width().map(|v| v as usize).unwrap_or(img_w).max(1);
    let th = ifd
        .tile_height()
        .map(|v| v as usize)
        .unwrap_or_else(|| ifd.rows_per_strip().unwrap_or(img_h as u32) as usize)
        .min(img_h)
        .max(1);

    let (tiles_x, tiles_y) = (img_w.div_ceil(tw), img_h.div_ceil(th));
    let (tx0, tx1) = (
        win.col_start / tw,
        ((win.col_end.saturating_sub(1)) / tw).min(tiles_x.saturating_sub(1)),
    );
    let (ty0, ty1) = (
        win.row_start / th,
        ((win.row_end.saturating_sub(1)) / th).min(tiles_y.saturating_sub(1)),
    );

    let offsets = ifd
        .tile_offsets()
        .or_else(|| ifd.strip_offsets())
        .ok_or("Missing chunk offsets")?;
    let byte_counts = ifd
        .tile_byte_counts()
        .or_else(|| ifd.strip_byte_counts())
        .ok_or("Missing chunk byte counts")?;

    let ctx = GridContext {
        reader,
        decoder: &**decoder,
        ifd,
        offsets,
        byte_counts,
        endianness,
        bits,
        sample_fmt,
        is_white_zero,
        win,
    };

    let grid = (tiles_x, tiles_y);
    for ty in ty0..=ty1 {
        let orig_y = ty * th;
        let buf_h = if is_tiled {
            th
        } else {
            th.min(img_h.saturating_sub(orig_y))
        };
        for tx in tx0..=tx1 {
            let (orig_x, buf_w) = (tx * tw, tw);
            if is_planar {
                for (i, &band) in target_bands.iter().enumerate() {
                    let idx = chunk_index(band, (tx, ty), grid).ok_or(TOO_LARGE)?;
                    if let Some(out_slice) = out_slices.get_mut(i) {
                        decode_and_blit_tile(
                            &ctx,
                            idx,
                            1,
                            true,
                            (orig_x, orig_y, buf_w, buf_h),
                            &[0],
                            &mut [out_slice],
                        )
                        .await?;
                    }
                }
            } else {
                let idx = chunk_index(0, (tx, ty), grid).ok_or(TOO_LARGE)?;
                decode_and_blit_tile(
                    &ctx,
                    idx,
                    samples,
                    false,
                    (orig_x, orig_y, buf_w, buf_h),
                    target_bands,
                    out_slices,
                )
                .await?;
            }
        }
    }
    Ok(())
}

const TOO_LARGE: &str = "TIFF chunk index too large";

/// Index of chunk `(tx, ty)` of `band` in a `(tiles_x, tiles_y)` grid per
/// band (planar files store the bands' grids one after another); `None`
/// when a corrupt header's grid overflows.
fn chunk_index(
    band: usize,
    (tx, ty): (usize, usize),
    (tiles_x, tiles_y): (usize, usize),
) -> Option<usize> {
    let in_band = ty.checked_mul(tiles_x)?.checked_add(tx)?;
    tiles_x
        .checked_mul(tiles_y)?
        .checked_mul(band)?
        .checked_add(in_band)
}

async fn decode_and_blit_tile(
    ctx: &GridContext<'_>,
    idx: usize,
    samples: usize,
    is_planar: bool,
    (orig_x, orig_y, buf_w, buf_h): (usize, usize, usize, usize),
    target_bands: &[usize],
    out_slices: &mut [&mut [f32]],
) -> Result<(), BlockStoreError> {
    let (Some(&offset), Some(&count)) = (ctx.offsets.get(idx), ctx.byte_counts.get(idx)) else {
        return Ok(());
    };
    let end = offset
        .checked_add(count)
        .ok_or("TIFF chunk byte range overflows")?;
    let raw = ctx
        .reader
        .get_bytes(offset..end)
        .await
        .map_err(|e| e.to_string())?;
    let decomp = ctx
        .decoder
        .decode_tile(
            raw,
            ctx.ifd.photometric_interpretation(),
            ctx.ifd.jpeg_tables(),
            samples as u16,
            ctx.bits,
            None,
        )
        .map_err(|e| e.to_string())?;
    let unpred = unpredict_buffer(
        decomp,
        ctx.ifd.predictor().unwrap_or(Predictor::None),
        samples,
        ctx.bits,
        buf_w,
        ctx.endianness,
    )
    .map_err(BlockStoreError::from)?;

    blit_chunk_to_window(
        &unpred,
        buf_w,
        buf_h,
        samples,
        is_planar,
        orig_x,
        orig_y,
        ctx.sample_fmt,
        ctx.bits,
        ctx.is_white_zero,
        ctx.win,
        target_bands,
        out_slices,
    );
    Ok(())
}
