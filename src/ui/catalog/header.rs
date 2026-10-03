use crate::ui::icons::{Icon, IconSize, IconTone, UiIconExt};
use crate::utils::stack_str;

pub fn render_header(ui: &mut egui::Ui, total_count: usize, should_close: &mut bool) {
    ui.horizontal(|ui| {
        ui.icon_toned(Icon::Catalog, IconSize::Md, IconTone::Strong);
        ui.heading("Dataset Catalog");

        let mut badge_buf = [0u8; 32];
        let badge_text = stack_str(&mut badge_buf, format_args!("{total_count} stores"));
        ui.label(
            egui::RichText::new(badge_text)
                .small()
                .color(ui.visuals().weak_text_color()),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.close_button("Close (Esc)").clicked() {
                *should_close = true;
            }
        });
    });

    ui.add(
        egui::Label::new(
            egui::RichText::new(
                "Curated cloud Zarr, Icechunk, and procedural ground-truth datasets. Select any dataset to load.",
            )
            .small()
            .italics(),
        )
        .wrap_mode(egui::TextWrapMode::Wrap),
    );
}
