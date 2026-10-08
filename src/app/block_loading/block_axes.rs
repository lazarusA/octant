//! Where a layer's selection falls inside a block: the indices of the
//! collapsed dimensions and the window of each shown one. Pure functions of
//! the block and the selection, so any layer can project its own blocks.

use crate::data::octant_block::OctantBlock;

/// Position of block dimension `i` in the variable's dimensions
/// (`orig_dim_names`), matched by name.
fn variable_dim(block: &OctantBlock, orig_dim_names: &[String], i: usize) -> usize {
    block
        .dimension_names
        .get(i)
        .and_then(|n| orig_dim_names.iter().position(|o| o == n))
        .unwrap_or(i)
}

/// Block-local, oriented index of every dimension: `step` along the animated
/// dimension, the selected index (`indices`, by variable dimension) elsewhere.
pub(crate) fn fixed_indices(
    block: &OctantBlock,
    orig_dim_names: &[String],
    anim_dim: Option<usize>,
    indices: &[usize],
    step: usize,
) -> Vec<usize> {
    (0..block.rank())
        .map(|i| {
            let orig_idx = variable_dim(block, orig_dim_names, i);
            let idx = if Some(orig_idx) == anim_dim {
                step
            } else {
                indices.get(orig_idx).copied().unwrap_or(0)
            };
            let local = idx.saturating_sub(block.origin.get(i).copied().unwrap_or(0));
            block.oriented_index(i, local)
        })
        .collect()
}

/// The selected range (`ranges`, by variable dimension) along block dimension
/// `dim_idx`, and its block-local `[start, end)` window (the whole block when
/// they do not overlap).
pub(crate) fn dim_bounds(
    block: &OctantBlock,
    orig_dim_names: &[String],
    dim_idx: usize,
    ranges: &[(usize, usize)],
) -> ((usize, usize), (usize, usize)) {
    let dim_len = block.shape.get(dim_idx).copied().unwrap_or(1);
    let block_orig = block.origin.get(dim_idx).copied().unwrap_or(0);
    let orig_idx = variable_dim(block, orig_dim_names, dim_idx);

    let (req_start, req_end) = ranges
        .get(orig_idx)
        .copied()
        .unwrap_or((0, dim_len.saturating_sub(1)));

    let local = if super::view_filter::overlaps(block_orig, dim_len, (req_start, req_end)) {
        let s = req_start
            .saturating_sub(block_orig)
            .min(dim_len.saturating_sub(1));
        let e = (req_end + 1)
            .saturating_sub(block_orig)
            .clamp(s + 1, dim_len);
        (s, e)
    } else {
        (0, dim_len)
    };

    ((req_start, req_end), local)
}
