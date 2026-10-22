//! An overlay's channel range: overlays have no band slider, so the bands
//! they read follow their composite, unless the base shares their band
//! dimension, whose slider then picks them for both.

use crate::app::OctantApp;
use crate::app::layers::{LayerId, dim_named};

impl OctantApp {
    /// Overlay `id`'s channel axis when it sets its own range: a named or OME
    /// channel axis (never `layer_channel_dim`'s dimension-0 fallback) that
    /// the base lacks. A shared one follows the base's slider like any
    /// dimension (`follow_base_window`).
    pub(crate) fn overlay_own_channel(&self, id: LayerId) -> Option<usize> {
        if id == LayerId::BASE {
            return None;
        }
        let var = self.layer_variable_info(id)?;
        let c = var.channel_axis()?;
        let name = var.dimension_names.get(c)?;
        let shared = self
            .plotted_variable_info()
            .is_some_and(|base| dim_named(base, name).is_some());
        (!shared).then_some(c)
    }

    /// Layer `id`'s selected channel range (an overlay's shared band follows
    /// the base's slider), or every channel without a channel dimension and
    /// for an overlay whose channels follow its composite.
    pub(crate) fn layer_channel_range(&self, id: LayerId) -> (usize, usize) {
        match self.layer_channel_dim(id) {
            Some(c) if self.overlay_own_channel(id).is_none() => self.layer_dim_range(id, c),
            _ => (0, usize::MAX),
        }
    }

    /// Sets overlay `id`'s channel range to the bands its composite draws
    /// (CMYK's inks, the ticked channels or the R/G/B bands), or to its one
    /// band while the composite is off, when it owns that axis
    /// (`overlay_own_channel`).
    pub(crate) fn fit_overlay_channels(&mut self, id: LayerId) {
        let (Some(c), Some(var)) = (self.overlay_own_channel(id), self.layer_variable_info(id))
        else {
            return;
        };
        let last = var
            .shape
            .get(c)
            .map_or(0, |&n| (n as usize).saturating_sub(1));
        let cmyk = self.layer_is_cmyk(id);
        let Some(layer) = self.layers.get_mut(id) else {
            return;
        };
        let composite = &layer.composite;
        let index = layer.selection().dim_indices.get(c).copied().unwrap_or(0);
        let ticked = composite.channel_configs.iter().filter(|c| c.visible);
        let range = if !composite.enabled {
            Some((index, index))
        } else if cmyk {
            Some((0, last))
        } else if !composite.channel_configs.is_empty() {
            span(ticked.map(|c| c.index))
        } else {
            span(composite.rgb_channels.into_iter())
        };
        let Some((lo, hi)) = range else {
            return;
        };
        if let Some(r) = layer.selection_mut().dim_ranges.get_mut(c) {
            *r = (lo.min(last), hi.min(last));
        }
    }
}

/// The smallest and largest of `bands`.
fn span(bands: impl Iterator<Item = usize>) -> Option<(usize, usize)> {
    bands.fold(None, |acc, b| match acc {
        None => Some((b, b)),
        Some((lo, hi)) => Some((lo.min(b), hi.max(b))),
    })
}
