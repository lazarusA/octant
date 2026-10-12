use crate::i18n::{CommonKey, DatasetKey, fill, tr};
use crate::ui::icons::{Icon, IconSize, IconTone, UiIconExt};

pub fn render_header(ui: &mut egui::Ui, total_count: usize, should_close: &mut bool) {
    ui.horizontal(|ui| {
        ui.icon_toned(Icon::Catalog, IconSize::Md, IconTone::Strong);
        ui.heading(tr(DatasetKey::CatalogTitle));

        let mut badge_buf = [0u8; 64];
        let badge_text = fill(
            &mut badge_buf,
            tr(DatasetKey::CatalogStoreCount),
            &[&total_count],
        );
        ui.label(
            egui::RichText::new(badge_text)
                .small()
                .color(ui.visuals().weak_text_color()),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.close_button(tr(CommonKey::CloseEsc)).clicked() {
                *should_close = true;
            }
        });
    });

    ui.add(
        egui::Label::new(
            egui::RichText::new(tr(DatasetKey::CatalogBlurb))
                .small()
                .italics(),
        )
        .wrap_mode(egui::TextWrapMode::Wrap),
    );
}
