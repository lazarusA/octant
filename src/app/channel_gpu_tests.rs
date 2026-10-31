//! Multi-channel composites blended on the GPU: the in-memory pyramid's
//! three `t2m` steps as tinted channels of a heatmap, plotted with a real
//! (headless) GPU render state, against the same plot blended on the CPU.

use crate::app::OctantApp;
use crate::app::layers::LayerId;
use crate::app::state::AnimationRole;
use crate::app::test_support::{poll_until, pyramid_app, render_state, selection_of};
use crate::data::slicing::ChannelColorConfig;

/// `t2m` plotted as a three-channel composite on a 20 px canvas (level 2
/// by Auto), with a GPU when `gpu` (`None` without one).
pub(super) fn plotted(gpu: bool) -> Option<OctantApp> {
    let (mut app, meta) = pyramid_app();
    if gpu {
        app.wgpu_render_state = Some(render_state()?);
    }
    app.layout.canvas_px = [20, 20];
    let mut sel = selection_of(&mut app, &meta, "0/t2m");
    sel.dim_config[0].animation = AnimationRole::None;
    sel.dim_config[0].range = (0, 2);
    sel.dim_ranges[0] = (0, 2);
    sel.animated_dim = None;
    app.selected = sel;
    app.plot_selection();
    poll_until(&mut app, |app| app.layers.base.data.matrix.is_some());
    // Turned on as the Settings checkbox does (a new variable takes its defaults).
    let composite = &mut app.layers.base.composite;
    composite.enabled = true;
    composite.channel_configs = (0..3)
        .map(|i| {
            let tint = [[255, 0, 0], [0, 255, 0], [0, 0, 255]][i];
            ChannelColorConfig::new(i, format!("t{i}"), tint)
        })
        .collect();
    app.load_layer_block(LayerId::BASE);
    poll_until(&mut app, |app| {
        app.layers.base.data.composite_probe.is_some()
    });
    Some(app)
}

#[test]
fn the_gpu_blends_what_the_cpu_blends() {
    let (Some(gpu), Some(cpu)) = (plotted(true), plotted(false)) else {
        eprintln!("SKIPPED the_gpu_blends_what_the_cpu_blends: no GPU device");
        return;
    };
    assert!(gpu.layers.base.blends_on_gpu(), "planes on the GPU");
    assert!(!cpu.layers.base.blends_on_gpu(), "colors on the CPU");
    let (Some(cpu_matrix), Some(gpu_matrix)) =
        (&cpu.layers.base.data.matrix, &gpu.layers.base.data.matrix)
    else {
        panic!("both plotted");
    };
    assert_eq!(
        (gpu_matrix.width, gpu_matrix.height),
        (cpu_matrix.width, cpu_matrix.height)
    );
    // What the hover reads: each cell's color as the CPU packed it.
    for py in 0..cpu_matrix.height {
        for px in 0..cpu_matrix.width {
            let want = cpu_matrix.values[py * cpu_matrix.width + px];
            let got = gpu.layers.base.gpu_blended_cell(px, py).expect("blended");
            assert_eq!(got.to_bits(), want.to_bits(), "cell {px}, {py}");
        }
    }
}

#[test]
fn display_changes_reload_only_without_the_gpu() {
    let (Some(gpu), Some(cpu)) = (plotted(true), plotted(false)) else {
        eprintln!("SKIPPED display_changes_reload_only_without_the_gpu: no GPU device");
        return;
    };
    assert!(!gpu.channel_display_reloads(LayerId::BASE));
    assert!(cpu.channel_display_reloads(LayerId::BASE));
}

#[test]
fn turning_the_composite_off_drops_the_planes() {
    let Some(mut app) = plotted(true) else {
        eprintln!("SKIPPED turning_the_composite_off_drops_the_planes: no GPU device");
        return;
    };
    app.layers.base.composite.enabled = false;
    app.load_layer_block(LayerId::BASE);
    poll_until(&mut app, |app| app.layers.base.renderers.channels.is_none());
    assert!(!app.layers.base.blends_on_gpu());
}
