//! Overlay alignment against the base layer, and overlay selections that read its window.

use std::collections::HashMap;
use std::sync::Arc;

use super::{Alignment, VariableSelection, classify, overlay_selection};
use crate::app::{AnimationRole, DimConfig, SpatialRole};
use crate::data::{CoordValues, DatasetMetadata, VariableInfo};

fn var(name: &str, dims: &[&str], shape: &[u64]) -> VariableInfo {
    VariableInfo {
        name: name.to_string(),
        dimension_names: dims.iter().map(|d| d.to_string()).collect(),
        shape: shape.to_vec(),
        chunk_shape: shape.to_vec(),
        ..Default::default()
    }
}

fn regular(start: f64, step: f64, len: usize) -> CoordValues {
    CoordValues::Regular { start, step, len }
}

/// A dataset at `target` with `t2m(time, lat, lon)` and `sst(time, lat, lon)`
/// on a 5x4 grid, and the given coordinates.
fn dataset(target: &str, coords: &[(&str, CoordValues)]) -> VariableSelection {
    let dims = ["time", "lat", "lon"];
    let metadata = DatasetMetadata {
        variables: vec![var("t2m", &dims, &[3, 5, 4]), var("sst", &dims, &[3, 5, 4])],
        dimension_coordinates: coords
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect::<HashMap<_, _>>(),
        ..Default::default()
    };
    VariableSelection {
        store_target: target.to_string(),
        metadata: Some(metadata),
        ..Default::default()
    }
}

/// `t2m` of `dataset` plotted as a heatmap of lat x lon, animated along time.
fn base_of(dataset: &VariableSelection) -> VariableSelection {
    let mut base = dataset.clone();
    base.variable_idx = 0;
    base.dim_config = vec![
        DimConfig::new(SpatialRole::None, AnimationRole::Animated, true, 1, (0, 2)),
        DimConfig::new(SpatialRole::Y, AnimationRole::None, true, 0, (0, 4)),
        DimConfig::new(SpatialRole::X, AnimationRole::None, true, 0, (0, 3)),
    ];
    base.dim_indices = vec![1, 0, 0];
    base.dim_ranges = vec![(0, 2), (0, 4), (0, 3)];
    base.spatial_dims = vec![2, 1];
    base.animated_dim = Some(0);
    base
}

fn grid_coords() -> [(&'static str, CoordValues); 2] {
    [
        ("lat", regular(-40.0, 20.0, 5)),
        ("lon", regular(0.0, 90.0, 4)),
    ]
}

#[test]
fn an_overlay_selection_reads_the_base_window_by_dimension_name() {
    let data = dataset("a", &[]);
    let base = base_of(&data);
    let overlay = overlay_selection(&base, &data, 1).expect("overlay");
    assert_eq!(overlay.variable_idx, 1);
    assert_eq!(overlay.dim_ranges, base.dim_ranges);
    assert_eq!(overlay.dim_indices, base.dim_indices);
    assert_eq!(overlay.animated_dim, Some(0));
    assert_eq!(DimConfig::x_dim(&overlay.dim_config), Some(2));
    assert_eq!(DimConfig::y_dim(&overlay.dim_config), Some(1));
}

#[test]
fn a_variable_of_the_same_dataset_shares_the_grid() {
    let data = dataset("a", &[]);
    let base = base_of(&data);
    let overlay = overlay_selection(&base, &data, 1).expect("overlay");
    assert_eq!(classify(&base, &overlay), Alignment::SameGrid);
}

#[test]
fn another_dataset_shares_the_grid_only_when_its_coordinates_match() {
    let base = base_of(&dataset("a", &grid_coords()));
    let same = dataset("b", &grid_coords());
    let overlay = overlay_selection(&base, &same, 1).expect("overlay");
    assert_eq!(classify(&base, &overlay), Alignment::SameGrid);

    let values: Arc<[f64]> = Arc::from(vec![-40.0, -20.0, 0.0, 20.0, 40.0]);
    let as_values = dataset(
        "b",
        &[
            ("lat", CoordValues::Values(values)),
            grid_coords()[1].clone(),
        ],
    );
    let overlay = overlay_selection(&base, &as_values, 1).expect("overlay");
    assert_eq!(
        classify(&base, &overlay),
        Alignment::SameGrid,
        "equal values, other storage"
    );

    let shifted = dataset(
        "b",
        &[("lat", regular(-30.0, 20.0, 5)), grid_coords()[1].clone()],
    );
    let overlay = overlay_selection(&base, &shifted, 1).expect("overlay");
    assert!(matches!(classify(&base, &overlay), Alignment::Geo { .. }));
}

#[test]
fn another_dataset_without_coordinates_is_index_only() {
    let base = base_of(&dataset("a", &[]));
    let overlay = overlay_selection(&base, &dataset("b", &[]), 1).expect("overlay");
    assert_eq!(classify(&base, &overlay), Alignment::IndexOnly);
}

#[test]
fn a_different_window_or_grid_is_not_drawn() {
    let data = dataset("a", &[]);
    let base = base_of(&data);
    let mut overlay = overlay_selection(&base, &data, 1).expect("overlay");
    overlay.dim_ranges[2] = (0, 1);
    assert!(!classify(&base, &overlay).is_drawn(), "another lon window");

    let mut other = dataset("b", &[]);
    if let Some(meta) = other.metadata.as_mut() {
        meta.variables[1] = var("sst", &["time", "y", "x"], &[3, 7, 9]);
    }
    let overlay = overlay_selection(&base, &other, 1).expect("overlay");
    assert_eq!(classify(&base, &overlay), Alignment::Incompatible);
}

#[test]
fn only_heatmaps_align() {
    let data = dataset("a", &[]);
    let mut base = base_of(&data);
    let overlay = overlay_selection(&base, &data, 1).expect("overlay");
    base.plot_type = crate::plots::PlotType::Volume;
    assert_eq!(classify(&base, &overlay), Alignment::Incompatible);
}
