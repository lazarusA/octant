//! The plotted layers, drawn in order. The base layer always exists and
//! decides the canvas (plot type, axes, colorbar, hover); overlays draw over it.

use super::{Layer, LayerId, Slot, Source};
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
            base: Layer::new(LayerId::BASE, Source::default(), Slot::Bottom(0)),
            overlays: Vec::new(),
            next_id: LayerId::BASE.next(),
        }
    }
}

impl LayerStack {
    /// Adds an overlay drawn from `source` on top of the others, under a new
    /// id, its colorbar in the first free slot.
    pub fn push(&mut self, source: Source) -> LayerId {
        let id = self.next_id;
        self.next_id = id.next();
        let taken: Vec<Slot> = self.iter().map(|layer| layer.colorbar.slot).collect();
        let slot = Slot::first_free(&taken);
        self.overlays.push(Layer::new(id, source, slot));
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

    /// The ids of the layers drawn on the canvas (`Layer::is_drawn`), in
    /// drawing order: hidden or unaligned overlays load and prefetch nothing.
    pub fn drawn_ids(&self) -> Vec<LayerId> {
        self.iter()
            .filter(|layer| layer.is_drawn())
            .map(Layer::id)
            .collect()
    }

    /// The overlays' ids in drawing order.
    pub fn overlay_ids(&self) -> Vec<LayerId> {
        self.overlays.iter().map(Layer::id).collect()
    }

    /// The layer `id`, while it exists.
    pub fn get(&self, id: LayerId) -> Option<&Layer> {
        self.iter().find(|layer| layer.id() == id)
    }

    /// The base layer and overlay `id` (mutably) together, while it exists.
    pub fn base_and_overlay_mut(&mut self, id: LayerId) -> Option<(&Layer, &mut Layer)> {
        let overlay = self.overlays.iter_mut().find(|layer| layer.id() == id)?;
        Some((&self.base, overlay))
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
