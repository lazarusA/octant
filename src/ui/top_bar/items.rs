//! Drawing and width measurement for individual top-bar items.

use super::layout::TopBarItem;
use crate::app::OctantApp;
use crate::i18n::{CommonKey, tr};
use crate::ui::icons::{Icon, TOOLBAR_ITEM_HEIGHT, ToolbarButton};
use crate::ui::toolbar::roving::{BarNav, Stop};
use crate::ui::toolbar::{ItemWidths, SEPARATOR_WIDTH};
use crate::ui::{cache, plot_type};

const BRAND_LABEL: &str = "Octant";

/// Label of a toolbar-button item. Brand and Status are measured separately.
fn label(item: TopBarItem, app: &OctantApp) -> &'static str {
    match item {
        TopBarItem::Brand => BRAND_LABEL,
        TopBarItem::Dataset => tr(CommonKey::Dataset),
        TopBarItem::Variables => tr(CommonKey::Variables),
        TopBarItem::Dimensions => tr(CommonKey::Dimensions),
        TopBarItem::PlotType => app.selected.plot_type.display_name(),
        TopBarItem::Settings => tr(CommonKey::Settings),
        TopBarItem::Status => "",
        TopBarItem::Cache => tr(CommonKey::Cache),
        TopBarItem::Language => crate::i18n::language().code(),
        TopBarItem::Theme => theme_icon_label(app).1,
    }
}

fn is_dark(app: &OctantApp) -> bool {
    app.layout.theme_preference == egui::ThemePreference::Dark
}

fn theme_icon_label(app: &OctantApp) -> (Icon, &'static str) {
    if is_dark(app) {
        (Icon::Sun, tr(CommonKey::Light))
    } else {
        (Icon::Moon, tr(CommonKey::Dark))
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
fn show_brand(app: &mut OctantApp, ui: &mut egui::Ui, compact: bool, nav: &mut BarNav) {
    let icon_resp =
        crate::ui::hero::draw_octant_widget(ui, TOOLBAR_ITEM_HEIGHT, [-1.0, -1.0, -1.0], 0.0, 1.0)
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .on_hover_text(tr(CommonKey::ToggleHeroHover));
    if icon_resp.clicked() {
        app.layout.show_hero = !app.layout.show_hero;
    }
    if icon_resp.secondary_clicked() {
        app.layout.show_about_window = !app.layout.show_about_window;
    }
    nav.push(&icon_resp, Stop::Button);

    if !compact {
        let label_resp = ui
            .add(egui::Label::new(brand_label_text()).sense(egui::Sense::click()))
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .on_hover_text(tr(CommonKey::AboutOctantHover));
        if label_resp.clicked() {
            app.layout.show_about_window = !app.layout.show_about_window;
        }
        nav.push(&label_resp, Stop::Button);
    }
    ui.separator();
}

/// Draw a button-like item in full or icon-only mode, adding its focusable
/// widgets to `nav`.
pub(super) fn show_item(
    item: TopBarItem,
    compact: bool,
    app: &mut OctantApp,
    ui: &mut egui::Ui,
    nav: &mut BarNav,
) {
    let (response, stop) = match item {
        TopBarItem::Brand => return show_brand(app, ui, compact, nav),
        // Drawn by `show_contents`, which owns the fetch progress snapshot.
        TopBarItem::Status => return,
        TopBarItem::Dataset => {
            let response = ui.add(button(Icon::Dataset, item, app, compact));
            if response.clicked() {
                app.layout.show_dataset_panel = !app.layout.show_dataset_panel;
            }
            (response, Stop::Button)
        }
        TopBarItem::Variables => {
            let response = ui.add(button(Icon::Variables, item, app, compact));
            if response.clicked() {
                app.layout.show_variables_overlay = !app.layout.show_variables_overlay;
                revert_if_panels_closed(app);
            }
            (response, Stop::Button)
        }
        TopBarItem::Dimensions => {
            let btn = button(Icon::Dimensions, item, app, compact)
                .hover(tr(CommonKey::ToggleVariableControlsHover));
            let response = ui.add(btn);
            if response.clicked() {
                app.layout.show_variable_controls = !app.layout.show_variable_controls;
                revert_if_panels_closed(app);
            }
            (response, Stop::Button)
        }
        TopBarItem::PlotType => menu(plot_type::show_plot_type_menu(app, ui, compact)),
        TopBarItem::Settings => {
            let response = ui.add(button(Icon::Settings, item, app, compact));
            if response.clicked() {
                app.layout.show_settings_panel = !app.layout.show_settings_panel;
            }
            (response, Stop::Button)
        }
        TopBarItem::Cache => menu(cache::show_cache_menu(app, ui, compact)),
        TopBarItem::Language => menu(crate::ui::language_picker::show_language_menu(ui, compact)),
        TopBarItem::Theme => (show_theme_toggle(app, ui, compact), Stop::Button),
    };
    nav.push(&response, stop);
}

fn menu(response: egui::Response) -> (egui::Response, Stop) {
    let stop = Stop::menu(&response);
    (response, stop)
}

fn button(icon: Icon, item: TopBarItem, app: &OctantApp, compact: bool) -> ToolbarButton<'static> {
    ToolbarButton::new(icon, label(item, app))
        .compact(compact)
        .active(is_panel_open(item, app))
}

/// Whether the panel or overlay toggled by a panel button is currently shown.
fn is_panel_open(item: TopBarItem, app: &OctantApp) -> bool {
    match item {
        TopBarItem::Dataset => app.layout.show_dataset_panel,
        TopBarItem::Variables => app.layout.show_variables_overlay,
        TopBarItem::Dimensions => app.layout.show_variable_controls,
        TopBarItem::Settings => app.layout.show_settings_panel,
        _ => false,
    }
}

fn revert_if_panels_closed(app: &mut OctantApp) {
    if !app.layout.show_variables_overlay && !app.layout.show_variable_controls {
        app.revert_selected_state_to_plotted();
    }
}

fn show_theme_toggle(app: &mut OctantApp, ui: &mut egui::Ui, compact: bool) -> egui::Response {
    let dark = is_dark(app);
    let (icon, text) = theme_icon_label(app);
    let hover = if dark {
        tr(CommonKey::SwitchToLightHover)
    } else {
        tr(CommonKey::SwitchToDarkHover)
    };
    let btn = ToolbarButton::new(icon, text).compact(compact).hover(hover);
    let response = ui.add(btn);
    if response.clicked() {
        app.layout.theme_preference = if dark {
            egui::ThemePreference::Light
        } else {
            egui::ThemePreference::Dark
        };
        ui.ctx().set_theme(app.layout.theme_preference);
    }
    response
}
