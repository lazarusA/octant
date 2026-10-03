//! Dispatch from [`Icon`] to its procedural drawing routine.

use super::canvas::IconCanvas;
use super::{Icon, files, marks, nav, palette, playback, plots, status, store};
use egui::emath::GuiRounding;
use egui::{Color32, Painter, Rect};

impl Icon {
    /// Paint the vector icon into `rect` using `painter`.
    pub fn paint(&self, painter: &Painter, rect: Rect, color: Color32, is_dark: bool) {
        // Snap to physical pixels so strokes stay crisp at any display scale.
        let rect = rect.round_to_pixels(painter.pixels_per_point());
        let c = &IconCanvas::new(painter, rect, color, is_dark);

        match self {
            // Nav
            Icon::Globe => nav::draw_globe(c),
            Icon::Variables => nav::draw_variables(c),
            Icon::Dimensions => nav::draw_dimensions(c),
            Icon::Settings => nav::draw_settings(c),
            Icon::Cache => nav::draw_cache(c),
            Icon::Sun => nav::draw_sun(c),
            Icon::Moon => nav::draw_moon(c),

            // Playback
            Icon::Play => playback::draw_play(c),
            Icon::Pause => playback::draw_pause(c),
            Icon::Stop => playback::draw_stop(c),
            Icon::StepBackward => playback::draw_step_backward(c),
            Icon::StepForward => playback::draw_step_forward(c),
            Icon::SeekStart => playback::draw_seek_start(c),
            Icon::SeekEnd => playback::draw_seek_end(c),
            Icon::Loop => playback::draw_loop(c),
            Icon::Reset => playback::draw_reset(c),
            Icon::Gauge => playback::draw_gauge(c),

            // Plots
            Icon::PlotPlane => palette::draw_plot_plane(c),
            Icon::PlotLine => plots::draw_plot_line(c),
            Icon::PlotSurface => plots::draw_plot_surface(c),
            Icon::PlotGlobe => plots::draw_plot_globe(c),
            Icon::PlotVolume => plots::draw_plot_volume(c),
            Icon::PlotPointCloud => plots::draw_plot_point_cloud(c),
            Icon::Colormap => palette::draw_colormap(c),

            // Store & Files
            Icon::Dataset => files::draw_dataset(c),
            Icon::Folder => files::draw_folder(c),
            Icon::FolderOpen => files::draw_folder_open(c),
            Icon::VariableDoc => store::draw_variable_doc(c),
            Icon::Icechunk => store::draw_icechunk(c),
            Icon::Catalog => store::draw_catalog(c),
            Icon::Save => store::draw_save(c),
            Icon::Snapshot => store::draw_snapshot(c),
            Icon::DropTray => store::draw_drop_tray(c),
            Icon::Search => store::draw_search(c),
            Icon::Trash => store::draw_trash(c),
            Icon::Clipboard => store::draw_clipboard(c),

            // Status & Badges
            Icon::Scissors => status::draw_scissors(c),
            Icon::Check => marks::draw_check(c),
            Icon::Cross => marks::draw_cross(c),
            Icon::Lock => status::draw_lock(c),
            Icon::Unlock => status::draw_unlock(c),
            Icon::Bolt => status::draw_bolt(c),
            Icon::Hourglass => status::draw_hourglass(c),
            Icon::Warning => marks::draw_warning(c),
            Icon::Info => marks::draw_info(c),
            Icon::Bullet => marks::draw_bullet(c),
            Icon::ChevronRight => marks::draw_chevron_right(c),
            Icon::ChevronDown => marks::draw_chevron_down(c),
        }
    }
}
