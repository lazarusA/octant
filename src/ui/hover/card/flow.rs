//! Coordinate rows as a wrapping line of `label value` pairs split by hairline dividers.

use crate::ui::hover::card::layout::{field_font, key_font, line, muted_color};
use crate::ui::hover::field::HoverField;
use egui::{Painter, Pos2, Stroke, Visuals, pos2};

/// Space between a coordinate label and its value.
pub const KEY_GAP: f32 = 5.0;
/// Space on each side of the divider between two pairs.
pub const ITEM_GAP: f32 = 9.0;
/// Width a divider adds between two pairs on the same row.
pub const SEPARATOR: f32 = 2.0 * ITEM_GAP + 1.0;
/// Vertical space between wrapped rows.
pub const ROW_GAP: f32 = 5.0;

/// Where one pair lands: its x offset within the row, its row, and whether a divider
/// precedes it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlowSlot {
    pub x: f32,
    pub row: usize,
    pub divided: bool,
}

/// Lays out item widths left to right, wrapping before an item that would overflow
/// `max_width`. An item wider than a whole row gets a row of its own.
pub fn flow(widths: impl Iterator<Item = f32>, max_width: f32) -> impl Iterator<Item = FlowSlot> {
    let mut cursor: Option<(f32, usize)> = None;
    widths.map(move |w| {
        let slot = match cursor {
            Some((end, row)) if end + SEPARATOR + w <= max_width => FlowSlot {
                x: end + SEPARATOR,
                row,
                divided: true,
            },
            Some((_, row)) => FlowSlot {
                x: 0.0,
                row: row + 1,
                divided: false,
            },
            None => FlowSlot {
                x: 0.0,
                row: 0,
                divided: false,
            },
        };
        cursor = Some((slot.x + w, slot.row));
        slot
    })
}

/// Natural `(label, value)` widths and the taller of the two line heights.
pub fn pair_metrics(painter: &Painter, visuals: &Visuals, field: &HoverField) -> (f32, f32, f32) {
    let color = visuals.text_color();
    let key = line(painter, &field.label, key_font(), color, f32::INFINITY).size();
    let val = line(painter, &field.value, field_font(), color, f32::INFINITY).size();
    (key.x, val.x, key.y.max(val.y))
}

pub fn pair_width((key_w, val_w, _): (f32, f32, f32)) -> f32 {
    key_w + KEY_GAP + val_w
}

/// Paints the pairs from `origin`, wrapping within `max_width`, with rows `row_h` tall.
pub fn paint_fields(
    painter: &Painter,
    visuals: &Visuals,
    fields: &[HoverField],
    origin: Pos2,
    max_width: f32,
    row_h: f32,
) {
    let key_color = muted_color(visuals);
    let val_color = visuals.text_color();
    let divider = Stroke::new(1.0, visuals.widgets.noninteractive.bg_stroke.color);
    let widths = fields
        .iter()
        .map(|f| pair_width(pair_metrics(painter, visuals, f)).min(max_width));

    for (field, slot) in fields.iter().zip(flow(widths, max_width)) {
        let x = origin.x + slot.x;
        let mid_y = origin.y + slot.row as f32 * (row_h + ROW_GAP) + row_h * 0.5;
        if slot.divided {
            let div_x = (x - ITEM_GAP - 0.5).round() + 0.5;
            let half = (row_h * 0.32).round();
            painter.vline(div_x, (mid_y - half)..=(mid_y + half), divider);
        }

        let key = line(
            painter,
            &field.label,
            key_font(),
            key_color,
            max_width * 0.5,
        );
        let key_w = key.size().x;
        painter.galley(pos2(x, mid_y - key.size().y * 0.5), key, key_color);

        let val_x = x + key_w + KEY_GAP;
        let val_max = (origin.x + max_width - val_x).max(0.0);
        let val = line(painter, &field.value, field_font(), val_color, val_max);
        painter.galley(pos2(val_x, mid_y - val.size().y * 0.5), val, val_color);
    }
}
