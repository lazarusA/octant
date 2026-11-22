//! Composite volumes from bricks: a brick's read spans the channel dimension as the plotted one does, so its
//! channels arrive together and never show at different levels; it is
//! blended on the CPU into RGBA8 with the stretches the plotted level was
//! drawn with (recorded once from its block), so bricks of every level share
//! the plot's colors.

use crate::app::OctantApp;
use crate::app::block_loading::block_axes;
use crate::app::detail::SETTLE;
use crate::app::layers::LayerId;
use crate::data::OctantBlock;
use crate::data::VolumeData;
use crate::data::slicing::composite::{
    ChannelColorConfig, CompositeStretch, slice_multichannel_volume_stretched,
    slice_rgb_volume_stretched,
};

/// How a plot's composite colors its bricks.
pub(crate) struct CompositeLod {
    c_dim: usize,
    blend: Blend,
    stretch: CompositeStretch,
}

enum Blend {
    /// Tinted channels added up.
    Channels(Vec<ChannelColorConfig>),
    /// Channels shown as red, green and blue (or CMYK inks).
    Rgb([Option<usize>; 3]),
}

type Ranges = ((usize, usize), (usize, usize), (usize, usize));

impl CompositeLod {
    /// Tinted `configs` over channel dimension `c_dim`, stretches recorded
    /// over `block` along `(x, y, z)` dimensions `dims`.
    #[cfg(test)]
    pub(super) fn channels(
        c_dim: usize,
        configs: Vec<ChannelColorConfig>,
        block: &OctantBlock,
        dims: (usize, usize, usize),
    ) -> Self {
        let mut lod = Self {
            c_dim,
            blend: Blend::Channels(configs),
            stretch: CompositeStretch::default(),
        };
        let whole = |d: usize| (0, block.shape.get(d).copied().unwrap_or(1));
        let ranges = (whole(dims.0), whole(dims.1), whole(dims.2));
        let fixed = vec![0; block.shape.len()];
        let mut stretch = CompositeStretch::default();
        if let Blend::Channels(c) = &lod.blend {
            let d = (c_dim, dims.0, dims.1, dims.2);
            let _ = slice_multichannel_volume_stretched(block, d, ranges, &fixed, c, &mut stretch);
        }
        lod.stretch = stretch;
        lod
    }

    /// Packed colors of `block` over `ranges` of its `(x, y, z)` dimensions,
    /// as the plotted composite volume holds them.
    pub(super) fn slice(
        &self,
        block: &OctantBlock,
        (x, y, z): (usize, usize, usize),
        ranges: Ranges,
        fixed: &[usize],
    ) -> Option<VolumeData> {
        let dims = (self.c_dim, x, y, z);
        let stretch = &mut self.stretch.clone();
        match &self.blend {
            Blend::Channels(c) => {
                slice_multichannel_volume_stretched(block, dims, ranges, fixed, c, stretch)
            }
            Blend::Rgb(ch) => slice_rgb_volume_stretched(block, dims, ranges, fixed, *ch, stretch),
        }
    }
}

impl OctantApp {
    /// The block the base layer's volume was sliced from, while cached: its
    /// latest request's (`load.block_key` is cleared once a block is
    /// accepted, `load.slice_request` stays), else the one still pending.
    pub(super) fn plotted_block(&self) -> Option<&OctantBlock> {
        let [requested, pending] = self.plotted_block_keys();
        requested
            .iter()
            .chain(pending.iter())
            .find_map(|k| self.block_cache.peek(k))
    }

    /// The cache keys `plotted_block` looks under: the latest request's,
    /// then the pending one.
    fn plotted_block_keys(&self) -> [Option<crate::data::BlockCacheKey>; 2] {
        let base = &self.layers.base;
        let requested = base.load.slice_request.as_ref().map(|slice| {
            let source_id = self.selection_source_id(base.selection());
            crate::data::BlockCacheKey::new(source_id, slice)
        });
        [requested, base.load.block_key.clone()]
    }

    /// Marks the plotted block used (`BlockCache::touch`), for a composite.
    pub(super) fn keep_plotted_block(&mut self) {
        if !self.layers.base.composite.enabled {
            return;
        }
        for key in self.plotted_block_keys().iter().flatten() {
            self.block_cache.touch(key);
        }
    }

