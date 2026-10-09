//! Top-level application state struct definition (defaults in `app_defaults.rs`).

use std::sync::Arc;

use crate::app::layers::{LayerStack, VariableSelection};
use crate::data::DatasetMetadata;
use crate::plots::{Coastline3DRenderer, CoastlineRenderer};

pub struct OctantApp {
    /// The variable and dimensions staged in the UI (the Variables overlay
    /// and sliders); Plot copies it to the base layer.
    pub selected: VariableSelection,
    pub cached_variable_tree: Option<crate::data::VariableTreeGroup>,
    /// Filtered variable tree for `(selected.metadata_generation, query)`.
    pub cached_search: Option<crate::ui::variables_overlay::SearchCache>,
    /// The plotted layers; the base layer is the plot on the canvas.
    pub layers: LayerStack,
    pub current_timestep: usize,
    pub preview_colormap: Option<u32>,
    pub colormaps: super::ColormapState,
    pub status_message: String,
    pub is_loading: bool,
    pub sphere_rotation_y: f32,
    pub sphere_rotation_x: f32,
    pub sphere_auto_rotate: bool,
    pub sphere_zoom: f32,
    pub sphere_displacement_strength: f32,
    pub sphere_mode: u32,
    pub surface_displacement_strength: f32,
    pub surface_mode: u32,
    pub volume_opacity: f32,
    /// Volume raymarching samples per voxel crossed by each ray.
    pub volume_quality: f32,
    pub volume_transparency: bool,
    /// Surfaces, spheres and point clouds with translucent colors draw
    /// without depth writes, so no part hides the parts behind it.
    pub plot_transparency: bool,
    /// Lights transparent DVR samples by their gradient.
    pub volume_lighting: bool,
    /// The user is rotating or zooming the 3D view this frame: volumes render
    /// at reduced resolution until it settles.
    pub view_interacting: bool,
    pub volume_attenuation: f32,
    pub volume_algorithm: u32,
    pub volume_isovalue: f32,
    pub volume_isorange: f32,
    pub volume_z_scale: f32,
    pub point_cloud_size: f32,
    pub line_profile_dim_idx: usize,
    pub line_profile_slice_idx: usize,
    pub line_plot_all_series: bool,
    pub line_color: [f32; 4],
    pub line_use_custom_color: bool,
    pub line_show_lines: bool,
    pub line_show_points: bool,
    pub line_point_size: f32,
    pub show_colorbar: bool,
    pub colorbar_transparency: f32,
    pub wgpu_render_state: Option<eframe::egui_wgpu::RenderState>,

    // Block-cache & Prefetcher State
    pub dataset_manager: crate::data::DatasetManager,
    pub block_cache: crate::data::BlockCache,
    pub block_prefetcher: crate::data::BlockPrefetcher,
    /// Reads a variable's coordinates in the background when it is chosen.
    pub coordinate_loader: crate::data::blocks::CoordinateLoader,
    /// Incremented whenever coordinates arrive; keys caches built from them.
    pub coordinates_revision: u64,
    pub max_cache_mb: usize,
    pub block_window_size: usize,
    pub prefetch_threads: usize,

    // Animation & Playback Controls
    pub metadata_rx: Option<
        std::sync::mpsc::Receiver<Result<(DatasetMetadata, crate::data::StoreHandle), String>>,
    >,
    pub is_playing: bool,
    pub playback_fps: f32,
    pub loop_playback: bool,
    pub enable_prefetch: bool,
    pub last_step_time: web_time::Instant,

    // Catalog State
    pub show_catalog_window: bool,
    pub show_about_window: bool,
    pub show_icon_gallery_window: bool,
    pub catalog_search_query: String,
    pub catalog_category_filter: crate::catalog::CatalogCategoryFilter,

    // Panel Visibility State
    pub show_left_panel: bool,
    pub show_hero: bool,
    pub hero_state: crate::ui::hero::HeroState,
    pub show_variables_overlay: bool,
    pub show_settings_panel: bool,
    pub show_variable_controls: bool,
    /// The Dimensions panel's Plot button adds the staged variable as an
    /// overlay instead of replacing the plot (off again after each add).
    pub plot_as_overlay: bool,
    pub variables_overlay_width: f32,
    pub variable_search: String,
    pub show_bottom_bar: bool,
    pub show_hover_card: bool,
    pub settings_overlay_width: f32, // tracks prev-frame width to position Variable Controls to the right
    pub theme_preference: egui::ThemePreference,
    pub enforce_data_aspect_ratio: bool,

    // 2D Flatmap Heatmap & 1D Line Plot Viewport Zoom / Pan State
    pub heatmap_zoom: f32,
    pub heatmap_pan: egui::Vec2,
    pub line_zoom: f32,
    pub line_pan: egui::Vec2,
    pub enable_pyramid_resampling: bool,
    pub pyramid_aggregation_op: crate::data::AggregationOp,

    // Canvas Save & Export System
    pub export_settings: crate::export::ExportSettings,
    pub show_export_modal: bool,
    pub show_crop_overlay: bool,
    pub roi_crop_box: crate::export::RoiCropBox,
    pub pending_export: Option<crate::export::PendingExportRequest>,
    pub export_flash_timer: Option<web_time::Instant>,
    /// Warnings, errors and results shown over the canvas.
    pub toasts: crate::ui::toast::Toasts,

    // Coastline Overlay
    /// Whether to render the coastline overlay on geographic plot types.
    pub show_coastlines: bool,
    /// RGBA line color for coastlines [0..1]. Theme-aware default applied at
    /// render time if this is `None`; set to `Some` when the user picks a color.
    pub coastline_color: Option<[f32; 4]>,
    /// GPU renderer — initialised lazily on first pipeline build.
    pub coastline_renderer: Option<Arc<CoastlineRenderer>>,
    /// GPU renderer for coastlines projected onto 3D surfaces and spheres.
    pub coastline_3d_renderer: Option<Arc<Coastline3DRenderer>>,
    /// Current LOD loaded in the GPU buffer.
    pub coastline_current_lod: crate::plots::CoastlineLod,
    /// Receiver for background coastline LOD downloads.
    pub coastline_rx: Option<crate::plots::CoastlineReceiver>,
    /// Whether a higher LOD coastline is currently downloading.
    pub coastline_is_loading: bool,
    /// Whether to clip coastlines strictly to the spatial boundary of the active dataset.
    pub coastline_crop_to_data_domain: bool,
    /// Line width for coastline rendering (1.0 to 4.0).
    pub coastline_line_width: f32,
}
