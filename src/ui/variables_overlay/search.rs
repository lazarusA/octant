use crate::ui::icons::{Icon, IconSize, UiIconExt};

pub fn render_search_bar(ui: &mut egui::Ui, variable_search: &mut String) {
    ui.horizontal(|ui| {
        ui.icon(Icon::Search, IconSize::Sm);
        let search_has_text = !variable_search.is_empty();
        let edit_width = if search_has_text {
            (ui.available_width() - 26.0).max(60.0)
        } else {
            ui.available_width()
        };

        ui.add(
            egui::TextEdit::singleline(variable_search)
                .hint_text("Search variables...")
                .desired_width(edit_width),
        );

        if search_has_text && ui.close_button("Clear search").clicked() {
            variable_search.clear();
        }
    });
}
