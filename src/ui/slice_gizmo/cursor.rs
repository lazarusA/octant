//! The resize cursor over a grip, as the crop tool's: pointing the way the
//! grip moves on screen (across an edge, out of a box corner, along a
//! plane's axis), snapped to the four resize cursors.

use super::Grip;
use super::guide::Guide;
use super::space::Space;
use crate::app::Slicer;
use egui::{CursorIcon, Vec2};

/// The cursor over `grip`, grabbed `t` along it.
pub fn icon(space: &Space, slice: &Slicer, grip: Grip, t: f32) -> CursorIcon {
    direction(space, slice, grip, t).map_or(CursorIcon::Grab, resize_icon)
}

fn direction(space: &Space, slice: &Slicer, grip: Grip, t: f32) -> Option<Vec2> {
    match grip {
        Grip::Box(h) if h.is_corner(space.dims) => {
            let mid = std::array::from_fn(|k| 0.5 * (slice.lo[k] + slice.hi[k]));
            Some(space.screen(h.point(slice, 0.0))? - space.screen(mid)?)
        }
        Grip::Box(h) => {
            let along = h.sides.iter().position(Option::is_none)?;
            let tangent = space.screen_dir(h.point(slice, t), along)?;
            Some(Vec2::new(-tangent.y, tangent.x))
        }
        Grip::Plane(e) => space.screen_dir(e.point(slice, t), e.plane),
    }
}

/// The resize cursor nearest `dir` (screen y down), either way along it.
pub fn resize_icon(dir: Vec2) -> CursorIcon {
    if dir.length_sq() < 1e-6 {
        return CursorIcon::Grab;
    }
    match dir.y.atan2(dir.x).to_degrees().rem_euclid(180.0) {
        a if !(22.5..157.5).contains(&a) => CursorIcon::ResizeHorizontal,
        a if a < 67.5 => CursorIcon::ResizeNwSe,
        a if a < 112.5 => CursorIcon::ResizeVertical,
        _ => CursorIcon::ResizeNeSw,
    }
}
