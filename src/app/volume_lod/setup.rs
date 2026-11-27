//! When the base volume is bricked, and building its bricks for a plot: the
//! plotted window on level 0's grid, the root bricks (the plotted level's),
//! the atlas and the uniforms placing it all for the shader.

use super::Lod;
use super::placement::{byte_offset, bytes_hold, table_window, value_encoding, wide_integers};
use super::residency::Residency;
use super::select::root_bricks;
use crate::app::OctantApp;
use crate::app::detail::plot_hash;
use crate::data::multiscale::Multiscale;
use crate::data::multiscale::bricks::{BRICK, Bricks, table_cells};
use crate::plots::PlotType;
use crate::plots::volume::bricks::AtlasTexel;
use crate::plots::volume::bricks::MAX_LEVELS;

/// Most table cells (64^3 level-0 voxels each): 256 MiB of table, on the
/// CPU and the GPU alike.
const TABLE_CELLS: usize = 1 << 26;

/// Why a plot that asked for detail by view stays dense (a notice).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Off {
    /// Its roots (first) would fill more than half the atlas's slots.
    Roots(usize, usize),
    /// Its levels do not brick the table (`Bricks::aligned`).
    Uneven,
    /// Integer values an f16 atlas would merge (`wide_integers`).
    WideIntegers,
    /// The GPU cannot run the bricked shader (`reset.rs`).
    Unsupported,
    /// The table of `grid` cells is past the GPU's 3D texture size (`max`).
    TooLarge([u32; 3], u32),
}

/// What an atlas holds and its values' `(offset, scale)`.
type Encoded = (AtlasTexel, (f32, f32));

/// Atlas slots per axis: 384 bricks (220 MiB) on the desktop, 192 on the
/// web's smaller memory; twice the desktop's on a GPU with its own memory
/// (`atlas_slots`).
#[cfg(not(target_arch = "wasm32"))]
pub const SLOTS: [u32; 3] = [8, 8, 6];
#[cfg(target_arch = "wasm32")]
pub const SLOTS: [u32; 3] = [8, 8, 3];

/// The atlas's slots on `adapter` for `texel`s: `SLOTS`, doubled on a
/// discrete GPU (integrated ones share the system's memory), doubled again
/// for byte texels (the same memory as f16's).
fn atlas_slots(adapter: Option<&wgpu::Adapter>, texel: AtlasTexel) -> [u32; 3] {
    let discrete =
        adapter.is_some_and(|a| a.get_info().device_type == wgpu::DeviceType::DiscreteGpu);
    let gpu = if discrete && cfg!(not(target_arch = "wasm32")) {
        2
    } else {
        1
    };
    let byte = if texel == AtlasTexel::Byte { 2 } else { 1 };
    [SLOTS[0], SLOTS[1], SLOTS[2] * gpu * byte]
}

/// `slots` with fewer layers along z until every resident brick's block
/// (`f32` values of `channels` channels) fits in half of a cache of
/// `cache_bytes`: neighbours' aprons and the hover read placed bricks'
/// blocks from the cache, so an atlas holding more than it keeps would
/// show seams.
fn cached_slots(slots: [u32; 3], cache_bytes: usize, channels: usize) -> [u32; 3] {
    let brick = (BRICK as usize).pow(3) * std::mem::size_of::<f32>() * channels;
    let layer = (slots[0] as usize * slots[1] as usize)
        .saturating_mul(brick)
        .max(1);
    let z = (cache_bytes / 2 / layer).clamp(1, slots[2] as usize);
    [slots[0], slots[1], z as u32]
}

impl OctantApp {
    /// The plot the bricks belong to, when the base layer is bricked: "Detail
    /// by view" on, a volume (composites too) at rest, in any mode,
    /// reading a level of a multiscale variable chosen by Auto, not animated
    /// along x, y or z.
    pub(super) fn volume_lod_context(&self, plot_type: PlotType) -> Option<u64> {
        let base = &self.layers.base;
        let plotted = base.selection();
        let var = plotted.variable_info()?;
        let (x, y, z) = var.resolve_spatial_dim_indices(&plotted.dim_config);
        let spatial = [x?, y?, z?];
        let animated_space = plotted.animated_dim().is_some_and(|a| spatial.contains(&a));
        let volume = &self.plot_configs.volume;
        let bricked = volume.lod
            && plot_type == PlotType::Volume
            && !self.playback.is_playing
            && plotted.level.is_auto()
            && var
                .multiscale
                .as_ref()
                .is_some_and(|r| r.level < MAX_LEVELS)
            && !animated_space
            && base.data.volume.is_some();
        let step = self.playback.current_timestep;
        // Composite settings recolor the bricks once they rest
        // (`composite.rs` `recolor_volume_lod`), not on every change.
        bricked.then(|| plot_hash(plotted, step, &base.data.flipped_dims, None))
    }

