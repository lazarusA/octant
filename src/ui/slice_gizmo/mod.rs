//! The slicer's gizmo, chosen in Settings > View (`Slicer::mode`, state and
//! its readings in `app::state::slicer`): the box mode's cut box drawn over
//! the plot, cut by dragging its edges (the two faces they border) and
//! corners (all of theirs); the planes mode's planes through volumes and
//! point clouds, each dragged along its axis. Released handles snap to cells.

mod cursor;
mod drag;
mod guide;
mod handles;
mod paint;
#[cfg(test)]
mod placement_tests;
mod planes;
mod shape;
mod space;
mod strokes;
#[cfg(test)]
mod tests;

use crate::app::{OctantApp, SliceMode, Slicer};
use handles::Handle;
use planes::PlaneEdge;
use space::Space;

pub use paint::draw;

/// How near (points) the pointer must be to grab a handle.
const GRAB_RADIUS: f32 = 8.0;

/// What the pointer holds: a box handle or a plane's outline edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Grip {
    Box(Handle),
    Plane(PlaneEdge),
}

impl Grip {
    /// Whether `self` lights up while `hovered` is: the same handle, or any
    /// edge of the same plane.
    fn lit_by(self, hovered: Option<Grip>) -> bool {
        match (self, hovered) {
            (Grip::Plane(a), Some(Grip::Plane(b))) => a.plane == b.plane,
            (a, b) => Some(a) == b,
        }
    }
}

/// The grip hovered and the one held, kept in egui temp memory.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct State {
    pub(super) hovered: Option<Grip>,
    /// The grip being dragged and where along it it was grabbed.
    held: Option<(Grip, f32)>,
}

fn state_id(response: &egui::Response) -> egui::Id {
    response.id.with("slice_gizmo")
}

pub(super) fn state(ui: &egui::Ui, response: &egui::Response) -> State {
    ui.data(|d| d.get_temp(state_id(response)))
        .unwrap_or_default()
}

/// The canvas plot's space when the gizmo shows: its guides shown, no
/// export running, and planes only through volumes and point clouds.
pub(super) fn shown_space(app: &OctantApp, rect: egui::Rect) -> Option<Space<'_>> {
    let slice = &app.plot_configs.slice;
    if !slice.guides_shown() || app.pending_export.is_some() {
        return None;
    }
    Space::of(app, rect).filter(|s| slice.mode == SliceMode::Box || s.is_box())
}

/// Whether the pointer is on (or holds) a handle: the hover card then
/// stays away.
pub fn is_hovered(ui: &egui::Ui, response: &egui::Response) -> bool {
    state(ui, response).hovered.is_some()
}

/// Hovers, drags and double-clicks the gizmo's handles, through the canvas
/// `response`; whether the gizmo took the pointer this frame, so the canvas
/// neither pans, orbits nor resets its view. While a handle is dragged the
/// view counts as interacting (`NavigationState::view_interacting`), so a
/// volume previews the moving cut at half resolution as it does an orbit.
pub fn interact(app: &mut OctantApp, ui: &egui::Ui, response: &egui::Response) -> bool {
    let mut state = state(ui, response);
    // A copy: the space borrows the app.
    let mut slice = app.plot_configs.slice;
    let Some(space) = shown_space(app, response.rect) else {
        ui.data_mut(|d| d.remove::<State>(state_id(response)));
        return false;
    };
    let under = response
        .hover_pos()
        .and_then(|p| grip_at(&space, &slice, p));
    if response.drag_started_by(egui::PointerButton::Primary) {
        let origin = ui.input(|i| i.pointer.press_origin());
        state.held = origin.and_then(|p| grip_at(&space, &slice, p));
    }
    let took = if let Some((grip, t)) = state.held {
        drag(&space, &mut slice, grip, t, response);
        if !response.dragged() {
            state.held = None;
        }
        true
    } else if response.double_clicked()
        && let Some((grip, _)) = under
    {
        reset(&mut slice, grip, space.dims);
        true
    } else {
        false
    };
    let over = state.held.or(under);
    state.hovered = over.map(|(g, _)| g);
    if let Some((grip, t)) = over {
        let icon = cursor::icon(&space, &slice, grip, t);
        ui.ctx().set_cursor_icon(icon);
    }
    ui.data_mut(|d| d.insert_temp(state_id(response), state));
    app.plot_configs.slice = slice;
    if took {
        app.nav.view_interacting = state.held.is_some();
    }
    took
}

/// Follows the pointer's motion with `grip`, snapping it to cells once
/// released.
fn drag(space: &Space, slice: &mut Slicer, grip: Grip, t: f32, response: &egui::Response) {
    let motion = response.drag_delta();
    let released = !response.dragged();
    match grip {
        Grip::Box(handle) => {
            drag::apply(space, slice, handle, t, motion);
            if released {
                drag::snap(space, slice, handle);
            }
        }
        Grip::Plane(edge) => {
            planes::apply(space, slice, edge, t, motion);
            let k = edge.plane;
            if released && let Some(at) = slice.planes[k] {
                slice.planes[k] = Some(space.snap_center(k, at));
            }
        }
    }
}

/// The grip under `pos` in the slicer's mode.
fn grip_at(space: &Space, slice: &Slicer, pos: egui::Pos2) -> Option<(Grip, f32)> {
    if slice.mode == SliceMode::Planes {
        let (edges, n) = planes::edges(&slice.planes);
        let (edge, t) = guide::hit(space, slice, &edges[..n], pos, GRAB_RADIUS)?;
        return Some((Grip::Plane(edge), t));
    }
    let (all, n) = handles::all(space.dims);
    let (handle, t) = guide::hit(space, slice, &all[..n], pos, GRAB_RADIUS)?;
    Some((Grip::Box(handle), t))
}

/// Back to the whole plot along the faces a box handle borders; a plane
/// back to the middle.
fn reset(slice: &mut Slicer, grip: Grip, dims: usize) {
    match grip {
        Grip::Box(handle) => {
            for k in (0..dims).filter(|&k| handle.sides[k].is_some()) {
                slice.lo[k] = 0.0;
                slice.hi[k] = 1.0;
            }
        }
        Grip::Plane(edge) => slice.planes[edge.plane] = Some(0.5),
    }
}
