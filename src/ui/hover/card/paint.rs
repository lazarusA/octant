//! Card surface, headline value with color chip, and the coordinate section.

use crate::ui::hover::card::flow::paint_fields;
use crate::ui::hover::card::layout::{
    CHIP, CHIP_GAP, CORNER_RADIUS, CardLayout, PAD, SECTION_GAP, TITLE_GAP, UNITS_GAP, line,
    muted_color, title_font, units_font, value_font,
};
use crate::ui::hover::card::model::{HoverCard, HoverValue};
use egui::epaint::Shadow;
use egui::{Color32, Painter, Pos2, Rect, Stroke, StrokeKind, Visuals, pos2, vec2};

/// Paints the whole card into `rect`, which must have the size from [`CardLayout::measure`].
pub fn paint_card(
    painter: &Painter,
    visuals: &Visuals,
    card: &HoverCard,
    value: &str,
    layout: &CardLayout,
    rect: Rect,
) {
    paint_surface(painter, visuals, rect);
    let origin = rect.min + PAD;
    let inner_w = layout.inner_width();
    let value_y = origin.y + layout.title_h + TITLE_GAP;

    paint_header(painter, visuals, card, origin, inner_w);
    paint_value_row(
        painter,
        visuals,
        card,
        value,
        layout,
        pos2(origin.x, value_y),
        inner_w,
    );

    if !card.fields.is_empty() {
        let divider_y = value_y + layout.value_h + SECTION_GAP + 0.5;
        painter.hline(
            rect.x_range().shrink(PAD.x),
            divider_y,
            Stroke::new(1.0, visuals.widgets.noninteractive.bg_stroke.color),
        );
        let grid_origin = pos2(origin.x, divider_y + 0.5 + SECTION_GAP);
        paint_fields(
            painter,
            visuals,
            card.fields,
            grid_origin,
            inner_w,
            layout.field_h,
        );
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

fn paint_header(
    painter: &Painter,
    visuals: &Visuals,
    card: &HoverCard,
    origin: Pos2,
    inner_w: f32,
) {
    let strong = visuals.strong_text_color();
    let title = line(painter, card.title, title_font(), strong, inner_w);
    painter.galley(origin, title, strong);
}

fn paint_value_row(
    painter: &Painter,
    visuals: &Visuals,
    card: &HoverCard,
    value: &str,
    layout: &CardLayout,
    origin: Pos2,
    inner_w: f32,
) {
    let mid_y = origin.y + layout.value_h * 0.5;
    let chip = Rect::from_center_size(pos2(origin.x + CHIP * 0.5, mid_y), vec2(CHIP, CHIP));
    paint_chip(
        painter,
        visuals,
        chip,
        card.swatch,
        card.value == HoverValue::NoData,
    );

    let strong = visuals.strong_text_color();
    let text_color = if card.value == HoverValue::NoData {
        muted_color(visuals)
    } else {
        strong
    };
    let text_x = chip.right() + CHIP_GAP;
    let galley = line(
        painter,
        value,
        value_font(),
        text_color,
        inner_w - (text_x - origin.x),
    );
    let (value_w, value_h) = (galley.size().x, galley.size().y);
    let value_top = mid_y - value_h * 0.5;
    painter.galley(pos2(text_x, value_top), galley, text_color);

    if card.value.shows_units() && !card.units.is_empty() {
        let units_x = text_x + value_w + UNITS_GAP;
        let max_w = (origin.x + inner_w - units_x).max(0.0);
        let units = line(
            painter,
            card.units,
            units_font(),
            muted_color(visuals),
            max_w,
        );
        // Same font as the value, so a shared top keeps the baselines aligned.
        painter.galley(pos2(units_x, value_top), units, muted_color(visuals));
    }
}

fn paint_chip(painter: &Painter, visuals: &Visuals, rect: Rect, color: Color32, no_data: bool) {
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
