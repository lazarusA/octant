//! Hero header title, status pill, idle hints, drag cues, and warning banners.

pub fn header_title(ui: &mut egui::Ui) {
    let avail_w = ui.available_width();
    let font_size = if avail_w < 380.0 {
        11.5
    } else if avail_w < 480.0 {
        12.5
    } else {
        13.5
    };

    let mut job = egui::text::LayoutJob::default();
    job.wrap.max_width = (avail_w - 24.0).max(100.0);
    job.halign = egui::Align::Center;
    job.append(
        "Bring data into ",
        0.0,
        egui::TextFormat {
            font_id: egui::FontId::monospace(font_size),
            color: ui.visuals().weak_text_color(),
            ..Default::default()
        },
    );
    job.append(
        "Octant",
        0.0,
        egui::TextFormat {
            font_id: egui::FontId::monospace(font_size),
            color: ui.visuals().strong_text_color(),
            ..Default::default()
        },
    );
    job.append(
        ". Start exploring.",
        0.0,
        egui::TextFormat {
            font_id: egui::FontId::monospace(font_size),
            color: ui.visuals().weak_text_color(),
            ..Default::default()
        },
    );
    ui.label(job);
}

pub fn render_status_pill(
    ui: &mut egui::Ui,
    icon: crate::ui::icons::Icon,
    icon_color: egui::Color32,
    text: &str,
    text_color: egui::Color32,
) {
    ui.add_space(16.0);
    let font_id = egui::FontId::monospace(11.0);
    let max_text_w = (ui.available_width() - 48.0).max(60.0);
    let galley = ui
        .painter()
        .layout(text.to_string(), font_id, text_color, max_text_w);
    let icon_size = 12.0;
    let gap = 6.0;
    let total_w = icon_size + gap + galley.size().x;
    let pad = ((ui.available_width() - total_w) * 0.5).max(0.0);

    ui.horizontal(|ui| {
        ui.set_width(ui.available_width());
        if pad > 0.0 {
            ui.add_space(pad);
        }
        crate::ui::icons::UiIconExt::icon_colored(ui, icon, icon_size, icon_color);
        ui.add_space(gap);
        ui.label(
            egui::RichText::new(galley.text())
                .monospace()
                .size(11.0)
                .color(text_color),
        );
    });
}

pub fn render_idle_hint(ui: &mut egui::Ui) {
    let avail_w = ui.available_width();
    let text = if avail_w < 340.0 {
        "paste URL, path, or drag & drop files"
    } else {
        "paste URL, local path, or drag & drop files anywhere"
    };
    ui.label(
        egui::RichText::new(text)
            .monospace()
            .size(10.0)
            .color(ui.visuals().strong_text_color()),
    );
}

pub fn render_drag_hover_cue(ui: &mut egui::Ui) {
    let width = (ui.available_width() - 24.0).clamp(180.0, 460.0);
    let height = 38.0;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());

    if ui.is_rect_visible(rect) {
        let is_dark = ui.visuals().dark_mode;
        let accent = if is_dark {
            egui::Color32::from_rgb(0, 190, 255)
        } else {
            egui::Color32::from_rgb(0, 125, 220)
        };
        let bg = if is_dark {
            egui::Color32::from_rgba_unmultiplied(0, 190, 255, 22)
        } else {
            egui::Color32::from_rgba_unmultiplied(0, 125, 220, 16)
        };

        ui.painter().rect(
            rect,
            6.0,
            bg,
            egui::Stroke::new(1.2, accent),
            egui::StrokeKind::Inside,
        );

        let icon_rect = egui::Rect::from_center_size(
            egui::pos2(rect.left() + 20.0, rect.center().y),
            egui::vec2(14.0, 14.0),
        );
        crate::ui::icons::Icon::DropTray.paint(ui.painter(), icon_rect, accent, is_dark);

        let msg = if width < 330.0 {
            "Drop dataset (.nc, .zarr, .icechunk, .tif)"
        } else {
            "Drop dataset to load (.nc, .h5, .zarr, .icechunk, .tif)"
        };

        ui.painter().text(
            egui::pos2(rect.left() + 34.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            msg,
            egui::FontId::monospace(10.5),
            accent,
        );
    }
}

pub fn render_warning_banner(ui: &mut egui::Ui) {
    let width = (ui.available_width() - 24.0).clamp(180.0, 460.0);
    let height = 36.0;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());

    if ui.is_rect_visible(rect) {
        let is_dark = ui.visuals().dark_mode;
        let warning_color = egui::Color32::from_rgb(255, 130, 60);
        let bg = if is_dark {
            egui::Color32::from_rgba_unmultiplied(255, 110, 50, 26)
        } else {
            egui::Color32::from_rgba_unmultiplied(255, 130, 60, 18)
        };

        ui.painter().rect(
            rect,
            6.0,
            bg,
            egui::Stroke::new(1.0, warning_color),
            egui::StrokeKind::Inside,
        );

        let icon_rect = egui::Rect::from_center_size(
            egui::pos2(rect.left() + 18.0, rect.center().y),
            egui::vec2(14.0, 14.0),
        );
        crate::ui::icons::Icon::Warning.paint(ui.painter(), icon_rect, warning_color, is_dark);

        let msg = if width < 340.0 {
            "Unsupported format (.nc, .zarr, .icechunk, .tif)"
        } else {
            "Unsupported type — supported: .nc, .h5, .zarr, .icechunk, .tif"
        };

        ui.painter().text(
            egui::pos2(rect.left() + 32.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            msg,
            egui::FontId::monospace(10.5),
            warning_color,
        );
    }
}
