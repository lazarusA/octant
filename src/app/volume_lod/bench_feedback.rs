//! Ray feedback rounds of the Beechnut benchmark (`bench_tests.rs`):
//! a resting frame, its readback, a settle.

use super::PX;
use crate::app::OctantApp;
use crate::plots::PlotType;

/// The resting frame of the current view, built in 16 bands as the app
/// does, so its rays report the cells they reach.
fn render_resting(app: &OctantApp) {
    let (Some(rs), Some(renderer), Some(lod)) = (
        app.wgpu_render_state.as_ref(),
        app.layers.base.renderers.volume_bricked.clone(),
        app.layers.base.data.volume_lod.lod.as_ref(),
    ) else {
        return;
    };
    let params = app.get_volume_uniform_params(&app.layers.base, PX[0] as f32 / PX[1] as f32);
    if let Some(bricks) = renderer.bricks() {
        bricks.set_dims(lod.level_dims(lod.finest));
    }
    let mut resources = eframe::egui_wgpu::CallbackResources::default();
    crate::plots::colormap_atlas::prepare(&rs.device, &rs.queue, &mut resources);
    // The first call draws whole when nothing is shown yet; 16 bands follow.
    for _ in 0..17 {
        let mut encoder = rs.device.create_command_encoder(&Default::default());
        let target = (&params, PX, 16);
        renderer.render_frame_banded(&rs.device, &rs.queue, &mut encoder, target, &resources);
        rs.queue.submit([encoder.finish()]);
    }
}

/// Rounds of resting frame, readback and settle: the cells the rays
/// reached and the bricks wanted after each.
pub(super) fn report_feedback(app: &mut OctantApp, clock: &mut f64, label: &str) {
    for round in 1..=3 {
        render_resting(app);
        if let Some(rs) = app.wgpu_render_state.clone() {
            let _ = rs.device.poll(wgpu::PollType::wait_indefinitely());
        }
        // Refreshes until the readback is in (the app keeps repainting).
        let view = super::view();
        for _ in 0..50 {
            *clock += 0.05;
            app.refresh_volume_lod(PlotType::Volume, &view, *clock);
            if app.layers.base.data.volume_lod.feedback.is_some() {
                break;
            }
        }
        let s = super::settle(app, clock);
        let state = &app.layers.base.data.volume_lod;
        let seen = state.feedback.as_ref().map(|f| f.count());
        let wanted = state.lod.as_ref().map_or(0, |l| l.wanted.len());
        eprintln!(
            "{label}: ray feedback round {round}: {seen:?} cells reached, {wanted} bricks wanted, \
             settled in {:.1} s, bricks per level {:?}",
            s.wall.as_secs_f64(),
            s.levels
        );
    }
}
