//! Which bricks sit in the atlas's slots, and the CPU mirror of the
//! indirection table: one texel per level-0
//! table cell holding the slot (`xyz`) and `level + 1` (`w`, 0 when none)
//! of the finest resident brick over it. A brick placed writes its cells
//! where only coarser levels or nothing were; one evicted hands its cells
//! back to the nearest resident coarser brick.

use std::collections::HashMap;
use std::ops::Range;

use crate::data::multiscale::bricks::{BrickKey, Bricks, cell_index, table_cells};

/// Table cells along x, y and z (half-open).
pub type CellBox = [Range<u32>; 3];

struct Slot<T> {
    key: BrickKey,
    /// What the brick was read as (the app keeps its block for the hover).
    data: T,
    /// The last selection that wanted it.
    last: u64,
    pinned: bool,
}

pub struct Residency<T> {
    slots: [u32; 3],
    used: Vec<Option<Slot<T>>>,
    by_key: HashMap<BrickKey, u32>,
    grid: [u32; 3],
    levels: usize,
    table: Vec<[u8; 4]>,
}

impl<T> Residency<T> {
    /// Empty slots `slots` per axis over a table of `grid` cells, for a
    /// stack of `levels` levels.
    pub fn new(grid: [u32; 3], slots: [u32; 3], levels: usize) -> Self {
        let cells = table_cells(grid).unwrap_or(0);
        let count = table_cells(slots).unwrap_or(0);
        Self {
            slots,
            used: (0..count).map(|_| None).collect(),
            by_key: HashMap::new(),
            grid,
            levels,
            table: vec![[0; 4]; cells],
        }
    }

    pub fn slot_count(&self) -> usize {
        self.used.len()
    }

    pub fn holds(&self, key: &BrickKey) -> bool {
        self.by_key.contains_key(key)
    }

    /// Marks `key` as wanted by selection `generation`.
    pub fn touch(&mut self, key: &BrickKey, generation: u64) {
        if let Some(&s) = self.by_key.get(key)
            && let Some(Some(slot)) = self.used.get_mut(s as usize)
        {
            slot.last = generation;
        }
    }

    /// Resident `key`'s slot.
    pub fn slot_of(&self, key: &BrickKey) -> Option<[u32; 3]> {
        Some(self.coords(*self.by_key.get(key)?))
    }

    /// What resident `key` was read as, to change.
    pub fn get_mut(&mut self, key: &BrickKey) -> Option<&mut T> {
        let s = *self.by_key.get(key)?;
        Some(&mut self.used.get_mut(s as usize)?.as_mut()?.data)
    }

    /// What resident `key` was read as.
    pub fn get(&self, key: &BrickKey) -> Option<&T> {
        let s = *self.by_key.get(key)?;
        Some(&self.used.get(s as usize)?.as_ref()?.data)
    }

    /// Places `key` (read as `data`), wanted by selection `generation`, in
    /// a free slot or the least recently wanted one that is neither pinned
    /// nor wanted by that selection: its slot and the table cells changed;
    /// `None` when every slot is wanted (it waits for the next view). Ages
    /// count selections, not frames, so resting (no repaints) never
    /// stalls eviction, and nothing the view wants is evicted for another.
    pub fn insert(
        &mut self,
        bricks: &Bricks,
        key: BrickKey,
        data: T,
        generation: u64,
        pinned: bool,
    ) -> Option<([u32; 3], Vec<CellBox>)> {
        if let Some(&s) = self.by_key.get(&key) {
            return Some((self.coords(s), Vec::new()));
        }
        let s = self.free_slot(generation)?;
        let mut changed = Vec::new();
        if let Some(old) = self.used.get_mut(s as usize).and_then(Option::take) {
            self.by_key.remove(&old.key);
            changed.push(self.release(bricks, old.key, s));
        }
        self.by_key.insert(key, s);
        if let Some(slot) = self.used.get_mut(s as usize) {
            *slot = Some(Slot {
                key,
                data,
                last: generation,
                pinned,
            });
        }
        let coords = self.coords(s);
        let cells = bricks.cells(key);
        let entry = [
            coords[0] as u8,
            coords[1] as u8,
            coords[2] as u8,
            key.level + 1,
        ];
        self.each_cell(&cells, |e| {
            if e[3] == 0 || e[3] > key.level + 1 {
                *e = entry;
            }
        });
        changed.push(cells);
        Some((coords, changed))
    }

