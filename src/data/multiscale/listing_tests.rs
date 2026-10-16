//! Where a stack is listed: once, under its level 0, in the group its levels share.

use super::tests::var;
use super::types::{Multiscale, ScaleLevel};
use crate::data::VariableTreeGroup;
use crate::data::metadata::VariableInfo;

/// Links `paths` (finest first, each halving `y` and `x`) as stack `name`.
fn stacked(name: &str, paths: &[&str], vars: &mut [VariableInfo]) {
    let levels = paths
        .iter()
        .enumerate()
        .map(|(l, p)| {
            let f = f64::from(1u32 << l);
            ScaleLevel {
                path: p.to_string(),
                shape: vec![16 >> l, 16 >> l],
                factor: vec![f, f],
                offset: vec![0.0; 2],
            }
        })
        .collect();
    let dims = vec!["y".into(), "x".into()];
    let set = Multiscale::new(name.into(), dims, levels);
    super::link(set.expect("stack"), vars);
}

fn vars(paths: &[&str]) -> Vec<VariableInfo> {
    paths
        .iter()
        .enumerate()
        .map(|(i, p)| var(p, &[16 >> (i % 2), 16 >> (i % 2)], &["y", "x"]))
        .collect()
}

fn set_of(vars: &[VariableInfo], i: usize) -> &Multiscale {
    &vars[i].multiscale.as_ref().expect("linked").set
}

#[test]
fn stacks_are_listed_where_their_levels_meet() {
    // OME-Zarr image and its label image: levels are arrays of one group.
    let mut v = vars(&[
        "plate/A/1/0",
        "plate/A/1/1",
        "plate/A/1/labels/cells/0",
        "plate/A/1/labels/cells/1",
    ]);
    stacked("img", &["plate/A/1/0", "plate/A/1/1"], &mut v);
    stacked(
        "cells",
        &["plate/A/1/labels/cells/0", "plate/A/1/labels/cells/1"],
        &mut v,
    );
    assert_eq!(set_of(&v, 0).display_group(), "plate/A");
    assert_eq!(set_of(&v, 2).display_group(), "plate/A/1/labels");

    // GeoZarr levels in sibling groups, at the root and nested.
    let mut v = vars(&["0/t2m", "1/t2m", "grp/0/sst", "grp/1/sst"]);
    stacked("t2m", &["0/t2m", "1/t2m"], &mut v);
    stacked("sst", &["grp/0/sst", "grp/1/sst"], &mut v);
    assert_eq!(set_of(&v, 0).display_group(), "");
    assert_eq!(set_of(&v, 2).display_group(), "grp");

    // GeoTIFF overviews.
    let mut v = vars(&["band_1", "overview_1/band_1"]);
    stacked("band_1", &["band_1", "overview_1/band_1"], &mut v);
    assert_eq!(set_of(&v, 0).display_group(), "");
}

#[test]
fn the_tree_lists_level_0_alone() {
    let mut v = vars(&["0/t2m", "1/t2m", "0/sst", "1/sst"]);
    v.push(var("lat", &[16], &["y"]));
    stacked("t2m", &["0/t2m", "1/t2m"], &mut v);
    stacked("sst", &["0/sst", "1/sst"], &mut v);
    let root = VariableTreeGroup::build_tree_from_variables(&v);
    assert_eq!(root.variable_indices, [0, 2, 4]);
    assert!(root.subgroups.is_empty(), "the level groups are gone");
    assert_eq!(root.total_variable_count(), 3);
    let found = root.filter("t2m", &v).expect("match");
    assert_eq!(found.variable_indices, [0]);
}
