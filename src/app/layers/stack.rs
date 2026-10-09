//! The plotted layers, drawn in order. The base layer always exists and
//! decides the canvas (plot type, axes, colorbar, hover); overlays draw over it.

use super::{Layer, LayerId, Source};
use crate::data::BlockCacheKey;

pub struct LayerStack {
    pub base: Layer,
    overlays: Vec<Layer>,
    /// The id the next pushed layer gets.
    next_id: LayerId,
}

impl Default for LayerStack {
    fn default() -> Self {
        Self {
            base: Layer::new(LayerId::BASE, Source::default()),
            overlays: Vec::new(),
            next_id: LayerId::BASE.next(),
        }
    }
}

impl LayerStack {
    /// Adds an overlay drawn from `source` on top of the others, under a new id.
    pub fn push(&mut self, source: Source) -> LayerId {
        let id = self.next_id;
        self.next_id = id.next();
        self.overlays.push(Layer::new(id, source));
        id
    }

    /// Removes overlay `id` (the base layer stays); whether it existed.
    pub fn remove(&mut self, id: LayerId) -> bool {
        let before = self.overlays.len();
        self.overlays.retain(|layer| layer.id() != id);
        self.overlays.len() != before
    }

    /// Moves overlay `id` one place up (drawn later, on top) or down (drawn
    /// earlier); the base layer always draws first. Whether it moved.
    pub fn move_overlay(&mut self, id: LayerId, up: bool) -> bool {
        let Some(i) = self.overlays.iter().position(|l| l.id() == id) else {
            return false;
        };
        let j = if up { i + 1 } else { i.wrapping_sub(1) };
        if j >= self.overlays.len() {
            return false;
        }
        self.overlays.swap(i, j);
        true
    }

    /// The overlays in drawing order.
    pub fn overlays(&self) -> &[Layer] {
        &self.overlays
    }

    /// Every layer in drawing order.
    pub fn iter(&self) -> impl Iterator<Item = &Layer> {
        std::iter::once(&self.base).chain(&self.overlays)
    }

    /// Every layer in drawing order, mutably.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Layer> {
        std::iter::once(&mut self.base).chain(&mut self.overlays)
    }

    /// Every layer's id in drawing order, for loops that change the app.
    pub fn ids(&self) -> Vec<LayerId> {
        self.iter().map(Layer::id).collect()
    }

    /// The overlays' ids in drawing order.
    pub fn overlay_ids(&self) -> Vec<LayerId> {
        self.overlays.iter().map(Layer::id).collect()
    }

    /// The layer `id`, while it exists.
    pub fn get(&self, id: LayerId) -> Option<&Layer> {
        self.iter().find(|layer| layer.id() == id)
    }

    /// Overlay `id` mutably, or the base layer when there is no such overlay.
    pub fn get_or_base_mut(&mut self, id: LayerId) -> &mut Layer {
        match self.overlays.iter_mut().find(|layer| layer.id() == id) {
            Some(layer) => layer,
            None => &mut self.base,
        }
    }

    /// The layer `id`, mutably, while it exists.
    pub fn get_mut(&mut self, id: LayerId) -> Option<&mut Layer> {
        self.iter_mut().find(|layer| layer.id() == id)
    }

    /// The layer whose pending request is the block `key`.
    pub fn find_by_key(&self, key: &BlockCacheKey) -> Option<LayerId> {
        self.iter()
            .find(|layer| layer.load.block_key.as_ref() == Some(key))
            .map(Layer::id)
    }
}
