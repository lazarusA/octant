//! Stable layer identity, for routing requests and keying per-layer UI state.

/// Identifies a layer for as long as it lives in the `LayerStack`, which
/// hands out every id: there is no `Default`, so a layer cannot take the
/// base layer's id by accident.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LayerId(u32);

impl LayerId {
    /// The base layer, which always exists and decides the canvas.
    pub const BASE: Self = Self(0);

    /// The id handed out after this one.
    pub(super) fn next(self) -> Self {
        Self(self.0.wrapping_add(1))
    }
}
