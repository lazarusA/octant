//! The slice index finds each character's slice and survives a round trip.

use super::slice_index::{SliceIndex, slice_file};

fn sample() -> SliceIndex {
    // Slice 0 holds scattered common characters, slice 1 a run of the rest.
    SliceIndex::from_slices(&[
        vec![0x4E2D, 0x6587, 0x3002, 0x4E2E],
        vec![0x4E2F, 0x4E30, 0x4E31, 0x9FA5],
    ])
}

#[test]
fn characters_map_to_their_slice() {
    let index = sample();
    assert_eq!(index.slice_count(), 2);
    assert_eq!(index.slice_of('\u{4E2D}'), Some(0));
    assert_eq!(index.slice_of('\u{4E2E}'), Some(0));
    assert_eq!(index.slice_of('\u{4E30}'), Some(1));
    assert_eq!(index.slice_of('\u{9FA5}'), Some(1));
    assert_eq!(index.slice_of('\u{4E2C}'), None);
    assert_eq!(index.slice_of('a'), None);
}

#[test]
fn the_index_round_trips_and_rejects_damage() {
    let index = sample();
    let bytes = index.encode();
    assert_eq!(SliceIndex::decode(&bytes), Some(index));
    assert_eq!(SliceIndex::decode(&bytes[..bytes.len() - 1]), None);
    let mut wrong_magic = bytes.clone();
    wrong_magic[0] = b'X';
    assert_eq!(SliceIndex::decode(&wrong_magic), None);
    assert_eq!(slice_file(7), "007.otf");
}
