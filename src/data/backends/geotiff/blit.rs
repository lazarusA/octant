//! Sample blitting and hyperslab window mapping routines.

/// Source sample buffer and geometry descriptor for blitting.
#[derive(Debug, Clone, Copy)]
pub struct BlitSource<'a> {
    pub samples: &'a [f32],
    pub origin_x: usize,
    pub origin_y: usize,
    pub width: usize,
    pub height: usize,
    pub samples_per_pixel: usize,
    pub is_planar: bool,
}

/// Sliced hyperslab window coordinate ranges.
#[derive(Debug, Clone, Copy)]
pub struct ReadWindow {
    pub row_start: usize,
    pub row_end: usize,
    pub col_start: usize,
    pub col_end: usize,
    pub nodata_val: Option<f64>,
}

/// Blit decoded samples from a tile/strip into the output hyperslab windows.
pub fn blit_samples_to_window(
    src: &BlitSource<'_>,
    win: &ReadWindow,
    target_bands: &[usize],
    out_slices: &mut [&mut [f32]],
) {
    let out_w = win.col_end.saturating_sub(win.col_start).max(1);
    let r_min = win.row_start.max(src.origin_y);
    let r_max = win.row_end.min(src.origin_y + src.height);
    let c_min = win.col_start.max(src.origin_x);
    let c_max = win.col_end.min(src.origin_x + src.width);

    for r in r_min..r_max {
        let local_r = r - src.origin_y;
        let dst_r = r - win.row_start;
        for c in c_min..c_max {
            let local_c = c - src.origin_x;
            let dst_c = c - win.col_start;
            let dst_idx = dst_r * out_w + dst_c;

            for (i, &band) in target_bands.iter().enumerate() {
                if let Some(out_slice) = out_slices.get_mut(i) {
                    let s_idx = if src.is_planar {
                        band * src.width * src.height + local_r * src.width + local_c
                    } else {
                        local_r * src.width * src.samples_per_pixel
                            + local_c * src.samples_per_pixel
                            + band
                    };
                    let val = src.samples.get(s_idx).copied().unwrap_or(f32::NAN);
                    out_slice[dst_idx] = if is_nodata(val, win.nodata_val) {
                        f32::NAN
                    } else {
                        val
                    };
                }
            }
        }
    }
}

/// Check if sample equals the specified nodata sentinel value.
pub fn is_nodata(val: f32, nodata: Option<f64>) -> bool {
    if val.is_nan() {
        return true;
    }
    if let Some(nd) = nodata {
        let nd_f32 = nd as f32;
        if nd_f32.is_nan() {
            val.is_nan()
        } else {
            (val - nd_f32).abs() < 1e-6
        }
    } else {
        false
    }
}
