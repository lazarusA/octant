//! GPU render pipeline initialization, caching, and buffer allocation: 3D
//! volume renderers and coastline LODs here, 2D renderers in `rebuild_2d`.

use crate::app::OctantApp;
use crate::app::layers::{Layer, LayerId};
use crate::data::VolumeData;
use crate::plots::common::{MAX_GPU_STORAGE_BUFFER_BYTES, MAX_GPU_STORAGE_BUFFER_ELEMENTS};
use crate::plots::{PointCloudRenderer, VolumeRenderer};
use std::sync::Arc;

/// Starts the color range at the data's `range` when the layer shows a new
/// variable (`dims` "2d" or "3d"; `clear` drops the other kind's data), else
/// follows the data's extent; then applies a greyscale image's display at
/// `step`. Returns whether the variable is new.
pub(super) fn follow_variable(
    layer: &mut Layer,
    dims: &str,
    clear: fn(&mut Layer),
    step: usize,
    range: (f32, f32),
) -> bool {
    let sel = layer.selection();
    let var_key = format!("{}:{}:{dims}", sel.store_target, sel.variable_idx);
    let is_new = layer.data.var_key.as_ref() != Some(&var_key);
    if is_new {
        clear(layer);
        layer.data.var_key = Some(var_key);
        layer.color.reset_to_extent(range.0, range.1);
        layer.data.greyscale_channel = None;
    } else {
        layer.color.follow_extent(range.0, range.1);
    }
    super::greyscale::apply_greyscale_display(layer, step, range);
    is_new
}

impl OctantApp {
    /// Rebuilds or updates layer `id`'s GPU buffers for 3D volume data.
    pub fn rebuild_pipeline_with_volume_data(&mut self, id: LayerId, data: VolumeData) {
        if data.values.len() > MAX_GPU_STORAGE_BUFFER_ELEMENTS {
            use crate::i18n::{NoticesKey, filled, tr};
            let len = data.values.len();
            let detail = filled(
                tr(NoticesKey::VolumeBufferTooLargeDetail),
                &[&len, &(len * 4), &MAX_GPU_STORAGE_BUFFER_BYTES],
            );
            let title = tr(NoticesKey::VolumeTooLargeForGpu);
            self.notify(crate::ui::toast::Severity::Error, title, detail);
            return;
        }
        let step = self.layer_step(id);
        let Some(layer) = self.layers.get_mut(id) else {
            return;
        };

        let mut upload_later = false;
        if let Some(rs) = &self.wgpu_render_state {
            let encoding = layer.volume_encoding();
            let renderers = &mut layer.renderers;
            let same_dimensions = layer.data.volume.as_ref().is_some_and(|v| {
                v.width == data.width && v.height == data.height && v.depth == data.depth
            }) && renderers
                .volume
                .as_ref()
                .is_some_and(|r| r.encoding() == encoding);

            if same_dimensions && renderers.point_cloud.is_some() {
                // Uploaded before the next paint, to the renderer on screen.
                upload_later = true;
            } else {
                let (w, h) = (data.width as u32, data.height as u32);
                let (device, format) = (&rs.device, rs.target_format);
                let volume =
                    VolumeRenderer::new(device, &rs.queue, format, &data.values, w, h, encoding);
                let point_cloud = PointCloudRenderer::new(device, format, &data.values, w, h);
                renderers.volume = volume.map(Arc::new);
                renderers.point_cloud = Some(Arc::new(point_cloud));
                // New renderers start from `data`: nothing is pending.
                renderers.volume_dirty = None;
                renderers.point_cloud_dirty = None;
            }
        }
        follow_variable(
            layer,
            "3d",
            Layer::clear_2d,
            step,
            (data.min_val, data.max_val),
        );

        let depth = data.depth;
        layer.data.volume = Some(data);
        layer.data.touch_volume();
        if upload_later {
            layer.renderers.mark_volume_dirty(0..depth);
        }
    }

    /// Dispatches an asynchronous fetch for the requested coastline LOD
    /// and swaps vertices into GPU buffers when ready without blocking UI frames.
    pub fn reload_coastline_lod(&mut self, lod: crate::plots::CoastlineLod) {
        if lod == self.coastline_current_lod || self.coastline_is_loading {
            return;
        }
        let (tx, rx) = std::sync::mpsc::channel();
        self.coastline_rx = Some(rx);
        self.coastline_is_loading = true;
        crate::plots::fetch_coastline_async(lod, tx);
        log::info!("Coastline LOD fetch requested for {:?}", lod);
    }
}
