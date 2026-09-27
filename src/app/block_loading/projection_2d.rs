//! 2D matrix data and composite planar block projection routines.

use crate::app::OctantApp;
use crate::data::octant_block::OctantBlock;

impl OctantApp {
    /// Projects a 2D scalar or composite block hyperslab onto the 2D render pipeline.
    #[allow(clippy::too_many_arguments)]
    pub fn apply_2d_projection(
        &mut self,
        block: &OctantBlock,
        x_dim: usize,
        y_dim: usize,
        x_range: (usize, usize),
        y_range: (usize, usize),
        fixed_indices: &[usize],
        compute_bounds: bool,
        c_dim: usize,
    ) {
        let mdata_opt = if self.rgb_composite_mode
            && block.shape.len() >= 2
            && block.shape.get(c_dim).copied().unwrap_or(0) >= 1
        {
            self.slice_2d_composite_data(
                block,
                c_dim,
                x_dim,
                y_dim,
                x_range,
                y_range,
                fixed_indices,
            )
        } else {
            None
        };

        let mdata_opt = mdata_opt.or_else(|| {
            block.slice_2d_with_ranges(
                x_dim,
                y_dim,
                x_range,
                y_range,
                fixed_indices,
                self.animated_dim_extent(),
                &format!("Block Cache [{}]", block.variable_name),
                compute_bounds,
            )
        });

        if let Some(mdata) = mdata_opt {
            self.rebuild_pipeline_with_matrix_data(mdata);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn slice_2d_composite_data(
        &self,
        block: &OctantBlock,
        c_dim: usize,
        x_dim: usize,
        y_dim: usize,
        x_range: (usize, usize),
        y_range: (usize, usize),
        fixed_indices: &[usize],
    ) -> Option<crate::data::matrix_data::MatrixData> {
        if !self.is_geotiff() && !self.composite_channel_configs.is_empty() {
            crate::data::slicing::slice_multichannel_composite_nd(
                block,
                c_dim,
                x_dim,
                y_dim,
                x_range,
                y_range,
                fixed_indices,
                &self.composite_channel_configs,
                self.animated_dim_extent(),
            )
        } else {
            let opt_channels = [
                Some(self.rgb_composite_channels[0]),
                Some(self.rgb_composite_channels[1]),
                Some(self.rgb_composite_channels[2]),
            ];
            crate::data::slicing::slice_rgb_composite_nd(
                block,
                c_dim,
                x_dim,
                y_dim,
                x_range,
                y_range,
                fixed_indices,
                opt_channels,
                self.animated_dim_extent(),
            )
        }
    }
}
