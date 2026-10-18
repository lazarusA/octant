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
    free: bool,
    /// Escape on the landing screen returns to the plot.
    hero: bool,
    canvas: bool,
    playback: bool,
}

impl Gate {
    /// `plot_shown`: a plot fills the canvas (the landing screen is not up).
    pub fn new(app: &OctantApp, ctx: &Context, plot_shown: bool) -> Self {
        let focused = ctx.memory(|m| m.focused());
        let panel = focus::escaped_area(ctx)
            .and_then(|area| Panel::ALL.into_iter().find(|p| p.area_id() == area));
        let unblocked = !egui::Popup::is_any_open(ctx) && !modal_open(ctx);
        let free = unblocked && !ctx.text_edit_focused();
        let idle = focused.is_none_or(|id| id == crate::app::canvas::canvas_id());
        let canvas = free && idle && plot_shown;
        // The landing screen's own widgets (the URL field) let Escape go;
        // its sample chips keep it, and so keep focus.
        let hero = unblocked
            && app.layout.show_hero
            && app.has_plot()
            && panel.is_none()
            && focused.is_none();
        Self {
            panel,
            free,
            hero,
            canvas,
            playback: canvas && app.has_animated_dimension(),
        }
    }

    pub fn allows(self, scope: Scope) -> bool {
        match scope {
            Scope::Global => true,
            Scope::Panel => self.panel.is_some(),
            Scope::Free => self.free,
            Scope::Hero => self.hero,
            Scope::Canvas => self.canvas,
            Scope::Playback => self.playback,
        }
    }
}

/// Whether a modal window (one keeping focus until it closes, `windows::trap`)
/// was open last frame.
fn modal_open(ctx: &Context) -> bool {
    ctx.memory(|m| m.top_modal_layer()).is_some()
}

/// Tracks the keyboard, then runs every shortcut pressed this frame whose
/// scope is active.
pub fn dispatch(app: &mut OctantApp, ctx: &Context, plot_shown: bool) {
    focus::begin_frame(ctx);
    regions::begin_frame(ctx);
    if !keymap::any_pressed(ctx) {
        return;
    }
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
        Action::GoToPanel(panel) => app.go_to_panel(ctx, panel),
        Action::GoToColorbar => {
            if !app.hero_shown() {
                crate::ui::colorbar::focus_next(app, ctx);
            }
        }
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
        Action::ShowShortcuts => app.layout.show_shortcuts = true,
        Action::ToggleHero => app.toggle_hero(),
        Action::LeaveHero => app.layout.show_hero = false,
        Action::PlotTypePicker => {
            let shown = app
                .selected_variable_info()
                .or_else(|| app.plotted_variable_info());
            app.layout.show_plot_picker = shown.is_some();
        }
        Action::PlotStaged => {
            if app.selected.variable_info().is_some() {
                app.plot_from_panel();
            }
        }
        // Regions hold focus outside modal windows only.
        Action::NextRegion | Action::PreviousRegion | Action::FocusTopBar if modal_open(ctx) => {}
        Action::NextRegion => regions::cycle(ctx, true),
        Action::PreviousRegion => regions::cycle(ctx, false),
        Action::FocusTopBar => regions::enter(ctx, regions::Region::TopBar),
        // Left for the canvas (`app::canvas::keys`), never taken here.
        Action::Canvas(_) => {}
    }
}