    /// While the plotted block is missing: waits for it when a switch or a
    /// pending read will bring it; else nothing will (it left the cache),
    /// so the plot is rebuilt without bricks, in its new colors, instead of
    /// polling for good.
    fn wait_for_plotted_block(&mut self, toggled: bool, colors: u64, now: f64) -> Option<f64> {
        let prefetcher = &self.block_prefetcher;
        let keys = self.plotted_block_keys();
        let reading = keys.iter().flatten().any(|k| prefetcher.is_pending(k));
        let state = &mut self.layers.base.data.volume_lod;
        if toggled || reading {
            state.colors.get_or_insert((colors, now));
            return Some(SETTLE);
        }
        let context = state.context;
        self.reset_volume_lod(context, None);
        None
    }

    /// The base layer's composite as bricks blend it, its stretches recorded
    /// over the plotted block (`Err` when that block left the cache); `None`
    /// when the base layer is no composite.
    pub(super) fn composite_lod(&self, dims: [usize; 3]) -> Option<Result<CompositeLod, ()>> {
        let base = &self.layers.base;
        if !base.composite.enabled {
            return None;
        }
        let Some(c_dim) = self.layer_channel_dim(LayerId::BASE) else {
            return Some(Err(()));
        };
        let blend = match !self.layer_is_geotiff(LayerId::BASE)
            && !base.composite.channel_configs.is_empty()
        {
            true => Blend::Channels(base.composite.channel_configs.clone()),
            false => Blend::Rgb(base.composite.rgb_channels.map(Some)),
        };
        let mut lod = CompositeLod {
            c_dim,
            blend,
            stretch: CompositeStretch::default(),
        };
        let Some(block) = self.plotted_block() else {
            return Some(Err(()));
        };
        let whole = |d: usize| (0, block.shape.get(d).copied().unwrap_or(1));
        let ranges = (whole(dims[0]), whole(dims[1]), whole(dims[2]));
        // At the plotted step, as the bricks read it (`encode.rs`): a block
        // holding several steps would give another step's percentiles.
        let plotted = self.plotted();
        let var = plotted.variable_info()?;
        let step = self.layer_step(LayerId::BASE);
        let anim = plotted.animated_dim();
        let fixed =
            block_axes::fixed_indices(block, &var.dimension_names, anim, &plotted.dim_config, step);
        let dims3 = (dims[0], dims[1], dims[2]);
        let (stretch, d) = (&mut lod.stretch, (c_dim, dims3.0, dims3.1, dims3.2));
        let recorded = match &lod.blend {
            Blend::Channels(c) => {
                slice_multichannel_volume_stretched(block, d, ranges, &fixed, c, stretch)
            }
            Blend::Rgb(ch) => slice_rgb_volume_stretched(block, d, ranges, &fixed, *ch, stretch),
        };
        Some(recorded.map(|_| lod).ok_or(()))
    }
}

/// A hash of what the base layer's composite draws (`hash_into`).
pub(super) fn colors_of(app: &OctantApp) -> u64 {
    use std::hash::Hasher;
    let mut h = std::collections::hash_map::DefaultHasher::new();
    app.layers.base.composite.hash_into(&mut h);
    h.finish()
}

impl OctantApp {
    /// Recolors a composite's bricks once its settings rested `SETTLE`
    /// seconds (egui time `now`): dragging a contrast slider rebuilds
    /// nothing until it stops; turning the composite on or off rebuilds at
    /// once. The renderer and its atlas are kept when they fit; the bricks
    /// are placed anew, from the block cache. How long to wait.
    pub(super) fn recolor_volume_lod(&mut self, now: f64) -> Option<f64> {
        let colors = colors_of(self);
        let enabled = self.layers.base.composite.enabled;
        let state = &mut self.layers.base.data.volume_lod;
        state.context?;
        if state.colored == Some(colors) {
            state.colors = None;
            return None;
        }
        // Turned on or off (one click, nothing to wait for): the bricks hold
        // the wrong kind of texel, so rebuild them at once. Without bricks
        // (the last build stayed dense), settings wait to rest like others.
        let built = state.lod.as_ref().map(|l| l.composite.is_some());
        let toggled = built.is_some_and(|b| b != enabled);
        // A composite blends from the plotted block, which a switch to it
        // reads anew: wait for it (a build without it would turn the bricks
        // off for the plot). Looked up only while a rebuild is pending.
        if enabled && self.plotted_block().is_none() {
            return self.wait_for_plotted_block(toggled, colors, now);
        }
        let state = &mut self.layers.base.data.volume_lod;
        match state.colors {
            _ if toggled => {}
            Some((c, since)) if c == colors && now - since >= SETTLE => {}
            Some((c, since)) if c == colors => return Some(SETTLE - (now - since)),
            _ => {
                state.colors = Some((colors, now));
                return Some(SETTLE);
            }
        }
        let context = state.context;
        let renderer = self.layers.base.renderers.volume_bricked.take();
        self.reset_volume_lod(context, renderer);
        None
    }
}
