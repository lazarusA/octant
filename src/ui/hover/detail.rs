//! The hover over a multiscale heatmap's detail tile (`app::detail`): the
//! tile's cell under the pointer, its value and level indices, read instead
//! of the coarse plot's beneath it.

use crate::app::OctantApp;
use crate::data::{DatasetMetadata, VariableInfo};
use crate::i18n::{HoverKey, fill, tr};
use crate::ui::hover::enrich::{flipped_offset, is_flipped};
use crate::ui::hover::field::HoverField;
use crate::ui::hover::format::format_dimension_coord;

/// The tile cell at a point of the plotted quad.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DetailCell {
    pub value: f32,
    /// The cell's center on the quad (`0..1` from its left and top edges).
    pub center: (f32, f32),
    /// The tile's level and its variable.
    pub level: usize,
    pub var_idx: usize,
    /// Its x and y dimensions and the cell's index along each.
    pub x: (usize, usize),
    pub y: (usize, usize),
}

/// The tile cell at `(norm_x, norm_y)` on the plotted quad, when a tile is
/// shown there.
pub fn detail_cell(app: &OctantApp, norm_x: f32, norm_y: f32) -> Option<DetailCell> {
    let tile = app.layers.base.data.detail.tile.as_ref()?;
    let matrix = tile.matrix.as_ref()?;
    let [u0, v0, u1, v1] = tile.uv;
    let inside = |t: f32, lo: f32, hi: f32| hi > lo && t >= lo && t <= hi;
    if !inside(norm_x, u0, u1) || !inside(norm_y, v0, v1) || matrix.width == 0 || matrix.height == 0
    {
        return None;
    }
    let (w, h) = (matrix.width, matrix.height);
    let lx = (norm_x - u0) / (u1 - u0);
    let ly = (norm_y - v0) / (v1 - v0);
    let (px, py) = matrix.grid.find_cell_from_norm(lx, ly, w, h);
    let value = matrix.values.get(py * w + px).copied().unwrap_or(f32::NAN);
    let center = (
        u0 + (px as f32 + 0.5) / w as f32 * (u1 - u0),
        v0 + (py as f32 + 0.5) / h as f32 * (v1 - v0),
    );
    let var = app
        .plotted()
        .metadata
        .as_ref()?
        .variables
        .get(tile.var_idx)?;
    let index = |axis: usize, i: usize, len: usize| {
        let dim = tile.dims[axis];
        let flipped = var
            .dimension_names
            .get(dim)
            .is_some_and(|n| is_flipped(app, n));
        (
            dim,
            tile.origin[axis] as usize + flipped_offset(flipped, i, len),
        )
    };
    Some(DetailCell {
        value,
        center,
        level: tile.level,
        var_idx: tile.var_idx,
        x: index(0, px, w),
        y: index(1, py, h),
    })
}

/// The y and x fields of `cell`: its level's coordinates, else indices.
pub fn detail_fields(
    app: &OctantApp,
    meta: Option<&DatasetMetadata>,
    cell: &DetailCell,
) -> Option<(HoverField, HoverField)> {
    let var: &VariableInfo = meta?.variables.get(cell.var_idx)?;
    let field = |(dim, i): (usize, usize)| {
        let name = var.dimension_names.get(dim)?;
        let len = var.shape.get(dim).copied().unwrap_or(1) as usize;
        let store = Some(app.plotted().store_target.as_str());
        Some(format_dimension_coord(
            meta,
            Some(var),
            store,
            name,
            i,
            len,
            None,
        ))
    };
    Some((field(cell.y)?, field(cell.x)?))
}

/// The level row of a multiscale plot's hover: the tile's level under the
/// pointer (`cell`), else the plotted one, of the stack's level count.
pub fn level_field(app: &OctantApp, cell: Option<&DetailCell>) -> Option<HoverField> {
    let shown = app.plotted().variable_info()?.multiscale.as_ref()?;
    let level = cell.map_or(shown.level, |c| c.level);
    let mut buf = [0; 64];
    let value = fill(
        &mut buf,
        tr(HoverKey::LevelOfTotal),
        &[&level, &shown.set.len()],
    );
    Some(HoverField::new(tr(HoverKey::Level), value))
}
