//! Spatial axis resolution and block projection dispatch onto 2D and 3D pipelines.

use super::block_axes;
use crate::app::OctantApp;
use crate::data::octant_block::OctantBlock;

impl OctantApp {
    /// Resolves the 3D spatial axis indices `(x_dim, y_dim, z_dim)` for block projections,
    /// honoring explicit user SpatialRoles (X/Y/Z) or falling back to non-animated dimensions.
    pub fn resolve_spatial_axes(
        rank: usize,
        block_dim_names: &[String],
        orig_dim_names: &[String],
        dim_config: &[crate::app::DimConfig],
    ) -> (usize, usize, usize) {
        if rank <= 1 {
            return (0, 0, usize::MAX);
        }

        let anim_dim = crate::app::DimConfig::animated_dim(dim_config);

        let find_explicit_spatial = |role: crate::app::SpatialRole| -> Option<usize> {
            (0..rank).find(|&d| {
                let Some(name) = block_dim_names.get(d) else {
                    return false;
                };
                let Some(orig_idx) = orig_dim_names.iter().position(|n| n == name) else {
                    return false;
                };
                dim_config.get(orig_idx).is_some_and(|c| c.spatial == role)
            })
        };

        if let Some(grid_dim) = find_explicit_spatial(crate::app::SpatialRole::Grid) {
            let z_dim = find_explicit_spatial(crate::app::SpatialRole::Z).unwrap_or(usize::MAX);
            return (grid_dim, grid_dim, z_dim);
        }

        let is_channel = |d: usize| -> bool {
            block_dim_names
                .get(d)
                .is_some_and(|n| crate::data::coordinates::naming::is_channel_dim_name(n))
        };

        let x_dim = find_explicit_spatial(crate::app::SpatialRole::X).unwrap_or_else(|| {
            (0..rank)
                .rev()
                .find(|&d| Some(d) != anim_dim && !is_channel(d))
                .unwrap_or(0)
        });

        let y_dim = find_explicit_spatial(crate::app::SpatialRole::Y).unwrap_or_else(|| {
            (0..rank)
                .rev()
                .find(|&d| d != x_dim && Some(d) != anim_dim && !is_channel(d))
                .unwrap_or_else(|| (0..rank).find(|&d| d != x_dim).unwrap_or(0))
        });

        let z_dim = find_explicit_spatial(crate::app::SpatialRole::Z).unwrap_or_else(|| {
            (0..rank)
                .find(|&d| d != x_dim && d != y_dim && Some(d) != anim_dim && !is_channel(d))
                .unwrap_or(usize::MAX)
        });

        (x_dim, y_dim, z_dim)
    }

    /// Projects a resident block into current 2D or 3D views.
    pub fn apply_block_projection(&mut self, block: &OctantBlock) {
        let dim_configs = self.effective_dim_config();
        let anim_dim = self
            .plotted()
            .animated_dim
            .or_else(|| crate::app::DimConfig::animated_dim(dim_configs));
        let orig_dim_names = self.resolve_orig_dim_names(block);

        let (x_dim, y_dim, z_dim) = Self::resolve_spatial_axes(
            block.rank(),
            &block.dimension_names,
            &orig_dim_names,
            dim_configs,
        );

        let fixed_indices = block_axes::fixed_indices(
            block,
            &orig_dim_names,
            anim_dim,
            self.effective_selected_dim_indices(),
            self.current_timestep,
        );
        let ranges = self.effective_selected_dim_ranges();
        let (req_x, local_x_range) = block_axes::dim_bounds(block, &orig_dim_names, x_dim, ranges);
        let (req_y, local_y_range) = block_axes::dim_bounds(block, &orig_dim_names, y_dim, ranges);
        let compute_bounds = !self.layers.base.color.lock_bounds;
        let c_dim = self.channel_dim_index().unwrap_or(0);
        let (req_z, local_z_range) =
            if z_dim < block.rank() && (!self.layers.base.composite.enabled || z_dim != c_dim) {
                block_axes::dim_bounds(block, &orig_dim_names, z_dim, ranges)
            } else {
                ((0, 0), (0, 1))
            };
        let is_vol_allowed = crate::ui::variables_panel::is_volume_allowed_for_selection(self);

        if !is_vol_allowed
            && (self.selected.plot_type == crate::plots::PlotType::Volume
                || self.selected.plot_type == crate::plots::PlotType::PointCloud)
        {
            self.selected.plot_type = crate::plots::PlotType::Heatmap;
        }

        let is_3d_plot = (self.selected.plot_type == crate::plots::PlotType::Volume
            || self.selected.plot_type == crate::plots::PlotType::PointCloud)
            && is_vol_allowed;

        let is_3d_anim = anim_dim.is_some_and(|a| a == x_dim || a == y_dim || a == z_dim);
        let axes = [(x_dim, req_x), (y_dim, req_y), (z_dim, req_z)];
        if !self.block_in_view(block, anim_dim, is_3d_anim, axes) {
            return;
        }

        if is_3d_plot {
            self.apply_3d_volume_projection(
                block,
                x_dim,
                y_dim,
                z_dim,
                req_x,
                req_y,
                req_z,
                local_x_range,
                local_y_range,
                local_z_range,
                &fixed_indices,
                is_3d_anim,
                compute_bounds,
                c_dim,
            );
        } else {
            self.apply_2d_projection(
                block,
                x_dim,
                y_dim,
                block.oriented_range(x_dim, local_x_range),
                block.oriented_range(y_dim, local_y_range),
                &fixed_indices,
                compute_bounds,
                c_dim,
            );
        }
    }

    fn resolve_orig_dim_names(&self, block: &OctantBlock) -> Vec<String> {
        self.effective_dataset_metadata()
            .and_then(|meta| {
                meta.variables
                    .get(self.plotted().variable_idx)
                    .or_else(|| meta.variables.get(self.selected.variable_idx))
            })
            .map(|v| v.dimension_names.clone())
            .unwrap_or_else(|| block.dimension_names.clone())
    }
}
