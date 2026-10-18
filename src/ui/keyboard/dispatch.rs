//! Runs the keymap once per frame, before any panel draws, so shortcuts win
//! over the widgets that would otherwise read their keys.

use super::keymap::{self, Action, Scope, TABLE};
use super::{focus, regions};
use crate::app::OctantApp;
use egui::Context;

/// Which scopes are active this frame.
#[derive(Clone, Copy, Debug)]
pub struct Gate {
    canvas: bool,
}

impl Gate {
    /// `plot_shown`: a plot fills the canvas (the landing screen is not up).
    pub fn new(app: &OctantApp, ctx: &Context, plot_shown: bool) -> Self {
        let focused = ctx.memory(|m| m.focused());
        let free = focused.is_none_or(|id| id == crate::app::canvas::canvas_id())
            && !egui::Popup::is_any_open(ctx)
            && !modal_open(app);
        Self {
            canvas: free && plot_shown,
        }
    }

    pub fn allows(self, scope: Scope) -> bool {
        match scope {
            Scope::Global => true,
            Scope::Canvas => self.canvas,
        }
    }
}

/// A window that holds focus until it closes.
fn modal_open(app: &OctantApp) -> bool {
    let layout = &app.layout;
    layout.show_catalog_window
        || layout.show_about_window
        || layout.show_icon_gallery_window
        || app.show_export_modal
}

/// Tracks the keyboard, then runs every shortcut pressed this frame whose
/// scope is active.
pub fn dispatch(app: &mut OctantApp, ctx: &Context, plot_shown: bool) {
    focus::begin_frame(ctx);
    regions::begin_frame(ctx);
    let gate = Gate::new(app, ctx, plot_shown);
    for shortcut in TABLE.iter().filter(|s| gate.allows(s.scope)) {
        if keymap::take(ctx, shortcut).is_some() {
            run(app, ctx, shortcut.action);
        }
    }
}

fn run(app: &mut OctantApp, ctx: &Context, action: Action) {
    match action {
        Action::QuickSave => app.quick_save_canvas(),
        Action::ExportDialog => app.show_export_modal = true,
        Action::ToggleCrop => app.show_crop_overlay = !app.show_crop_overlay,
        // Regions hold focus outside modal windows only.
        Action::NextRegion | Action::PreviousRegion | Action::FocusTopBar if modal_open(app) => {}
        Action::NextRegion => regions::cycle(ctx, true),
        Action::PreviousRegion => regions::cycle(ctx, false),
        Action::FocusTopBar => regions::enter(ctx, regions::Region::TopBar),
    }
}
