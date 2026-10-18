//! Language selection popup menu for the top bar and settings panel.

use crate::i18n::{CommonKey, Language, tr};
use crate::ui::icons::{Icon, IconSize, ToolbarButton, UiIconExt};
use crate::utils::stack_str;

/// The language button and its menu; the button's response.
pub fn show_language_menu(ui: &mut egui::Ui, compact: bool) -> egui::Response {
    let current_lang = crate::i18n::language();
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
            ui.label(egui::RichText::new(tr(CommonKey::Language)).strong());
        });
        ui.separator();

        for &lang in Language::ALL {
            let mut buf = [0; 48];
            let label = stack_str(
                &mut buf,
                format_args!("{} ({})", lang.display_name(), lang.code()),
            );
            if ui.selectable_label(current_lang == lang, label).clicked() {
                crate::i18n::set_language(lang);
                crate::ui::fonts::FontEngine::on_language_changed(ui.ctx(), lang);
                ui.close();
            }
        }
    });
    button_response
}
