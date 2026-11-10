//! SliceRequest parsing and hyperslab window calculations for TIFF datasets.

use async_tiff::ImageFileDirectory;
use async_tiff::tags::PhotometricInterpretation;

use crate::data::coordinates::naming::contains_ascii_case_insensitive;
use crate::data::slice_request::SliceRequest;

/// Parsed slice request details for TIFF extraction.
pub struct ParsedSlice {
    pub band_idx: Option<usize>,
    pub row_start: usize,
    pub row_end: usize,
    pub col_start: usize,
    pub col_end: usize,
}

/// Parse a `SliceRequest` against an `ImageFileDirectory`.
pub fn parse_slice_request(request: &SliceRequest, ifd: &ImageFileDirectory) -> ParsedSlice {
    let (img_w, img_h) = (ifd.image_width() as usize, ifd.image_height() as usize);
    let var = request.variable.trim();
    let band_idx = if contains_ascii_case_insensitive(var, "band_") {
        var.rsplit(|c: char| !c.is_ascii_digit())
            .next()
            .and_then(|s| s.parse::<usize>().ok())
            .map(|idx| idx.saturating_sub(1))
    } else if (ifd.samples_per_pixel() == 1
        && ifd.photometric_interpretation() != PhotometricInterpretation::RGBPalette)
        || !contains_ascii_case_insensitive(var, "raster")
    {
        Some(0)
    } else {
        None
    };

    // Rows and columns are the first two selections, or the last two of a multi-band raster.
    let skip = usize::from(band_idx.is_none() && request.selections.len() > 2);
    let (row_start, row_end) = request.clamped(skip, img_h);
    let (col_start, col_end) = request.clamped(skip + 1, img_w);

    ParsedSlice {
        band_idx,
        row_start,
        row_end,
        col_start,
        col_end,
    }
}

/// The bands `[start, end)` a multi-band request of `total` bands reads: its
/// first selection when it selects bands, rows and columns, else every band;
/// at least one band.
pub fn band_window(request: &SliceRequest, total: usize) -> (usize, usize) {
    match request.selections.as_slice() {
        [band, _, _] => band.clamp(total),
        _ => (0, total),
    }
}

/// Identify the RGB channel index (0 = Red, 1 = Green, 2 = Blue) from a variable name.
pub fn palette_channel(var: &str) -> Option<usize> {
    let clean = var.rsplit('/').next().unwrap_or(var).trim();
    if contains_ascii_case_insensitive(clean, "red") {
        Some(0)
    } else if contains_ascii_case_insensitive(clean, "green") {
        Some(1)
    } else if contains_ascii_case_insensitive(clean, "blue") {
        Some(2)
    } else {
        None
    }
}
