//! Multi-channel composites blended on the GPU in 3D: a point cloud layer's
//! composite keeps its block's raw channels as one volume per channel
//! (`ChannelVolume`, stacked along Z), uploaded with the volume's planes,
//! and its tints, windows and curves are drawn from the layer's style every
//! frame, so changing them reloads nothing.

use crate::app::OctantApp;
use crate::app::layers::{ChannelVolume, Layer, LayerId};
use crate::data::VolumeData;
use crate::data::octant_block::OctantBlock;
use crate::data::slicing::CompositeStretch;
use crate::plots::common::MAX_GPU_STORAGE_BUFFER_BYTES;
use crate::plots::{ChannelRenderer, MAX_CHANNELS, PlotType};

/// The ranges of a slab within a block, in its oriented order.
pub(super) type SlabRanges = ((usize, usize), (usize, usize), (usize, usize));

impl OctantApp {
    /// How many channels of `block` (along `c_dim`, not the volume's Z)
    /// the GPU blends for layer `id`'s volume of `cells` voxels: a
    /// multi-channel composite (not GeoTIFF) drawn as a point cloud, whose
    /// channels fit one storage buffer; `None` when the CPU blends it.
    pub(super) fn gpu_channel_count(
        &self,
        id: LayerId,
        block: &OctantBlock,
        (c_dim, z_dim): (usize, usize),
        cells: usize,
    ) -> Option<usize> {
        let points = self
            .layer_selections(id)
            .is_some_and(|(_, staged)| staged.plot_type == PlotType::PointCloud);
        let style = &self.layers.get(id)?.composite;
        let multichannel = style.enabled && !style.channel_configs.is_empty();
        if !points || !multichannel || self.layer_is_geotiff(id) || c_dim == z_dim {
            return None;
        }
        self.wgpu_render_state.as_ref()?;
        let count = block.shape.get(c_dim).copied()?;
        let bytes = count.checked_mul(cells)?.checked_mul(4)?;
        ((1..=MAX_CHANNELS).contains(&count) && bytes <= MAX_GPU_STORAGE_BUFFER_BYTES)
            .then_some(count)
    }

    /// Keeps layer `id`'s channel volume and planes for `count` channels
    /// from `c_start` of its allocated volume, empty again (stretch
    /// restarted) when `reset` (the volume was reallocated) or their shape
    /// changed; drops both when `count` is `None`.
    pub(super) fn sync_channel_volume(
        &mut self,
        id: LayerId,
        count: Option<usize>,
        c_start: usize,
        reset: bool,
    ) {
        let rs = self.wgpu_render_state.clone();
        let Some(layer) = self.layers.get_mut(id) else {
            return;
        };
        let (Some(count), Some(rs), Some(v)) = (count, rs, &layer.data.volume) else {
            layer.data.channel_volume = None;
            layer.renderers.channels = None;
            return;
        };
        let (w, h, d) = (v.width, v.height, v.depth);
        let fits = layer.data.channel_volume.as_ref().is_some_and(|cv| {
            (cv.c_start, cv.channels) == (c_start, count)
                && (cv.volume.width, cv.volume.height, cv.volume.depth) == (w, h, d * count)
        });
        if reset || !fits {
            let stacked = vec![f32::NAN; w * h * d * count];
            layer.data.channel_volume = Some(ChannelVolume {
                volume: VolumeData::new(w, h, d * count, stacked, 0.0, 0.0, String::new()),
                c_start,
                channels: count,
            });
            layer.data.composite_stretch = Some(CompositeStretch::default());
            // Nothing written yet: the next upload sends every plane.
            layer.renderers.mark_volume_dirty(0..d);
        }
        let slot = &mut layer.renderers.channels;
        ChannelRenderer::sync(slot, &rs.device, rs.target_format, count, w * h * d);
    }

    /// Every channel of `block` over `ranges` as a slab of its own, each
    /// channel's stretch recorded the first time it is seen; `None` when the
    /// GPU does not blend layer `id` or a channel cannot be sliced.
    pub(super) fn channel_slabs(
        &mut self,
        id: LayerId,
        block: &OctantBlock,
        (x_dim, y_dim, z_dim, c_dim): (usize, usize, usize, usize),
        (x, y, z): SlabRanges,
        fixed_indices: &[usize],
    ) -> Option<Vec<VolumeData>> {
        let layer = self.layers.get_mut(id)?;
        let c_start = layer.data.channel_volume.as_ref()?.c_start;
        let stretch = layer.data.composite_stretch.get_or_insert_default();
        let mut fixed = fixed_indices.to_vec();
        fixed.resize(fixed.len().max(block.rank()), 0);
        let count = block.shape.get(c_dim).copied()?;
        (0..count)
            .map(|c| {
                fixed[c_dim] = c;
                let slab =
                    block.volume_with_ranges(x_dim, y_dim, z_dim, x, y, z, &fixed, "", false)?;
                stretch.channel(c_start + c, &slab.values, None, 1.0);
                Some(slab)
            })
            .collect()
    }
}

impl Layer {
    /// The packed composite color of voxel `idx` of this layer's volume
    /// when the GPU blends its channel volumes; `None` when the volume holds
    /// the colors.
    pub fn gpu_blended_voxel(&self, idx: usize) -> Option<f32> {
        let cv = self
            .data
            .channel_volume
            .as_ref()
            .filter(|_| self.blends_on_gpu())?;
        let stretch = self.data.composite_stretch.as_ref()?;
        let plane = cv.volume.values.len() / cv.channels.max(1);
        Some(crate::data::slicing::blend_samples(
            &self.composite.channel_configs,
            stretch,
            (cv.c_start, cv.channels),
            |c| {
                let local = c.checked_sub(cv.c_start)?;
                cv.volume.values.get(local * plane + idx).copied()
            },
        ))
    }
}
