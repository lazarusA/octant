//! Minimal floating top-center canvas bouncing loading bar.

use crate::app::OctantApp;
use egui::{Color32, Ui, Vec2};
use std::time::Duration;

// =============================================================================
// ProgressSize / ProgressVariant
// =============================================================================

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ProgressSize {
    #[default]
    Size1,
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
    pub value: Option<f32>, // None = indeterminate bouncing bar
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
            size: ProgressSize::Size1,
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
// Bouncing Progress Bar Render Function
// =============================================================================

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

    // Foreground / Indeterminate animation
    if let Some(value) = props.value {
        let progress_ratio = (value / props.max).clamp(0.0, 1.0);
        let progress_width = rect.width() * progress_ratio;

        if progress_width > 0.0 {
            let progress_rect =
                egui::Rect::from_min_size(rect.min, Vec2::new(progress_width, height));
            ui.painter().rect_filled(progress_rect, rounding, fg_color);
        }
    } else {
        // Smooth bouncing bar animation
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
// Top-Center Canvas Floating Bouncing Bar Overlay
// =============================================================================

pub fn show_canvas_loading_bar(app: &OctantApp, ctx: &egui::Context, canvas_rect: egui::Rect) {
    let is_active =
        app.metadata_rx.is_some() || app.is_loading || app.block_prefetcher.pending_count() > 0;

    if !is_active {
        return;
    }

    let width = 220.0;
    let pos = egui::pos2(
        canvas_rect.center().x - (width / 2.0),
        canvas_rect.top() + 6.0,
    );

    egui::Area::new(egui::Id::new("octant_canvas_loading_bar"))
        .order(egui::Order::Foreground)
        .fixed_pos(pos)
        .show(ctx, |ui| {
            ui.set_width(width);
            progress(
                ui,
                ProgressProps::new(None)
                    .size(ProgressSize::Size1)
                    .variant(ProgressVariant::Surface),
            );
        });
}
