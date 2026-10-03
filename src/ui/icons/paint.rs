//! Dispatch from [`Icon`] to its procedural drawing routine.

use super::{Icon, nav, playback, plots, status, store, style};
use egui::emath::GuiRounding;
use egui::{Color32, Painter, Rect, Stroke};

impl Icon {
    /// Paint the vector icon into `rect` using `painter`.
    pub fn paint(&self, painter: &Painter, rect: Rect, color: Color32, is_dark: bool) {
        // Snap to physical pixels so strokes stay crisp at any display scale.
        let rect = rect.round_to_pixels(painter.pixels_per_point());
        let dim = rect.width().min(rect.height());
        let stroke_w = style::stroke_width(dim);
        let stroke = Stroke::new(stroke_w, color);
        let subtle_fill = if is_dark {
            Color32::from_rgba_unmultiplied(255, 255, 255, 18)
        } else {
            Color32::from_rgba_unmultiplied(0, 0, 0, 14)
        };

        match self {
            // Nav
            Icon::Globe => nav::draw_globe(painter, rect, stroke),
            Icon::Variables => nav::draw_variables(painter, rect, stroke, subtle_fill),
            Icon::Dimensions => nav::draw_dimensions(painter, rect, stroke, subtle_fill),
            Icon::Settings => nav::draw_settings(painter, rect, stroke, subtle_fill),
            Icon::Cache => nav::draw_cache(painter, rect, stroke, subtle_fill),
            Icon::Sun => nav::draw_sun(painter, rect, stroke),
            Icon::Moon => nav::draw_moon(painter, rect, stroke, color.gamma_multiply(0.25)),
            Icon::Overflow => nav::draw_overflow(painter, rect, color),

            // Playback
            Icon::Play => playback::draw_play(painter, rect, color, Stroke::NONE),
            Icon::Pause => playback::draw_pause(painter, rect, color, Stroke::NONE),
            Icon::Stop => playback::draw_stop(painter, rect, color, Stroke::NONE),
            Icon::StepBackward => playback::draw_step_backward(painter, rect, color, Stroke::NONE),
            Icon::StepForward => playback::draw_step_forward(painter, rect, color, Stroke::NONE),
            Icon::SeekStart => playback::draw_seek_start(painter, rect, color, Stroke::NONE),
            Icon::SeekEnd => playback::draw_seek_end(painter, rect, color, Stroke::NONE),
            Icon::Loop => playback::draw_loop(painter, rect, stroke, color),
            Icon::Reset => playback::draw_reset(painter, rect, stroke, color),
            Icon::Gauge => playback::draw_gauge(painter, rect, stroke, color),

            // Plots
            Icon::PlotPlane => plots::draw_plot_plane(painter, rect, stroke, subtle_fill),
            Icon::PlotLine => plots::draw_plot_line(painter, rect, stroke),
            Icon::PlotSurface => plots::draw_plot_surface(painter, rect, stroke, subtle_fill),
            Icon::PlotGlobe => plots::draw_plot_globe(painter, rect, stroke, subtle_fill),
            Icon::PlotVolume => plots::draw_plot_volume(painter, rect, stroke, subtle_fill),
            Icon::PlotPointCloud => plots::draw_plot_point_cloud(painter, rect, color),
            Icon::Colormap => plots::draw_colormap(painter, rect, stroke),

            // Store & Files
            Icon::Dataset => store::draw_dataset(painter, rect, stroke, subtle_fill),
            Icon::Folder => store::draw_folder(painter, rect, stroke, subtle_fill),
            Icon::FolderOpen => store::draw_folder_open(painter, rect, stroke, subtle_fill),
            Icon::VariableDoc => store::draw_variable_doc(painter, rect, stroke, subtle_fill),
            Icon::Icechunk => store::draw_icechunk(painter, rect, stroke, subtle_fill),
            Icon::Catalog => store::draw_catalog(painter, rect, stroke, subtle_fill),
            Icon::Save => store::draw_save(painter, rect, stroke, subtle_fill),
            Icon::Snapshot => store::draw_snapshot(painter, rect, stroke, subtle_fill),
            Icon::DropTray => store::draw_drop_tray(painter, rect, stroke),
            Icon::Search => store::draw_search(painter, rect, stroke),
            Icon::Trash => store::draw_trash(painter, rect, stroke, subtle_fill),
            Icon::Clipboard => store::draw_clipboard(painter, rect, stroke, subtle_fill),

            // Status & Badges
            Icon::Scissors => status::draw_scissors(painter, rect, stroke),
            Icon::Check => status::draw_check(painter, rect, stroke),
            Icon::Cross => status::draw_cross(painter, rect, stroke),
            Icon::Lock => status::draw_lock(painter, rect, stroke, subtle_fill),
            Icon::Unlock => status::draw_unlock(painter, rect, stroke, subtle_fill),
            Icon::Bolt => status::draw_bolt(painter, rect, color, Stroke::NONE),
            Icon::Hourglass => status::draw_hourglass(painter, rect, stroke, subtle_fill),
            Icon::Warning => status::draw_warning(painter, rect, stroke, subtle_fill),
            Icon::Info => status::draw_info(painter, rect, stroke),
            Icon::Bullet => status::draw_bullet(painter, rect, color),
            Icon::ChevronRight => status::draw_chevron_right(painter, rect, stroke),
        }
    }
}
