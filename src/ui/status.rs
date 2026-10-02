use crate::{
    app::OctantApp,
    ui::icons::{Icon, IconTone, ToolbarButton},
};

/// Spinner diameter shown next to the fetch progress label.
const SPINNER_SIZE: f32 = 12.0;

/// Snapshot of an ongoing block fetch, used to size and draw the status item.
pub struct FetchProgress {
    label: String,
    compact: bool,
}

impl FetchProgress {
    /// Returns `None` when no fetch is in flight.
    pub fn from_app(app: &OctantApp, compact: bool) -> Option<Self> {
        if app.block_prefetcher.pending_count() == 0 {
            return None;
        }
        let completed_bytes = app.block_prefetcher.completed_bytes();
        let total_bytes = app
            .block_prefetcher
            .total_bytes()
            .max(completed_bytes)
            .max(1);
        let progress = (completed_bytes as f32 / total_bytes as f32).clamp(0.0, 1.0);

        let label = if compact {
            format!("{:.0}%", progress * 100.0)
        } else {
            format!(
                "Fetching {} / {} ({:.0}%)",
                crate::ui::variables_panel::format_byte_size(completed_bytes),
                crate::ui::variables_panel::format_byte_size(total_bytes),
                progress * 100.0
            )
        };
        Some(Self { label, compact })
    }

    fn rich_label(&self, ui: &egui::Ui) -> egui::RichText {
        egui::RichText::new(self.label.as_str())
            .small()
            .strong()
            .color(IconTone::Warning.color(ui.visuals()))
    }

    /// Width of the status item (abort button, spinner and label).
    pub fn width(&self, ui: &egui::Ui) -> f32 {
        let spacing = ui.spacing().item_spacing.x;
        let galley = egui::WidgetText::from(self.rich_label(ui)).into_galley(
            ui,
            Some(egui::TextWrapMode::Extend),
            f32::INFINITY,
            egui::TextStyle::Small,
        );
        ToolbarButton::width(ui, "Abort", self.compact)
            + SPINNER_SIZE
            + galley.size().x
            + spacing * 2.0
    }
}

pub fn show_status_bar(app: &mut OctantApp, ui: &mut egui::Ui, progress: &FetchProgress) {
    ui.horizontal(|ui| {
        // Abort button to interrupt ongoing calls
        if ui
            .add(
                ToolbarButton::new(Icon::Stop, "Abort")
                    .compact(progress.compact)
                    .hover("Interrupt and abort ongoing data transfer"),
            )
            .clicked()
        {
            app.abort_current_fetch();
        }

        ui.add(egui::Spinner::new().size(SPINNER_SIZE));
        ui.label(progress.rich_label(ui));
    });
}