    /// The table texels over `cells`, x fastest, then y, then z.
    pub fn table_region(&self, cells: &CellBox) -> Vec<[u8; 4]> {
        let mut out = Vec::new();
        for z in cells[2].clone() {
            for y in cells[1].clone() {
                for x in cells[0].clone() {
                    out.push(self.table[self.index([x, y, z])]);
                }
            }
        }
        out
    }

    /// The finest resident brick over table cell `cell` and what it was
    /// read as.
    pub fn lookup(&self, bricks: &Bricks, cell: [u32; 3]) -> Option<(BrickKey, &T)> {
        let e = self.table.get(self.index(cell))?;
        let level = usize::from(e[3].checked_sub(1)?);
        let key = bricks.cell_brick(level, cell);
        let slot = self.used.get(*self.by_key.get(&key)? as usize)?.as_ref()?;
        Some((key, &slot.data))
    }

    /// A free slot, else the least recently wanted evictable one.
    fn free_slot(&self, generation: u64) -> Option<u32> {
        if let Some(s) = self.used.iter().position(Option::is_none) {
            return Some(s as u32);
        }
        self.used
            .iter()
            .enumerate()
            .filter_map(|(s, slot)| slot.as_ref().map(|slot| (s, slot)))
            .filter(|(_, slot)| !slot.pinned && slot.last < generation)
            .min_by_key(|(_, slot)| slot.last)
            .map(|(s, _)| s as u32)
    }

    /// Hands the cells `key` (in slot `s`) showed to the nearest resident
    /// coarser brick, or none; the cells it covered.
    fn release(&mut self, bricks: &Bricks, key: BrickKey, s: u32) -> CellBox {
        let coords = self.coords(s);
        let mine = [
            coords[0] as u8,
            coords[1] as u8,
            coords[2] as u8,
            key.level + 1,
        ];
        let cells = bricks.cells(key);
        let (levels, by_key) = (self.levels, &self.by_key);
        let (slots, grid) = (self.slots, self.grid);
        for z in cells[2].clone() {
            for y in cells[1].clone() {
                for x in cells[0].clone() {
                    let i = cell_index(grid, [x, y, z]);
                    if self.table[i] != mine {
                        continue;
                    }
                    self.table[i] = (usize::from(key.level) + 1..levels)
                        .find_map(|l| {
                            let s = *by_key.get(&bricks.cell_brick(l, [x, y, z]))?;
                            let c = coords_of(slots, s);
                            Some([c[0] as u8, c[1] as u8, c[2] as u8, l as u8 + 1])
                        })
                        .unwrap_or([0; 4]);
                }
            }
        }
        cells
    }

    fn each_cell(&mut self, cells: &CellBox, mut f: impl FnMut(&mut [u8; 4])) {
        for z in cells[2].clone() {
            for y in cells[1].clone() {
                for x in cells[0].clone() {
                    let i = self.index([x, y, z]);
                    if let Some(e) = self.table.get_mut(i) {
                        f(e);
                    }
                }
            }
        }
    }

    fn index(&self, cell: [u32; 3]) -> usize {
        cell_index(self.grid, cell)
    }

    fn coords(&self, s: u32) -> [u32; 3] {
        coords_of(self.slots, s)
    }
}

/// Slot `s`'s place in the atlas, x fastest.
fn coords_of(slots: [u32; 3], s: u32) -> [u32; 3] {
    [
        s % slots[0],
        (s / slots[0]) % slots[1],
        s / (slots[0] * slots[1]),
    ]
}
