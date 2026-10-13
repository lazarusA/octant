//! Label colors on the CPU, exactly as `labels.wgsl` draws them: the
//! palette uploaded to the GPU, and the color of one label for the hover.

use crate::data::ome::OmeLabelInfo;

/// `(id, RGBA packed red-first)` of every color with a 32-bit id, sorted.
pub fn palette_entries(info: &OmeLabelInfo) -> Vec<[u32; 2]> {
    info.colors
        .iter()
        .filter_map(|&(id, rgba)| Some([u32::try_from(id).ok()?, u32::from_le_bytes(rgba)]))
        .collect()
}

/// PCG hash of a label id, as `labels.wgsl` computes it.
pub fn label_hash(v: u32) -> u32 {
    let state = v.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
    let word = ((state >> ((state >> 28) + 4)) ^ state).wrapping_mul(277_803_737);
    (word >> 22) ^ word
}

/// The color `labels.wgsl` draws label `id` with (before opacity): its
/// recorded color, else one from its hash; `None` for the background (0).
pub fn label_color(info: &OmeLabelInfo, id: u32) -> Option<[u8; 4]> {
    if id == 0 {
        return None;
    }
    if let Some(rgba) = info.color_of(u64::from(id)) {
        return Some(rgba);
    }
    let hue = (label_hash(id) & 0xFFFF) as f32 / 65535.0;
    let (s, v) = (0.65f32, 0.95f32);
    let channel = |k: f32| {
        let p = (((hue + k / 6.0).fract() * 6.0 - 3.0).abs() - 1.0).clamp(0.0, 1.0);
        let c = v * (1.0 + (p - 1.0) * s);
        (c * 255.0).round().clamp(0.0, 255.0) as u8
    };
    Some([channel(5.0), channel(3.0), channel(1.0), 255])
}
