//! Arrow-key navigation between the intake field and the sample chips:
//! Down from the field enters the chips, Left/Right (Home/End) move between
//! them, Up or Escape returns to the field, Enter/Space loads the sample.

use super::focus::intake_id;
use crate::ui::key_focus;
use egui::{Context, Id, Key, Response};

/// Stable id of sample chip `i`, so focus can move to it by index.
pub fn chip_id(i: usize) -> Id {
    Id::new(("hero_sample_chip", i))
}

/// Down in the intake field moves focus to the first chip.
pub fn from_intake(ctx: &Context, intake: &Response) {
    if intake.has_focus() && key_focus::pressed(ctx, [Key::ArrowDown]).is_some() {
        key_focus::request(ctx, chip_id(0));
    }
}

/// Move focus between `count` chips. Call before drawing them so the newly
/// focused chip draws focused on the same frame.
pub fn handle_keys(ctx: &Context, count: usize) {
    let Some(focused) = ctx.memory(|m| m.focused()) else {
        return;
    };
    let Some(at) = (0..count).find(|&i| chip_id(i) == focused) else {
        return;
    };
    let keys = [
        Key::ArrowLeft,
        Key::ArrowRight,
        Key::ArrowUp,
        Key::Home,
        Key::End,
        Key::Escape,
    ];
    let Some(key) = key_focus::pressed(ctx, keys) else {
        return;
    };
    let focus = |id: Id| key_focus::request(ctx, id);
    match key {
        Key::ArrowLeft if at > 0 => focus(chip_id(at - 1)),
        Key::ArrowRight if at + 1 < count => focus(chip_id(at + 1)),
        Key::Home => focus(chip_id(0)),
        Key::End => focus(chip_id(count - 1)),
        // Through `key_focus` so egui's own spatial Up can't move focus past the field.
        Key::ArrowUp | Key::Escape => focus(intake_id()),
        _ => {}
    }
}
