//! Hover card: laid out once, placed inside the canvas, then painted with its leader on a
//! single tooltip layer so the leader always meets the card edge.

pub mod flow;
pub mod layout;
#[cfg(test)]
mod layout_tests;
pub mod leader;
pub mod model;
pub mod paint;
pub mod place;
#[cfg(test)]
mod sheet;
#[cfg(test)]
mod tests;

pub use model::{HoverCard, HoverValue};

use egui::{Id, LayerId, Order, Pos2, Rect};
use layout::CardLayout;
use place::{card_bounds, place_connected, place_following};

/// How the card positions itself relative to the hover.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Anchoring {
    /// Beside the data point, joined by a leader (maps, surfaces, volumes).
    Connected,
    /// Trailing the pointer (line plots, which draw their own guides).
    FollowPointer,
}

/// Paints the card for one frame.
pub fn show_card(
    ctx: &egui::Context,
    canvas: Rect,
    hover_pos: Pos2,
    target: Option<Pos2>,
    anchoring: Anchoring,
    card: &HoverCard,
) {
    let painter = ctx.layer_painter(LayerId::new(Order::Tooltip, Id::new("octant_hover_card")));
    let visuals = &ctx.style_of(ctx.theme()).visuals;

    let mut buf = [0u8; 32];
    let layout = CardLayout::new(&painter, visuals, card, card.value.format(&mut buf));

    let viewport = ctx.input(|i| i.viewport_rect());
    let bounds = card_bounds(canvas, viewport, layout.size);
    // The tooltip layer is unclipped: a target off the visible canvas (such as the centre
    // of a zoomed-in cell) gets no leader, so it cannot draw over other panels.
    let visible = canvas.intersect(viewport);
    let rect = match (anchoring, target.filter(|t| visible.contains(*t))) {
        (Anchoring::Connected, Some(t)) => {
            let placement = place_connected(t, layout.size, bounds);
            leader::draw_leader(&painter, visuals, t, &placement);
            placement.rect
        }
        _ => place_following(hover_pos, layout.size, bounds),
    };
    paint::paint_card(&painter, visuals, card, &layout, rect);
}
