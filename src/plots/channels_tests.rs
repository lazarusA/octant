//! The channel shader against the CPU blend, on a GPU.

use crate::data::ome::model::ChannelFamily;
use crate::data::slicing::{ChannelColorConfig, CompositeStretch};

/// Three 8 x 8 channels: a ramp with missing cells, int8-like values (read
/// as bit patterns) and wide floats.
pub(super) fn block() -> crate::data::octant_block::OctantBlock {
    let n = 64;
    let ramp = (0..n).map(|i| if i % 9 == 0 { f32::NAN } else { i as f32 });
    let int8 = (0..n).map(|i| (i as f32) - 20.0);
    let wide = (0..n).map(|i| (i * i) as f32 * 0.25);
    let values: Vec<f32> = ramp.chain(int8).chain(wide).collect();
    crate::data::octant_block::OctantBlock::new(
        "c".into(),
        vec![3, 8, 8],
        ["c", "y", "x"].map(String::from).to_vec(),
        vec![0; 3],
        values,
        Default::default(),
        std::collections::HashMap::new(),
    )
}

/// Every curve family, a window, an inversion and distinct tints.
pub(super) fn configs() -> Vec<ChannelColorConfig> {
    let mut c: Vec<_> = (0..3)
        .map(|i| {
            ChannelColorConfig::new(
                i,
                format!("c{i}"),
                [[255, 40, 0], [0, 200, 90], [60, 0, 255]][i],
            )
        })
        .collect();
    c[0].display.family = ChannelFamily::Logarithmic;
    c[1].display.family = ChannelFamily::Polynomial;
    c[1].display.coefficient = 2.0;
    c[2].display.family = ChannelFamily::Exponential;
    c[2].display.coefficient = 0.5;
    c[2].display.inverted = true;
    c[2].window = Some((100.0, 900.0));
    c
}

/// The CPU blend packed, and each channel's plane, as the app slices them.
pub(super) fn cpu(
    block: &crate::data::octant_block::OctantBlock,
    configs: &[ChannelColorConfig],
    stretch: &mut CompositeStretch,
) -> (Vec<f32>, Vec<Vec<f32>>) {
    let packed = crate::data::slicing::slice_multichannel_composite_nd(
        block,
        0,
        2,
        1,
        (0, 8),
        (0, 8),
        &[0, 0, 0],
        configs,
        1,
        stretch,
    )
    .expect("blend");
    let planes = (0..3)
        .map(|c| {
            let m = block.slice_2d_with_ranges(2, 1, (0, 8), (0, 8), &[c, 0, 0], 1, "p", true);
            m.expect("plane").values.to_vec()
        })
        .collect();
    (packed.values.to_vec(), planes)
}

/// The shader draws every cell in the CPU blend's color (within one level
/// of rounding), missing cells transparent. Skipped without a GPU.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn channels_blend_as_the_cpu_does() {
    use super::labels_tests::{SIZE, read_back, texture};
    let (Some((device, queue)), Some(rt)) = (
        super::test_gpu::device(),
        tokio::runtime::Runtime::new().ok(),
    ) else {
        eprintln!("SKIPPED channels_blend_as_the_cpu_does: no GPU device");
        return;
    };
    let (block, mut configs) = (block(), configs());
    let mut stretch = CompositeStretch::default();
    // All three ticked, then the first alone (as a click unticks the others).
    for ticked in [3, 1] {
        configs
            .iter_mut()
            .enumerate()
            .for_each(|(i, c)| c.visible = i < ticked);
        let (packed, planes) = cpu(&block, &configs, &mut stretch);
        let format = wgpu::TextureFormat::Rgba8Unorm;
        let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let heatmap = super::HeatmapRenderer::new(&device, format, &planes[0], 8, 8);
        let renderer = super::ChannelRenderer::new(&device, format, 3, 64);
        for (i, plane) in planes.iter().enumerate() {
            renderer.write_plane(&queue, i, plane);
        }
        let params = super::ChannelParams::of(&configs, &stretch, (0, 3), 64, None);
        heatmap.update_uniforms(&queue, &Default::default(), &Default::default());
        renderer.update_params(&queue, &params);
        let usage = wgpu::TextureUsages::RENDER_ATTACHMENT;
        let target = texture(&device, format, usage | wgpu::TextureUsages::COPY_SRC);
        let depth = texture(&device, wgpu::TextureFormat::Depth32Float, usage);
        let depth_view = depth.create_view(&Default::default());
        let mut encoder = device.create_command_encoder(&Default::default());
        {
            let view = target.create_view(&Default::default());
            let mut rpass = encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("channel test pass"),
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
            renderer.draw(&mut rpass, &heatmap);
        }
        let pixels = read_back(&device, &queue, encoder, &target);
        if let Some(error) = rt.block_on(scope.pop()) {
            panic!("channel pipeline validation failed: {error}");
        }
        for (cell, &want) in packed.iter().enumerate().take((SIZE * SIZE) as usize) {
            let (x, y) = (cell % 8, cell / 8);
            let at = y * 256 + x * 4;
            let got = pixels.get(at..at + 4).expect("pixel");
            let want = match want.is_nan() {
                true => [0; 4],
                false => {
                    let (r, g, b) = crate::data::slicing::composite::utils::unpack_rgb(want);
                    [r as u8, g as u8, b as u8, 255]
                }
            };
            let close = got.iter().zip(want).all(|(&g, w)| g.abs_diff(w) <= 1);
            assert!(
                close,
                "{ticked} ticked, cell {cell}: drew {got:?}, CPU {want:?}"
            );
        }
    }
}

/// The uniforms pick the channels `drawn_channels` picks, in order, and
/// leave the planes the CPU blend does not read.
#[test]
fn uniforms_draw_the_channels_the_cpu_draws() {
    let mut configs = configs();
    let mut stretch = CompositeStretch::default();
    let (block, _) = (block(), ());
    cpu(&block, &configs, &mut stretch);
    configs[1].visible = false;
    let params = super::ChannelParams::of(&configs, &stretch, (0, 3), 64, None);
    let drawn = crate::data::slicing::composite::utils::drawn_channels(&configs, 0, 3);
    assert_eq!(
        params.planes(),
        drawn.iter().map(|(_, l)| *l as u32).collect::<Vec<_>>()
    );
    configs.iter_mut().for_each(|c| c.visible = false);
    let params = super::ChannelParams::of(&configs, &stretch, (0, 3), 64, None);
    assert_eq!(params.planes(), [0], "none ticked: the first");
}

/// The hover's one-cell blend is the plane's, missing cells included.
#[test]
fn one_cell_blends_as_the_plane() {
    let (block, configs) = (block(), configs());
    let mut stretch = CompositeStretch::default();
    let (packed, _) = cpu(&block, &configs, &mut stretch);
    let probe =
        crate::data::slicing::CompositeProbe::new(&block, 0, 2, 1, (0, 8), (0, 8), &[0, 0, 0]);
    for (cell, want) in packed.iter().enumerate() {
        let got =
            crate::data::slicing::blend_pixel(&probe, &configs, &stretch, (cell % 8, cell / 8));
        assert_eq!(got.to_bits(), want.to_bits(), "cell {cell}");
    }
}
