//! Oriented block ranges, mirrored slab placement, and transposing square lon-first grids.

use std::collections::HashMap;

use super::block_orientation::slab_destination;
use super::octant_block::OctantBlock;
use crate::utils::grid::check_and_orient_block_grid;

fn block(flipped: &[&str]) -> OctantBlock {
    let dims = ["lat", "lon"].map(String::from).to_vec();
    let mut b = OctantBlock::new(
        "v".into(),
        vec![10, 4],
        dims,
        vec![0, 0],
        vec![0.0; 40],
        HashMap::new(),
        HashMap::new(),
    );
    b.flipped_dims = flipped.iter().map(|s| s.to_string()).collect();
    b
}

#[test]
fn stored_ranges_mirror_only_along_flipped_dimensions() {
    let b = block(&["lat"]);
    assert!(b.is_flipped(0) && !b.is_flipped(1) && !b.is_flipped(7));
    // Stored rows 2..5 of a 10-row flipped block are its rows 5..8.
    assert_eq!(b.oriented_range(0, (2, 5)), (5, 8));
    assert_eq!(b.oriented_range(0, (0, 10)), (0, 10));
    assert_eq!(b.oriented_range(1, (1, 3)), (1, 3));
    assert_eq!(block(&[]).oriented_range(0, (2, 5)), (2, 5));
    // A fixed index along a flipped dimension reads the mirrored row.
    assert_eq!(b.oriented_index(0, 0), 9);
    assert_eq!(b.oriented_index(0, 9), 0);
    assert_eq!(b.oriented_index(1, 2), 2);
}

#[test]
fn flipped_slabs_are_placed_mirrored_across_the_volume() {
    // Two 5-row blocks of a 10-row volume: the northern one (stored 5..10) goes on top.
    assert_eq!(slab_destination(true, (0, 0, 5), (0, 10)), 5);
    assert_eq!(slab_destination(true, (5, 0, 5), (0, 10)), 0);
    assert_eq!(slab_destination(false, (5, 0, 5), (0, 10)), 5);
    // A request starting at stored row 3 of a 4-row volume, slab of stored rows 3..5.
    assert_eq!(slab_destination(true, (0, 3, 2), (3, 4)), 2);
}

#[test]
fn square_lon_first_grids_swap_their_names_and_origins() {
    let values: Vec<f32> = (0..9u8).map(f32::from).collect(); // (lon i, lat j) = 3 * i + j
    let mut shape = vec![3, 3];
    let mut dims = vec!["lon".to_string(), "lat".to_string()];
    let mut origin = vec![10, 20];
    let mut coords = HashMap::from([
        ("lat".to_string(), vec![30.0, 20.0, 10.0]),
        ("lon".to_string(), vec![0.0, 1.0, 2.0]),
    ]);
    let attrs = serde_json::Map::new();
    let (out, flipped) = check_and_orient_block_grid(
        values,
        &mut shape,
        &mut dims,
        &mut origin,
        &attrs,
        &mut coords,
    );
    assert_eq!(dims, ["lat", "lon"], "names follow the transposed data");
    assert_eq!(origin, [20, 10]);
    assert!(
        flipped.is_empty(),
        "lat descends and lon ascends: nothing flips"
    );
    // Row j (lat), column i (lon) now holds the stored (lon i, lat j) value.
    assert_eq!(out, [0.0, 3.0, 6.0, 1.0, 4.0, 7.0, 2.0, 5.0, 8.0]);
}

#[test]
fn rows_and_columns_flip_together_with_an_odd_row_count() {
    // 3 rows (lat ascending) x 2 columns (lon descending), two time slices.
    let values: Vec<f32> = (0..12u8).map(f32::from).collect();
    let mut shape = vec![2, 3, 2];
    let mut dims = ["time", "lat", "lon"].map(String::from).to_vec();
    let mut origin = vec![0, 0, 0];
    let mut coords = HashMap::from([
        ("lat".to_string(), vec![-10.0, 0.0, 10.0]),
        ("lon".to_string(), vec![20.0, 10.0]),
    ]);
    let attrs = serde_json::Map::new();
    let (out, flipped) = check_and_orient_block_grid(
        values,
        &mut shape,
        &mut dims,
        &mut origin,
        &attrs,
        &mut coords,
    );
    assert_eq!(flipped, ["lat", "lon"]);
    let first_slice = [5.0, 4.0, 3.0, 2.0, 1.0, 0.0];
    assert_eq!(
        out[..6],
        first_slice,
        "north first, west first, middle row reversed only"
    );
    assert_eq!(
        out[6..],
        first_slice.map(|v| v + 6.0),
        "every slice the same way"
    );
    assert_eq!(coords["lat"], [10.0, 0.0, -10.0]);
    assert_eq!(coords["lon"], [10.0, 20.0]);
}
