//! Reticle marker and leader connector line rendering.

use egui::{Pos2, Rect, Stroke};

/// Renders the reticle marker dot, calculates tooltip anchor, and draws the leader elbow line.
pub fn draw_leader_callout(
    painter: &egui::Painter,
    ctx: &egui::Context,
    target_pos: Pos2,
    tooltip_rect: Rect,
) {
    let visuals = &ctx.style_of(ctx.theme()).visuals;
    let strong_color = visuals.strong_text_color();
    let text_color = visuals.text_color();
    let line_color = visuals.widgets.noninteractive.fg_stroke.color;

    // 1. Target reticle marker dot
    painter.circle_filled(target_pos, 7.0, text_color.linear_multiply(0.12));
    painter.circle_filled(target_pos, 4.5, text_color.linear_multiply(0.25));
    painter.circle_stroke(target_pos, 3.5, Stroke::new(1.2, strong_color));
    painter.circle_filled(target_pos, 1.8, strong_color);

    // 2. Compute anchor on tooltip box
    let box_anchor = if tooltip_rect.min.x >= target_pos.x {
        // card is to the right — connect at left edge, vertically centred
        Pos2::new(tooltip_rect.min.x, tooltip_rect.center().y)
    } else if tooltip_rect.max.x <= target_pos.x {
        // card is to the left — connect at right edge, vertically centred
        Pos2::new(tooltip_rect.max.x, tooltip_rect.center().y)
    } else if tooltip_rect.min.y >= target_pos.y {
        Pos2::new(target_pos.x, tooltip_rect.min.y)
    } else {
        Pos2::new(target_pos.x, tooltip_rect.max.y)
    };

    // 3. Elbow connector line
    let elbow = Pos2::new(target_pos.x, box_anchor.y);
    let leader_stroke = Stroke::new(1.2, line_color.linear_multiply(0.85));

    painter.line_segment([target_pos, elbow], leader_stroke);
    painter.line_segment([elbow, box_anchor], leader_stroke);

    // 4. Subtle junction dot at the elbow vertex
    painter.circle_filled(elbow, 1.5, strong_color.linear_multiply(0.8));
}
