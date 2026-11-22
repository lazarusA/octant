//! Which bricks a view wants: the roots
//! (every brick of the plotted level over the window), then, largest on
//! screen first, the children of bricks in the frustum whose voxels project
//! larger than `threshold` pixels, within `budget` bricks.

use std::collections::{BinaryHeap, HashSet};
use std::ops::Range;

use crate::data::multiscale::bricks::{BRICK, BrickKey, Bricks};
use crate::plots::camera::{FOV_SCALE, OrbitCamera};

/// Where level-0 positions sit in the volume's box.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placement {
    /// The plotted window's level-0 edges along x, y and z.
    pub lo: [f64; 3],
    pub hi: [f64; 3],
    /// Axes running the other way in the unit box (x right, y up, z toward
    /// the viewer) than in storage.
    pub mirror: [bool; 3],
    /// The box's world size.
    pub aspect: [f32; 3],
}

impl Placement {
    /// Object space position of level-0 position `q`.
    pub fn object_position(&self, q: [f64; 3]) -> [f32; 3] {
        std::array::from_fn(|i| {
            let s = (q[i] - self.lo[i]) / (self.hi[i] - self.lo[i]).max(1e-9);
            let unit = if self.mirror[i] { 1.0 - s } else { s };
            ((unit - 0.5) as f32) * self.aspect[i]
        })
    }

    /// Object space bounds of level-0 table cells `cells`, within the
    /// window.
    fn bounds(&self, cells: &[Range<u32>; 3]) -> ([f32; 3], [f32; 3]) {
        let edge = |c: u32, i: usize| (f64::from(c) * BRICK as f64).clamp(self.lo[i], self.hi[i]);
        let a = self.object_position(std::array::from_fn(|i| edge(cells[i].start, i)));
        let b = self.object_position(std::array::from_fn(|i| edge(cells[i].end, i)));
        (
            std::array::from_fn(|i| a[i].min(b[i])),
            std::array::from_fn(|i| a[i].max(b[i])),
        )
    }
}

/// What a selection asks of each brick: `empty` ones (drawing nothing) are
/// never split, `hidden` ones (behind matter) never wanted, and `refined`
/// ones (with finer bricks resident) stay split until their voxels shrink
/// below `KEEP_SPLIT` of the threshold, so a brick near the threshold does
/// not swap levels back and forth as the view moves a little.
#[derive(Clone, Copy)]
pub struct Skip<'a> {
    pub empty: &'a dyn Fn(BrickKey) -> bool,
    pub hidden: &'a dyn Fn(BrickKey) -> bool,
    pub refined: &'a dyn Fn(BrickKey) -> bool,
}

/// Share of the split threshold a refined brick's voxels must fall below
/// before it merges back (hysteresis).
pub const KEEP_SPLIT: f32 = 0.7;

/// The view a selection is made for.
pub struct View<'a> {
    pub camera: &'a OrbitCamera,
    pub place: Placement,
    /// Table cells inside the plotted window.
    pub window: [Range<u32>; 3],
    /// Canvas height in pixels.
    pub px_height: u32,
    /// Screen pixels a voxel may span before its brick splits.
    pub threshold: f32,
}

/// The bricks `view` wants, roots (`roots`' level) first, then refinements
/// largest on screen first, at most `budget` in all.
pub fn select(
    bricks: &Bricks,
    roots: usize,
    view: &View,
    budget: usize,
    skip: Skip,
) -> Vec<BrickKey> {
    let mut wanted = root_bricks(bricks, roots, &view.window);
    // Pyramids offset by half a cell give two parents a shared child: it is
    // chosen (and counts against the budget) once.
    let mut chosen: HashSet<BrickKey> = wanted.iter().copied().collect();
    let mut heap = BinaryHeap::new();
    for &key in &wanted {
        if !(skip.empty)(key) {
            push_if_coarse(bricks, view, key, skip, &mut heap);
        }
    }
    while let Some((_, key)) = heap.pop() {
        // Children out of view stay with the parent's (finest-wins table).
        let children: Vec<BrickKey> = bricks
            .children(key)
            .filter(|c| !chosen.contains(c) && in_view(bricks, view, *c) && !(skip.hidden)(*c))
            .collect();
        if wanted.len() + children.len() > budget {
            break;
        }
        for child in children {
            chosen.insert(child);
            wanted.push(child);
            if !(skip.empty)(child) {
                push_if_coarse(bricks, view, child, skip, &mut heap);
            }
        }
    }
    wanted
}

