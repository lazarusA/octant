//! Deferred GPU uploads of the CPU volume: blocks mark the Z planes they
//! changed, and once per frame only the renderer the active plot shows
//! receives them (several blocks in a frame upload once). The other renderer
//! keeps its pending planes until it is shown.

use crate::app::OctantApp;
use crate::app::layers::Layer;
use crate::plots::PlotType;
use std::ops::Range;

fn union(pending: Option<Range<usize>>, z: Range<usize>) -> Option<Range<usize>> {
    Some(match pending {
        Some(p) => p.start.min(z.start)..p.end.max(z.end),
        None => z,
    })
}

impl OctantApp {
    /// Marks Z planes `z` of the plotted volume as changed for both 3D renderers.
    pub(crate) fn mark_volume_dirty(&mut self, z: Range<usize>) {
        if z.is_empty() {
            return;
        }
        self.layers.base.renderers.volume_dirty =
            union(self.layers.base.renderers.volume_dirty.take(), z.clone());
        self.layers.base.renderers.point_cloud_dirty =
            union(self.layers.base.renderers.point_cloud_dirty.take(), z);
    }

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

/// Uploads `layer`'s changed planes to its renderer for `plot_type`.
fn flush_layer(layer: &mut Layer, queue: &wgpu::Queue, plot_type: PlotType) {
    let Some(vdata) = &layer.data.volume else {
        return;
    };
    let renderers = &mut layer.renderers;
    match plot_type {
        PlotType::Volume => {
            if let (Some(z), Some(r)) = (renderers.volume_dirty.take(), &renderers.volume) {
                r.update_planes(queue, &vdata.values, z);
            }
        }
        PlotType::PointCloud => {
            if let (Some(z), Some(r)) = (renderers.point_cloud_dirty.take(), &renderers.point_cloud)
            {
                let range = vdata.plane_range(z.start, z.len());
                if let Some(planes) = vdata.values.get(range.clone()) {
                    r.update_data_range(queue, range.start, planes);
                }
            }
        }
        _ => {}
    }
}
