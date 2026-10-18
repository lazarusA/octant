//! Runs the keymap once per frame, before any panel draws, so shortcuts win
//! over the widgets that would otherwise read their keys.

use super::keymap::{self, Action, Scope, TABLE};
use super::{focus, regions};
use crate::app::OctantApp;
use crate::ui::panel_layout::Panel;
use egui::Context;

/// Which scopes are active this frame.
#[derive(Clone, Copy, Debug)]
pub struct Gate {
    /// The panel Escape closes.
    panel: Option<Panel>,
    canvas: bool,
    playback: bool,
}

impl Gate {
    /// `plot_shown`: a plot fills the canvas (the landing screen is not up).
    pub fn new(app: &OctantApp, ctx: &Context, plot_shown: bool) -> Self {
        let focused = ctx.memory(|m| m.focused());
        let panel = focus::escaped_layer(ctx)
            .and_then(|layer| Panel::ALL.into_iter().find(|p| p.area_id() == layer.id));
        let free = focused.is_none_or(|id| id == crate::app::canvas::canvas_id())
            && !egui::Popup::is_any_open(ctx)
            && !modal_open(app);
        let canvas = free && plot_shown;
        Self {
            panel,
            canvas,
            playback: canvas && app.has_animated_dimension(),
        }
    }

    pub fn allows(self, scope: Scope) -> bool {
        match scope {
            Scope::Global => true,
            Scope::Panel => self.panel.is_some(),
            Scope::Canvas => self.canvas,
            Scope::Playback => self.playback,
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
    let here =
        |s: &&keymap::Shortcut| gate.allows(s.scope) && !matches!(s.action, Action::Canvas(_));
    for shortcut in TABLE.iter().filter(here) {
        if keymap::take(ctx, shortcut).is_some() {
            run(app, ctx, gate, shortcut.action);
        }
    }
}

fn run(app: &mut OctantApp, ctx: &Context, gate: Gate, action: Action) {
    match action {
        Action::QuickSave => app.quick_save_canvas(),
        Action::ExportDialog => app.show_export_modal = true,
        Action::ToggleCrop => app.show_crop_overlay = !app.show_crop_overlay,
        Action::TogglePanel(panel) => app.toggle_panel(ctx, panel),
        Action::SearchVariables => {
            app.set_panel_open(ctx, Panel::Variables, true);
            crate::ui::variables_overlay::focus_search(ctx);
        }
        Action::ClosePanel => {
            if let Some(panel) = gate.panel {
                app.set_panel_open(ctx, panel, false);
            }
        }
        Action::PlayPause => app.toggle_playback(),
        Action::StepPrev => app.step_prev(),
        Action::StepNext => app.step_next(),
        Action::FirstStep => app.step_first(),
        Action::LastStep => app.step_last(),
        Action::ToggleLoop => app.playback.loop_playback = !app.playback.loop_playback,
        // Regions hold focus outside modal windows only.
        Action::NextRegion | Action::PreviousRegion | Action::FocusTopBar if modal_open(app) => {}
        Action::NextRegion => regions::cycle(ctx, true),
        Action::PreviousRegion => regions::cycle(ctx, false),
        Action::FocusTopBar => regions::enter(ctx, regions::Region::TopBar),
        // Left for the canvas (`app::canvas::keys`), never taken here.
        Action::Canvas(_) => {}
    }
}