/// Every brick of `level` over table cells `window`.
pub fn root_bricks(bricks: &Bricks, level: usize, window: &[Range<u32>; 3]) -> Vec<BrickKey> {
    bricks.over(level, window).collect()
}

/// Queues `key` for splitting when it has finer levels, shows in the
/// frustum and its voxels span more than the threshold (less when already
/// refined, `KEEP_SPLIT`).
fn push_if_coarse(
    bricks: &Bricks,
    view: &View,
    key: BrickKey,
    skip: Skip,
    heap: &mut BinaryHeap<(u32, BrickKey)>,
) {
    if key.level == 0 {
        return;
    }
    if !in_view(bricks, view, key) {
        return;
    }
    let (lo, hi) = view.place.bounds(&bricks.cells(key));
    let px = voxel_pixels(bricks, view, key, lo, hi);
    let keep = if (skip.refined)(key) { KEEP_SPLIT } else { 1.0 };
    if px > view.threshold * keep {
        // Positive floats order as their bits.
        heap.push((px.to_bits(), key));
    }
}

/// Screen pixels one of `key`'s voxels spans at the box `lo..hi`'s point
/// nearest the camera.
fn voxel_pixels(bricks: &Bricks, view: &View, key: BrickKey, lo: [f32; 3], hi: [f32; 3]) -> f32 {
    let Some(level) = bricks.set.level(usize::from(key.level)) else {
        return 0.0;
    };
    let place = &view.place;
    let size = (0..3)
        .map(|i| {
            let f = level.factor.get(bricks.dims[i]).copied().unwrap_or(1.0);
            place.aspect[i] * (f / (place.hi[i] - place.lo[i]).max(1e-9)) as f32
        })
        .fold(0.0, f32::max);
    let eye = view.camera.eye();
    let nearest: [f32; 3] = std::array::from_fn(|i| eye[i].clamp(lo[i], hi[i]));
    let d = (0..3)
        .map(|i| (nearest[i] - eye[i]).powi(2))
        .sum::<f32>()
        .sqrt();
    let dist = d.max(view.camera.near());
    size / dist * FOV_SCALE * view.px_height as f32 * 0.5
}

/// Whether `key` overlaps the window and the frustum.
fn in_view(bricks: &Bricks, view: &View, key: BrickKey) -> bool {
    let cells = bricks.cells(key);
    let (lo, hi) = view.place.bounds(&cells);
    overlaps(&cells, &view.window) && !culled(view.camera, lo, hi)
}

/// Whether the box `lo..hi` lies wholly outside one of the frustum's planes.
fn culled(camera: &OrbitCamera, lo: [f32; 3], hi: [f32; 3]) -> bool {
    (0..5).any(|plane| {
        (0..8).all(|c| {
            let p = std::array::from_fn(|i| if c >> i & 1 == 0 { lo[i] } else { hi[i] });
            inside(camera, plane, p) < 0.0
        })
    })
}

fn overlaps(a: &[Range<u32>; 3], b: &[Range<u32>; 3]) -> bool {
    (0..3).all(|i| a[i].start.max(b[i].start) < a[i].end.min(b[i].end))
}

/// How far object point `p` lies inside frustum plane `plane` (0 near,
/// 1-4 left, right, bottom, top): negative outside.
fn inside(camera: &OrbitCamera, plane: usize, p: [f32; 3]) -> f32 {
    let v = camera.to_view(p);
    let dist = -v[2];
    let (w, h) = (camera.aspect * dist / FOV_SCALE, dist / FOV_SCALE);
    match plane {
        0 => dist - camera.near(),
        1 => w + v[0],
        2 => w - v[0],
        3 => h + v[1],
        _ => h - v[1],
    }
}
