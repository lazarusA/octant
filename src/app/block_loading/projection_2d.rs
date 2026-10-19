//! 2D matrix data and composite planar block projection routines.

use crate::app::OctantApp;
use crate::app::layers::LayerId;
use crate::data::MatrixData;
use crate::data::octant_block::OctantBlock;
use crate::data::slicing::{CompositeProbe, CompositeStretch};

/// The plane of a block a 2D slice reads: its x and y dimensions, their local
/// (oriented) ranges and the indices fixing the others.
pub(crate) struct PlaneWindow<'a> {
    pub x_dim: usize,
    pub y_dim: usize,
    pub x_range: (usize, usize),
    pub y_range: (usize, usize),
    pub fixed: &'a [usize],
}

/// A 2D composite: its packed colors, the block window behind them and how
/// its channels were stretched.
pub(crate) struct SlicedComposite {
    pub matrix: MatrixData,
    pub probe: CompositeProbe,
    pub stretch: CompositeStretch,
}

impl OctantApp {
    /// Projects a 2D scalar or composite block hyperslab onto layer `id`'s 2D render pipeline.
    #[allow(clippy::too_many_arguments)]
    pub fn apply_2d_projection(
        &mut self,
        id: LayerId,
        block: &OctantBlock,
        x_dim: usize,
        y_dim: usize,
        x_range: (usize, usize),
        y_range: (usize, usize),
        fixed_indices: &[usize],
        compute_bounds: bool,
        c_dim: usize,
    ) {
        let window = PlaneWindow {
            x_dim,
            y_dim,
            x_range,
            y_range,
            fixed: fixed_indices,
        };
        let composite = self.layers.get(id).is_some_and(|l| l.composite.enabled)
            && block.shape.len() >= 2
            && block.shape.get(c_dim).copied().unwrap_or(0) >= 1;
        let sliced = composite
            .then(|| self.slice_composite(id, block, c_dim, &window, CompositeStretch::default()))
            .flatten();
        let Some(layer) = self.layers.get_mut(id) else {
            return;
        };
        layer.data.flipped_dims.clone_from(&block.flipped_dims);
        layer.data.composite_probe = None;
        let before = layer.data.composite_stretch.take();
        let mdata_opt = sliced.map(|c| {
            // Tiles stretched otherwise would show seams.
            if before.as_ref() != Some(&c.stretch) {
                layer.data.detail.discard_tile();
            }
            layer.data.composite_probe = Some(c.probe);
            layer.data.composite_stretch = Some(c.stretch);
            c.matrix
        });

        let mdata_opt = mdata_opt.or_else(|| {
            block.slice_2d_with_ranges(
                x_dim,
                y_dim,
                x_range,
                y_range,
                fixed_indices,
                self.layer_animated_dim_extent(id),
                &format!("Block Cache [{}]", block.variable_name),
                compute_bounds,
            )
        });

        if let Some(mdata) = mdata_opt {
            self.rebuild_pipeline_with_matrix_data(id, mdata);
        }
    }

    /// `window` of `block` as layer `id`'s composite over channel dimension
    /// `c_dim`, its channels stretched as `stretch` records (else over their
    /// values, recorded); `None` when the block holds no composite.
    pub(crate) fn slice_composite(
        &self,
        id: LayerId,
        block: &OctantBlock,
        c_dim: usize,
        window: &PlaneWindow,
        mut stretch: CompositeStretch,
    ) -> Option<SlicedComposite> {
        let anim_extent = self.layer_animated_dim_extent(id);
        let style = &self.layers.get(id)?.composite;
        let PlaneWindow {
            x_dim,
            y_dim,
            x_range,
            y_range,
            fixed,
        } = *window;
        let matrix = if !self.layer_is_geotiff(id) && !style.channel_configs.is_empty() {
            crate::data::slicing::slice_multichannel_composite_nd(
                block,
                c_dim,
                x_dim,
                y_dim,
                x_range,
                y_range,
                fixed,
                &style.channel_configs,
                anim_extent,
                &mut stretch,
            )
        } else {
            crate::data::slicing::slice_rgb_composite_nd(
                block,
                c_dim,
                x_dim,
                y_dim,
                x_range,
                y_range,
                fixed,
                style.rgb_channels.map(Some),
                anim_extent,
                &mut stretch,
            )
        }?;
        let probe = CompositeProbe::new(block, c_dim, x_dim, y_dim, x_range, y_range, fixed);
        Some(SlicedComposite {
            matrix,
            probe,
            stretch,
        })
    }
}
