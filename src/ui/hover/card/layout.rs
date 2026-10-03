//! Card typography, spacing, and the one-pass layout that sizes and feeds the painter.

use crate::ui::hover::card::flow::FieldRows;
use crate::ui::hover::card::model::{HoverCard, HoverValue};
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

/// Narrowest units worth showing; below this they are dropped rather than ellipsized.
const MIN_UNITS_WIDTH: f32 = 12.0;

/// Every galley the card paints, laid out once per frame, plus the card size they imply.
pub struct CardLayout {
    pub size: Vec2,
    pub title: Arc<Galley>,
    pub value: Arc<Galley>,
    pub units: Option<Arc<Galley>>,
    /// Height of the value row (the taller of the value text and the color chip).
    pub value_h: f32,
    pub fields: FieldRows,
}

impl CardLayout {
    pub fn new(painter: &Painter, visuals: &Visuals, card: &HoverCard, value: &str) -> Self {
        // Everything is laid out at the widest card; the card then shrinks to its content,
        // which keeps every truncation and row break valid.
        let max_inner = MAX_WIDTH - 2.0 * PAD.x;
        let title = line(
            painter,
            card.title,
            title_font(),
            visuals.strong_text_color(),
            max_inner,
        );
        let (value, units) =
            value_galleys(painter, visuals, card, value, max_inner - CHIP - CHIP_GAP);
        let fields = FieldRows::layout(painter, visuals, card.fields, max_inner);

        let units_w = units.as_ref().map_or(0.0, |u| UNITS_GAP + u.size().x);
        let value_w = CHIP + CHIP_GAP + value.size().x + units_w;
        let content_w = title.size().x.max(value_w).max(fields.width);
        // Round up so painted text never truncates by a sub-pixel.
        let width = (content_w + 2.0 * PAD.x).ceil().clamp(MIN_WIDTH, MAX_WIDTH);

        let value_h = value.size().y.max(CHIP);
        let fields_h = if fields.rows == 0 {
            0.0
        } else {
            2.0 * SECTION_GAP + 1.0 + fields.height()
        };
        let height = 2.0 * PAD.y + title.size().y + TITLE_GAP + value_h + fields_h;

        Self {
            size: vec2(width, height.ceil()),
            title,
            value,
            units,
            value_h,
            fields,
        }
    }
}

/// The headline value and, when it is a number and there is room, its units.
fn value_galleys(
    painter: &Painter,
    visuals: &Visuals,
    card: &HoverCard,
    text: &str,
    max_width: f32,
) -> (Arc<Galley>, Option<Arc<Galley>>) {
    let muted = muted_color(visuals);
    let color = if card.value == HoverValue::NoData {
        muted
    } else {
        visuals.strong_text_color()
    };
    let value = line(painter, text, value_font(), color, max_width);
    let room = max_width - value.size().x - UNITS_GAP;
    let units = (card.value.shows_units() && !card.units.is_empty() && room >= MIN_UNITS_WIDTH)
        .then(|| line(painter, card.units, units_font(), muted, room));
    (value, units)
}
