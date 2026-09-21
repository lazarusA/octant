use crate::ui::icons::{Icon, UiIconExt};

pub fn render_header(ui: &mut egui::Ui, total_count: usize, should_close: &mut bool) {
    ui.horizontal(|ui| {
        ui.icon_colored(Icon::Catalog, 16.0, ui.visuals().strong_text_color());
        ui.heading("Dataset Catalog");

        let mut badge_buf = [0u8; 32];
        let badge_text = format_count(&mut badge_buf, total_count, " stores");
        ui.label(
            egui::RichText::new(badge_text)
                .small()
                .color(ui.visuals().weak_text_color()),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .icon_button(Icon::Cross, "")
                .on_hover_text("Close (Esc)")
                .clicked()
            {
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

pub fn format_count<'a>(buf: &'a mut [u8; 32], count: usize, suffix: &str) -> &'a str {
    use std::io::Write;
    let mut cursor = std::io::Cursor::new(&mut buf[..]);
    let _ = write!(cursor, "{}{}", count, suffix);
    let len = cursor.position() as usize;
    std::str::from_utf8(&buf[..len]).unwrap_or("")
}
