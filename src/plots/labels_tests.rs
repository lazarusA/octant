//! The label pipeline against a heatmap renderer's data, on a GPU.

/// Builds a label renderer over a heatmap's data and draws it once,
/// failing on any wgpu validation error. Skipped without a GPU.
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
    let ids = [0u32, 7, 16_777_217, 3_000_000_000].map(f32::from_bits);
    let info = crate::data::ome::OmeLabelInfo {
        colors: vec![(7, [255, 0, 0, 255])],
        ..Default::default()
    };
    let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let heatmap = std::sync::Arc::new(super::HeatmapRenderer::new(&device, format, &ids, 2, 2));
    // A second palette over the same pipeline, as a new label variable gets.
    let labels = super::LabelRenderer::new(&device, format, &Default::default());
    let labels = labels.with_palette(&device, &info);
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("label test target"),
        size: wgpu::Extent3d {
            width: 8,
            height: 8,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let depth = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("label test depth"),
        size: wgpu::Extent3d {
            width: 8,
            height: 8,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let view = target.create_view(&Default::default());
    let depth_view = depth.create_view(&Default::default());
    heatmap.update_uniforms(&queue, &Default::default(), [0.0; 2], 1.0, [1.0; 2], 0);
    labels.update_params(&queue, true, 1.0);
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
    queue.submit([encoder.finish()]);
    if let Some(error) = rt.block_on(scope.pop()) {
        panic!("label pipeline validation failed: {error}");
    }
}
