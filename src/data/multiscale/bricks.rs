//! Bricks of a multiscale stack (bricked volume LOD, `app::volume_lod`):
//! cubes of
//! `BRICK`^3 cells of one level over three of its dimensions (x, y, z),
//! stored with an `APRON` of their neighbours' cells on every side so
//! trilinear filtering inside a brick never needs another one.
//!
//! Level 0 is cut into cells of `BRICK`^3 level-0 cells (the indirection
//! table's grid). Each level-0 cell belongs, at every level, to the brick
//! holding the cell's center (`cell_brick`); a brick covers the contiguous
//! level-0 cells that map to it (`cells`), and its children are the finer
//! level's bricks over those cells. Levels whose cells do not start on
//! level 0's cells (OME pyramids centered by half a cell) are off by at
//! most half a level cell, which the apron absorbs; other pyramids are not
//! bricked (`aligned`).

use std::ops::Range;

use super::types::Multiscale;

/// Interior cells per brick side.
pub const BRICK: u64 = 64;
/// Neighbour cells read on every side.
pub const APRON: u64 = 1;
/// Stored cells per brick side.
pub const STORED: u64 = BRICK + 2 * APRON;

/// A brick: its level and index along x, y and z.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BrickKey {
    pub level: u8,
    pub b: [u32; 3],
}

/// The bricks of `set` over its dimensions `dims` (x, y, z).
#[derive(Clone, Copy)]
pub struct Bricks<'a> {
    pub set: &'a Multiscale,
    pub dims: [usize; 3],
}

impl Bricks<'_> {
    /// Bricks of `level` along each axis.
    pub fn counts(&self, level: usize) -> [u32; 3] {
        let shape = |d: usize| {
            self.set
                .level(level)
                .and_then(|l| l.shape.get(d).copied())
                .unwrap_or(1)
        };
        self.dims.map(|d| shape(d).div_ceil(BRICK).max(1) as u32)
    }

    /// Whether levels `0..levels` brick level 0's table cells within their
    /// aprons: whole factors, and brick edges at most half a level cell
    /// from a table cell's edge (OME pyramids offset by half a cell are).
    /// Else a cell's edge lies past its brick's apron, and rays there would
    /// read the apron's clamped edge instead of the data.
    pub fn aligned(&self, levels: usize) -> bool {
        let cell = BRICK as f64;
        (0..levels).all(|l| {
            let Some(lvl) = self.set.level(l) else {
                return false;
            };
            self.dims.iter().all(|&d| {
                let f = lvl.factor.get(d).copied().unwrap_or(1.0);
                let off = lvl.offset.get(d).copied().unwrap_or(0.0).rem_euclid(cell);
                let gap = off.min(cell - off);
                f >= 1.0 && (f - f.round()).abs() < 1e-6 && gap <= f / 2.0 + 1e-6
            })
        })
    }

    /// Level 0's grid of `BRICK`-cell cells (the table's).
    pub fn grid(&self) -> [u32; 3] {
        self.counts(0)
    }

    /// The cells of `key`'s level it reads: its interior, clamped to the
    /// level. Its stored cube starts `APRON` cells before (filled from its
    /// neighbours), so reads stay inside chunks whose edges fall on brick
    /// edges.
    pub fn window(&self, key: BrickKey) -> Option<[Range<u64>; 3]> {
        let lvl = self.set.level(usize::from(key.level))?;
        let mut out = [0..0, 0..0, 0..0];
        for (i, &d) in self.dims.iter().enumerate() {
            let len = *lvl.shape.get(d)?;
            let start = (u64::from(key.b[i]) * BRICK).min(len.saturating_sub(1));
            out[i] = start..(start + BRICK).min(len);
        }
        Some(out)
    }

    /// The brick of `level` holding the center of level-0 table cell
    /// `cell` along axis `i`, clamped to the level's bricks.
    pub fn brick_along(&self, level: usize, i: usize, cell: u32) -> u32 {
        let Some(lvl) = self.set.level(level) else {
            return 0;
        };
        let d = self.dims[i];
        let (f, off) = (
            lvl.factor.get(d).copied().unwrap_or(1.0),
            lvl.offset.get(d).copied().unwrap_or(0.0),
        );
        let center = (f64::from(cell) + 0.5) * BRICK as f64;
        let b = ((center - off) / (f * BRICK as f64)).floor().max(0.0);
        (b as u32).min(self.counts(level)[i].saturating_sub(1))
    }

    /// The brick of `level` holding the center of table cell `cell`.
    pub fn cell_brick(&self, level: usize, cell: [u32; 3]) -> BrickKey {
        BrickKey {
            level: level as u8,
            b: std::array::from_fn(|i| self.brick_along(level, i, cell[i])),
        }
    }

    /// The table cells mapping to `key` (`cell_brick`): contiguous, since
    /// the brick a cell maps to never falls as the cell rises.
    pub fn cells(&self, key: BrickKey) -> [Range<u32>; 3] {
        let grid = self.grid();
        let level = usize::from(key.level);
        std::array::from_fn(|i| {
            let first_at = |b: u32| lower_bound(grid[i], |c| self.brick_along(level, i, c) >= b);
            first_at(key.b[i])..first_at(key.b[i] + 1)
        })
    }

    /// The bricks one level finer covering `key`'s table cells.
    pub fn children(&self, key: BrickKey) -> impl Iterator<Item = BrickKey> + use<> {
        let cells = match key.level {
            0 => std::array::from_fn(|_| 0..0),
            _ => self.cells(key),
        };
        self.over(usize::from(key.level).saturating_sub(1), &cells)
    }

    /// The bricks of `level` over the table cells `cells` (x fastest).
    pub fn over(
        &self,
        level: usize,
        cells: &[Range<u32>; 3],
    ) -> impl Iterator<Item = BrickKey> + use<> {
        let [sx, sy, sz]: [Range<u32>; 3] = std::array::from_fn(|i| {
            let r = &cells[i];
            if r.is_empty() {
                return 0..0;
            }
            self.brick_along(level, i, r.start)..self.brick_along(level, i, r.end - 1) + 1
        });
        let level = level as u8;
        sz.flat_map(move |z| {
            let sx = sx.clone();
            sy.clone().flat_map(move |y| {
                sx.clone().map(move |x| BrickKey {
                    level,
                    b: [x, y, z],
                })
            })
        })
    }
}

