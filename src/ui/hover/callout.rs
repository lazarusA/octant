//! Reticle marker and elbow leader arm from the hovered point to the card.

use crate::ui::hover::card::layout::CORNER_RADIUS;
use crate::ui::hover::card::place::{Placement, Side};
use egui::{Painter, Pos2, Stroke, Visuals, pos2};

/// How far the leader runs under the card edge, so the card always covers its end.
const UNDERLAP: f32 = 4.0;

/// Paints the reticle at `target` and the elbow arm to the card. Must be painted before
/// the card on the same layer: the card covers the arm's end, so they never look detached.
pub fn draw_leader_callout(
    painter: &Painter,
    visuals: &Visuals,
    target: Pos2,
    placement: &Placement,
) {
    let strong = visuals.strong_text_color();
    let stroke = Stroke::new(1.2, visuals.widgets.noninteractive.fg_stroke.color);

    let anchor = leader_anchor(target, placement);
    let elbow = leader_elbow(target, anchor, placement.side);
    painter.line_segment([target, elbow], stroke);
    painter.line_segment([elbow, anchor], stroke);
    if elbow.distance(target) > 2.0 {
        painter.circle_filled(elbow, 1.6, strong);
    }

    painter.circle_filled(target, 7.0, visuals.text_color().gamma_multiply(0.12));
    painter.circle_filled(target, 4.5, visuals.window_fill.gamma_multiply(0.6));
    painter.circle_stroke(target, 3.5, Stroke::new(1.2, strong));
    painter.circle_filled(target, 1.8, strong);
}

/// Point just inside the card edge facing `target`: the vertical middle of a side edge,
/// or straight above/below the target on a top/bottom edge.
pub fn leader_anchor(target: Pos2, placement: &Placement) -> Pos2 {
    let rect = placement.rect;
    let inset = f32::from(CORNER_RADIUS) + 2.0;
    let x = target.x.min(rect.right() - inset).max(rect.left() + inset);
    match placement.side {
        Side::Right => pos2(rect.left() + UNDERLAP, rect.center().y),
        Side::Left => pos2(rect.right() - UNDERLAP, rect.center().y),
        Side::Below => pos2(x, rect.top() + UNDERLAP),
        Side::Above => pos2(x, rect.bottom() - UNDERLAP),
    }
}

/// Corner of the arm: vertical from the target, then horizontal into a side edge.
pub fn leader_elbow(target: Pos2, anchor: Pos2, side: Side) -> Pos2 {
    match side {
        Side::Right | Side::Left => pos2(target.x, anchor.y),
        Side::Below | Side::Above => pos2(anchor.x, target.y),
    }
}
