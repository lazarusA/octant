use crate::app::OctantApp;
use crate::data::{DatasetMetadata, VariableInfo};
use crate::ui::hover::enrich::{
    enrich_entries_with_animated_and_collapsed_dims, get_dimension_origin_and_full_len,
};
use crate::ui::hover::field::HoverField;
use crate::ui::hover::format::format_dimension_coord;
use crate::ui::hover::raycast_volume::VolumeSampler;
use std::collections::HashSet;

pub(crate) fn resolve_3d_dim_entries(
    app: &OctantApp,
    meta: Option<&DatasetMetadata>,
    var: Option<&VariableInfo>,
    sampler: &VolumeSampler,
    hit_x: usize,
    hit_y: usize,
    hit_z: usize,
) -> Vec<HoverField> {
    let data_x = (hit_x + sampler.shift_x) % sampler.width;
    let data_y = (hit_y + sampler.shift_y) % sampler.height;
    let data_z = (hit_z + sampler.shift_z) % sampler.depth;

    let mut used_dims = HashSet::new();

    if let Some(v) = var {
        let (explicit_x, explicit_y, explicit_z) =
            v.resolve_spatial_dim_indices(app.effective_dim_config());

        let x_idx = explicit_x.unwrap_or(v.dimension_names.len().saturating_sub(1));
        let y_idx = explicit_y.unwrap_or(v.dimension_names.len().saturating_sub(2));
        let z_idx =
            explicit_z.or_else(|| (0..v.dimension_names.len()).find(|&i| i != x_idx && i != y_idx));

        used_dims.insert(x_idx);
        used_dims.insert(y_idx);

        let dim_y_name = explicit_y
            .and_then(|i| v.dimension_names.get(i).cloned())
            .or_else(|| app.get_spatial_dim_name(1))
            .unwrap_or_else(|| "y".to_string());

        let dim_x_name = explicit_x
            .and_then(|i| v.dimension_names.get(i).cloned())
            .or_else(|| app.get_spatial_dim_name(0))
            .unwrap_or_else(|| "x".to_string());

        let dim_z_name = z_idx
            .and_then(|i| v.dimension_names.get(i).cloned())
            .or_else(|| app.get_spatial_dim_name(2))
            .unwrap_or_else(|| "z".to_string());

        let (origin_x, full_x_len) = get_dimension_origin_and_full_len(app, Some(v), x_idx);
        let (origin_y, full_y_len) = get_dimension_origin_and_full_len(app, Some(v), y_idx);

        let global_x = (origin_x + data_x).min(full_x_len.saturating_sub(1));
        let global_y = (origin_y + data_y).min(full_y_len.saturating_sub(1));

        let loc_y = format_dimension_coord(
            meta,
            Some(v),
            Some(&app.plotted_store_target_input),
            &dim_y_name,
            global_y,
            full_y_len,
            None,
        );
        let loc_x = format_dimension_coord(
            meta,
            Some(v),
            Some(&app.plotted_store_target_input),
            &dim_x_name,
            global_x,
            full_x_len,
            None,
        );

        let mut list = vec![loc_y, loc_x];

        if sampler.depth > 1 {
            let z_dim = z_idx.unwrap_or(0);
            if let Some(zi) = z_idx {
                used_dims.insert(zi);
            }
            let (origin_z, full_z_len) = get_dimension_origin_and_full_len(app, Some(v), z_dim);
            let global_z = (origin_z + data_z).min(full_z_len.saturating_sub(1));

            let loc_z = format_dimension_coord(
                meta,
                Some(v),
                Some(&app.plotted_store_target_input),
                &dim_z_name,
                global_z,
                full_z_len,
                None,
            );
            list.insert(0, loc_z);
        }

        enrich_entries_with_animated_and_collapsed_dims(
            app,
            meta,
            Some(v),
            &mut list,
            &mut used_dims,
        );

        list
    } else if sampler.depth > 1 {
        let dim_z_name = app
            .get_spatial_dim_name(2)
            .unwrap_or_else(|| "z".to_string());
        let dim_y_name = app
            .get_spatial_dim_name(1)
            .unwrap_or_else(|| "y".to_string());
        let dim_x_name = app
            .get_spatial_dim_name(0)
            .unwrap_or_else(|| "x".to_string());
        vec![
            HoverField::index_of(dim_z_name, data_z, sampler.depth),
            HoverField::index_of(dim_y_name, data_y, sampler.height),
            HoverField::index_of(dim_x_name, data_x, sampler.width),
        ]
    } else {
        let dim_y_name = app
            .get_spatial_dim_name(1)
            .unwrap_or_else(|| "y".to_string());
        let dim_x_name = app
            .get_spatial_dim_name(0)
            .unwrap_or_else(|| "x".to_string());
        vec![
            HoverField::index_of(dim_y_name, data_y, sampler.height),
            HoverField::index_of(dim_x_name, data_x, sampler.width),
        ]
    }
}
