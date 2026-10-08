//! Composite hash computation and target dimensionality calculation for block loading.

use std::hash::{DefaultHasher, Hash, Hasher};

use crate::app::layers::CompositeStyle;
use crate::data::octant_block::OctantBlock;

/// Computes the 64-bit deterministic hash of a layer's active composite channels and
/// windows (`is_geotiff`: the layer's GeoTIFF bands always map to RGB).
pub fn compute_composite_hash(composite: &CompositeStyle, is_geotiff: bool) -> u64 {
    let mut hasher = DefaultHasher::new();
    composite.enabled.hash(&mut hasher);
    if composite.enabled {
        if !is_geotiff && !composite.channel_configs.is_empty() {
            for c in &composite.channel_configs {
                (c.index, c.visible, c.color_rgb).hash(&mut hasher);
                if let Some((s, e)) = c.window {
                    (s.to_bits(), e.to_bits()).hash(&mut hasher);
                }
            }
        } else {
            composite.rgb_channels.hash(&mut hasher);
        }
    }
    hasher.finish()
}

/// Computes the clamped target dimensions `(nx, ny, nz)` for GPU storage buffer allocation.
pub fn compute_target_dims(
    block: &OctantBlock,
    z_dim: usize,
    req_x: (usize, usize),
    req_y: (usize, usize),
    req_z: (usize, usize),
) -> (usize, usize, usize) {
    let target_nx = (req_x.1 + 1).saturating_sub(req_x.0).max(1);
    let target_ny = (req_y.1 + 1).saturating_sub(req_y.0).max(1);
    let full_nz = if z_dim < block.rank() {
        (req_z.1 + 1).saturating_sub(req_z.0).max(1)
    } else {
        1
    };
    let slice_elements = target_nx.saturating_mul(target_ny).max(1);
    let max_z =
        (crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS / slice_elements).clamp(1, full_nz);
    (target_nx, target_ny, full_nz.min(max_z))
}
