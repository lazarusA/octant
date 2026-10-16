//! Stack validation, index mapping, and the OME-Zarr and GeoTIFF adapters.

use super::geotiff::{IfdShape, is_overview, link_overviews};
use super::types::{Multiscale, MultiscaleError, ScaleLevel};
use crate::data::metadata::VariableInfo;

pub(super) fn var(name: &str, shape: &[u64], dims: &[&str]) -> VariableInfo {
    VariableInfo {
        name: name.to_string(),
        shape: shape.to_vec(),
        dimension_names: dims.iter().map(|d| d.to_string()).collect(),
        ..Default::default()
    }
}

fn level(path: &str, shape: &[u64], factor: &[f64], offset: &[f64]) -> ScaleLevel {
    ScaleLevel {
        path: path.to_string(),
        shape: shape.to_vec(),
        factor: factor.to_vec(),
        offset: offset.to_vec(),
    }
}

fn stack(levels: Vec<ScaleLevel>) -> Result<Multiscale, MultiscaleError> {
    let dims = vec!["time".to_string(), "y".to_string(), "x".to_string()];
    Multiscale::new("t2m".into(), dims, levels)
}

fn three_levels() -> Multiscale {
    stack(vec![
        level("0/t2m", &[4, 11, 20], &[1.0, 1.0, 1.0], &[0.0; 3]),
        level("1/t2m", &[4, 5, 10], &[1.0, 2.0, 2.0], &[0.0; 3]),
        level("2/t2m", &[4, 2, 5], &[1.0, 4.0, 4.0], &[0.0; 3]),
    ])
    .expect("stack")
}

#[test]
fn stacks_are_validated() {
    let base = || level("0", &[4, 8, 8], &[1.0; 3], &[0.0; 3]);
    let half = || level("1", &[4, 4, 4], &[1.0, 2.0, 2.0], &[0.0; 3]);
    assert_eq!(stack(vec![base()]), Err(MultiscaleError::TooFewLevels));
    let rank = level("1", &[4, 4], &[1.0, 2.0], &[0.0; 2]);
    assert_eq!(
        stack(vec![base(), rank]),
        Err(MultiscaleError::Rank { level: 1 })
    );
    let shifted = level("0", &[4, 8, 8], &[1.0; 3], &[0.0, 0.5, 0.0]);
    assert_eq!(
        stack(vec![shifted, half()]),
        Err(MultiscaleError::BaseNotIdentity)
    );
    let finer = level("1", &[4, 8, 8], &[1.0, 0.5, 1.0], &[0.0; 3]);
    let bad = MultiscaleError::BadFactor { level: 1, dim: 1 };
    assert_eq!(stack(vec![base(), finer]), Err(bad));
    let back = level("2", &[4, 8, 4], &[1.0, 1.0, 2.0], &[0.0; 3]);
    let shrink = MultiscaleError::NotCoarsening { level: 2, dim: 1 };
    assert_eq!(stack(vec![base(), half(), back]), Err(shrink));
    let resized = level("1", &[3, 4, 4], &[1.0, 2.0, 2.0], &[0.0; 3]);
    let shape = MultiscaleError::Shape { level: 1, dim: 0 };
    assert_eq!(stack(vec![base(), resized]), Err(shape));
    assert!(stack(vec![base(), half()]).is_ok());
}

#[test]
fn ranges_map_between_levels() {
    let s = three_levels();
    // Whole extents, an uneven trimmed edge (11 rows -> 5) included.
    assert_eq!(s.to_level(1, 1, 0..11), Some(0..5));
    assert_eq!(s.to_level(1, 2, 0..20), Some(0..10));
    assert_eq!(s.to_level(2, 2, 3..9), Some(0..3));
    // A single level-0 cell maps to the cell holding it.
    assert_eq!(s.to_level(1, 2, 7..8), Some(3..4));
    assert_eq!(s.to_base(1, 2, 3..4), Some(6..8));
    assert_eq!(s.to_base(2, 1, 0..2), Some(0..8));
    // Clamped to level 0's 11 rows.
    assert_eq!(s.to_base(1, 1, 0..6), Some(0..11));
    assert_eq!(s.level_index(1, 2, 7), Some(3));
    assert_eq!(s.base_index(1, 2, 3), Some(7));
}

#[test]
fn undownsampled_dimensions_map_to_themselves() {
    let s = three_levels();
    for l in 0..3 {
        assert_eq!(s.to_level(l, 0, 1..3), Some(1..3));
        assert_eq!(s.to_base(l, 0, 1..3), Some(1..3));
        assert_eq!(s.level_index(l, 0, 2), Some(2));
    }
    assert_eq!(s.to_level(0, 2, 5..9), Some(5..9));
    assert_eq!(s.to_level(3, 0, 0..1), None);
    assert_eq!(s.to_level(1, 3, 0..1), None);
}

