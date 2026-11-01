//! Multi-channel composites drawn as point clouds or volumes blend on the GPU: a
//! `(c, z, y, x)` block projected with a real (headless) GPU render state
//! keeps its raw channels, whose blend matches the CPU's packed volume.

use crate::app::OctantApp;
use crate::app::layers::{LayerId, VariableSelection};
use crate::app::test_support::render_state;
use crate::data::octant_block::OctantBlock;
use crate::data::slicing::ChannelColorConfig;
use crate::plots::PlotType;

const SHAPE: [usize; 4] = [3, 2, 4, 5];

fn block() -> OctantBlock {
    block_of(SHAPE)
}

/// A `(c, z, y, x)` block of `shape` with a NaN every seventh value.
fn block_of(shape: [usize; 4]) -> OctantBlock {
    let len: usize = shape.iter().product();
    let values: Vec<f32> = (0..len)
        .map(|i| match i % 7 {
            0 => f32::NAN,
            k => (i * 13 % 251) as f32 + k as f32,
        })
        .collect();
    OctantBlock::new(
        "img".into(),
        shape.to_vec(),
        ["c", "z", "y", "x"].map(String::from).to_vec(),
        vec![0; 4],
        values,
        Default::default(),
        std::collections::HashMap::new(),
    )
}

/// The block projected as a point cloud composite of three tinted
/// channels, with a GPU when `gpu` (`None` without one).
fn projected(gpu: bool) -> Option<OctantApp> {
    projected_as(gpu, PlotType::PointCloud, SHAPE)
}

/// A block of `shape` projected as a `plot_type` composite of three tinted
/// channels, with a GPU when `gpu` (`None` without one).
fn projected_as(gpu: bool, plot_type: PlotType, shape: [usize; 4]) -> Option<OctantApp> {
    let mut app = OctantApp {
        selected: VariableSelection {
            plot_type,
            ..Default::default()
        },
        ..Default::default()
    };
    if gpu {
        app.wgpu_render_state = Some(render_state()?);
    }
    let composite = &mut app.layers.base.composite;
    composite.enabled = true;
    composite.channel_configs = (0..3)
        .map(|i| {
            let tint = [[255, 0, 0], [0, 255, 0], [0, 0, 255]][i];
            ChannelColorConfig::new(i, format!("c{i}"), tint)
        })
        .collect();
    let [_, z, y, x] = shape;
    app.apply_3d_volume_projection(
        LayerId::BASE,
        &block_of(shape),
        3,
        2,
        1,
        (0, x - 1),
        (0, y - 1),
        (0, z - 1),
        (0, x),
        (0, y),
        (0, z),
        &[0, 0, 0, 0],
        false,
        true,
        0,
    );
    Some(app)
}

#[test]
fn points_blend_on_the_gpu_what_the_cpu_blends() {
    let (Some(gpu), Some(cpu)) = (projected(true), projected(false)) else {
        eprintln!("SKIPPED points_blend_on_the_gpu_what_the_cpu_blends: no GPU device");
        return;
    };
    assert!(gpu.layers.base.blends_on_gpu(), "channels on the GPU");
    assert!(!gpu.channel_display_reloads(LayerId::BASE));
    assert!(
        cpu.layers.base.data.channel_volume.is_none(),
        "colors on the CPU"
    );
    let cv = gpu
        .layers
        .base
        .data
        .channel_volume
        .as_ref()
        .expect("channels");
    assert_eq!(cv.channels, 3);
    let want = &cpu.layers.base.data.volume.as_ref().expect("volume").values;
    // What the hover reads: each voxel's color as the CPU packed it.
    for (idx, packed) in want.iter().enumerate() {
        let got = gpu.layers.base.gpu_blended_voxel(idx).expect("blended");
        assert_eq!(got.to_bits(), packed.to_bits(), "voxel {idx}");
    }
}

#[test]
fn a_volume_keeps_the_gpu_blend() {
    let Some(mut app) = projected(true) else {
        eprintln!("SKIPPED a_volume_keeps_the_gpu_blend: no GPU device");
        return;
    };
    app.selected.plot_type = PlotType::Volume;
    let [_, z, y, x] = SHAPE;
    app.apply_3d_volume_projection(
        LayerId::BASE,
        &block(),
        3,
        2,
        1,
        (0, x - 1),
        (0, y - 1),
        (0, z - 1),
        (0, x),
        (0, y),
        (0, z),
        &[0, 0, 0, 0],
        false,
        true,
        0,
    );
    assert!(app.layers.base.blends_on_gpu(), "baked into its textures");
    assert!(!app.channel_display_reloads(LayerId::BASE));
}

#[test]
fn a_narrow_volume_blends_on_the_cpu() {
    // One voxel wide: 12 MB of channels, but the bake's validity rows are
    // padded to 256 bytes, 256 MiB, past one storage buffer (WebGPU's
    // default: the ignored benches record their device's, often larger).
    crate::plots::limits::set_from_device(&wgpu::Limits::default());
    let Some(app) = projected_as(true, PlotType::Volume, [3, 1024, 1024, 1]) else {
        eprintln!("SKIPPED a_narrow_volume_blends_on_the_cpu: no GPU device");
        return;
    };
    assert!(
        app.layers.base.data.channel_volume.is_none(),
        "colors on the CPU"
    );
    assert!(app.layers.base.data.volume.is_some(), "still drawn");
}
