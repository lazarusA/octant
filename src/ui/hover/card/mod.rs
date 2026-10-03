//! Hover card: measured once, placed inside the canvas, then painted with its leader on a
//! single tooltip layer so the leader always meets the card edge.

pub mod flow;
pub mod layout;
pub mod model;
pub mod paint;
pub mod place;
#[cfg(test)]
mod sheet;
#[cfg(test)]
mod tests;

pub use model::{HoverCard, HoverValue};

use crate::ui::hover::callout::draw_leader_callout;
use egui::{Id, LayerId, Order, Pos2, Rect};
use layout::CardLayout;
use place::{Placement, Side, place_connected, place_following};

/// How the card positions itself relative to the hover.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Anchoring {
    /// Beside the data point, joined by a leader (maps, surfaces, volumes).
    Connected,
    /// Trailing the pointer (line plots, which draw their own guides).
    FollowPointer,
}

/// Paints the card for one frame and returns where it was placed.
pub fn show_card(
    ctx: &egui::Context,
    canvas: Rect,
    hover_pos: Pos2,
    target: Option<Pos2>,
    anchoring: Anchoring,
    card: &HoverCard,
) -> Rect {
    let painter = ctx.layer_painter(LayerId::new(Order::Tooltip, Id::new("octant_hover_card")));
    let visuals = &ctx.style_of(ctx.theme()).visuals;

    let mut buf = [0u8; 32];
    let value = card.value.format(&mut buf);
    let layout = CardLayout::measure(&painter, visuals, card, value);

    let bounds = canvas.intersect(ctx.input(|i| i.viewport_rect()));
    let placement = match (anchoring, target) {
        (Anchoring::Connected, Some(t)) => place_connected(t, layout.size, bounds),
        _ => {
            let rect = place_following(hover_pos, layout.size, bounds);
            let side = Side::facing(rect, target.unwrap_or(hover_pos));
            Placement { rect, side }
        }
    };

    if let Some(t) = target {
        draw_leader_callout(&painter, visuals, t, &placement);
    }
    paint::paint_card(&painter, visuals, card, value, &layout, placement.rect);
    placement.rect
}
