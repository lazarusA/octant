//! Sub-brick ranges for empty-space skipping inside a brick: per 8^3
//! sub-block of a brick's interior, the min and max of its valid values and
//! whether any is missing, over the sub-block and one cell more on each side
//! (the trilinear taps near it), as the dense volume's 8^3 blocks hold them
//! (`plots/volume/blocks.rs`). Rays skip sub-blocks that cannot show anything
//! (`sample_bricked.wgsl` `next_open_sample`).

use std::ops::Range;

use super::encode::Encoding;
use crate::data::multiscale::bricks::{APRON, STORED};

/// Sub-blocks per brick side; each covers `BRICK / SUB` = 8 cells.
pub const SUB: usize = 8;
/// Sub-blocks per brick.
pub const SUBS: usize = SUB * SUB * SUB;
const _: () = assert!(SUBS as u64 == crate::plots::volume::bricks::SUBS_PER_BRICK);

/// A brick's sub-block ranges, x fastest: `[min, max, missing, 0]`.
pub type SubRanges = Vec<[f32; 4]>;

/// Every sub-block open (a composite's, always drawn).
pub(super) fn open_subs() -> SubRanges {
    vec![[f32::MIN, f32::MAX, 0.0, 0.0]; SUBS]
}

/// The sub-blocks along one axis whose range covers stored cell `c`: those
/// whose cells (stored `8s + 1 ..`) or the cell either side reach it.
fn subs_of(c: usize) -> Range<usize> {
    let width = (STORED as usize - 2 * APRON as usize) / SUB;
    let lo = c.saturating_sub(width + 1).div_ceil(width);
    let hi = (c / width).min(SUB - 1);
    lo.min(hi)..hi + 1
}

/// Widens `subs` with the f16 `values` (x fastest, decoded by `encoding`)
/// over the stored box `region`, returning the decoded `(min, max)` range
/// and whether any voxel in `values` was missing.
pub(super) fn widen_subs(
    subs: &mut [[f32; 4]],
    region: &[Range<usize>; 3],
    values: &[u16],
    encoding: Encoding,
) -> ((f32, f32), bool) {
    let mut values = values.iter();
    let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
    let mut missing = false;
    for z in region[2].clone() {
        let sz = subs_of(z);
        for y in region[1].clone() {
            let sy = subs_of(y);
            for x in region[0].clone() {
                let Some(&h) = values.next() else {
                    return ((lo, hi), missing);
                };
                let v = encoding.decode(h);
                if v.is_finite() {
                    lo = lo.min(v);
                    hi = hi.max(v);
                } else {
                    missing = true;
                }
                for k in sz.clone() {
                    for j in sy.clone() {
                        for i in subs_of(x) {
                            if let Some(s) = subs.get_mut((k * SUB + j) * SUB + i) {
                                widen(s, v);
                            }
                        }
                    }
                }
            }
        }
    }
    ((lo, hi), missing)
}

fn widen(s: &mut [f32; 4], v: f32) {
    if v.is_finite() {
        s[0] = s[0].min(v);
        s[1] = s[1].max(v);
    } else {
        s[2] = 1.0;
    }
}

/// Single-pass valid range, missing flag, and sub-block ranges of a whole
/// stored cube of f16 `values` (apron included: rays near its faces read it).
pub(super) fn sub_ranges(values: &[u16], encoding: Encoding) -> ((f32, f32), bool, SubRanges) {
    let mut subs = vec![[f32::INFINITY, f32::NEG_INFINITY, 0.0, 0.0]; SUBS];
    let n = STORED as usize;
    let (range, missing) = widen_subs(&mut subs, &[0..n, 0..n, 0..n], values, encoding);
    (range, missing, subs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_cell_reaches_its_sub_block_and_the_one_beside_at_an_edge() {
        // Stored cell 0 is the apron before sub-block 0; cells 1..9 are
        // sub-block 0's, 9 its neighbour's first, reached by both.
        assert_eq!(subs_of(0), 0..1);
        assert_eq!(subs_of(1), 0..1);
        assert_eq!(subs_of(8), 0..2);
        assert_eq!(subs_of(9), 0..2);
        assert_eq!(subs_of(10), 1..2);
        assert_eq!(subs_of(65), 7..8);
    }

    #[test]
    fn a_bright_voxel_opens_its_sub_block_only() {
        let n = STORED as usize;
        let zero = half::f16::from_f32(0.0).to_bits();
        let mut values = vec![zero; n * n * n];
        // Stored cell (20, 20, 20): sub-block (2, 2, 2) (cells 17..25).
        values[(20 * n + 20) * n + 20] = half::f16::from_f32(1.0).to_bits();
        let encoding = Encoding {
            offset: 0.0,
            scale: 100.0,
            byte: false,
        };
        let (range, missing, subs) = sub_ranges(&values, encoding);
        assert_eq!(range, (0.0, 100.0));
        assert!(!missing);
        let at = |i: usize, j: usize, k: usize| subs[(k * SUB + j) * SUB + i];
        assert_eq!(at(2, 2, 2)[1], 100.0);
        assert_eq!(at(3, 2, 2)[1], 0.0, "a sub-block away stays dark");
        assert_eq!(at(0, 0, 0)[1], 0.0);
    }
}
