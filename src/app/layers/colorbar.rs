//! Where a layer's colorbar sits on the canvas: its default slot, its
//! orientation and, once dragged, its own position. Kept for the session only.

use egui::Pos2;

/// A default spot on the canvas. The base layer's colorbar is `Bottom(0)`;
/// overlays take the first free slot of `OVERLAY_SLOTS`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    /// Centered above the bottom edge, `n` panels up.
    Bottom(u8),
    Top,
    Right,
    Left,
}

/// The order overlays fill the canvas edges in.
pub const OVERLAY_SLOTS: [Slot; 4] = [Slot::Top, Slot::Right, Slot::Left, Slot::Bottom(1)];

impl Slot {
    /// The first slot of `OVERLAY_SLOTS` that no slot of `taken` holds; when
    /// every one is taken, the next bottom slot up.
    pub fn first_free(taken: &[Slot]) -> Slot {
        let free = |slot: &Slot| !taken.contains(slot);
        if let Some(slot) = OVERLAY_SLOTS.into_iter().find(free) {
            return slot;
        }
        (2..=u8::MAX)
            .map(Slot::Bottom)
            .find(free)
            .unwrap_or(Slot::Bottom(u8::MAX))
    }

    /// The orientation the slot's colorbar starts in: vertical on the sides.
    pub fn default_orientation(self) -> BarOrientation {
        match self {
            Slot::Left | Slot::Right => BarOrientation::Vertical,
            Slot::Top | Slot::Bottom(_) => BarOrientation::Horizontal,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarOrientation {
    Horizontal,
    Vertical,
}

impl BarOrientation {
    pub fn flipped(self) -> Self {
        match self {
            Self::Horizontal => Self::Vertical,
            Self::Vertical => Self::Horizontal,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorbarPlacement {
    pub slot: Slot,
    pub orientation: BarOrientation,
    /// The dragged panel's pivot point as a fraction of the canvas (so it
    /// keeps its place when the canvas resizes); `None` sits in `slot`.
    pub pos: Option<Pos2>,
}

impl ColorbarPlacement {
    /// The undragged placement of `slot`, in its default orientation.
    pub fn at(slot: Slot) -> Self {
        Self {
            slot,
            orientation: slot.default_orientation(),
            pos: None,
        }
    }
}