#[test]
fn offsets_and_fractional_factors_map() {
    let dims = vec!["y".to_string(), "x".to_string()];
    let levels = vec![
        level("band_1", &[1001, 2001], &[1.0, 1.0], &[0.0, 0.0]),
        level(
            "o/band_1",
            &[501, 1001],
            &[1001.0 / 501.0, 2001.0 / 1001.0],
            &[0.0, -0.5],
        ),
    ];
    let s = Multiscale::new("b".into(), dims, levels).expect("stack");
    assert_eq!(s.to_level(1, 0, 0..1001), Some(0..501));
    assert_eq!(s.to_level(1, 1, 0..2001), Some(0..1001));
    // Level cell 0 starts half a level-0 cell before level 0.
    assert_eq!(s.to_base(1, 1, 0..1), Some(0..2));
    assert_eq!(s.level_index(1, 1, 0), Some(0));
    assert_eq!(s.level_index(1, 1, 2), Some(1));
}

#[test]
fn ome_images_become_stacks() {
    use crate::data::ome::test_support::{Fixture, czyx_axes, multiscale_v04};
    let scales = [
        vec![1.0, 0.5, 0.36, 0.36],
        vec![1.0, 0.5, 0.72, 0.72],
        vec![1.0, 0.5, 1.44, 1.44],
    ];
    let mut ms = multiscale_v04(czyx_axes(), &scales);
    ms["name"] = "cells".into();
    let found = Fixture::default()
        .v2_group("", serde_json::json!({ "multiscales": [ms] }))
        .v2_array("0", &[2, 10, 64, 64])
        .v2_array("1", &[2, 10, 32, 32])
        .v2_array("2", &[2, 10, 16, 16])
        .discover()
        .expect("ome");
    let refs: Vec<_> = found
        .variables
        .iter()
        .map(|v| v.multiscale.as_ref())
        .collect();
    let set = &refs[0].expect("linked").set;
    assert_eq!(set.name, "cells");
    assert_eq!(set.dims, ["c", "z", "y", "x"]);
    for (l, r) in refs.iter().enumerate() {
        assert_eq!(r.expect("linked").level, l);
    }
    let top = set.level(2).expect("level 2");
    assert!((top.factor[3] - 4.0).abs() < 1e-6 && top.factor[1] == 1.0);
    // Unshifted translations: each level's first cell starts before level 0's.
    assert!((top.offset[3] + 1.5).abs() < 1e-6);
    assert_eq!(set.to_level(1, 3, 0..64), Some(0..32));
}

fn ifd(t: Option<u32>, width: u64, height: u64, samples: u64) -> IfdShape {
    IfdShape {
        new_subfile_type: t,
        width,
        height,
        samples,
    }
}

#[test]
fn overviews_are_reduced_resolution_ifds_but_not_masks() {
    let base = ifd(None, 200, 100, 3);
    assert!(is_overview(&base, &ifd(Some(1), 100, 50, 3)));
    assert!(!is_overview(&base, &ifd(Some(5), 100, 50, 1)));
    assert!(!is_overview(&base, &ifd(Some(4), 200, 100, 1)));
    assert!(!is_overview(&base, &ifd(Some(0), 100, 50, 3)));
    assert!(
        !is_overview(&base, &ifd(Some(1), 100, 50, 1)),
        "other samples"
    );
    // Without the tag: smaller pages with the same samples.
    assert!(is_overview(&base, &ifd(None, 100, 50, 3)));
    assert!(!is_overview(&base, &ifd(None, 100, 50, 1)));
    assert!(!is_overview(&base, &ifd(None, 200, 100, 3)));
}

#[test]
fn geotiff_variables_stack_with_their_overviews() {
    let mut vars = vec![
        var("raster", &[3, 101, 200], &["band", "y", "x"]),
        var("band_1", &[101, 200], &["y", "x"]),
        var("overview_1/raster", &[3, 51, 100], &["band", "y", "x"]),
        var("overview_1/band_1", &[51, 100], &["y", "x"]),
        var("overview_2/band_1", &[101, 200], &["y", "x"]),
        var("overview_3/band_1", &[26, 50], &["y", "x"]),
    ];
    // IFD 2 is a mask: not listed.
    link_overviews(&mut vars, &[3, 1]);
    let band = vars[1].multiscale.as_ref().expect("band stack");
    let paths: Vec<&str> = band.set.levels.iter().map(|l| l.path.as_str()).collect();
    assert_eq!(paths, ["band_1", "overview_1/band_1", "overview_3/band_1"]);
    assert!(vars[4].multiscale.is_none());
    assert_eq!(vars[5].multiscale.as_ref().map(|r| r.level), Some(2));
    let raster = vars[2].multiscale.as_ref().expect("raster stack");
    assert_eq!((raster.set.name.as_str(), raster.level), ("raster", 1));
    let lvl = raster.info().expect("level");
    assert_eq!(lvl.factor[0], 1.0);
    assert!((lvl.factor[1] - 101.0 / 51.0).abs() < 1e-12);
    assert_eq!(raster.set.to_level(1, 1, 0..101), Some(0..51));
}

#[test]
fn geotiff_without_overviews_links_nothing() {
    let mut vars = vec![var("band_1", &[10, 10], &["y", "x"])];
    link_overviews(&mut vars, &[]);
    assert!(vars[0].multiscale.is_none());
}
