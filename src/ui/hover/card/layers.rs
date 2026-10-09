//! One row per overlay under the headline value: its color chip, its value
//! with units, and its name, laid out once per frame.

use crate::app::overlays::MAX_OVERLAYS;
use crate::ui::hover::card::layout::{CHIP_GAP, field_font, key_font, line, muted_color};
use crate::ui::hover::card::model::HoverValue;
use egui::text::{LayoutJob, TextFormat, TextWrapping};
use egui::{Color32, Galley, Painter, Pos2, Rect, Visuals, pos2, vec2};
use std::sync::Arc;

/// Side of an overlay row's color chip.
pub const ROW_CHIP: f32 = 10.0;
/// Space between overlay rows.
pub const ROW_GAP: f32 = 4.0;
/// Space between an overlay's value and its name.
const NAME_GAP: f32 = 10.0;

/// One overlay's reading at the hovered cell.
#[derive(Clone, Copy)]
pub struct LayerValue<'a> {
    pub name: &'a str,
    pub value: HoverValue,
    pub units: &'a str,
    pub swatch: Color32,
}

impl LayerValue<'_> {
    /// An unused slot of a fixed row array.
    pub const EMPTY: LayerValue<'static> = LayerValue {
        name: "",
        value: HoverValue::NoData,
        units: "",
        swatch: Color32::TRANSPARENT,
    };
}

struct LayerRow {
    value: Arc<Galley>,
    name: Arc<Galley>,
    swatch: Color32,
    no_data: bool,
}

/// The laid-out overlay rows and the size they take.
pub struct LayerRows {
    rows: [Option<LayerRow>; MAX_OVERLAYS],
    pub count: usize,
    pub width: f32,
    pub row_h: f32,
}

impl LayerRows {
    /// Lays out up to [`MAX_OVERLAYS`] rows of `layers` within `max_width`.
    pub fn layout(
        painter: &Painter,
        visuals: &Visuals,
        layers: &[LayerValue],
        max_width: f32,
    ) -> Self {
        let mut rows: [Option<LayerRow>; MAX_OVERLAYS] = Default::default();
        let (mut count, mut width, mut row_h) = (0, 0.0_f32, 0.0_f32);
        let room = max_width - ROW_CHIP - CHIP_GAP;
        for (slot, layer) in rows.iter_mut().zip(layers) {
            let value = value_galley(painter, visuals, layer, room * 0.6);
            let name_room = (room - value.size().x - NAME_GAP).max(0.0);
            let muted = muted_color(visuals);
            let name = line(painter, layer.name, key_font(), muted, name_room);
            let row_w = ROW_CHIP + CHIP_GAP + value.size().x + NAME_GAP + name.size().x;
            width = width.max(row_w);
            row_h = row_h.max(value.size().y.max(name.size().y).max(ROW_CHIP));
            let no_data = layer.value == HoverValue::NoData;
            *slot = Some(LayerRow {
                value,
                name,
                swatch: layer.swatch,
                no_data,
            });
            count += 1;
        }
        Self {
            rows,
            count,
            width,
            row_h,
        }
    }

    /// Height of every row with the gaps between them.
    pub fn height(&self) -> f32 {
        if self.count == 0 {
            return 0.0;
        }
        self.count as f32 * self.row_h + (self.count - 1) as f32 * ROW_GAP
    }

    /// Paints the rows from `origin`, names right-aligned at `right`.
    pub fn paint(&self, painter: &Painter, visuals: &Visuals, origin: Pos2, right: f32) {
        for (i, row) in self.rows.iter().flatten().enumerate() {
            let mid_y = origin.y + i as f32 * (self.row_h + ROW_GAP) + self.row_h * 0.5;
            let chip_center = pos2(origin.x + ROW_CHIP * 0.5, mid_y);
            let chip = Rect::from_center_size(chip_center, vec2(ROW_CHIP, ROW_CHIP));
            super::paint::paint_chip(painter, visuals, chip, row.swatch, row.no_data);
            let value_pos = pos2(chip.right() + CHIP_GAP, mid_y - row.value.size().y * 0.5);
            painter.galley(value_pos, Arc::clone(&row.value), visuals.text_color());
            let name_pos = pos2(right - row.name.size().x, mid_y - row.name.size().y * 0.5);
            painter.galley(name_pos, Arc::clone(&row.name), visuals.text_color());
        }
    }
}

/// The value in the field font, then its units muted, truncated at `max_width`.
fn value_galley(
    painter: &Painter,
    visuals: &Visuals,
    layer: &LayerValue,
    max_width: f32,
) -> Arc<Galley> {
    let muted = muted_color(visuals);
    let mut buf = [0u8; 32];
    let text = layer.value.format(&mut buf);
    let color = if layer.value == HoverValue::NoData {
        muted
    } else {
        visuals.strong_text_color()
    };
    let mut job = LayoutJob::default();
    job.append(text, 0.0, TextFormat::simple(field_font(), color));
    if layer.value.shows_units() && !layer.units.is_empty() {
        job.append(layer.units, 4.0, TextFormat::simple(field_font(), muted));
    }
    job.wrap = TextWrapping::truncate_at_width(max_width);
    painter.layout_job(job)
}
