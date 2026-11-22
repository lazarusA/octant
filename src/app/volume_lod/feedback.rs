//! Ray feedback in the selection: the table cells the last resting frame's rays reached (`plots/volume/
//! feedback.rs`) replace the CPU's guess (`occlusion.rs`) once read back for
//! the view resting now, and select again; each finished frame refines the
//! next selection, so cells a finer brick opens up are refined in turn.

use super::select::Placement;
use super::{Lod, ViewKey, occlusion};
use crate::app::OctantApp;
use crate::app::pipeline::paint_layer::CanvasView;
use crate::data::multiscale::bricks::CellBits;

/// Seconds between looks while a readback runs.
const READBACK_WAIT: f64 = 0.05;

impl OctantApp {
    /// Moves the readback along and takes its cells when they are of the
    /// frame drawn now and differ from the ones used: the next settle
    /// selects with them. How long to wait while a readback runs.
    pub(super) fn take_ray_feedback(&mut self, view: &CanvasView) -> Option<f64> {
        let rs = self.wgpu_render_state.clone()?;
        let renderer = self.layers.base.renderers.volume_bricked.clone()?;
        let feedback = &renderer.bricks()?.feedback;
        feedback.poll(&rs.device, &rs.queue);
        // Only the bits of the frame drawn now: a frame of another mode,
        // color range or opacity reached other cells.
        let screen = crate::plots::common::compute_aspect_ratio(&view.plot_rect);
        let drawn = self.get_volume_uniform_params(&self.layers.base, screen);
        let state = &mut self.layers.base.data.volume_lod;
        if let Some((frame, seen)) = feedback.take()
            && frame == drawn
            && state.view.is_some()
            && state.feedback.as_ref() != Some(&seen)
        {
            state.feedback = Some(seen);
            state.view = None;
        }
        feedback.waiting().then_some(READBACK_WAIT)
    }
}

/// The table cells rays reach as the CPU guesses them over the plotted
/// `volume` (`occlusion.rs`), `None` in modes nothing hides in.
pub(super) fn guessed_cells(
    lod: &Lod,
    volume: &crate::data::VolumeData,
    key: &ViewKey,
) -> Option<CellBits> {
    let place = Placement {
        aspect: key.aspect,
        ..lod.place
    };
    let grid = lod.bricks().grid();
    let opacity = key.occluding?;
    Some(occlusion::seen_cells(
        volume,
        &key.camera,
        &place,
        lod.flip,
        grid,
        key.px,
        opacity,
    ))
}
