use crate::app::OctantApp;
use crate::data::{CoordinateGrid, DatasetMetadata, MatrixData, VariableInfo};
use crate::ui::hover::detail::{DetailCell, detail_fields};
use crate::ui::hover::enrich::enrich_entries_with_animated_and_collapsed_dims;
use crate::ui::hover::field::HoverField;
use crate::utils::units::{COORD_DECIMALS, format_cardinal_degrees};
use std::collections::HashSet;

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_2d_plot_entries(
    app: &OctantApp,
    matrix: &MatrixData,
    meta: Option<&DatasetMetadata>,
    var: Option<&VariableInfo>,
    (norm_x, norm_y): (f32, f32),
    cell: Option<(usize, usize)>,
    geo_coords: Option<(f32, f32)>,
) -> (f32, Vec<HoverField>, usize, usize) {
    let (orig_w, orig_h) = if let Some(pyr) = &app.layers.base.data.pyramid {
        (pyr.original_width, pyr.original_height)
    } else {
        (matrix.width, matrix.height)
    };
    let (px, py) = cell.unwrap_or_else(|| {
        matrix
            .grid
            .find_cell_from_norm(norm_x, norm_y, orig_w, orig_h)
    });

    // The matrix is the full-resolution data, also under a pyramid; a
    // multiscale plot's detail tile, where it is drawn, is finer still.
    let idx = py * matrix.width + px;
    let detail = super::detail::detail_cell(app, norm_x, norm_y);
    let base = &app.layers.base;
    let cell = || {
        base.gpu_blended_cell(px, py)
            .unwrap_or_else(|| matrix.values.get(idx).copied().unwrap_or(f32::NAN))
    };
    let val = detail.map_or_else(cell, |d| d.value);

    // A composite mixes every channel, so no single band is selected.
    let mut used_dims: HashSet<usize> = composite_channel_dim(app).into_iter().collect();
    let entries = resolve_2d_dim_entries(
        app,
        meta,
        var,
        px,
        py,
        orig_w,
        orig_h,
        geo_coords,
        &mut used_dims,
        detail.as_ref(),
    );

    (val, entries, px, py)
}

/// The channel dimension a composite plot mixes, which the hover lists per channel instead.
pub(crate) fn composite_channel_dim(app: &OctantApp) -> Option<usize> {
    app.layers
        .base
        .composite
        .enabled
        .then(|| app.channel_dim_index())
        .flatten()
}

#[allow(clippy::too_many_arguments)]
fn resolve_2d_dim_entries(
    app: &OctantApp,
    meta: Option<&DatasetMetadata>,
    var: Option<&VariableInfo>,
    px: usize,
    py: usize,
    orig_w: usize,
    orig_h: usize,
    geo_coords: Option<(f32, f32)>,
    used_dims: &mut HashSet<usize>,
    detail: Option<&DetailCell>,
) -> Vec<HoverField> {
    if let CoordinateGrid::Healpix { nside, .. } = &app
        .layers
        .base
        .data
        .matrix
        .as_ref()
        .map(|m| &m.grid)
        .unwrap_or(&CoordinateGrid::GlobalRegular)
    {
        return resolve_healpix_dim_entries(
            app, meta, var, px, py, orig_w, orig_h, *nside, used_dims,
        );
    }

    if let Some(v) = var {
        let (x_idx, y_idx, _) = super::entries::plane_dims(app, v);
        used_dims.insert(x_idx);
        used_dims.insert(y_idx);
        let tile_fields = detail.and_then(|d| detail_fields(app, meta, d));
        let (loc_y, loc_x) = tile_fields.unwrap_or_else(|| {
            let cells = (px, py, orig_w, orig_h);
            let [y, x] =
                super::entries::plane_fields(app, meta, v, (x_idx, y_idx), cells, geo_coords);
            (y, x)
        });

        let mut list = vec![loc_y, loc_x];
        enrich_entries_with_animated_and_collapsed_dims(app, meta, Some(v), &mut list, used_dims);

        list
    } else {
        vec![
            HoverField::index_of("y", py, orig_h),
            HoverField::index_of("x", px, orig_w),
        ]
    }
}

#[allow(clippy::too_many_arguments)]
fn resolve_healpix_dim_entries(
    app: &OctantApp,
    meta: Option<&DatasetMetadata>,
    var: Option<&VariableInfo>,
    px: usize,
    py: usize,
    orig_w: usize,
    orig_h: usize,
    nside: usize,
    used_dims: &mut HashSet<usize>,
) -> Vec<HoverField> {
    let (ring, _) = crate::data::coordinates::healpix::pix2ring(nside, px);
    let (cell_lon_rad, cell_lat_rad) = app
        .layers
        .base
        .data
        .matrix
        .as_ref()
        .map(|m| m.grid.cell_center_lon_lat_rad(px, py, orig_w, orig_h))
        .unwrap_or((0.0, 0.0));
    let lat_deg = cell_lat_rad.to_degrees();
    let lon_deg = cell_lon_rad.to_degrees();

    let lon_norm = ((lon_deg % 360.0) + 360.0) % 360.0;
    let lon_signed = if lon_norm <= 180.0 {
        lon_norm
    } else {
        lon_norm - 360.0
    };
    let lat_str = HoverField::new(
        "lat",
        format_cardinal_degrees(lat_deg as f64, false, COORD_DECIMALS),
    );
    let lon_str = HoverField::new(
        "lon",
        format_cardinal_degrees(lon_signed as f64, true, COORD_DECIMALS),
    );
    let healpix_str = HoverField::new("cell", format!("#{} (ring #{}, nside {})", px, ring, nside));

    let mut list = vec![healpix_str, lat_str, lon_str];
    if let Some(v) = var {
        let (explicit_x, _, _) = v.resolve_spatial_dim_indices(app.effective_dim_config());
        if let Some(x_idx) = explicit_x {
            used_dims.insert(x_idx);
        } else if let Some(cell_idx) = v
            .dimension_names
            .iter()
            .position(|d| crate::data::coordinates::naming::is_healpix_dim_name(d))
        {
            used_dims.insert(cell_idx);
        }
        enrich_entries_with_animated_and_collapsed_dims(app, meta, Some(v), &mut list, used_dims);
    }
    list
}
