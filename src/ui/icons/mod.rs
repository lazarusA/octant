//! Native procedural vector icons for Octant.
//!
//! Provides resolution-independent, theme-adaptive vector icons drawn directly into
//! egui's GPU vertex stream with `egui::Painter`. Replaces font-dependent emojis with
//! crisp scientific icons.

pub mod nav;
pub mod playback;
pub mod plots;
pub mod status;
pub mod store;

use egui::{Color32, Painter, Rect, Response, Sense, Stroke, Ui, WidgetText, vec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Icon {
    // Nav & Panels
    Globe,
    Variables,
    Dimensions,
    Settings,
    Cache,
    Sun,
    Moon,
    Overflow,

    // Playback
    Play,
    Pause,
    Stop,
    StepBackward,
    StepForward,
    SeekStart,
    SeekEnd,
    Loop,
    Reset,

    // Plots
    PlotPlane,
    PlotLine,
    PlotSurface,
    PlotGlobe,
    PlotVolume,
    PlotPointCloud,
    Colormap,

    // Store & Files
    Folder,
    FolderOpen,
    VariableDoc,
    Icechunk,
    Catalog,
    Save,
    Snapshot,
    DropTray,
    Search,
    Trash,
    Clipboard,

    // Status & Badges
    Scissors,
    Check,
    Cross,
    Lock,
    Unlock,
    Bolt,
    Hourglass,
    Warning,
    Info,
    Bullet,
    ChevronRight,
}

impl Icon {
    /// Paint the vector icon into `rect` using `painter`.
    pub fn paint(&self, painter: &Painter, rect: Rect, color: Color32, is_dark: bool) {
        let stroke_w = if rect.width() > 20.0 { 1.3 } else { 1.1 };
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

            // Plots
            Icon::PlotPlane => plots::draw_plot_plane(painter, rect, stroke, subtle_fill),
            Icon::PlotLine => plots::draw_plot_line(painter, rect, stroke),
            Icon::PlotSurface => plots::draw_plot_surface(painter, rect, stroke, subtle_fill),
            Icon::PlotGlobe => plots::draw_plot_globe(painter, rect, stroke, subtle_fill),
            Icon::PlotVolume => plots::draw_plot_volume(painter, rect, stroke, subtle_fill),
            Icon::PlotPointCloud => plots::draw_plot_point_cloud(painter, rect, color),
            Icon::Colormap => plots::draw_colormap(painter, rect, stroke),

            // Store & Files
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

/// Helper extension trait for easy rendering in egui UIs.
pub trait UiIconExt {
    /// Render an icon with exact `size` (width & height).
    fn icon(&mut self, icon: Icon, size: f32) -> Response;

    /// Render an icon with custom color and exact `size`.
    fn icon_colored(&mut self, icon: Icon, size: f32, color: Color32) -> Response;

    /// Render an interactive button containing a vector icon and label text side-by-side.
    fn icon_button(&mut self, icon: Icon, text: impl Into<WidgetText>) -> Response;

    /// Render a non-interactive label containing a vector icon and text side-by-side.
    fn icon_label(&mut self, icon: Icon, text: impl Into<WidgetText>) -> Response;
}

impl UiIconExt for Ui {
    fn icon(&mut self, icon: Icon, size: f32) -> Response {
        let (rect, response) = self.allocate_exact_size(vec2(size, size), Sense::hover());
        if self.is_rect_visible(rect) {
            let color = self.visuals().text_color();
            let is_dark = self.visuals().dark_mode;
            icon.paint(self.painter(), rect, color, is_dark);
        }
        response
    }

    fn icon_colored(&mut self, icon: Icon, size: f32, color: Color32) -> Response {
        let (rect, response) = self.allocate_exact_size(vec2(size, size), Sense::hover());
        if self.is_rect_visible(rect) {
            let is_dark = self.visuals().dark_mode;
            icon.paint(self.painter(), rect, color, is_dark);
        }
        response
    }

    fn icon_button(&mut self, icon: Icon, text: impl Into<WidgetText>) -> Response {
        let widget_text = text.into();
        let font_id = egui::TextStyle::Button.resolve(self.style());
        let galley = widget_text.into_galley(self, None, self.available_width(), font_id);

        let icon_size = 14.0;
        let gap = 6.0;
        let padding = self.spacing().button_padding;

        let content_size = vec2(
            icon_size + gap + galley.size().x,
            icon_size.max(galley.size().y),
        );
        let desired_size = content_size + padding * 2.0;

        let (rect, response) = self.allocate_exact_size(desired_size, Sense::click());

        if self.is_rect_visible(rect) {
            let is_dark = self.visuals().dark_mode;
            let visuals = self.style().interact(&response);

            // Draw button frame background
            self.painter().rect(
                rect,
                visuals.corner_radius,
                visuals.bg_fill,
                visuals.bg_stroke,
                egui::StrokeKind::Inside,
            );

            let content_origin = rect.min + padding;
            let icon_rect = Rect::from_min_size(
                egui::pos2(
                    content_origin.x,
                    content_origin.y + (content_size.y - icon_size) * 0.5,
                ),
                vec2(icon_size, icon_size),
            );

            let icon_color = visuals.text_color();
            icon.paint(self.painter(), icon_rect, icon_color, is_dark);

            let text_pos = egui::pos2(
                content_origin.x + icon_size + gap,
                content_origin.y + (content_size.y - galley.size().y) * 0.5,
            );
            self.painter().galley(text_pos, galley, icon_color);
        }

        response
    }

    fn icon_label(&mut self, icon: Icon, text: impl Into<WidgetText>) -> Response {
        let widget_text = text.into();
        let font_id = egui::TextStyle::Body.resolve(self.style());
        let galley = widget_text.into_galley(self, None, self.available_width(), font_id);

        let icon_size = 13.0;
        let gap = 5.0;

        let desired_size = vec2(
            icon_size + gap + galley.size().x,
            icon_size.max(galley.size().y),
        );
        let (rect, response) = self.allocate_exact_size(desired_size, Sense::hover());

        if self.is_rect_visible(rect) {
            let is_dark = self.visuals().dark_mode;
            let color = self.visuals().text_color();

            let icon_rect = Rect::from_min_size(
                egui::pos2(rect.min.x, rect.min.y + (desired_size.y - icon_size) * 0.5),
                vec2(icon_size, icon_size),
            );
            icon.paint(self.painter(), icon_rect, color, is_dark);

            let text_pos = egui::pos2(
                rect.min.x + icon_size + gap,
                rect.min.y + (desired_size.y - galley.size().y) * 0.5,
            );
            self.painter().galley(text_pos, galley, color);
        }

        response
    }
}

impl Icon {
    pub const ALL: &'static [Icon] = &[
        // Nav
        Icon::Globe,
        Icon::Variables,
        Icon::Dimensions,
        Icon::Settings,
        Icon::Cache,
        Icon::Sun,
        Icon::Moon,
        Icon::Overflow,
        // Playback
        Icon::Play,
        Icon::Pause,
        Icon::Stop,
        Icon::StepBackward,
        Icon::StepForward,
        Icon::SeekStart,
        Icon::SeekEnd,
        Icon::Loop,
        Icon::Reset,
        // Plots
        Icon::PlotPlane,
        Icon::PlotLine,
        Icon::PlotSurface,
        Icon::PlotGlobe,
        Icon::PlotVolume,
        Icon::PlotPointCloud,
        Icon::Colormap,
        // Store
        Icon::Folder,
        Icon::FolderOpen,
        Icon::VariableDoc,
        Icon::Icechunk,
        Icon::Catalog,
        Icon::Save,
        Icon::Snapshot,
        Icon::DropTray,
        Icon::Search,
        Icon::Trash,
        Icon::Clipboard,
        // Status
        Icon::Scissors,
        Icon::Check,
        Icon::Cross,
        Icon::Lock,
        Icon::Unlock,
        Icon::Bolt,
        Icon::Hourglass,
        Icon::Warning,
        Icon::Info,
        Icon::Bullet,
        Icon::ChevronRight,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Icon::Globe => "Globe",
            Icon::Variables => "Variables",
            Icon::Dimensions => "Dimensions",
            Icon::Settings => "Settings",
            Icon::Cache => "Cache",
            Icon::Sun => "Sun",
            Icon::Moon => "Moon",
            Icon::Overflow => "Overflow",
            Icon::Play => "Play",
            Icon::Pause => "Pause",
            Icon::Stop => "Stop",
            Icon::StepBackward => "StepBackward",
            Icon::StepForward => "StepForward",
            Icon::SeekStart => "SeekStart",
            Icon::SeekEnd => "SeekEnd",
            Icon::Loop => "Loop",
            Icon::Reset => "Reset",
            Icon::PlotPlane => "PlotPlane",
            Icon::PlotLine => "PlotLine",
            Icon::PlotSurface => "PlotSurface",
            Icon::PlotGlobe => "PlotGlobe",
            Icon::PlotVolume => "PlotVolume",
            Icon::PlotPointCloud => "PlotPointCloud",
            Icon::Colormap => "Colormap",
            Icon::Folder => "Folder",
            Icon::FolderOpen => "FolderOpen",
            Icon::VariableDoc => "VariableDoc",
            Icon::Icechunk => "Icechunk",
            Icon::Catalog => "Catalog",
            Icon::Save => "Save",
            Icon::Snapshot => "Snapshot",
            Icon::DropTray => "DropTray",
            Icon::Search => "Search",
            Icon::Trash => "Trash",
            Icon::Clipboard => "Clipboard",
            Icon::Scissors => "Scissors",
            Icon::Check => "Check",
            Icon::Cross => "Cross",
            Icon::Lock => "Lock",
            Icon::Unlock => "Unlock",
            Icon::Bolt => "Bolt",
            Icon::Hourglass => "Hourglass",
            Icon::Warning => "Warning",
            Icon::Info => "Info",
            Icon::Bullet => "Bullet",
            Icon::ChevronRight => "ChevronRight",
        }
    }

    pub fn category(&self) -> &'static str {
        match self {
            Icon::Globe
            | Icon::Variables
            | Icon::Dimensions
            | Icon::Settings
            | Icon::Cache
            | Icon::Sun
            | Icon::Moon
            | Icon::Overflow => "Navigation & Menus",

            Icon::Play
            | Icon::Pause
            | Icon::Stop
            | Icon::StepBackward
            | Icon::StepForward
            | Icon::SeekStart
            | Icon::SeekEnd
            | Icon::Loop
            | Icon::Reset => "Playback & Timeline",

            Icon::PlotPlane
            | Icon::PlotLine
            | Icon::PlotSurface
            | Icon::PlotGlobe
            | Icon::PlotVolume
            | Icon::PlotPointCloud
            | Icon::Colormap => "Plot Types & Colormaps",

            Icon::Folder
            | Icon::FolderOpen
            | Icon::VariableDoc
            | Icon::Icechunk
            | Icon::Catalog
            | Icon::Save
            | Icon::Snapshot
            | Icon::DropTray
            | Icon::Search
            | Icon::Trash
            | Icon::Clipboard => "Data Store & Files",

            Icon::Scissors
            | Icon::Check
            | Icon::Cross
            | Icon::Lock
            | Icon::Unlock
            | Icon::Bolt
            | Icon::Hourglass
            | Icon::Warning
            | Icon::Info
            | Icon::Bullet
            | Icon::ChevronRight => "Tools & Status Badges",
        }
    }
}