/// Cells of a box of `n` along x, y and z, `None` past `usize`.
pub fn table_cells(n: [u32; 3]) -> Option<usize> {
    n.iter().try_fold(1usize, |a, &g| a.checked_mul(g as usize))
}

/// The index of table cell `[x, y, z]` in a table of `grid` cells, x
/// fastest (`table_cells` bounds it).
pub fn cell_index(grid: [u32; 3], [x, y, z]: [u32; 3]) -> usize {
    (z as usize * grid[1] as usize + y as usize) * grid[0] as usize + x as usize
}

/// One bit per table cell (`cell_index`), in 32-bit words as the GPU's ray
/// feedback writes them: the cells rays reach.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CellBits {
    words: Vec<u32>,
}

impl CellBits {
    /// `cells` bits, none set.
    pub fn new(cells: usize) -> Self {
        Self {
            words: vec![0; cells.div_ceil(32)],
        }
    }

    /// The bits of `cells` cells in `words` (a readback; missing words unset).
    pub fn from_words(mut words: Vec<u32>, cells: usize) -> Self {
        words.resize(cells.div_ceil(32), 0);
        Self { words }
    }

    /// Cell `i`'s bit, `None` past the table.
    pub fn get(&self, i: usize) -> Option<bool> {
        self.words.get(i / 32).map(|w| w >> (i % 32) & 1 != 0)
    }

    pub fn set(&mut self, i: usize) {
        if let Some(w) = self.words.get_mut(i / 32) {
            *w |= 1 << (i % 32);
        }
    }

    /// Cells set.
    pub fn count(&self) -> usize {
        self.words.iter().map(|w| w.count_ones() as usize).sum()
    }
}

/// The first `c` in `0..len` where `pred` holds (`len` when none), for a
/// `pred` that once true stays true.
fn lower_bound(len: u32, pred: impl Fn(u32) -> bool) -> u32 {
    let (mut lo, mut hi) = (0, len);
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if pred(mid) {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }
    lo
}
