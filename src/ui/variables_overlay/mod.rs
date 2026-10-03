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
use nav::SearchJump;

pub fn show_variables_overlay(app: &mut OctantApp, ctx: &egui::Context, canvas_rect: egui::Rect) {
    let shown = app
        .show_variables_overlay
        .then(|| dataset_key(app))
        .flatten();
    focus::note_shown(ctx, shown);
    if !app.show_variables_overlay {
        return;
    }

    let screen_size = ctx.input(|i| i.viewport_rect().size());
    let width = (screen_size.x * 0.28).clamp(280.0, 520.0);
    let max_height = (screen_size.y * 0.65).clamp(250.0, 750.0);
    app.variables_overlay_width = width;

    let origin = canvas_rect.left_top() + egui::vec2(8.0, 8.0);
    let area_resp = egui::Area::new(egui::Id::new("octant_variables_area"))
        .fixed_pos(origin)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_width(width);
                if show_panel(ui, app, max_height) {
                    close(app);
                }
            });
        });
    app.variables_overlay_width = area_resp.response.rect.width();
}

/// Collapsible "Variables" header with the search field and list; returns
/// `true` when the close button was clicked.
fn show_panel(ui: &mut egui::Ui, app: &mut OctantApp, max_height: f32) -> bool {
    let header_id = ui.make_persistent_id("variables_overlay_header");
    let mut should_close = false;
    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), header_id, true)
        .show_header(ui, |ui| {
            should_close = ui.panel_header(Icon::Variables, "Variables", "Close Variables Window");
        })
        .body(|ui| {
            let search =
                ui.search_field_response(&mut app.variable_search, "Search variables...", None);
            if focus::take_open_focus(ui.ctx()) {
                search.request_focus();
            }
            let jump = search_jump(ui, &search);
            egui::ScrollArea::vertical()
                .max_height(max_height)
                .min_scrolled_height(max_height)
                .auto_shrink([false, false])
                .show(ui, |ui| body::show_list(ui, app, search.id, jump));
        });
    should_close
}

/// Hide the overlay; with the controls panel also closed, drop the unplotted
/// selection.
fn close(app: &mut OctantApp) {
    app.show_variables_overlay = false;
    if !app.show_variable_controls {
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
    app.active_dataset_metadata
        .as_ref()
        .map(|_| egui::Id::new(("variables_dataset", app.metadata_generation)))
}
