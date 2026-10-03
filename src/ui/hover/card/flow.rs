//! Coordinate rows as a wrapping line of `label value` pairs split by hairline dividers.

use crate::ui::hover::card::layout::{field_font, key_font, line, muted_color};
use crate::ui::hover::field::HoverField;
use egui::{Galley, Painter, Pos2, Stroke, Visuals, pos2};
use std::sync::Arc;

/// Space between a coordinate label and its value.
pub const KEY_GAP: f32 = 5.0;
/// Space on each side of the divider between two pairs.
pub const ITEM_GAP: f32 = 9.0;
/// Width a divider adds between two pairs on the same row.
pub const SEPARATOR: f32 = 2.0 * ITEM_GAP + 1.0;
/// Vertical space between wrapped rows.
pub const ROW_GAP: f32 = 5.0;
/// Pairs beyond this are dropped; datasets rarely have more than a handful of dimensions.
pub const MAX_FIELDS: usize = 12;

/// Where one pair lands: its x offset within the row, its row, and whether a divider
/// precedes it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
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
            None => FlowSlot::default(),
        };
        cursor = Some((slot.x + w, slot.row));
        slot
    })
}

/// One laid-out `label value` pair and its flow position.
pub struct FieldPair {
    pub key: Arc<Galley>,
    pub value: Arc<Galley>,
    pub slot: FlowSlot,
}

impl FieldPair {
    pub fn width(&self) -> f32 {
        self.key.size().x + KEY_GAP + self.value.size().x
    }
}

/// The coordinate section, laid out once per frame and painted from the same galleys.
pub struct FieldRows {
    pairs: [Option<FieldPair>; MAX_FIELDS],
    pub rows: usize,
    /// Width of the widest row.
    pub width: f32,
    pub row_h: f32,
}

impl FieldRows {
    pub fn layout(
        painter: &Painter,
        visuals: &Visuals,
        fields: &[HoverField],
        max_width: f32,
    ) -> Self {
        let (key_color, val_color) = (muted_color(visuals), visuals.text_color());
        let mut pairs: [Option<FieldPair>; MAX_FIELDS] = Default::default();
        for (cell, field) in pairs.iter_mut().zip(fields) {
            let (key, value) = fit_pair(painter, field, key_color, val_color, max_width);
            *cell = Some(FieldPair {
                key,
                value,
                slot: FlowSlot::default(),
            });
        }

        let widths: [f32; MAX_FIELDS] =
            std::array::from_fn(|i| pairs[i].as_ref().map_or(0.0, FieldPair::width));
        let count = fields.len().min(MAX_FIELDS);
        let (mut rows, mut width, mut row_h) = (0, 0.0_f32, 0.0_f32);
        let slots = flow(widths[..count].iter().copied(), max_width);
        for (pair, slot) in pairs.iter_mut().flatten().zip(slots) {
            pair.slot = slot;
            rows = slot.row + 1;
            width = width.max(slot.x + pair.width());
            row_h = row_h.max(pair.key.size().y.max(pair.value.size().y));
        }
        Self {
            pairs,
            rows,
            width,
            row_h,
        }
    }

    pub fn height(&self) -> f32 {
        let rows = self.rows as f32;
        (rows * self.row_h + (rows - 1.0) * ROW_GAP).max(0.0)
    }

    pub fn paint(&self, painter: &Painter, visuals: &Visuals, origin: Pos2) {
        let divider = Stroke::new(1.0, visuals.widgets.noninteractive.bg_stroke.color);
        for pair in self.pairs.iter().flatten() {
            let x = origin.x + pair.slot.x;
            let mid_y = origin.y + pair.slot.row as f32 * (self.row_h + ROW_GAP) + self.row_h * 0.5;
            if pair.slot.divided {
                let div_x = (x - ITEM_GAP - 0.5).round() + 0.5;
                let half = (self.row_h * 0.32).round();
                painter.vline(div_x, (mid_y - half)..=(mid_y + half), divider);
            }
            let (key, value) = (&pair.key, &pair.value);
            let key_pos = pos2(x, mid_y - key.size().y * 0.5);
            let val_pos = pos2(x + key.size().x + KEY_GAP, mid_y - value.size().y * 0.5);
            painter.galley(key_pos, Arc::clone(key), visuals.text_color());
            painter.galley(val_pos, Arc::clone(value), visuals.text_color());
        }
    }
}

/// Lays out a pair at its natural width; a pair wider than a whole row truncates, with
/// the label keeping its natural width or at least half the row, and the value the rest.
fn fit_pair(
    painter: &Painter,
    field: &HoverField,
    key_color: egui::Color32,
    val_color: egui::Color32,
    max_width: f32,
) -> (Arc<Galley>, Arc<Galley>) {
    let key = line(painter, &field.label, key_font(), key_color, f32::INFINITY);
    let value = line(
        painter,
        &field.value,
        field_font(),
        val_color,
        f32::INFINITY,
    );
    let (key_w, val_w) = (key.size().x, value.size().x);
    if key_w + KEY_GAP + val_w <= max_width {
        return (key, value);
    }
    let key_max = key_w.min((max_width * 0.5).max(max_width - KEY_GAP - val_w));
    let key = if key_w > key_max {
        line(painter, &field.label, key_font(), key_color, key_max)
    } else {
        key
    };
    let val_max = (max_width - KEY_GAP - key.size().x).max(0.0);
    let value = line(painter, &field.value, field_font(), val_color, val_max);
    (key, value)
}
