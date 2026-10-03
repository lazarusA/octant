//! The overlay's list area: loading and empty states, or the variable tree
//! (filtered by the search), and applying a picked variable.

use super::nav::SearchJump;
use super::search;
use super::tree::{VariableTreeContext, render_tree_group};
use crate::app::OctantApp;
use crate::ui::icons::{Icon, UiIconExt};

/// List area under the search field: the state that applies, or the tree.
pub(super) fn show_list(
    ui: &mut egui::Ui,
    app: &mut OctantApp,
    search_id: egui::Id,
    jump: Option<SearchJump>,
) {
    if app.is_loading {
        show_loading(ui);
        return;
    }
    let Some(var_count) = app
        .active_dataset_metadata
        .as_ref()
        .map(|m| m.variables.len())
    else {
        if empty_state(
            ui,
            "No store metadata loaded yet.",
            "Fetch / Load Store Metadata",
        ) {
            let target = app.store_target_input.clone();
            app.submit_or_activate_source(&target, Some(app.selected_store_kind));
        }
        return;
    };
    if var_count == 0 {
        if empty_state(
            ui,
            "No variables found in this dataset store.",
            "Refresh Store Metadata",
        ) {
            app.inspect_active_store();
        }
        return;
    }
    if let Some(idx) = show_tree(ui, app, search_id, jump) {
        apply_selection(app, idx);
    }
}

fn show_loading(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(10.0);
        ui.spinner();
        ui.label(egui::RichText::new("Inspecting store metadata in background...").italics());
        ui.add_space(10.0);
    });
}

/// Centered message with one action button; returns `true` when clicked.
fn empty_state(ui: &mut egui::Ui, message: &str, action: &str) -> bool {
    ui.vertical_centered(|ui| {
        ui.add_space(10.0);
        ui.label(message);
        ui.add_space(6.0);
        let clicked = ui.icon_button(Icon::Search, action).clicked();
        ui.add_space(10.0);
        clicked
    })
    .inner
}

/// Draw the tree, or the search's no-match note; returns a newly picked variable.
fn show_tree(
    ui: &mut egui::Ui,
    app: &mut OctantApp,
    search_id: egui::Id,
    jump: Option<SearchJump>,
) -> Option<usize> {
    let metadata = app.active_dataset_metadata.as_ref()?;
    let tree = app
        .cached_variable_tree
        .get_or_insert_with(|| metadata.build_variable_tree());
    let query = app.variable_search.trim();
    let search_active = !query.is_empty();
    let root = if search_active {
        let generation = app.metadata_generation;
        search::filtered(
            &mut app.cached_search,
            generation,
            query,
            tree,
            &metadata.variables,
        )
    } else {
        // Don't keep a filtered copy of the tree around while not searching.
        app.cached_search = None;
        Some(&*tree)
    };

    let Some(root) = root else {
        ui.vertical_centered(|ui| {
            ui.add_space(10.0);
            ui.label(egui::RichText::new(format!("No variables matching '{query}'")).italics());
            ui.add_space(10.0);
        });
        return None;
    };

    let mut tree_ctx = VariableTreeContext {
        variables: &metadata.variables,
        selected_idx: app.selected_variable_idx,
        search_active,
        newly_selected_idx: None,
        search_id,
        search_jump: jump,
    };
    render_tree_group(ui, root, &mut tree_ctx);
    tree_ctx.newly_selected_idx
}

/// Select variable `idx`: reset its colorbar label and dimension defaults and
/// open its controls.
fn apply_selection(app: &mut OctantApp, idx: usize) {
    app.selected_variable_idx = idx;
    app.reset_colorbar_label();
    let var_info = app
        .active_dataset_metadata
        .as_ref()
        .and_then(|meta| meta.variables.get(idx).cloned());
    if let Some(var_info) = var_info {
        crate::ui::variables_panel::init_variable_dimension_defaults(app, &var_info);
        app.show_variable_controls = true;
    }
}
