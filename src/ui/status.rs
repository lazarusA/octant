use crate::{
    app::OctantApp,
    i18n::{CommonKey, fill, tr},
    ui::icons::{Icon, IconTone, ToolbarButton},
    utils::{ByteSize, stack_str},
};

/// Spinner diameter shown next to the fetch progress label.
const SPINNER_SIZE: f32 = 12.0;
/// Stack buffer large enough for the longest progress label.
type LabelBuf = [u8; 160];

/// Numbers of an ongoing block fetch. Labels are formatted on demand into
/// stack buffers, so building one per frame does not allocate.
#[derive(Clone, Copy, Debug)]
pub struct FetchProgress {
    completed: u64,
    total: u64,
    fraction: f32,
}

impl FetchProgress {
    /// Returns `None` when no fetch is in flight.
    pub fn from_app(app: &OctantApp) -> Option<Self> {
        let prefetcher = &app.block_prefetcher;
        if prefetcher.pending_count() == 0 {
            return None;
        }
        let completed = prefetcher.completed_bytes();
        let total = prefetcher.total_bytes().max(completed).max(1);
        let fraction = (completed as f32 / total as f32).clamp(0.0, 1.0);
        Some(Self {
            completed,
            total,
            fraction,
        })
    }

    fn label<'a>(&self, buf: &'a mut LabelBuf, compact: bool) -> &'a str {
        let percent = self.fraction * 100.0;
        if compact {
            stack_str(buf, format_args!("{percent:.0}%"))
        } else {
            let mut pct = [0; 8];
            let pct = stack_str(&mut pct, format_args!("{percent:.0}"));
            let (done, total) = (ByteSize(self.completed), ByteSize(self.total));
            fill(buf, tr(CommonKey::FetchingProgress), &[&done, &total, &pct])
        }
    }

    /// Width of the status item (abort button, spinner and label).
    pub fn width(&self, ui: &egui::Ui, compact: bool) -> f32 {
        let mut buf: LabelBuf = [0; 160];
        let galley = egui::WidgetText::from(rich_label(ui, self.label(&mut buf, compact)))
            .into_galley(
                ui,
                Some(egui::TextWrapMode::Extend),
                f32::INFINITY,
                egui::TextStyle::Small,
            );
        let spacing = ui.spacing().item_spacing.x;
        abort_button(compact).width(ui) + SPINNER_SIZE + galley.size().x + spacing * 2.0
    }
}

fn rich_label(ui: &egui::Ui, text: &str) -> egui::RichText {
    egui::RichText::new(text)
        .small()
        .strong()
        .color(IconTone::Warning.color(ui.visuals()))
}

fn abort_button(compact: bool) -> ToolbarButton<'static> {
    ToolbarButton::new(Icon::Stop, tr(CommonKey::Abort))
        .compact(compact)
        .hover(tr(CommonKey::AbortHover))
}

pub fn show_status_bar(
    app: &mut OctantApp,
    ui: &mut egui::Ui,
    progress: FetchProgress,
    compact: bool,
) {
    ui.horizontal(|ui| {
        if ui.add(abort_button(compact)).clicked() {
            app.abort_current_fetch();
        }
        ui.add(egui::Spinner::new().size(SPINNER_SIZE));
        let mut buf: LabelBuf = [0; 160];
        ui.label(rich_label(ui, progress.label(&mut buf, compact)));
    });
}
