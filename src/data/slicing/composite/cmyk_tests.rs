//! CMYK conversion through the N-dimensional composite path the app projects with.

use std::collections::HashMap;

use super::{slice_cmyk_composite, slice_rgb_composite_nd};
use crate::data::octant_block::OctantBlock;

/// A `[4, 2, 3]` CMYK block (or plain bands without the photometric tag).
fn ink_block(cmyk: bool) -> OctantBlock {
    let values: Vec<f32> = (0..24).map(|i| (i * 10) as f32).collect();
    let attributes = if cmyk {
        HashMap::from([("photometric".to_string(), "cmyk".to_string())])
    } else {
        HashMap::new()
    };
    let dims = ["band", "y", "x"].map(String::from).to_vec();
    OctantBlock::new(
        "ink".into(),
        vec![4, 2, 3],
        dims,
        vec![0, 0, 0],
        values,
        HashMap::new(),
        attributes,
    )
}

fn nd(block: &OctantBlock) -> Vec<f32> {
    let channels = [Some(0), Some(1), Some(2)];
    let matrix = slice_rgb_composite_nd(block, 0, 2, 1, (0, 3), (0, 2), &[0, 0, 0], channels, 1);
    matrix.expect("composite").values.to_vec()
}

#[test]
fn nd_composite_converts_cmyk_inks() {
    let block = ink_block(true);
    let expected = slice_cmyk_composite(&block, 3, 2, 6, 1).expect("cmyk composite");
    assert_eq!(nd(&block), expected.values.to_vec());
    assert_ne!(
        nd(&block),
        nd(&ink_block(false)),
        "inks are not drawn as RGB bands"
    );
}

#[test]
fn nd_cmyk_composite_reads_only_the_requested_window() {
    let block = ink_block(true);
    let channels = [Some(0), Some(1), Some(2)];
    let window = slice_rgb_composite_nd(&block, 0, 2, 1, (1, 3), (1, 2), &[0, 0, 0], channels, 1);
    let window = window.expect("windowed composite");
    assert_eq!((window.width, window.height), (2, 1));

    // The same pixels (y = 1, x = 1..3 of each ink) as a block of their own.
    let pixels: Vec<f32> = (0..4)
        .flat_map(|ink| [4, 5].map(|x| block.values[ink * 6 + x]))
        .collect();
    let dims = ["band", "y", "x"].map(String::from).to_vec();
    let cropped = OctantBlock::new(
        "ink".into(),
        vec![4, 1, 2],
        dims,
        vec![0, 0, 0],
        pixels,
        HashMap::new(),
        block.attributes.clone(),
    );
    let expected = slice_cmyk_composite(&cropped, 2, 1, 2, 1).expect("cropped composite");
    assert_eq!(window.values.to_vec(), expected.values.to_vec());
}
