//! An overlay's channel range: overlays have no band slider, so the bands
//! they read follow their composite.

use crate::app::OctantApp;
use crate::app::layers::LayerId;

impl OctantApp {
    /// Sets overlay `id`'s channel range to the bands its composite draws
    /// (CMYK's inks, the ticked channels or the R/G/B bands), or to its one
    /// band while the composite is off. The base layer's follows its slider.
    pub(crate) fn fit_overlay_channels(&mut self, id: LayerId) {
        if id == LayerId::BASE {
            return;
        }
        // A named or OME channel axis only, never the dimension-0 fallback
        // (`layer_channel_dim`), and not one the base has: that follows the base.
        let base_has = |name: &str| {
            self.plotted_variable_info().is_some_and(|b| {
                b.dimension_names
                    .iter()
                    .any(|n| n.eq_ignore_ascii_case(name))
            })
        };
        let Some(var) = self.layer_variable_info(id) else {
            return;
        };
        let Some(c) = var
            .channel_axis()
            .filter(|&c| !var.dimension_names.get(c).is_some_and(|n| base_has(n)))
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
