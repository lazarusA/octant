//! Brick selection on a 256^3 isotropic stack of 3 levels (1, 8 and 64
//! bricks): a far view keeps the root, a close-up of a corner refines that
//! corner only, the budget caps the bricks.

use super::select::{Placement, Skip, View, select};
use crate::data::multiscale::bricks::{BrickKey, Bricks};
use crate::data::multiscale::{Multiscale, ScaleLevel};
use crate::plots::camera::OrbitCamera;

fn iso() -> Multiscale {
    Multiscale {
        name: "v".into(),
        dims: vec!["z".into(), "y".into(), "x".into()],
        levels: (0..3)
            .map(|l| ScaleLevel {
                path: format!("{l}"),
                shape: vec![256 >> l; 3],
                factor: vec![f64::from(1 << l); 3],
                offset: vec![0.0; 3],
            })
            .collect(),
    }
}

const PLACE: Placement = Placement {
    lo: [0.0; 3],
    hi: [256.0; 3],
    mirror: [false, true, true],
    aspect: [1.0; 3],
};

fn wanted(camera: &OrbitCamera, px: u32, budget: usize) -> Vec<BrickKey> {
    wanted_where(camera, px, budget, |_| false)
}

fn wanted_where(
    camera: &OrbitCamera,
    px: u32,
    budget: usize,
    empty: impl Fn(BrickKey) -> bool,
) -> Vec<BrickKey> {
    wanted_with(camera, px, budget, empty, |_| false)
}

fn wanted_with(
    camera: &OrbitCamera,
    px: u32,
    budget: usize,
    empty: impl Fn(BrickKey) -> bool,
    refined: impl Fn(BrickKey) -> bool,
) -> Vec<BrickKey> {
    let set = iso();
    let bricks = Bricks {
        set: &set,
        dims: [2, 1, 0],
    };
    let view = View {
        camera,
        place: PLACE,
        window: [0..4, 0..4, 0..4],
        px_height: px,
        threshold: 1.0,
    };
    let none = |_| false;
    let skip = Skip {
        empty: &empty,
        hidden: &none,
        refined: &refined,
    };
    select(&bricks, 2, &view, budget, skip)
}

const ROOT: BrickKey = BrickKey {
    level: 2,
    b: [0; 3],
};

#[test]
fn a_far_view_keeps_the_root() {
    assert_eq!(wanted(&OrbitCamera::default(), 64, 100), vec![ROOT]);
}

#[test]
fn a_close_up_refines_its_corner_only() {
    // Just in front of the corner at +x, +y (unit y up), the front face.
    let camera = OrbitCamera {
        rotation_x: 0.0,
        rotation_y: 0.0,
        distance: 0.05,
        target: [0.45, 0.45, 0.5],
        aspect: 1.0,
    };
    let keys = wanted(&camera, 1000, 1000);
    assert_eq!(keys[0], ROOT);
    let finest: Vec<_> = keys.iter().filter(|k| k.level == 0).collect();
    assert!(!finest.is_empty(), "{keys:?}");
    // +x is the last x brick; unit y up is storage row 0 (mirrored); the
    // front face (unit z 1) is storage plane 0.
    let nearest = BrickKey {
        level: 0,
        b: [3, 0, 0],
    };
    assert!(finest.contains(&&nearest), "{finest:?}");
    let far = BrickKey {
        level: 0,
        b: [0, 3, 3],
    };
    assert!(!keys.contains(&far), "the far corner stays coarse");
}

#[test]
fn the_budget_caps_the_bricks() {
    let camera = OrbitCamera {
        distance: 0.3,
        ..OrbitCamera::default()
    };
    let all = wanted(&camera, 2000, 1000);
    assert!(all.len() > 9, "{}", all.len());
    // Splitting the root needs 1 + 8 bricks.
    assert_eq!(wanted(&camera, 2000, 8), vec![ROOT]);
    assert!(wanted(&camera, 2000, 20).len() <= 20);
}

#[test]
fn bricks_drawing_nothing_never_split() {
    let camera = OrbitCamera {
        distance: 0.3,
        ..OrbitCamera::default()
    };
    assert_eq!(
        wanted_where(&camera, 2000, 1000, |k| k.level == 2),
        vec![ROOT]
    );
    // Level-1 bricks known empty keep their level-0 children out.
    let keys = wanted_where(&camera, 2000, 1000, |k| k.level == 1);
    assert!(keys.iter().all(|k| k.level > 0), "{keys:?}");
}

#[test]
fn a_refined_brick_stays_split_a_little_below_the_threshold() {
    let camera = OrbitCamera::default();
    // The smallest canvas on which the root splits.
    let split = (64..4096)
        .find(|&px| wanted(&camera, px, 1000).len() > 1)
        .expect("the root splits on some canvas");
    let below = (split as f32 * 0.85) as u32;
    assert_eq!(wanted(&camera, below, 1000), vec![ROOT], "not split yet");
    let kept = wanted_with(&camera, below, 1000, |_| false, |k| k == ROOT);
    assert!(kept.len() > 1, "already split: kept");
    let far = (split as f32 * 0.5) as u32;
    let merged = wanted_with(&camera, far, 1000, |_| false, |k| k == ROOT);
    assert_eq!(merged, vec![ROOT], "well below: merged back");
}

#[test]
fn a_child_of_two_parents_is_chosen_once() {
    // 512^3 over 3 levels, level 2 offset by -64 level-0 cells: its brick
    // edge (level-0 192) falls inside level-1 brick 1 (128..256), a child of
    // both level-2 bricks along each axis.
    let level = |l: u32, offset: f64| ScaleLevel {
        path: format!("{l}"),
        shape: vec![512 >> l; 3],
        factor: vec![f64::from(1 << l); 3],
        offset: vec![offset; 3],
    };
    let set = Multiscale {
        name: "v".into(),
        dims: vec!["z".into(), "y".into(), "x".into()],
        levels: vec![level(0, 0.0), level(1, 0.0), level(2, -64.0)],
    };
    let bricks = Bricks {
        set: &set,
        dims: [2, 1, 0],
    };
    let shared = BrickKey {
        level: 1,
        b: [1, 0, 0],
    };
    let parents = [0, 1].map(|x| BrickKey {
        level: 2,
        b: [x, 0, 0],
    });
    assert!(
        parents
            .iter()
            .all(|p| bricks.children(*p).any(|c| c == shared))
    );
    let camera = OrbitCamera {
        distance: 0.3,
        ..OrbitCamera::default()
    };
    let view = View {
        camera: &camera,
        place: Placement {
            hi: [512.0; 3],
            ..PLACE
        },
        window: [0..8, 0..8, 0..8],
        px_height: 4000,
        threshold: 1.0,
    };
    let none = |_| false;
    let skip = Skip {
        empty: &none,
        hidden: &none,
        refined: &none,
    };
    let keys = select(&bricks, 2, &view, 1000, skip);
    let parents_of = |k: &BrickKey| {
        let chosen = keys.iter().filter(|p| p.level == 2);
        chosen
            .filter(|p| bricks.children(**p).any(|c| c == *k))
            .count()
    };
    let shared_chosen = keys
        .iter()
        .filter(|k| k.level == 1)
        .any(|k| parents_of(k) > 1);
    assert!(shared_chosen, "a level-1 brick of two parents is chosen");
    let mut unique = keys.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(keys.len(), unique.len(), "chosen twice");
}
