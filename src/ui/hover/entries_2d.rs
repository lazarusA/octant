use crate::app::OctantApp;
use crate::data::{CoordinateGrid, DatasetMetadata, MatrixData, VariableInfo};
use crate::ui::hover::enrich::{
    enrich_entries_with_animated_and_collapsed_dims, get_dimension_origin_and_full_len,
    stored_offset,
};
use crate::ui::hover::field::HoverField;
use crate::ui::hover::format::format_dimension_coord;
use crate::utils::units::format_cardinal_degrees;
use std::collections::HashSet;

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_2d_plot_entries(
    app: &OctantApp,
    matrix: &MatrixData,
    meta: Option<&DatasetMetadata>,
    var: Option<&VariableInfo>,
    norm_x: f32,
    norm_y: f32,
    geo_coords: Option<(f32, f32)>,
) -> (f32, Vec<HoverField>, usize, usize) {
    let (orig_w, orig_h) = if let Some(pyr) = &app.active_pyramid {
        (pyr.original_width, pyr.original_height)
    } else {
        (matrix.width, matrix.height)
    };
    let (px, py) = matrix
        .grid
        .find_cell_from_norm(norm_x, norm_y, orig_w, orig_h);

    let val = if let Some(pyr) = &app.active_pyramid
        && let Some(base_lvl) = pyr.levels.first()
    {
        let idx = py * orig_w + px;
        base_lvl.values.get(idx).copied().unwrap_or(f32::NAN)
    } else {
        let idx = py * matrix.width + px;
        matrix.values.get(idx).copied().unwrap_or(f32::NAN)
    };

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
    );

    (val, entries, px, py)
}

/// The channel dimension a composite plot mixes, which the hover lists per channel instead.
pub(crate) fn composite_channel_dim(app: &OctantApp) -> Option<usize> {
    app.rgb_composite_mode
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
) -> Vec<HoverField> {
    if let CoordinateGrid::Healpix { nside, .. } = &app
        .matrix_data
        .as_ref()
        .map(|m| &m.grid)
        .unwrap_or(&CoordinateGrid::GlobalRegular)
    {
        return resolve_healpix_dim_entries(
            app, meta, var, px, py, orig_w, orig_h, *nside, used_dims,
        );
    }

    if let Some(v) = var {
        let (explicit_x, explicit_y, _) = v.resolve_spatial_dim_indices(app.effective_dim_config());

        let x_idx = explicit_x.unwrap_or(v.dimension_names.len().saturating_sub(1));
        let y_idx = explicit_y.unwrap_or(v.dimension_names.len().saturating_sub(2));

        used_dims.insert(x_idx);
        used_dims.insert(y_idx);

        // The plotted dimensions' own names, so flipped ones map back to stored rows.
        let dim_y_name = v
            .dimension_names
            .get(y_idx)
            .cloned()
            .unwrap_or_else(|| "y".to_string());

        let dim_x_name = v
            .dimension_names
            .get(x_idx)
            .cloned()
            .unwrap_or_else(|| "x".to_string());

        let geo_y = geo_coords.map(|(lat, _)| lat);
        let geo_x = geo_coords.map(|(_, lon)| lon);

        let (origin_x, full_x_len) = get_dimension_origin_and_full_len(app, Some(v), x_idx);
        let (origin_y, full_y_len) = get_dimension_origin_and_full_len(app, Some(v), y_idx);

        let x_offset = stored_offset(app, &dim_x_name, px, orig_w);
        let y_offset = stored_offset(app, &dim_y_name, py, orig_h);
        let global_x = (origin_x + x_offset).min(full_x_len.saturating_sub(1));
        let global_y = (origin_y + y_offset).min(full_y_len.saturating_sub(1));

        let loc_y = format_dimension_coord(
            meta,
            Some(v),
            Some(&app.plotted_store_target_input),
            &dim_y_name,
            global_y,
            full_y_len,
            geo_y,
        );
        let loc_x = format_dimension_coord(
            meta,
            Some(v),
            Some(&app.plotted_store_target_input),
            &dim_x_name,
            global_x,
            full_x_len,
            geo_x,
        );

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
        .matrix_data
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
    let lat_str = HoverField::new("lat", format_cardinal_degrees(lat_deg as f64, false));
    let lon_str = HoverField::new("lon", format_cardinal_degrees(lon_signed as f64, true));
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
