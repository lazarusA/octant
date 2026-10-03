//! Card placement next to the hovered point, kept inside the canvas.

use egui::{Pos2, Rect, Vec2, pos2};

/// Horizontal run of the leader arm, from the hovered point to the card edge.
pub const LEADER_GAP: f32 = 32.0;
/// Vertical clearance between the hovered point and the card, so the arm bends.
pub const ARM_RISE: f32 = 18.0;
/// Offset of a cursor-following card (line plots) from the pointer.
pub const FOLLOW_OFFSET: f32 = 16.0;
/// Minimum clearance between the card and the canvas border.
pub const EDGE_MARGIN: f32 = 8.0;

/// Which side of the target the card sits on; decides the leader's anchor edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Right,
    Left,
    Below,
    Above,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placement {
    pub rect: Rect,
    pub side: Side,
}

impl Side {
    /// Side of `target` that `rect` lies on, preferring horizontal sides.
    pub fn facing(rect: Rect, target: Pos2) -> Self {
        if rect.left() >= target.x {
            Self::Right
        } else if rect.right() <= target.x {
            Self::Left
        } else if rect.top() >= target.y {
            Self::Below
        } else {
            Self::Above
        }
    }
}

/// Region the card is kept inside: the visible canvas when the card fits in it with
/// margins, otherwise the whole viewport, so a card never spills off screen.
pub fn card_bounds(canvas: Rect, viewport: Rect, size: Vec2) -> Rect {
    let visible = canvas.intersect(viewport);
    let room = size + Vec2::splat(2.0 * EDGE_MARGIN);
    if visible.width() >= room.x && visible.height() >= room.y {
        visible
    } else {
        viewport
    }
}

/// Places a card connected to `target` by an elbow arm: diagonally off the point, toward
/// the canvas centre, flipping per axis when that side has no room. Falls back to
/// directly below or above when neither horizontal side fits.
pub fn place_connected(target: Pos2, size: Vec2, bounds: Rect) -> Placement {
    let inner = bounds.shrink(EDGE_MARGIN);
    let centre = inner.center();

    let right = target.x + LEADER_GAP;
    let left = target.x - LEADER_GAP - size.x;
    let fits_x = |x: f32| x >= inner.left() && x + size.x <= inner.right();
    let below = target.y + ARM_RISE;
    let above = target.y - ARM_RISE - size.y;
    let fits_y = |y: f32| y >= inner.top() && y + size.y <= inner.bottom();

    let (x_first, x_second) = if target.x <= centre.x {
        (right, left)
    } else {
        (left, right)
    };
    let (y_first, y_second) = if target.y <= centre.y {
        (below, above)
    } else {
        (above, below)
    };

    // Without vertical room the card centres on the point and the arm runs straight.
    let y = [y_first, y_second]
        .into_iter()
        .find(|&y| fits_y(y))
        .unwrap_or(target.y - size.y * 0.5);
    let min = match [x_first, x_second].into_iter().find(|&x| fits_x(x)) {
        Some(x) => pos2(x, y),
        None => pos2(target.x - size.x * 0.5, y),
    };
    let rect = clamp_into(Rect::from_min_size(min, size), inner);
    Placement {
        rect,
        side: Side::facing(rect, target),
    }
}

/// Places a card trailing the pointer, flipping to the other side near the canvas edges.
pub fn place_following(cursor: Pos2, size: Vec2, bounds: Rect) -> Rect {
    let inner = bounds.shrink(EDGE_MARGIN);
    let mut min = cursor + Vec2::splat(FOLLOW_OFFSET);
    if min.x + size.x > inner.right() {
        min.x = cursor.x - FOLLOW_OFFSET - size.x;
    }
    if min.y + size.y > inner.bottom() {
        min.y = cursor.y - FOLLOW_OFFSET - size.y;
    }
    clamp_into(Rect::from_min_size(min, size), inner)
}

/// Shifts `rect` inside `bounds`; an oversized card stays pinned to the top-left.
fn clamp_into(rect: Rect, bounds: Rect) -> Rect {
    let x = rect
        .min
        .x
        .min(bounds.right() - rect.width())
        .max(bounds.left());
    let y = rect
        .min
        .y
        .min(bounds.bottom() - rect.height())
        .max(bounds.top());
    Rect::from_min_size(pos2(x, y), rect.size())
}
