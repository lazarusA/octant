use crate::app::OctantApp;
use crate::ui::icons::{Icon, ToolbarButton, UiIconExt};

pub fn show_colormap_menu(app: &mut OctantApp, ui: &mut egui::Ui, compact: bool) {
    let button_response = ui.add(
        ToolbarButton::new(Icon::Colormap, "Colormap")
            .compact(compact)
            .owns_popup(),
    );
    egui::Popup::from_toggle_button_response(&button_response)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            render_colormap_contents(app, ui);
        });
}

fn render_colormap_contents(app: &mut OctantApp, ui: &mut egui::Ui) {
    ui.set_min_width(200.0);

    ui.horizontal(|ui| {
        ui.icon(Icon::Colormap, 14.0);
        ui.label(
            egui::RichText::new("Select Colormap Palette")
                .small()
                .weak(),
        );
    });
    ui.separator();

    let colormaps = [
        (0, "Viridis (Thermal)"),
        (1, "Plasma (Spectral)"),
        (2, "Inferno (Radiance)"),
        (3, "Magma (Density)"),
        (4, "Turbo (Rainbow)"),
        (5, "Coolwarm (Diverging)"),
        (6, "Cividis (Accessible)"),
    ];

    for (id, name) in colormaps {
        let is_active = app.active_colormap == id;
        let response = ui.selectable_label(is_active, name);

        if response.hovered() && app.preview_colormap != Some(id) {
            app.preview_colormap = Some(id);
            ui.ctx().request_repaint();
        }

        if response.clicked() {
            if app.rgb_composite_mode {
                app.rgb_composite_mode = false;
                app.load_selected_variable_block();
            }
            app.active_colormap = id;
            app.preview_colormap = None;
        }
    }

    ui.separator();
    ui.checkbox(&mut app.show_colorbar, "Show Colorbar");
    if app.show_colorbar {
        ui.add(
            egui::Slider::new(&mut app.colorbar_transparency, 0.0..=1.0)
                .custom_formatter(|n, _| format!("{:.0}%", n * 100.0)),
        );
        ui.label(egui::RichText::new("Box transparency"));
    }
}
