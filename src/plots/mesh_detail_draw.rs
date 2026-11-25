//! A multiscale sphere's finer detail tile drawn in the same pass as its
//! coarse mesh, and the per-frame upload both meshes share.

use super::channels::ChannelDraw;
use super::common::Mesh3DUniformParams;
use super::mesh::Mesh3DRenderer;
use std::sync::Arc;

/// A detail tile's mesh, drawn over the coarse one (which discards the
/// tile's cells via `hole_bounds`).
pub struct MeshDetailDraw {
    pub renderer: Arc<Mesh3DRenderer>,
    pub params: Mesh3DUniformParams,
    pub channels: Option<ChannelDraw>,
    pub thinning: super::Thinning,
    pub heights: Option<Arc<Mesh3DRenderer>>,
}

impl MeshDetailDraw {
    /// Uploads the tile's mesh; the key of what it draws, for the frame
    /// cache of the mesh it is drawn with.
    pub(super) fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
    ) -> (super::BakeId, u64) {
        let r = &self.renderer;
        let base_key = prepare_mesh(
            (r, &self.params),
            self.channels.as_ref(),
            self.thinning,
            self.heights.as_deref(),
            (device, queue, encoder),
        );
        (r.heights.id(), r.inputs.key(base_key))
    }
}

/// Uploads `renderer`'s uniforms, bakes a GPU composite into its data and
/// drapes it on `base`'s heights (an overlay's). Returns `base`'s key: a
/// draped overlay is drawn again when the base's heights change, and for a
/// new base, whose upload count starts over.
pub(super) fn prepare_mesh(
    (renderer, params): (&Mesh3DRenderer, &Mesh3DUniformParams),
    channels: Option<&ChannelDraw>,
    thinning: super::Thinning,
    base: Option<&Mesh3DRenderer>,
    (device, queue, encoder): (&wgpu::Device, &wgpu::Queue, &mut wgpu::CommandEncoder),
) -> Option<(super::BakeId, u64)> {
    renderer.update_uniforms(queue, params);
    if let Some(channels) = channels {
        let target = (renderer.bake_id.id(), &renderer.data_buffer, thinning);
        if channels.bake_into(device, queue, encoder, target) {
            renderer.inputs.data_changed();
        }
    }
    renderer
        .heights
        .drape(device, (renderer.width, renderer.height), base);
    base.map(|b| (b.heights.id(), b.inputs.key(())))
}
