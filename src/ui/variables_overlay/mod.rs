mod body;
mod focus;
mod item;
mod nav;
mod row;
mod search;
mod tree;

#[cfg(test)]
mod nav_tests;
#[cfg(test)]
mod open_tests;
#[cfg(test)]
mod search_tests;
#[cfg(test)]
mod tests;

pub use search::SearchCache;

use crate::app::OctantApp;
use crate::ui::icons::{Icon, UiIconExt};
use crate::ui::key_focus;
use crate::ui::panel_header::{self, PanelHeader};
use crate::ui::panel_layout::{self, Panel};
use nav::SearchJump;

/// Shortest list, however low the panel sits.
const MIN_LIST_HEIGHT: f32 = 120.0;

pub fn show_variables_overlay(app: &mut OctantApp, ctx: &egui::Context, canvas_rect: egui::Rect) {
    let shown = app
        .layout
        .show_variables_overlay
        .then(|| dataset_key(app))
        .flatten();
    focus::note_shown(ctx, shown);
    if !app.layout.show_variables_overlay {
        return;
    }

    let screen_size = ctx.input(|i| i.viewport_rect().size());
    let width = (screen_size.x * 0.28).clamp(280.0, 520.0);
    let list_height = (screen_size.y * 0.65).clamp(250.0, 750.0);
    let origin = panel_layout::origin(app, Panel::Variables, canvas_rect);
    app.layout.variables_overlay_width = width;

    let mut header = None;
    let area_resp = egui::Area::new(egui::Id::new("octant_variables_area"))
        .fixed_pos(origin)
        .constrain_to(canvas_rect)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_width(width);
                header = show_panel(ui, app, list_height, canvas_rect);
            });
        });
    let rect = area_resp.response.rect;
    app.layout.variables_overlay_width = rect.width();
    if let Some(header) = header {
        panel_layout::apply_grip(app, Panel::Variables, header.grip, rect, canvas_rect);
        if header.close {
            close(app);
        }
    }
}

/// Collapsible "Variables" header with the search field and list (at most
/// `list_height` tall, shorter rather than past the bottom of `canvas`);
/// returns what the header's close button and grip asked for.
fn show_panel(
    ui: &mut egui::Ui,
    app: &mut OctantApp,
    list_height: f32,
    canvas: egui::Rect,
) -> Option<PanelHeader> {
    let header_id = ui.make_persistent_id("variables_overlay_header");
    let mut header = None;
    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), header_id, true)
        .show_header(ui, |ui| {
            header = Some(panel_header::show(
                ui,
                Icon::Variables,
                "Variables",
                "Close Variables Window",
            ));
        })
        .body(|ui| {
            let search = ui.search_field_response(
                &mut app.layout.variable_search,
                "Search variables...",
                None,
            );
            if focus::take_open_focus(ui.ctx()) {
                search.request_focus();
            }
            let jump = search_jump(ui, &search);
            let max_height = list_height
                .min(panel_layout::room_below(ui, canvas))
                .max(MIN_LIST_HEIGHT);
            egui::ScrollArea::vertical()
                .max_height(max_height)
                .min_scrolled_height(max_height)
                .auto_shrink([false, false])
                .show(ui, |ui| body::show_list(ui, app, search.id, jump));
        });
    header
}

/// Hide the overlay; with the controls panel also closed, drop the unplotted
/// selection.
fn close(app: &mut OctantApp) {
    app.layout.show_variables_overlay = false;
    if !app.layout.show_variable_controls {
        app.revert_selected_state_to_plotted();
    }
}

/// Down in the search field moves to the first row; Enter (which ends the
/// single-line edit) moves to the first matching variable. With modifiers
/// (Shift+Down selects, Cmd+Down moves the cursor) the keys stay in the field.
fn search_jump(ui: &egui::Ui, search: &egui::Response) -> Option<SearchJump> {
    let pressed = |key| key_focus::pressed(ui.ctx(), [key]).is_some();
    if search.has_focus() && pressed(egui::Key::ArrowDown) {
        Some(SearchJump::FirstRow)
    } else if search.lost_focus() && pressed(egui::Key::Enter) {
        Some(SearchJump::FirstVariable)
    } else {
        None
    }
}

/// Identity of the dataset listed in the overlay (its load counter), `None`
/// while none is loaded.
fn dataset_key(app: &OctantApp) -> Option<egui::Id> {
    app.selected
        .metadata
        .as_ref()
        .map(|_| egui::Id::new(("variables_dataset", app.selected.metadata_generation)))
}
