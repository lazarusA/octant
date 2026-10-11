//! Language selection popup menu for the top bar and settings panel.

use crate::app::OctantApp;
use crate::i18n::Language;
use crate::ui::icons::{Icon, IconSize, ToolbarButton, UiIconExt};

pub fn show_language_menu(app: &mut OctantApp, ui: &mut egui::Ui, compact: bool) {
    let current_lang = app.layout.language;
    let button_response = ui.add(
        ToolbarButton::new(Icon::Language, current_lang.code())
            .compact(compact)
            .hover(current_lang.display_name())
            .owns_popup(),
    );

    egui::Popup::menu(&button_response).show(|ui| {
        ui.set_min_width(180.0);
        ui.horizontal(|ui| {
            ui.icon(Icon::Language, IconSize::Sm);
            ui.label(egui::RichText::new("Language").strong());
        });
        ui.separator();

        for &lang in Language::ALL_PHASE1 {
            let is_selected = app.layout.language == lang;
            let label = format!("{} ({})", lang.display_name(), lang.code());
            if ui.selectable_label(is_selected, label).clicked() {
                app.layout.language = lang;
                crate::ui::fonts::FontEngine::on_language_changed(ui.ctx(), lang);
                ui.close();
            }
        }
    });
}