    /// The plotted volume's bricks; `Err` when they are not drawn.
    pub(super) fn build_lod(&self) -> Option<Result<Lod, Off>> {
        let plotted = self.plotted();
        let var = plotted.variable_info()?;
        let r = var.multiscale.as_ref()?;
        let (Some(x), Some(y), Some(z)) = var.resolve_spatial_dim_indices(&plotted.dim_config)
        else {
            return None;
        };
        let dims = [x, y, z];
        let (place, flip) = self.lod_placement(dims)?;
        let bricks = Bricks { set: &r.set, dims };
        let grid = bricks.grid();
        // A table past `TABLE_CELLS` (a level 0 beyond ~26k voxels a side on
        // average) stays on the dense path rather than allocate it.
        if table_cells(grid).is_none_or(|c| c > TABLE_CELLS) {
            log::warn!("Volume LOD off: a table of {grid:?} cells is too large");
            return None;
        }
        if !bricks.aligned(r.level + 1) {
            return Some(Err(Off::Uneven));
        }
        let window = table_window(&place, grid);
        let roots = root_bricks(&bricks, r.level, &window);
        if roots.is_empty() {
            return None;
        }
        // A composite whose plotted block left the cache stays dense.
        let composite = match self.composite_lod(dims) {
            Some(Ok(c)) => Some(c),
            Some(Err(())) => return None,
            None => None,
        };
        let (texel, (offset, scale)) =
            match self.atlas_encoding(&var.data_type, composite.is_some())? {
                Ok(encoding) => encoding,
                Err(off) => return Some(Err(off)),
            };
        let adapter = self.wgpu_render_state.as_ref().map(|rs| &rs.adapter);
        // A composite brick's block holds every channel.
        let channels = composite
            .as_ref()
            .and(self.layer_channel_dim(crate::app::layers::LayerId::BASE))
            .and_then(|c| var.shape.get(c))
            .map_or(1, |&n| n.max(1) as usize);
        let slots = cached_slots(
            atlas_slots(adapter, texel),
            self.block_cache.max_bytes(),
            channels,
        );
        let mut lod = Lod::new(r.set.clone(), dims, r.level, slots, texel);
        if roots.len() > lod.residency.slot_count() / 2 {
            return Some(Err(Off::Roots(roots.len(), lod.residency.slot_count())));
        }
        (lod.place, lod.flip, lod.window, lod.roots) = (place, flip, window, roots);
        (lod.offset, lod.scale, lod.composite) = (offset, scale, composite);
        Some(Ok(lod))
    }

    /// What the atlas holds and how values encode into it, `(v - offset) /
    /// scale`: a composite's colors; 8-bit integer data as bytes, exactly
    /// (scale 255), when the plotted volume holds only whole values within
    /// the type's 256 (packed data, `scale_factor`/`add_offset`, does not)
    /// and misses none (a byte holds no NaN); else f16 over the plotted
    /// range, a unit scale when it holds one value (a coarse level of a mask
    /// can), so finer values stay finite; `Err` for integer ids too far
    /// apart for f16 (they stay dense).
    fn atlas_encoding(&self, data_type: &str, color: bool) -> Option<Result<Encoded, Off>> {
        let v = self.layers.base.data.volume.as_ref()?;
        if color {
            return Some(Ok((AtlasTexel::Color, (0.0, 1.0))));
        }
        if let Some(offset) = byte_offset(data_type).filter(|&o| bytes_hold(&v.values, o)) {
            return Some(Ok((AtlasTexel::Byte, (offset, 255.0))));
        }
        if wide_integers(data_type, &v.values, (v.min_val, v.max_val)) {
            return Some(Err(Off::WideIntegers));
        }
        Some(Ok((AtlasTexel::Half, value_encoding(v.min_val, v.max_val))))
    }
}

impl Lod {
    /// Bricks of `set` over `dims` with roots at `root_level`, in an atlas
    /// of `slots` holding `texel`s, before any is read: placed over the
    /// whole level 0, values stored as they are (offset 0, scale 1), no
    /// roots yet (`build_lod` sets what its plot needs).
    pub(super) fn new(
        set: std::sync::Arc<Multiscale>,
        dims: [usize; 3],
        root_level: usize,
        slots: [u32; 3],
        texel: AtlasTexel,
    ) -> Self {
        let grid = Bricks { set: &set, dims }.grid();
        let hi = dims.map(|d| set.base().shape.get(d).copied().unwrap_or(1) as f64);
        Self {
            dims,
            slots,
            root_level,
            roots: Vec::new(),
            place: super::select::Placement {
                lo: [0.0; 3],
                hi,
                mirror: [false, true, true],
                aspect: [1.0; 3],
            },
            flip: [false; 3],
            window: grid.map(|g| 0..g),
            offset: 0.0,
            scale: 1.0,
            residency: Residency::new(grid, slots, set.len()),
            pending: Default::default(),
            queue: Default::default(),
            arrived: Default::default(),
            placed_at: f64::NEG_INFINITY,
            wanted: Default::default(),
            finest: root_level,
            failure_reported: false,
            unreadable: Default::default(),
            composite: None,
            texel,
            set,
        }
    }

    /// `new`'s bricks with roots at `set`'s coarsest level.
    #[cfg(test)]
    pub(super) fn bare(set: std::sync::Arc<Multiscale>, dims: [usize; 3]) -> Self {
        let root = set.len() - 1;
        Self::new(set, dims, root, SLOTS, AtlasTexel::Half)
    }
}

#[cfg(test)]
mod slot_tests {
    #[test]
    fn the_atlas_never_outgrows_the_block_cache() {
        let gib = 1 << 30;
        // 64 slots a layer of 1 MiB bricks: 8 layers fit in half a GiB.
        assert_eq!(super::cached_slots([8, 8, 24], gib, 1), [8, 8, 8]);
        assert_eq!(super::cached_slots([8, 8, 6], gib, 1), [8, 8, 6]);
        assert_eq!(super::cached_slots([8, 8, 6], gib, 4), [8, 8, 2]);
        assert_eq!(super::cached_slots([8, 8, 6], 0, 1), [8, 8, 1]);
    }
}