/// Render a gallery showcase grid of all procedural vector icons.
pub fn show_icon_gallery(ui: &mut Ui) {
    ui.heading("Octant Native Vector Icons");
    ui.label(
        "Resolution-independent, GPU-rasterized vector shapes adapting dynamically to themes.",
    );
    ui.add_space(8.0);

    let categories = [
        "Navigation & Menus",
        "Playback & Timeline",
        "Plot Types & Colormaps",
        "Data Store & Files",
        "Tools & Status Badges",
    ];

    for cat in categories {
        ui.group(|ui| {
            ui.strong(cat);
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                for icon in Icon::ALL {
                    if icon.category() == cat {
                        ui.vertical_centered(|ui| {
                            let resp = ui.icon(*icon, 24.0);
                            resp.on_hover_text(icon.name());
                            ui.small(icon.name());
                        });
                        ui.add_space(12.0);
                    }
                }
            });
        });
        ui.add_space(6.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_icons_unique_and_non_empty() {
        let mut names = std::collections::HashSet::new();
        for &icon in Icon::ALL {
            let name = icon.name();
            assert!(!name.is_empty(), "Icon name cannot be empty");
            assert!(
                names.insert(name),
                "Duplicate icon name '{name}' found in Icon::ALL"
            );
            assert!(
                !icon.category().is_empty(),
                "Category for '{name}' cannot be empty"
            );
        }
        assert_eq!(Icon::ALL.len(), 46, "Expected 46 total procedural icons");
    }

    #[test]
    fn test_icon_painting_without_panics() {
        let rect = Rect::from_min_size(egui::pos2(0.0, 0.0), vec2(24.0, 24.0));
        for &icon in Icon::ALL {
            for is_dark in [true, false] {
                let painter =
                    Painter::new(egui::Context::default(), egui::LayerId::background(), rect);
                let color = if is_dark {
                    Color32::WHITE
                } else {
                    Color32::BLACK
                };
                // Paint icon into painter
                icon.paint(&painter, rect, color, is_dark);
            }
        }
    }
}
