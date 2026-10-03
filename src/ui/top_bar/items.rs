//! Drawing and width measurement for individual top-bar items.

use super::layout::TopBarItem;
use crate::app::OctantApp;
use crate::ui::icons::{Icon, TOOLBAR_ITEM_HEIGHT, ToolbarButton};
use crate::ui::toolbar::{ItemWidths, SEPARATOR_WIDTH};
use crate::ui::{cache, colormap, plot_type, store};

const BRAND_LABEL: &str = "Octant";

/// Label of a toolbar-button item. Brand and Status are measured separately.
fn label(item: TopBarItem, app: &OctantApp) -> &'static str {
    match item {
        TopBarItem::Brand => BRAND_LABEL,
        TopBarItem::Dataset => "Dataset",
        TopBarItem::Variables => "Variables",
        TopBarItem::Dimensions => "Dimensions",
        TopBarItem::PlotType => app.active_plot_type.display_name(),
        TopBarItem::Colormap => "Colormap",
        TopBarItem::Settings => "Settings",
        TopBarItem::Status => "",
        TopBarItem::Cache => "Cache",
        TopBarItem::Theme => theme_icon_label(app).1,
    }
}

fn is_dark(app: &OctantApp) -> bool {
    app.theme_preference == egui::ThemePreference::Dark
}

fn theme_icon_label(app: &OctantApp) -> (Icon, &'static str) {
    if is_dark(app) {
        (Icon::Sun, "Light")
    } else {
        (Icon::Moon, "Dark")
    }
}

fn brand_label_text() -> egui::RichText {
    egui::RichText::new(BRAND_LABEL).strong().heading()
}

/// Widths of a button-like item (everything except Status).
pub(super) fn item_widths(item: TopBarItem, app: &OctantApp, ui: &egui::Ui) -> ItemWidths {
    let spacing = ui.spacing().item_spacing.x;
    let separator = SEPARATOR_WIDTH + spacing;

    if item == TopBarItem::Brand {
        let base = TOOLBAR_ITEM_HEIGHT + spacing + separator;
        let galley = egui::WidgetText::from(brand_label_text()).into_galley(
            ui,
            Some(egui::TextWrapMode::Extend),
            f32::INFINITY,
            egui::TextStyle::Heading,
        );
        return ItemWidths {
            full: base + galley.size().x + spacing,
            compact: base,
        };
    }

    let trailing = match item {
        TopBarItem::Cache => spacing + separator,
        _ => spacing,
    };
    // Width depends only on label and mode, so any glyph measures the same.
    let measure = |compact| {
        ToolbarButton::new(Icon::Settings, label(item, app))
            .compact(compact)
            .width(ui)
    };
    ItemWidths {
        full: measure(false) + trailing,
        compact: measure(true) + trailing,
    }
}

/// Octant logo (toggles the hero view) and, unless compact, the brand label
/// (opens About). Right-clicking the logo also opens About.
fn show_brand(app: &mut OctantApp, ui: &mut egui::Ui, compact: bool) {
    let icon_resp =
        crate::ui::hero::draw_octant_widget(ui, TOOLBAR_ITEM_HEIGHT, [-1.0, -1.0, -1.0], 0.0, 1.0)
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .on_hover_text("Toggle Hero / Landing View (right-click for About)");
    if icon_resp.clicked() {
        app.show_hero = !app.show_hero;
    }
    if icon_resp.secondary_clicked() {
        app.show_about_window = !app.show_about_window;
    }

    if !compact {
        let label_resp = ui
            .add(egui::Label::new(brand_label_text()).sense(egui::Sense::click()))
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .on_hover_text("About Octant");
        if label_resp.clicked() {
            app.show_about_window = !app.show_about_window;
        }
    }
    ui.separator();
}

/// Draw a button-like item in full or icon-only mode.
pub(super) fn show_item(item: TopBarItem, compact: bool, app: &mut OctantApp, ui: &mut egui::Ui) {
    match item {
        TopBarItem::Brand => show_brand(app, ui, compact),
        // Drawn by `show_contents`, which owns the fetch progress snapshot.
        TopBarItem::Status => {}
        TopBarItem::Dataset => store::show_store_menu(app, ui, compact),
        TopBarItem::Variables => {
            if ui
                .add(button(Icon::Variables, item, app, compact))
                .clicked()
            {
                app.show_variables_overlay = !app.show_variables_overlay;
                revert_if_panels_closed(app);
            }
        }
        TopBarItem::Dimensions => {
            let btn = button(Icon::Dimensions, item, app, compact)
                .hover("Toggle Variable Controls Panel");
            if ui.add(btn).clicked() {
                app.show_variable_controls = !app.show_variable_controls;
                revert_if_panels_closed(app);
            }
        }
        TopBarItem::PlotType => plot_type::show_plot_type_menu(app, ui, compact),
        TopBarItem::Colormap => colormap::show_colormap_menu(app, ui, compact),
        TopBarItem::Settings => {
            if ui.add(button(Icon::Settings, item, app, compact)).clicked() {
                app.show_settings_panel = !app.show_settings_panel;
            }
        }
        TopBarItem::Cache => cache::show_cache_menu(app, ui, compact),
        TopBarItem::Theme => show_theme_toggle(app, ui, compact),
    }
}

fn button(icon: Icon, item: TopBarItem, app: &OctantApp, compact: bool) -> ToolbarButton<'static> {
    ToolbarButton::new(icon, label(item, app))
        .compact(compact)
        .active(is_panel_open(item, app))
}

/// Whether the panel or overlay toggled by a panel button is currently shown.
fn is_panel_open(item: TopBarItem, app: &OctantApp) -> bool {
    match item {
        TopBarItem::Variables => app.show_variables_overlay,
        TopBarItem::Dimensions => app.show_variable_controls,
        TopBarItem::Settings => app.show_settings_panel,
        _ => false,
    }
}

fn revert_if_panels_closed(app: &mut OctantApp) {
    if !app.show_variables_overlay && !app.show_variable_controls {
        app.revert_selected_state_to_plotted();
    }
}

fn show_theme_toggle(app: &mut OctantApp, ui: &mut egui::Ui, compact: bool) {
    let dark = is_dark(app);
    let (icon, text) = theme_icon_label(app);
    let hover = if dark {
        "Switch to Light mode"
    } else {
        "Switch to Dark mode"
    };
    let btn = ToolbarButton::new(icon, text).compact(compact).hover(hover);
    if ui.add(btn).clicked() {
        app.theme_preference = if dark {
            egui::ThemePreference::Light
        } else {
            egui::ThemePreference::Dark
        };
        ui.ctx().set_theme(app.theme_preference);
    }
}
