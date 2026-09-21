//! Sample slash chips and interactive quick-load chip buttons.

use crate::app::OctantApp;

pub fn sample_slash_chips_row(ui: &mut egui::Ui, app: &mut OctantApp) {
    let samples: [(&str, &str, &str); 3] = [
        (
            "/seasfire",
            "https://s3.bgc-jena.mpg.de:9000/misc/seasfire_rechunked.zarr",
            "Global wildfire & climate rechunked dataset (Zarr)",
        ),
        (
            "/sentinel-2",
            "https://sentinel-cogs.s3.us-west-2.amazonaws.com/sentinel-s2-l2a-cogs/36/Q/WD/2020/7/S2A_36QWD_20200701_0_L2A/TCI.tif",
            "Sentinel-2 L2A True Color COG (AWS S3)",
        ),
        (
            "/procedural-4d",
            "procedural://volume4d",
            "Synthetic 4D spatiotemporal volume",
        ),
    ];

    let prefix = "try:";
    let prefix_font = egui::FontId::monospace(11.0);
    let prefix_galley = ui.painter().layout_no_wrap(
        prefix.to_string(),
        prefix_font,
        ui.visuals().weak_text_color(),
    );

    let chip_font = egui::FontId::monospace(11.0);
    let chip_padding = egui::vec2(12.0, 6.0);
    let chip_widths: f32 = samples
        .iter()
        .map(|(label, _, _)| {
            let g = ui.painter().layout_no_wrap(
                label.to_string(),
                chip_font.clone(),
                ui.visuals().text_color(),
            );
            g.size().x + chip_padding.x
        })
        .sum();

    let total_w = prefix_galley.size().x + chip_widths + ((samples.len() - 1) as f32 * 6.0) + 6.0;

    if total_w <= ui.available_width() - 16.0 {
        let pad = ((ui.available_width() - total_w) * 0.5).max(0.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            if pad > 0.0 {
                ui.add_space(pad);
            }

            ui.label(
                egui::RichText::new(prefix)
                    .monospace()
                    .size(11.0)
                    .color(ui.visuals().weak_text_color().gamma_multiply(0.6)),
            );

            for (label, uri, desc) in samples {
                let resp = render_ghost_slash_chip(ui, label, desc);
                if resp.clicked() {
                    app.hero_state.input = uri.to_string();
                    app.submit_or_activate_source(uri, None);
                }
            }
        });
    } else {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(6.0, 4.0);
            ui.label(
                egui::RichText::new(prefix)
                    .monospace()
                    .size(11.0)
                    .color(ui.visuals().weak_text_color().gamma_multiply(0.6)),
            );

            for (label, uri, desc) in samples {
                let resp = render_ghost_slash_chip(ui, label, desc);
                if resp.clicked() {
                    app.hero_state.input = uri.to_string();
                    app.submit_or_activate_source(uri, None);
                }
            }
        });
    }
}

pub fn render_ghost_slash_chip(ui: &mut egui::Ui, label: &str, desc: &str) -> egui::Response {
    let font_id = egui::FontId::monospace(11.0);
    let padding = egui::vec2(10.0, 4.0);

    // Layout text with slash dimmer than the rest
    let mut job = egui::text::LayoutJob::default();
    if let Some(rest) = label.strip_prefix('/') {
        job.append(
            "/",
            0.0,
            egui::TextFormat {
                font_id: font_id.clone(),
                color: ui.visuals().weak_text_color().gamma_multiply(0.5),
                ..Default::default()
            },
        );
        job.append(
            rest,
            0.0,
            egui::TextFormat {
                font_id,
                color: ui.visuals().text_color(),
                ..Default::default()
            },
        );
    } else {
        job.append(
            label,
            0.0,
            egui::TextFormat {
                font_id,
                color: ui.visuals().text_color(),
                ..Default::default()
            },
        );
    }

    let galley = ui.painter().layout_job(job);
    let desired_size = galley.size() + padding;

    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if ui.is_rect_visible(rect) {
        let is_hovered = response.hovered();
        let bg_color = if is_hovered {
            ui.visuals().widgets.hovered.bg_fill
        } else {
            egui::Color32::TRANSPARENT
        };

        let stroke = if is_hovered {
            egui::Stroke::new(
                0.8,
                ui.visuals()
                    .widgets
                    .hovered
                    .bg_stroke
                    .color
                    .gamma_multiply(0.6),
            )
        } else {
            egui::Stroke::NONE
        };

        if is_hovered {
            ui.painter()
                .rect(rect, 4.0, bg_color, stroke, egui::StrokeKind::Inside);
        }

        let text_pos = rect.center() - galley.size() * 0.5;
        let text_color = if is_hovered {
            ui.visuals().strong_text_color()
        } else {
            ui.visuals().text_color()
        };
        ui.painter().galley(text_pos, galley, text_color);
    }

    response.on_hover_text(format!("Load sample: {desc}"))
}
