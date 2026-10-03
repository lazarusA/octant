//! Card typography, spacing, and exact size measurement ahead of painting.

use crate::ui::hover::card::flow::{ROW_GAP, flow, pair_metrics, pair_width};
use crate::ui::hover::card::model::HoverCard;
use egui::text::{LayoutJob, TextWrapping};
use egui::{Color32, FontId, Galley, Painter, Vec2, Visuals, vec2};
use std::sync::Arc;

pub const PAD: Vec2 = vec2(12.0, 10.0);
pub const MIN_WIDTH: f32 = 176.0;
pub const MAX_WIDTH: f32 = 300.0;
pub const CORNER_RADIUS: u8 = 8;
pub const CHIP: f32 = 14.0;
pub const CHIP_GAP: f32 = 8.0;
pub const UNITS_GAP: f32 = 6.0;
pub const TITLE_GAP: f32 = 4.0;
pub const SECTION_GAP: f32 = 8.0;

pub fn title_font() -> FontId {
    FontId::proportional(13.0)
}
pub fn value_font() -> FontId {
    FontId::proportional(20.0)
}
/// Same size as the value so the reading scans as one quantity; only the color differs.
pub fn units_font() -> FontId {
    value_font()
}
pub fn key_font() -> FontId {
    FontId::proportional(11.5)
}
pub fn field_font() -> FontId {
    FontId::proportional(12.0)
}

/// Secondary text: the body color pulled toward the card fill, so it stays opaque
/// and readable in both themes.
pub fn muted_color(visuals: &Visuals) -> Color32 {
    visuals
        .text_color()
        .lerp_to_gamma(visuals.window_fill, 0.35)
}

/// Single-line text, truncated with an ellipsis beyond `max_width`.
pub fn line(
    painter: &Painter,
    text: &str,
    font: FontId,
    color: Color32,
    max_width: f32,
) -> Arc<Galley> {
    let mut job = LayoutJob::simple_singleline(text.to_owned(), font, color);
    job.wrap = TextWrapping::truncate_at_width(max_width);
    painter.layout_job(job)
}

/// Exact card dimensions, computed from the same galleys the painter draws.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CardLayout {
    pub size: Vec2,
    pub title_h: f32,
    pub value_h: f32,
    /// Height of one row of coordinate pairs.
    pub field_h: f32,
}

impl CardLayout {
    pub fn measure(painter: &Painter, visuals: &Visuals, card: &HoverCard, value: &str) -> Self {
        let color = visuals.text_color();
        let natural =
            |text: &str, font: FontId| line(painter, text, font, color, f32::INFINITY).size();

        let title = natural(card.title, title_font());
        let value_size = natural(value, value_font());
        let units_w = if card.value.shows_units() && !card.units.is_empty() {
            UNITS_GAP + natural(card.units, units_font()).x
        } else {
            0.0
        };

        let header_w = title.x;
        let value_w = CHIP + CHIP_GAP + value_size.x + units_w;

        // Flow the pairs at the widest card, then shrink to the widest row they produced:
        // every row still fits, so the breaks stay where they are.
        let max_inner = MAX_WIDTH - 2.0 * PAD.x;
        let widths = || {
            card.fields
                .iter()
                .map(|f| pair_width(pair_metrics(painter, visuals, f)).min(max_inner))
        };
        let (mut rows_w, mut rows) = (0.0_f32, 0.0_f32);
        for (slot, w) in flow(widths(), max_inner).zip(widths()) {
            rows_w = rows_w.max(slot.x + w);
            rows = (slot.row + 1) as f32;
        }
        let field_h = card
            .fields
            .iter()
            .map(|f| pair_metrics(painter, visuals, f).2)
            .fold(0.0_f32, f32::max);

        let content_w = header_w.max(value_w).max(rows_w);
        // Round up so the painted rows never truncate by a sub-pixel.
        let width = (content_w + 2.0 * PAD.x).ceil().clamp(MIN_WIDTH, MAX_WIDTH);
        let value_h = value_size.y.max(CHIP);
        let grid_h = if rows == 0.0 {
            0.0
        } else {
            2.0 * SECTION_GAP + 1.0 + rows * field_h + (rows - 1.0) * ROW_GAP
        };
        let height = 2.0 * PAD.y + title.y + TITLE_GAP + value_h + grid_h;

        Self {
            size: vec2(width, height.round()),
            title_h: title.y,
            value_h,
            field_h,
        }
    }

    /// Width available for content between the side paddings.
    pub fn inner_width(&self) -> f32 {
        self.size.x - 2.0 * PAD.x
    }
}
