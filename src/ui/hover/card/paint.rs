//! Card surface, headline value with color chip, overlay rows, and the coordinate section.

use crate::ui::hover::card::layout::{
    CHIP, CHIP_GAP, CORNER_RADIUS, CardLayout, PAD, SECTION_GAP, TITLE_GAP, UNITS_GAP, muted_color,
};
use crate::ui::hover::card::model::{HoverCard, HoverValue};
use egui::epaint::Shadow;
use egui::{Color32, Painter, Pos2, Rect, Stroke, StrokeKind, Visuals, pos2, vec2};
use std::sync::Arc;

/// Paints the card into `rect`, which must have the size of `layout`.
pub fn paint_card(
    painter: &Painter,
    visuals: &Visuals,
    card: &HoverCard,
    layout: &CardLayout,
    rect: Rect,
) {
    paint_surface(painter, visuals, rect);
    let origin = rect.min + PAD;
    painter.galley(
        origin,
        Arc::clone(&layout.title),
        visuals.strong_text_color(),
    );

    let value_y = origin.y + layout.title.size().y + TITLE_GAP;
    paint_value_row(painter, visuals, card, layout, pos2(origin.x, value_y));
    let mut bottom = value_y + layout.value_h;
    if layout.layers.count > 0 {
        let layers_origin = pos2(origin.x, bottom + SECTION_GAP);
        let right = rect.right() - PAD.x;
        layout.layers.paint(painter, visuals, layers_origin, right);
        bottom = layers_origin.y + layout.layers.height();
    }

    if layout.fields.rows > 0 {
        let divider_y = bottom + SECTION_GAP + 0.5;
        painter.hline(
            rect.x_range().shrink(PAD.x),
            divider_y,
            Stroke::new(1.0, visuals.widgets.noninteractive.bg_stroke.color),
        );
        let fields_origin = pos2(origin.x, divider_y + 0.5 + SECTION_GAP);
        layout.fields.paint(painter, visuals, fields_origin);
    }
}

fn paint_surface(painter: &Painter, visuals: &Visuals, rect: Rect) {
    // Symmetric shadow so the card reads the same on either side of the target.
    let shadow = Shadow {
        offset: [0, 4],
        blur: 18,
        spread: 0,
        color: Color32::from_black_alpha(if visuals.dark_mode { 110 } else { 40 }),
    };
    painter.add(shadow.as_shape(rect, CORNER_RADIUS));
    painter.rect(
        rect,
        CORNER_RADIUS,
        visuals.window_fill,
        visuals.window_stroke,
        StrokeKind::Inside,
    );
}

fn paint_value_row(
    painter: &Painter,
    visuals: &Visuals,
    card: &HoverCard,
    layout: &CardLayout,
    origin: Pos2,
) {
    let mid_y = origin.y + layout.value_h * 0.5;
    let chip = Rect::from_center_size(pos2(origin.x + CHIP * 0.5, mid_y), vec2(CHIP, CHIP));
    let no_data = card.value == HoverValue::NoData;
    paint_chip(painter, visuals, chip, card.swatch, no_data);

    let value = &layout.value;
    let text_x = chip.right() + CHIP_GAP;
    let value_top = mid_y - value.size().y * 0.5;
    painter.galley(
        pos2(text_x, value_top),
        Arc::clone(value),
        visuals.text_color(),
    );

    if let Some(units) = &layout.units {
        // Same font as the value, so a shared top keeps the baselines aligned.
        let units_x = text_x + value.size().x + UNITS_GAP;
        painter.galley(
            pos2(units_x, value_top),
            Arc::clone(units),
            visuals.text_color(),
        );
    }
}

/// A rounded color chip, or a slashed empty one for no data.
pub(super) fn paint_chip(
    painter: &Painter,
    visuals: &Visuals,
    rect: Rect,
    color: Color32,
    no_data: bool,
) {
    let border = Stroke::new(1.0, visuals.widgets.noninteractive.bg_stroke.color);
    if no_data {
        painter.rect_filled(rect, 3, visuals.faint_bg_color);
        let slash = Stroke::new(1.2, muted_color(visuals));
        let r = rect.shrink(3.0);
        painter.line_segment([r.left_bottom(), r.right_top()], slash);
    } else {
        painter.rect_filled(rect, 3, color);
    }
    painter.rect_stroke(rect, 3, border, StrokeKind::Inside);
}
