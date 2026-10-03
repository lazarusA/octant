//! Keyboard focus for the intake field: focused when the hero appears (first
//! load, or shown again later) and whenever the hero background is clicked.

use egui::{Context, Id, Rect, Response, Sense, Ui};

/// Stable id of the intake text field, so focus can be requested by id.
pub fn intake_id() -> Id {
    Id::new("hero_intake_field")
}

/// `true` on the first pass the hero is drawn after not being drawn on the
/// previous one. Tracking the last drawn pass needs no hide notification.
pub fn just_appeared(ctx: &Context) -> bool {
    let id = Id::new("hero_last_drawn_pass");
    let pass = ctx.cumulative_pass_nr();
    let previous = ctx.data_mut(|d| {
        let previous = d.get_temp::<u64>(id);
        d.insert_temp(id, pass);
        previous
    });
    previous.is_none_or(|p| p.saturating_add(1) < pass)
}

/// Click target covering the hero's own area (`rect`, after side panels).
/// Register it before the hero's widgets so they stay on top and keep their
/// own clicks.
pub fn background(ui: &mut Ui, rect: Rect) -> Response {
    ui.interact(rect, Id::new("hero_background"), Sense::click())
}

/// Focus the intake field. Call after the field is drawn: clicking outside it
/// makes egui drop its focus while drawing it, which would undo an earlier call.
pub fn focus_intake(ctx: &Context) {
    ctx.memory_mut(|m| m.request_focus(intake_id()));
}
