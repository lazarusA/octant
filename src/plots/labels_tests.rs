//! The label pipeline against a heatmap renderer's data, on a GPU.

/// Side of the square test target, in pixels.
#[cfg(not(target_arch = "wasm32"))]
const SIZE: u32 = 8;

/// A `SIZE`-square 2D texture of `format`.
#[cfg(not(target_arch = "wasm32"))]
fn texture(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("label test texture"),
        size: wgpu::Extent3d {
            width: SIZE,
            height: SIZE,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        view_formats: &[],
    })
}

/// Copies `target` into a mappable buffer (rows 256 bytes apart) and reads it.
#[cfg(not(target_arch = "wasm32"))]
fn read_back(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    mut encoder: wgpu::CommandEncoder,
    target: &wgpu::Texture,
) -> Vec<u8> {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 256 * u64::from(SIZE),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let layout = wgpu::TexelCopyBufferLayout {
        offset: 0,
        bytes_per_row: Some(256),
        rows_per_image: Some(SIZE),
    };
    encoder.copy_texture_to_buffer(
        target.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout,
        },
        target.size(),
    );
    queue.submit([encoder.finish()]);
    let slice = buffer.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    let _ = device.poll(wgpu::PollType::wait_indefinitely());
    slice
        .get_mapped_range()
        .map(|view| view.to_vec())
        .unwrap_or_default()
}

/// Builds a label renderer over a 2x2 heatmap of ids, draws it filled and
/// reads it back: each quadrant shows its label's `label_color` (id 7
/// recorded, the others hashed), id 0 stays transparent. Fails on any wgpu
/// validation error. Skipped without a GPU.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn label_pipeline_validates_against_heatmap_data() {
    let (Some((device, queue)), Some(rt)) = (
        super::test_gpu::device(),
        tokio::runtime::Runtime::new().ok(),
    ) else {
        eprintln!("SKIPPED label_pipeline_validates_against_heatmap_data: no GPU device");
        return;
    };
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let ids = [0u32, 7, 16_777_217, 3_000_000_000];
    let info = std::sync::Arc::new(crate::data::ome::OmeLabelInfo {
        colors: vec![(7, [255, 0, 0, 255])],
        ..Default::default()
    });
    let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let values = ids.map(f32::from_bits);
    let heatmap = super::HeatmapRenderer::new(&device, format, &values, 2, 2);
    // A second palette over the same pipeline, as a new label variable gets.
    let labels = super::LabelRenderer::new(&device, format, &Default::default());
    let labels = labels.with_palette(&device, &info);
    let usage = wgpu::TextureUsages::RENDER_ATTACHMENT;
    let target = texture(&device, format, usage | wgpu::TextureUsages::COPY_SRC);
    let depth = texture(&device, wgpu::TextureFormat::Depth32Float, usage);
    let view = target.create_view(&Default::default());
    let depth_view = depth.create_view(&Default::default());
    heatmap.update_uniforms(&queue, &Default::default(), [0.0; 2], 1.0, [1.0; 2], 0);
    labels.update_params(&queue, false, 1.0);
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut rpass = encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("label test pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: None,
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            })
            .forget_lifetime();
        labels.draw(&mut rpass, &heatmap);
    }
    let pixels = read_back(&device, &queue, encoder, &target);
    if let Some(error) = rt.block_on(scope.pop()) {
        panic!("label pipeline validation failed: {error}");
    }
    // One pixel inside each quadrant, in the ids' row-major order.
    for (&id, (x, y)) in ids.iter().zip([(1, 1), (6, 1), (1, 6), (6, 6)]) {
        let at = y * 256 + x * 4;
        let got = pixels.get(at..at + 4).expect("pixel");
        let want = super::labels::label_color(&info, id).unwrap_or([0; 4]);
        let close = got.iter().zip(want).all(|(&g, w)| g.abs_diff(w) <= 1);
        assert!(close, "label {id}: drew {got:?}, expected {want:?}");
    }
}

/// The palette is rebuilt only for other label metadata, not per level.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn label_palettes_follow_their_metadata() {
    use std::sync::Arc;
    let Some((device, _queue)) = super::test_gpu::device() else {
        eprintln!("SKIPPED label_palettes_follow_their_metadata: no GPU device");
        return;
    };
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let info = Arc::new(crate::data::ome::OmeLabelInfo::default());
    let mut slot = None;
    super::LabelRenderer::sync(&mut slot, Some(&info), &device, format);
    let first = slot.clone().expect("renderer");
    super::LabelRenderer::sync(&mut slot, Some(&info), &device, format);
    assert!(Arc::ptr_eq(&first, slot.as_ref().expect("kept")));
    let other = Arc::new(crate::data::ome::OmeLabelInfo::default());
    super::LabelRenderer::sync(&mut slot, Some(&other), &device, format);
    assert!(!Arc::ptr_eq(&first, slot.as_ref().expect("rebuilt")));
    super::LabelRenderer::sync(&mut slot, None, &device, format);
    assert!(slot.is_none());
}
