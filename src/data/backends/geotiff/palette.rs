//! ColorMap palette expansion and lookup utilities.

/// Apply ColorMap palette lookup to convert index samples into RGB channel values.
pub fn apply_colormap(
    indices: &[f32],
    colormap: &[u16],
    bits_per_sample: u16,
    channel: usize, // 0 = Red, 1 = Green, 2 = Blue
) -> Vec<f32> {
    let num_colors = 1usize << (bits_per_sample as usize).min(16);
    if colormap.len() < num_colors * 3 {
        return indices.to_vec();
    }

    let channel_offset = channel * num_colors;
    let mut out = Vec::with_capacity(indices.len());

    for &idx_f32 in indices {
        if idx_f32.is_nan() {
            out.push(f32::NAN);
            continue;
        }
        let idx = (idx_f32 as usize).min(num_colors.saturating_sub(1));
        let lut_val = colormap.get(channel_offset + idx).copied().unwrap_or(0);
        // TIFF colormaps are 16-bit values (0..65535)
        let normalized = (lut_val as f32) / 65535.0 * 255.0;
        out.push(normalized);
    }

    out
}
