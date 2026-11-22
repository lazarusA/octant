//! Ray feedback end to end over the in-memory pyramid with a GPU: a resting
//! frame's rays report the cells they reached, the app reads them back and
//! selects with them; a frame of another mode is not taken. Skipped (with a
//! note) without an adapter.

use super::tests::{plotted, rest};
use crate::app::OctantApp;
use crate::plots::VolumeUniformParams;

/// The plotted app with a GPU and its bricked renderer, rested.
fn with_gpu() -> Option<OctantApp> {
    let mut app = plotted();
    app.wgpu_render_state = crate::app::test_support::render_state();
    app.wgpu_render_state.as_ref()?;
    // The bricked renderer is built with the bricks, now with a GPU.
    app.layers.base.data.volume_lod = Default::default();
    rest(&mut app);
    Some(app)
}

/// The base layer's frame parameters on the canvas.
fn drawn(app: &OctantApp) -> VolumeUniformParams {
    let px = app.layout.canvas_px;
    app.get_volume_uniform_params(&app.layers.base, px[0] as f32 / px[1] as f32)
}

/// Builds the frames of `frames` in 4 bands each, then rests until the
/// feedback is taken (at most 50 times; other tests share the GPU).
fn render_and_rest(app: &mut OctantApp, frames: &[VolumeUniformParams]) {
    let (Some(rs), Some(renderer)) = (
        app.wgpu_render_state.clone(),
        app.layers.base.renderers.volume_bricked.clone(),
    ) else {
        return;
    };
    let mut resources = eframe::egui_wgpu::CallbackResources::default();
    crate::plots::colormap_atlas::prepare(&rs.device, &rs.queue, &mut resources);
    let px = app.layout.canvas_px;
    // With nothing shown the renderer draws whole first, not in bands.
    if let Some(p) = frames.first() {
        let mut encoder = rs.device.create_command_encoder(&Default::default());
        renderer.render_frame_banded(&rs.device, &rs.queue, &mut encoder, (p, px, 1), &resources);
        rs.queue.submit([encoder.finish()]);
    }
    for p in frames {
        for _ in 0..4 {
            let mut encoder = rs.device.create_command_encoder(&Default::default());
            renderer.render_frame_banded(
                &rs.device,
                &rs.queue,
                &mut encoder,
                (p, px, 4),
                &resources,
            );
            rs.queue.submit([encoder.finish()]);
        }
    }
    for _ in 0..50 {
        let _ = rs.device.poll(wgpu::PollType::wait_indefinitely());
        rest(app);
        if app.layers.base.data.volume_lod.feedback.is_some() {
            return;
        }
    }
}

#[test]
fn a_resting_frame_reports_its_cells_to_the_selection() {
    let Some(mut app) = with_gpu() else {
        eprintln!("SKIPPED a_resting_frame_reports_its_cells_to_the_selection: no GPU adapter");
        return;
    };
    let params = drawn(&app);
    // A frame of another view first, whose copy still waits for its
    // readback when the resting view's frame ends: that one's is made later.
    let mut other = params;
    other.camera.eye[0] += 0.01;
    render_and_rest(&mut app, &[other, params]);
    let state = &app.layers.base.data.volume_lod;
    let seen = state
        .feedback
        .as_ref()
        .expect("feedback taken for the resting view");
    assert!(seen.count() > 0, "rays reached some cell");
}

#[test]
fn a_frame_of_another_mode_is_not_taken() {
    let Some(mut app) = with_gpu() else {
        eprintln!("SKIPPED a_frame_of_another_mode_is_not_taken: no GPU adapter");
        return;
    };
    // A DVR frame, read back once the plot shows MIP.
    let dvr = drawn(&app);
    app.plot_configs.volume.algorithm = 1;
    render_and_rest(&mut app, &[dvr]);
    assert!(
        app.layers.base.data.volume_lod.feedback.is_none(),
        "DVR's cells do not stand for MIP's"
    );
}

#[test]
fn a_new_step_reuses_the_renderer_without_the_last_steps_frame_or_feedback() {
    let Some(mut app) = with_gpu() else {
        eprintln!("SKIPPED a_new_step_reuses_the_renderer: no GPU adapter");
        return;
    };
    let (Some(rs), Some(before)) = (
        app.wgpu_render_state.clone(),
        app.layers.base.renderers.volume_bricked.clone(),
    ) else {
        return;
    };
    // A resting frame shown, its ray feedback read back but not taken.
    let p = drawn(&app);
    let mut resources = eframe::egui_wgpu::CallbackResources::default();
    crate::plots::colormap_atlas::prepare(&rs.device, &rs.queue, &mut resources);
    let px = app.layout.canvas_px;
    for bands in [1, 4, 4, 4, 4] {
        let mut encoder = rs.device.create_command_encoder(&Default::default());
        before.render_frame_banded(
            &rs.device,
            &rs.queue,
            &mut encoder,
            (&p, px, bands),
            &resources,
        );
        rs.queue.submit([encoder.finish()]);
    }
    let feedback = &before.bricks().expect("bricked").feedback;
    for _ in 0..100 {
        feedback.poll(&rs.device, &rs.queue);
        let _ = rs.device.poll(wgpu::PollType::wait_indefinitely());
        if !feedback.waiting() {
            break;
        }
    }
    assert!(before.frame_texture().is_some());
    // The next step: another plot context, same atlas size.
    app.layers.base.data.volume_lod.context = Some(u64::MAX);
    let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(px[0] as f32, px[1] as f32));
    let view = crate::app::pipeline::paint_layer::CanvasView {
        canvas_rect: rect,
        plot_rect: rect,
        pan: [0.0, 0.0],
        zoom: 1.0,
        aspect_scale: [1.0, 1.0],
    };
    app.refresh_volume_lod(crate::plots::PlotType::Volume, &view, 1e6);
    let after = app.layers.base.renderers.volume_bricked.clone();
    let after = after.expect("bricked again");
    assert!(
        std::sync::Arc::ptr_eq(&before, &after),
        "no new atlas per step"
    );
    assert!(
        after.frame_texture().is_none(),
        "the last step's frame is gone"
    );
    assert!(feedback.take().is_none(), "and so are its ray bits");
}
