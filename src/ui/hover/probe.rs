//! The keyboard probe: a stand-in pointer the arrow keys move over the plot,
//! so the hover card reads the data without a mouse. Enter on the canvas
//! starts and stops it, Escape or moving the mouse stops it.

use egui::{Context, Id, Pos2, Rect, Vec2};

fn probe_id() -> Id {
    Id::unique("octant_keyboard_probe")
}

/// Where the probe points, while it is on.
pub fn position(ctx: &Context) -> Option<Pos2> {
    ctx.data(|d| d.get_temp(probe_id()))
}

/// Starts the probe at `at`, or stops it.
pub fn toggle(ctx: &Context, at: Pos2) {
    ctx.data_mut(|d| {
        if d.remove_temp::<Pos2>(probe_id()).is_none() {
            d.insert_temp(probe_id(), at);
        }
    });
}

pub fn stop(ctx: &Context) {
    ctx.data_mut(|d| d.remove::<Pos2>(probe_id()));
}

/// Moves the probe by `delta`, kept inside `bounds`.
pub fn step(ctx: &Context, delta: Vec2, bounds: Rect) {
    ctx.data_mut(|d| {
        if let Some(at) = d.get_temp::<Pos2>(probe_id()) {
            d.insert_temp(probe_id(), bounds.clamp(at + delta));
        }
    });
}
