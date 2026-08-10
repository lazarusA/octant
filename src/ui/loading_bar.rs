//! Progress component and top-center canvas floating loading bar overlay.

use crate::app::OctantApp;
use egui::{Color32, Ui, Vec2};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

static RAW_TOTAL_REQUESTS: AtomicUsize = AtomicUsize::new(0);

// =============================================================================
// ProgressSize / ProgressVariant
// =============================================================================

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ProgressSize {
    Size1,
    #[default]
    Size2,
    Size3,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ProgressVariant {
    Classic,
    #[default]
    Surface,
    Soft,
}

// =============================================================================
// ProgressProps
// =============================================================================

#[derive(Clone, Debug)]
pub struct ProgressProps {
    pub value: Option<f32>, // None = indeterminate
    pub max: f32,
    pub size: ProgressSize,
    pub variant: ProgressVariant,
    pub color: Option<Color32>,
    pub radius: Option<f32>,
    pub duration_ms: u32,
}

impl ProgressProps {
    pub fn new(value: Option<f32>) -> Self {
        Self {
            value,
            max: 100.0,
            size: ProgressSize::Size2,
            variant: ProgressVariant::Surface,
            color: None,
            radius: None,
            duration_ms: 1500,
        }
    }

    pub fn max(mut self, max: f32) -> Self {
        self.max = max.max(0.01);
        self
    }

    pub fn size(mut self, size: ProgressSize) -> Self {
        self.size = size;
        self
    }

    pub fn variant(mut self, variant: ProgressVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = Some(radius);
        self
    }

    pub fn duration_ms(mut self, duration_ms: u32) -> Self {
        self.duration_ms = duration_ms;
        self
    }
}

// =============================================================================
// Main Progress Function
// =============================================================================

/// Render a progress bar.
pub fn progress(ui: &mut Ui, props: ProgressProps) {
    let primary = ui.visuals().selection.bg_fill;
    let muted = ui.visuals().widgets.noninteractive.bg_fill;

    let accent = props.color.unwrap_or(primary);

    let (bg_color, fg_color) = match props.variant {
        ProgressVariant::Classic => (muted, accent),
        ProgressVariant::Surface => (muted.gamma_multiply(0.4), accent),
        ProgressVariant::Soft => (accent.gamma_multiply(0.2), accent),
    };

    let height = match props.size {
        ProgressSize::Size1 => 4.0,
        ProgressSize::Size2 => 6.0,
        ProgressSize::Size3 => 10.0,
    };

    let available_width = ui.available_width();
    let rounding = props.radius.unwrap_or(height / 2.0);

    let (rect, _response) =
        ui.allocate_exact_size(Vec2::new(available_width, height), egui::Sense::hover());

    // Background track
    ui.painter().rect_filled(rect, rounding, bg_color);

    // Foreground progress
    if let Some(value) = props.value {
        let progress_ratio = (value / props.max).clamp(0.0, 1.0);
        let progress_width = rect.width() * progress_ratio;

        if progress_width > 0.0 {
            let progress_rect =
                egui::Rect::from_min_size(rect.min, Vec2::new(progress_width, height));
            ui.painter().rect_filled(progress_rect, rounding, fg_color);
        }
    } else {
        // Smooth indeterminate animation
        let time = ui.ctx().input(|i| i.time) as f32;
        let speed = 1000.0 / props.duration_ms.max(1) as f32 * 2.25;
        let anim_progress = ((time * speed).sin() + 1.0) / 2.0;
        let bar_width = rect.width() * 0.3;
        let offset = (rect.width() - bar_width) * anim_progress;

        let anim_rect = egui::Rect::from_min_size(
            rect.min + Vec2::new(offset, 0.0),
            Vec2::new(bar_width, height),
        );
        ui.painter().rect_filled(anim_rect, rounding, fg_color);
        ui.ctx().request_repaint_after(Duration::from_millis(33));
    }
}

// =============================================================================
// Top-Center Canvas Floating Loading Overlay Component
// =============================================================================

pub fn show_canvas_loading_bar(app: &OctantApp, ctx: &egui::Context, canvas_rect: egui::Rect) {
    let is_metadata_fetching = app.metadata_rx.is_some() || app.is_loading;
    let pending_raw_count = app.block_prefetcher.pending_count();
    let is_raw_data_fetching = pending_raw_count > 0;

    if !is_metadata_fetching && !is_raw_data_fetching {
        RAW_TOTAL_REQUESTS.store(0, Ordering::Relaxed);
        return;
    }

    // Track total requested raw data blocks for progress
    if pending_raw_count > 0 {
        let current_total = RAW_TOTAL_REQUESTS.load(Ordering::Relaxed);
        if pending_raw_count > current_total {
            RAW_TOTAL_REQUESTS.store(pending_raw_count, Ordering::Relaxed);
        }
    }

    let (label_text, progress_props) = if is_metadata_fetching {
        (
            "🔍 Inspecting store metadata...".to_string(),
            ProgressProps::new(None)
                .size(ProgressSize::Size2)
                .variant(ProgressVariant::Surface),
        )
    } else {
        let total = RAW_TOTAL_REQUESTS.load(Ordering::Relaxed).max(pending_raw_count);
        let fetched = total.saturating_sub(pending_raw_count);

        let label = if total > 1 {
            format!("⏳ Fetching data ({}/{})", fetched, total)
        } else {
            "⏳ Fetching data...".to_string()
        };

        (
            label,
            ProgressProps::new(Some(fetched as f32))
                .max(total.max(1) as f32)
                .size(ProgressSize::Size2)
                .variant(ProgressVariant::Surface),
        )
    };

    let card_width = 260.0;
    let pos = egui::pos2(
        canvas_rect.center().x - (card_width / 2.0),
        canvas_rect.top() + 14.0,
    );

    egui::Area::new(egui::Id::new("octant_canvas_loading_bar"))
        .order(egui::Order::Foreground)
        .fixed_pos(pos)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style())
                .inner_margin(egui::Margin::symmetric(12, 8))
                .show(ui, |ui| {
                    ui.set_width(card_width - 24.0);
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(egui::RichText::new(&label_text).strong().small());
                    });

                    ui.add_space(4.0);
                    progress(ui, progress_props);
                });
        });
}
