//! Which resident blocks belong to the plotted view, and re-projecting cached
//! blocks after a new selection.

use crate::app::OctantApp;
use crate::data::DimensionSelection;
use crate::data::octant_block::OctantBlock;

/// Whether a block spanning `origin..origin + len` overlaps the inclusive
/// requested range `req`.
pub fn overlaps(origin: usize, len: usize, req: (usize, usize)) -> bool {
    origin <= req.1 && origin + len > req.0
}

impl OctantApp {
    /// Whether `block` holds data of the current view: it overlaps the requested
    /// range on every spatial axis (`axes`: block dim and inclusive request),
    /// and covers the current step unless the animated axis is itself spatial
    /// (then the blocks along it together make up the volume).
    pub(crate) fn block_in_view(
        &self,
        block: &OctantBlock,
        anim_dim: Option<usize>,
        anim_is_spatial: bool,
        axes: [(usize, (usize, usize)); 3],
    ) -> bool {
        let span = |dim: usize| {
            let origin = block.origin.get(dim).copied().unwrap_or(0);
            (origin, block.shape.get(dim).copied().unwrap_or(1))
        };
        if let Some(dim) = anim_dim.filter(|_| !anim_is_spatial)
            && dim < block.rank()
        {
            let (origin, len) = span(dim);
            if !(origin..origin + len).contains(&self.current_timestep) {
                return false;
            }
        }
        axes.iter()
            .filter(|(dim, _)| *dim < block.rank())
            .all(|&(dim, req)| {
                let (origin, len) = span(dim);
                overlaps(origin, len, req)
            })
    }

    /// Plots resident `block` for the selection (`selections`, any step along
    /// `anim_dim`): syncs the plotted state, projects it (and the other blocks
    /// of a reset volume) and queues the rest of the animated range.
    pub(crate) fn show_cached_block(
        &mut self,
        block: &OctantBlock,
        selections: &[DimensionSelection],
        anim_dim: Option<usize>,
        shape: &[u64],
    ) {
        self.status_message = format!(
            "Block cache HIT for '{}' ({} bytes resident)",
            block.variable_name,
            block.bytes_size()
        );
        self.pending_target_step = None;
        self.sync_plotted_state_from_selected();
        let allocations = self.volume_allocations;
        self.apply_block_projection(block);
        let source_id = self.selected_source_id();
        self.project_cached_volume_blocks(
            &source_id,
            &block.variable_name,
            selections,
            anim_dim,
            allocations,
        );
        self.prefetch_selected_animated_range(shape);
    }

    /// Projects every cached block of the plotted selection after the volume
    /// was reset (`volume_allocations` moved past `since`, a new selection): a
    /// volume can be built from several resident blocks (along an animated
    /// spatial axis), and a cache hit only delivers the one covering the
    /// current step. Playback steps keep the volume and skip this.
    pub(crate) fn project_cached_volume_blocks(
        &mut self,
        source_id: &str,
        var_name: &str,
        selections: &[DimensionSelection],
        anim_dim: Option<usize>,
        since: u64,
    ) {
        if self.volume_allocations == since {
            return;
        }
        let is_volume = matches!(
            self.active_plot_type,
            crate::plots::PlotType::Volume | crate::plots::PlotType::PointCloud
        );
        if !is_volume {
            return;
        }
        let blocks = self
            .block_cache
            .matching_blocks(source_id, var_name, selections, anim_dim);
        for block in &blocks {
            self.apply_block_projection(block);
        }
    }
}
