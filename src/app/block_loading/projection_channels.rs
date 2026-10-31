//! Multi-channel composites blended on the GPU: a heatmap
//! layer's composite uploads its block's raw channel planes once
//! (`plots::ChannelRenderer`), and its tints, windows and curves are drawn
//! from the layer's style every frame, so changing them reloads nothing.

use std::sync::Arc;

use super::projection_2d::PlaneWindow;
use crate::app::OctantApp;
use crate::app::layers::{Layer, LayerId};
use crate::data::MatrixData;
use crate::data::octant_block::OctantBlock;
use crate::data::slicing::{CompositeProbe, CompositeStretch};
use crate::plots::{ChannelRenderer, PlotType};

impl OctantApp {
    /// Whether a change to layer `id`'s channel display (ticks, tints,
    /// windows, curves) needs a reload: not when the GPU blends its planes,
    /// unless it is an overlay, whose ticks pick the channels it reads
    /// (`fit_overlay_channels`).
    pub(crate) fn channel_display_reloads(&self, id: LayerId) -> bool {
        id != LayerId::BASE || !self.layers.get(id).is_some_and(Layer::blends_on_gpu)
    }

    /// Projects `window` of `block` as layer `id`'s channel planes, when its
    /// multi-channel composite is drawn as a heatmap and they fit one
    /// GPU buffer; `false` (planes dropped) when the CPU blends it instead.
    pub(crate) fn project_channel_planes(
        &mut self,
        id: LayerId,
        block: &OctantBlock,
        c_dim: usize,
        window: &PlaneWindow,
    ) -> bool {
        let planes = self.gpu_channel_planes(id, block, c_dim, window);
        let Some(layer) = self.layers.get_mut(id) else {
            return false;
        };
        let Some((renderer, matrix, stretch)) = planes else {
            layer.renderers.channels = None;
            return false;
        };
        layer.data.flipped_dims.clone_from(&block.flipped_dims);
        let probe = CompositeProbe::new(
            block,
            c_dim,
            window.x_dim,
            window.y_dim,
            window.x_range,
            window.y_range,
            window.fixed,
        );
        layer.data.composite_probe = Some(probe);
        // Tiles stretched otherwise would show seams.
        if layer.data.composite_stretch.as_ref() != Some(&stretch) {
            layer.data.detail.discard_tile();
        }
        layer.data.composite_stretch = Some(stretch);
        self.rebuild_pipeline_with_matrix_data(id, matrix);
        if let Some(layer) = self.layers.get_mut(id) {
            layer.renderers.channels = Some(renderer);
        }
        true
    }

    /// Every channel of `window` uploaded to layer `id`'s channel renderer,
    /// with the first channel's plane (the layer's grid) and each channel's
    /// stretch; `None` when the layer's composite is blended on the CPU.
    fn gpu_channel_planes(
        &mut self,
        id: LayerId,
        block: &OctantBlock,
        c_dim: usize,
        window: &PlaneWindow,
    ) -> Option<(Arc<ChannelRenderer>, MatrixData, CompositeStretch)> {
        let heatmap = self
            .layer_selections(id)
            .is_some_and(|(_, staged)| staged.plot_type == PlotType::Heatmap);
        let style = &self.layers.get(id)?.composite;
        let multichannel = !style.channel_configs.is_empty() && !self.layer_is_geotiff(id);
        let rs = self.wgpu_render_state.as_ref()?;
        let count = block.shape.get(c_dim).copied()?;
        let (w, h) = (
            window.x_range.1.saturating_sub(window.x_range.0),
            window.y_range.1.saturating_sub(window.y_range.0),
        );
        let plane_len = w.checked_mul(h)?;
        let gpu = heatmap && multichannel && ChannelRenderer::fits(count, plane_len);
        if !gpu {
            return None;
        }
        let c_start = block.origin.get(c_dim).copied().unwrap_or(0);
        let slot = &mut self.layers.get_mut(id)?.renderers.channels;
        let renderer = ChannelRenderer::sync(slot, &rs.device, rs.target_format, count, plane_len);
        let (mut first, mut stretch) = (None, CompositeStretch::default());
        each_plane(block, c_dim, window, |local, plane| {
            stretch.channel(c_start + local, &plane.values, None, 1.0);
            renderer.write_plane(&rs.queue, local, &plane.values);
            first.get_or_insert(plane);
        })?;
        Some((renderer, first?, stretch))
    }

    /// Uploads a detail tile's channel `planes` to layer `id`'s tile channel
    /// renderer, dropping it when they are not given or do not fit.
    pub(crate) fn upload_tile_planes(&mut self, id: LayerId, planes: Option<&[MatrixData]>) {
        let rs = self.wgpu_render_state.clone();
        let Some(layer) = self.layers.get_mut(id) else {
            return;
        };
        let slot = &mut layer.renderers.detail_channels;
        let (Some(rs), Some(planes)) = (rs, planes) else {
            *slot = None;
            return;
        };
        let plane_len = planes.first().map_or(0, |p| p.values.len());
        if !ChannelRenderer::fits(planes.len(), plane_len) {
            *slot = None;
            return;
        }
        let renderer =
            ChannelRenderer::sync(slot, &rs.device, rs.target_format, planes.len(), plane_len);
        for (i, plane) in planes.iter().enumerate() {
            renderer.write_plane(&rs.queue, i, &plane.values);
        }
    }
}

/// Calls `each` with every channel plane of `window` of `block` (along
/// `c_dim`) in order, the first with its bounds; `None` when one cannot be
/// sliced or lies on a HEALPix grid, whose cells are not laid out on the
/// quad's grid.
pub(crate) fn each_plane(
    block: &OctantBlock,
    c_dim: usize,
    window: &PlaneWindow,
    mut each: impl FnMut(usize, MatrixData),
) -> Option<()> {
    let count = block.shape.get(c_dim).copied()?;
    let mut fixed = window.fixed.to_vec();
    fixed.resize(fixed.len().max(block.shape.len()), 0);
    for local in 0..count {
        fixed[c_dim] = local;
        let plane = block.slice_2d_with_ranges(
            window.x_dim,
            window.y_dim,
            window.x_range,
            window.y_range,
            &fixed,
            1,
            &block.variable_name,
            local == 0,
        )?;
        if plane.grid.is_healpix() {
            return None;
        }
        each(local, plane);
    }
    Some(())
}

impl Layer {
    /// Whether this layer draws its composite from channel planes the GPU
    /// blends.
    pub fn blends_on_gpu(&self) -> bool {
        self.composite.enabled && self.renderers.channels.is_some()
    }

    /// The packed composite color of cell `(px, py)` of this layer's plane
    /// when the GPU blends it (its matrix then holds the first channel's raw
    /// values); `None` when the matrix holds the colors.
    pub fn gpu_blended_cell(&self, px: usize, py: usize) -> Option<f32> {
        if !self.blends_on_gpu() {
            return None;
        }
        let probe = self.data.composite_probe.as_ref()?;
        let stretch = self.data.composite_stretch.as_ref()?;
        let configs = &self.composite.channel_configs;
        Some(crate::data::slicing::blend_pixel(
            probe,
            configs,
            stretch,
            (px, py),
        ))
    }
}
