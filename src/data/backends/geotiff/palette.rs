//! ColorMap palette expansion and lookup utilities.

/// Apply ColorMap palette lookup to convert index samples into normalized RGB values in-place.
pub fn apply_colormap_into(
    indices: &[f32],
    colormap: &[u16],
    bits_per_sample: u16,
    channel: usize, // 0 = Red, 1 = Green, 2 = Blue
    out: &mut [f32],
) {
    let num_colors = 1usize << (bits_per_sample as usize).min(16);
    if colormap.len() < num_colors * 3 {
        let count = indices.len().min(out.len());
        out[..count].copy_from_slice(&indices[..count]);
        return;
    }

    let channel_offset = channel * num_colors;
    let count = indices.len().min(out.len());

    for i in 0..count {
        let idx_f32 = indices[i];
        if idx_f32.is_nan() {
            out[i] = f32::NAN;
            continue;
        }
        let idx = (idx_f32 as usize).min(num_colors.saturating_sub(1));
        let lut_val = colormap.get(channel_offset + idx).copied().unwrap_or(0);
        // TIFF colormaps are 16-bit values (0..65535)
        out[i] = (lut_val as f32) * (255.0 / 65535.0);
    }
}
