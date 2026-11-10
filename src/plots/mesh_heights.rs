//! Where a sphere or surface reads its heights (bind group 2,
//! `height_buffer` in `shaders/common/mesh3d.wgsl`): its own data, or the
//! base layer's under an overlay draped on the base, so every layer of the
//! stack shares the base's relief and only its colors are its own.

use std::sync::Mutex;

use super::BakeId;
use super::mesh::Mesh3DRenderer;

pub struct HeightSlot {
    layout: wgpu::BindGroupLayout,
    /// The renderer's own data buffer as its heights.
    own: wgpu::BindGroup,
    /// Names the renderer to the overlays draped on it; never reused.
    id: BakeId,
    /// Another renderer's data buffer as the heights, with that renderer's id.
    draped: Mutex<Option<(BakeId, wgpu::BindGroup)>>,
}

impl HeightSlot {
    pub fn new(device: &wgpu::Device, data_buffer: &wgpu::Buffer) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Mesh 3D Heights Layout"),
            entries: &[super::common::bgl_entry(
                0,
                wgpu::ShaderStages::VERTEX,
                wgpu::BufferBindingType::Storage { read_only: true },
            )],
        });
        let own = bind_group(device, &layout, data_buffer);
        Self {
            layout,
            own,
            id: BakeId::next(),
            draped: Mutex::new(None),
        }
    }

    /// The renderer's id, never reused: a rebuilt base is a new one.
    pub fn id(&self) -> BakeId {
        self.id
    }

    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }

    /// Reads the heights from `base`'s data from now on, or from the
    /// renderer's own without a base. A base whose grid differs in size
    /// (one side still being rebuilt) leaves the renderer on its own heights.
    pub fn drape(
        &self,
        device: &wgpu::Device,
        own_size: (usize, usize),
        base: Option<&Mesh3DRenderer>,
    ) {
        let mut draped = self.draped.lock().unwrap_or_else(|p| p.into_inner());
        let Some(base) = base.filter(|b| (b.width, b.height) == own_size) else {
            *draped = None;
            return;
        };
        let id = base.heights.id;
        if draped.as_ref().is_none_or(|(held, _)| *held != id) {
            *draped = Some((id, bind_group(device, &self.layout, &base.data_buffer)));
        }
    }

    /// Drops the base's buffer (its meshes were rebuilt): a hidden overlay
    /// would hold it until drawn again, which drapes it anew.
    pub fn release(&self) {
        *self.draped.lock().unwrap_or_else(|p| p.into_inner()) = None;
    }

    /// Binds group 2 for the next draw.
    pub fn bind(&self, pass: &mut wgpu::RenderPass<'_>) {
        let draped = self.draped.lock().unwrap_or_else(|p| p.into_inner());
        let group = draped.as_ref().map_or(&self.own, |(_, group)| group);
        pass.set_bind_group(2, group, &[]);
    }
}

fn bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Mesh 3D Heights"),
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
    })
}
