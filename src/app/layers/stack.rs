//! The plotted layers, drawn in order. The base layer always exists and
//! decides the canvas (plot type, axes, colorbar, hover).

use super::Layer;
use crate::data::BlockCacheKey;

#[derive(Default)]
pub struct LayerStack {
    pub base: Layer,
}

impl LayerStack {
    /// Every layer in drawing order.
    pub fn iter(&self) -> impl Iterator<Item = &Layer> {
        std::iter::once(&self.base)
    }

    /// The layer whose pending request is the block `key`.
    pub fn find_by_key(&self, key: &BlockCacheKey) -> Option<&Layer> {
        self.iter()
            .find(|layer| layer.load.block_key.as_ref() == Some(key))
    }

    /// Every layer in drawing order, mutably.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Layer> {
        std::iter::once(&mut self.base)
    }
}
