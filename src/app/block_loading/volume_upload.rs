//! Deferred GPU uploads of the CPU volume: blocks mark the Z planes they
//! changed, and once per frame only the renderer the active plot shows
//! receives them (several blocks in a frame upload once). The other renderer
//! keeps its pending planes until it is shown.

use crate::app::OctantApp;
use std::ops::Range;

fn union(pending: Option<Range<usize>>, z: Range<usize>) -> Option<Range<usize>> {
    Some(match pending {
        Some(p) => p.start.min(z.start)..p.end.max(z.end),
        None => z,
    })
}

impl OctantApp {
    /// Marks Z planes `z` of `volume_data` as changed for both 3D renderers.
    pub(crate) fn mark_volume_dirty(&mut self, z: Range<usize>) {
        if z.is_empty() {
            return;
        }
        self.volume_dirty = union(self.volume_dirty.take(), z.clone());
        self.point_cloud_dirty = union(self.point_cloud_dirty.take(), z);
    }

    /// Uploads the planes changed since the last upload to the renderer the
    /// active plot shows. Call once per frame before painting.
    pub fn flush_volume_uploads(&mut self) {
        let (Some(render_state), Some(vdata)) = (&self.wgpu_render_state, &self.volume_data) else {
            return;
        };
        let queue = &render_state.queue;
        match self.active_plot_type {
            crate::plots::PlotType::Volume => {
                if let (Some(z), Some(r)) = (self.volume_dirty.take(), &self.volume_renderer) {
                    r.update_planes(queue, &vdata.values, z);
                }
            }
            crate::plots::PlotType::PointCloud => {
                if let (Some(z), Some(r)) =
                    (self.point_cloud_dirty.take(), &self.point_cloud_renderer)
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
}
