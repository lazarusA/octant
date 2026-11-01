//! Deferred GPU uploads of the CPU volume: blocks mark the Z planes they
//! changed, and once per frame only the renderer the active plot shows
//! receives them (several blocks in a frame upload once). The other renderer
//! keeps its pending planes until it is shown.

use crate::app::OctantApp;
use crate::app::layers::{ChannelVolume, Layer};
use crate::plots::{ChannelRenderer, PlotType};

impl OctantApp {
    /// Uploads the planes changed since the last upload to the renderer the
    /// active plot shows. Call once per frame before painting.
    pub fn flush_volume_uploads(&mut self) {
        let Some(render_state) = &self.wgpu_render_state else {
            return;
        };
        let plot_type = self.selected.plot_type;
        for layer in self.layers.iter_mut() {
            flush_layer(layer, &render_state.queue, plot_type);
        }
    }
}

/// Uploads `layer`'s changed planes to its renderer for `plot_type`: a
/// composite the GPU blends sends its channels' planes instead, which the
/// renderer bakes.
fn flush_layer(layer: &mut Layer, queue: &wgpu::Queue, plot_type: PlotType) {
    let Some(vdata) = &layer.data.volume else {
        return;
    };
    let renderers = &mut layer.renderers;
    let channels = layer
        .data
        .channel_volume
        .as_ref()
        .zip(renderers.channels.as_ref());
    match plot_type {
        PlotType::Volume => {
            if let (Some(z), Some(r)) = (renderers.volume_dirty.take(), &renderers.volume) {
                match channels {
                    Some((cv, gpu)) => upload_channels(cv, gpu, queue, vdata.depth, z),
                    None => r.update_planes(queue, &vdata.values, z),
                }
            }
        }
        PlotType::PointCloud => {
            if let (Some(z), Some(r)) = (renderers.point_cloud_dirty.take(), &renderers.point_cloud)
            {
                if let Some(g) = &layer.data.points {
                    let planes = vdata.thinned_planes(g.stride, z.clone());
                    r.update_data_range(queue, z.start * g.width * g.height, &planes);
                } else {
                    let range = vdata.plane_range(z.start, z.len());
                    if let Some(planes) = vdata.values.get(range.clone()) {
                        r.update_data_range(queue, range.start, planes);
                    }
                }
                if let Some((cv, gpu)) = channels {
                    upload_channels(cv, gpu, queue, vdata.depth, z);
                }
            }
        }
        _ => {}
    }
}

/// Uploads planes `z` of each of `cv`'s channels (`depth` planes each).
fn upload_channels(
    cv: &ChannelVolume,
    gpu: &ChannelRenderer,
    queue: &wgpu::Queue,
    depth: usize,
    z: std::ops::Range<usize>,
) {
    for c in 0..cv.channels {
        let range = cv.volume.plane_range(c * depth + z.start, z.len());
        if let Some(planes) = cv.volume.values.get(range.clone()) {
            gpu.write_cells(queue, range.start, planes);
        }
    }
}
