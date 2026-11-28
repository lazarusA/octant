use crate::i18n::{CommonKey, DatasetKey, fill, tr};
use crate::ui::icons::{Icon, IconSize, IconTone, UiIconExt};

pub fn render_header(ui: &mut egui::Ui, total_count: usize, should_close: &mut bool) {
    ui.horizontal(|ui| {
        ui.icon_toned(Icon::Gallery, IconSize::Md, IconTone::Strong);
        ui.heading(tr(DatasetKey::GalleryTitle));

        let mut badge_buf = [0u8; 64];
        let badge_text = fill(
            &mut badge_buf,
            tr(DatasetKey::GalleryStoreCount),
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
            egui::RichText::new(tr(DatasetKey::GalleryBlurb))
                .small()
                .italics(),
        )
        .wrap_mode(egui::TextWrapMode::Wrap),
    );
    source_line(ui);
}

/// Where the list comes from: the docs gallery's page.
fn source_line(ui: &mut egui::Ui) {
    use crate::gallery::remote::GALLERY_PAGE;
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        let weak = ui.visuals().weak_text_color();
        let from = egui::RichText::new(tr(DatasetKey::GalleryFromDocs)).small();
        ui.label(from.color(weak));
        let page = GALLERY_PAGE.trim_start_matches("https://");
        ui.hyperlink_to(egui::RichText::new(page).small(), GALLERY_PAGE);
    });
}
