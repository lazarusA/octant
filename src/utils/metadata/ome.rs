//! OME-NGFF multiscales and OMERO channel metadata parsing and normalization.

use std::collections::HashMap;
use std::sync::Arc;

use zarrs::array::Array;
use zarrs::storage::ReadableStorageTraits;

use crate::data::metadata::VariableInfo;
use crate::utils::units::calculate_variable_size_bytes;

pub use super::ome_types::*;

/// Normalizes NGFF attributes across v0.1–v0.5 schemas into a standard root map.
pub fn normalize_ngff_attributes(
    attrs: serde_json::Map<String, serde_json::Value>,
) -> serde_json::Map<String, serde_json::Value> {
    if attrs.contains_key("multiscales") {
        return attrs;
    }
    let Some(ome) = attrs.get("ome").and_then(|v| v.as_object()).cloned() else {
        return attrs;
    };
    let mut out = ome;
    if !out.contains_key("omero")
        && let Some(omero) = attrs.get("omero").cloned()
    {
        out.insert("omero".to_string(), omero);
    }
    out
}

/// Extracts `VariableInfo` list for all pyramid levels in an OME-NGFF group.
pub fn extract_ome_multiscale_variables(
    store: Arc<dyn ReadableStorageTraits>,
    root_attrs_map: &serde_json::Map<String, serde_json::Value>,
    group_prefix: &str,
) -> Vec<VariableInfo> {
    let normalized = normalize_ngff_attributes(root_attrs_map.clone());
    let Ok(root_zattrs) =
        serde_json::from_value::<RootZattrs>(serde_json::Value::Object(normalized))
    else {
        return Vec::new();
    };

    let Some(multiscale) = root_zattrs.multiscales.first() else {
        return Vec::new();
    };

    let (labels, colors, windows, actives) = extract_omero_channel_lists(&root_zattrs.omero);
    let axes_names: Vec<String> = multiscale.axes.iter().map(|a| a.name.clone()).collect();
    let mut variables = Vec::new();

    for ds in &multiscale.datasets {
        let clean_path = ds.path.trim_matches('/');
        let var_name = if group_prefix.is_empty() {
            clean_path.to_string()
        } else {
            format!("{}/{}", group_prefix.trim_matches('/'), clean_path)
        };

        if let Some(var_info) = build_multiscale_variable(
            store.clone(),
            &var_name,
            clean_path,
            multiscale,
            ds,
            &axes_names,
            &root_zattrs,
            (&labels, &colors, &windows, &actives),
        ) {
            variables.push(var_info);
        }
    }

    variables
}

#[allow(clippy::type_complexity)]
fn extract_omero_channel_lists(
    omero: &Option<Omero>,
) -> (Vec<String>, Vec<String>, Vec<String>, Vec<String>) {
    let Some(o) = omero.as_ref() else {
        return (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    };

    let labels = o
        .channels
        .iter()
        .enumerate()
        .map(|(i, ch)| {
            ch.name
                .as_deref()
                .or(ch.label.as_deref())
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("Channel {i}"))
        })
        .collect();

    let colors = o
        .channels
        .iter()
        .map(|ch| ch.color.clone().unwrap_or_default())
        .collect();

    let windows = o
        .channels
        .iter()
        .map(|ch| {
            if let Some(ref w) = ch.window {
                format!("{}:{}", w.start, w.end)
            } else {
                String::new()
            }
        })
        .collect();

    let actives = o
        .channels
        .iter()
        .map(|ch| ch.active.map(|a| a.to_string()).unwrap_or_default())
        .collect();

    (labels, colors, windows, actives)
}

#[allow(clippy::too_many_arguments)]
fn build_multiscale_variable(
    store: Arc<dyn ReadableStorageTraits>,
    var_name: &str,
    clean_path: &str,
    multiscale: &Multiscale,
    ds: &MultiscaleDataset,
    axes_names: &[String],
    root_zattrs: &RootZattrs,
    channels: (&[String], &[String], &[String], &[String]),
) -> Option<VariableInfo> {
    let zarr_path = format!("/{var_name}");
    let arr = Array::open(store.clone(), &zarr_path)
        .ok()
        .or_else(|| Array::open(store.clone(), &format!("/{clean_path}")).ok())
        .or_else(|| {
            crate::utils::metadata::open_or_instantiate_array_normalized(store.clone(), &zarr_path)
                .ok()
        })?;

    let shape = arr.shape().to_vec();
    if shape.is_empty() {
        return None;
    }
    let zero_idx = vec![0u64; shape.len()];
    let chunk_shape = arr
        .chunk_shape(&zero_idx)
        .map(|v| v.into_iter().map(|n| n.get()).collect::<Vec<u64>>())
        .unwrap_or_else(|_| shape.clone());
    let data_type = format!("{:?}", arr.data_type());

    let dimension_names = if axes_names.len() == shape.len() {
        axes_names.to_vec()
    } else {
        fallback_ome_axes_for_rank(shape.len())
    };

    let attrs = build_ome_attributes(&dimension_names, ds, root_zattrs, channels);
    let file_size = calculate_variable_size_bytes(&shape, &data_type);

    Some(VariableInfo {
        name: var_name.to_string(),
        data_type,
        shape,
        dimension_names,
        chunk_shape,
        file_size,
        units: None,
        long_name: multiscale.name.clone(),
        time_coverage_start: None,
        time_coverage_end: None,
        temporal_resolution: None,
        attributes: attrs,
    })
}

fn build_ome_attributes(
    dimension_names: &[String],
    ds: &MultiscaleDataset,
    root_zattrs: &RootZattrs,
    (labels, colors, windows, actives): (&[String], &[String], &[String], &[String]),
) -> HashMap<String, String> {
    let mut attrs = HashMap::new();
    if !labels.is_empty() {
        attrs.insert("omero_channels".to_string(), labels.join(","));
    }
    if !colors.is_empty() {
        attrs.insert("omero_colors".to_string(), colors.join(","));
    }
    if !windows.is_empty() {
        attrs.insert("omero_windows".to_string(), windows.join(","));
    }
    if !actives.is_empty() {
        attrs.insert("omero_actives".to_string(), actives.join(","));
    }
    if let Some(ref omero) = root_zattrs.omero
        && let Some(ref rdefs) = omero.rdefs
    {
        if let Some(dz) = rdefs.default_z {
            attrs.insert("default_z".to_string(), dz.to_string());
        }
        if let Some(dt) = rdefs.default_t {
            attrs.insert("default_t".to_string(), dt.to_string());
        }
    }
    for trans in &ds.coordinate_transformations {
        if let CoordTransform::Scale { scale } = trans
            && scale.len() == dimension_names.len()
        {
            for (dim_name, &s_val) in dimension_names.iter().zip(scale.iter()) {
                if s_val > 0.0 {
                    attrs.insert(format!("scale_{dim_name}"), s_val.to_string());
                }
            }
        }
    }
    attrs
}

/// Fallback OME axis names when axes are omitted in older NGFF specifications.
pub fn fallback_ome_axes_for_rank(rank: usize) -> Vec<String> {
    match rank {
        5 => vec![
            "t".to_string(),
            "c".to_string(),
            "z".to_string(),
            "y".to_string(),
            "x".to_string(),
        ],
        4 => vec![
            "c".to_string(),
            "z".to_string(),
            "y".to_string(),
            "x".to_string(),
        ],
        3 => vec!["c".to_string(), "y".to_string(), "x".to_string()],
        2 => vec!["y".to_string(), "x".to_string()],
        _ => (0..rank).map(|i| format!("dim_{i}")).collect(),
    }
}
