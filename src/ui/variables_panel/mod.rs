//! Variables Panel and Dimension Controller Subsystem.

pub mod dimension_slider;
#[cfg(test)]
mod header_tests;
pub mod info;

pub use dimension_slider::{
    calculate_download_sizes, calculate_max_animated_steps, calculate_selected_2d_elements,
    calculate_selected_volume_elements, double_slider_with_inputs, format_byte_size,
    init_layer_composite_defaults, init_variable_dimension_defaults, is_volume_allowed,
    is_volume_allowed_for_selection, show_dimension_sliders,
};
pub use info::show_variable_info;

use crate::app::OctantApp;
use crate::ui::drag_grip::GripAction;
use crate::ui::icons::{Icon, IconSize, IconTone, ToolbarButton, UiIconExt};
use crate::ui::panel_header::{self, PanelHeader};
use crate::ui::panel_layout::{self, Panel};

/// Width of the Dimensions panel's content; every dimension box fills it.
const PANEL_MAX_W: f32 = 340.0;
/// Narrowest the panel gets when the canvas leaves little room.
const PANEL_MIN_W: f32 = 220.0;

/// Docked after the Variables and Settings panels (`panel_layout`), or where its grip dragged it.
pub fn show_variable_controls(app: &mut OctantApp, ctx: &egui::Context, canvas_rect: egui::Rect) {
    if !app.layout.show_variable_controls || app.selected.metadata.is_none() {
        return;
    }

    let origin = panel_layout::origin(app, Panel::Dimensions, canvas_rect);

    // Fixed width, shrinking only when the canvas would otherwise clip it.
    // The 24 px covers the popup frame margins and the gap to the canvas edge.
    let room = canvas_rect.right() - origin.x - 24.0;
    let panel_w = PANEL_MAX_W.min(room).max(PANEL_MIN_W);

    let mut grip = GripAction::None;
    let area_resp = egui::Area::new(egui::Id::new("octant_variables_panel"))
        .fixed_pos(origin)
        .constrain_to(canvas_rect)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style())
                .stroke(egui::Stroke::NONE)
                .show(ui, |ui| {
                    ui.set_width(panel_w);
                    grip = show_panel(app, ui);
                });
        });
    let rect = area_resp.response.rect;
    panel_layout::apply_grip(app, Panel::Dimensions, grip, rect, canvas_rect);
}

/// The staged variable's header, Plot Data row and dimension sliders; what
/// the header's grip asked for.
fn show_panel(app: &mut OctantApp, ui: &mut egui::Ui) -> GripAction {
    let Some(var_info) = app.selected.variable_info().cloned() else {
        ui.label("No variable selected.");
        return GripAction::None;
    };
    let header = header_row(ui, &var_info);
    if header.close {
        app.layout.show_variable_controls = false;
        if !app.layout.show_variables_overlay {
            app.revert_selected_state_to_plotted();
        }
    }
    if plot_row(app, ui) {
        app.plot_from_panel();
    }

    ui.add_space(4.0);

    egui::CollapsingHeader::new("Dimension Sliders")
        .default_open(true)
        .show(ui, |ui| {
            show_dimension_sliders(app, ui, &var_info);
        });
    header.grip
}

/// The header's first row: the variable's name, expanding to its details,
/// the drag grip and the close button, and what those two asked for.
fn header_row(ui: &mut egui::Ui, var_info: &crate::data::VariableInfo) -> PanelHeader {
    let header_id = ui.make_persistent_id(("var_info_header", &var_info.name));
    let mut header = PanelHeader::default();
    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), header_id, false)
        .show_header(ui, |ui| {
            ui.icon(Icon::VariableDoc, IconSize::Sm);
            let name = match var_info.group_path() {
                Some(group) => format!("{} ({})", var_info.leaf_name(), group),
                None => var_info.name.clone(),
            };
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                header = panel_header::buttons(ui, "Close Dimension Panel");
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.add(egui::Label::new(egui::RichText::new(name).strong()).truncate());
                });
            });
        })
        .body(|ui| show_variable_info(ui, var_info));
    header
}

/// The header's second row, right-aligned: the "Add Overlay" toggle
/// (highlighted while on; disabled with the reason while the staged variable
/// can't be overlaid), then "Plot Data", which adds the variable as an
/// overlay while the toggle is on. Whether Plot Data was clicked.
fn plot_row(app: &mut OctantApp, ui: &mut egui::Ui) -> bool {
    let unavailable = app.overlay_unavailable_for(app.selected.variable_idx);
    if unavailable.is_some() {
        app.layout.plot_as_overlay = false;
    }
    let plot_icon = crate::ui::plot_type::plot_type_icon(app.selected.plot_type);
    let mut plot = false;
    // One row tall: a bare right-to-left layout would take the panel's height.
    let row = egui::vec2(ui.available_width(), ui.spacing().interact_size.y);
    let layout = egui::Layout::right_to_left(egui::Align::Center);
    ui.allocate_ui_with_layout(row, layout, |ui| {
        let hover = if app.layout.plot_as_overlay {
            "Fetch this variable and add it as an overlay over the plot"
        } else {
            "Fetch this variable and plot it"
        };
        plot = ui
            .outlined_icon_button(plot_icon, "Plot Data", IconTone::Default)
            .on_hover_text(hover)
            .clicked();
        let toggle = ToolbarButton::new(Icon::Layers, "Add Overlay")
            .icon_size(IconSize::Sm)
            .toggled(app.layout.plot_as_overlay)
            .hover("Plot Data adds this variable as an overlay while on");
        let response = ui.add_enabled(unavailable.is_none(), toggle);
        let response = match unavailable {
            Some(reason) => response.on_disabled_hover_text(reason),
            None => response,
        };
        if response.clicked() {
            app.layout.plot_as_overlay = !app.layout.plot_as_overlay;
        }
    });
    plot
}
