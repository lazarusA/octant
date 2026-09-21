//! Top-level application state struct definition and default constructor.

use std::sync::Arc;

use super::dimension_state::DimConfig;
use super::layer_state::PlottedVariableState;
use super::store_kind::StoreKind;
use crate::data::DatasetMetadata;
use crate::data::matrix_data::MatrixData;
use crate::data::slice_request::SliceRequest;
use crate::plots::{
    Coastline3DRenderer, CoastlineRenderer, LineRenderer, MatrixRenderer, PlotType,
    PointCloudRenderer, SphereRenderer, SurfaceRenderer, VolumeRenderer,
};

pub struct OctantApp {
    pub selected_store_kind: StoreKind,
    pub store_target_input: String,
    pub active_dataset_metadata: Option<DatasetMetadata>,
    pub cached_variable_tree: Option<crate::data::VariableTreeGroup>,
    pub selected_variable_idx: usize,
    pub plotted_store_kind: StoreKind,
    pub plotted_store_target_input: String,
    pub plotted_dataset_metadata: Option<DatasetMetadata>,
    pub plotted_variable_idx: usize,
    pub plotted_dim_config: Vec<DimConfig>,
    pub plotted_selected_dim_indices: Vec<usize>,
    pub plotted_selected_dim_ranges: Vec<(usize, usize)>,
    pub plotted_spatial_dims: Vec<usize>,
    pub plotted_animated_dim: Option<usize>,
    pub current_plotted_var_key: Option<String>,
    /// Placeholder list for future multi-variable layer overlays (e.g. vector fields, RGB composites)
    pub multi_plotted_layers: Vec<PlottedVariableState>,
    pub current_timestep: usize,
    pub active_plot_type: PlotType,
    pub active_colormap: u32,
    pub preview_colormap: Option<u32>,
    pub status_message: String,
    pub is_loading: bool,
    pub matrix_data: Option<MatrixData>,
    pub active_pyramid: Option<Arc<crate::data::MatrixPyramid>>,
    pub resampler: crate::data::ViewportResampler,
    pub volume_data: Option<crate::data::VolumeData>,
    pub renderer: Option<Arc<MatrixRenderer>>,
    pub line_renderer: Option<Arc<LineRenderer>>,
    pub sphere_renderer: Option<Arc<SphereRenderer>>,
    pub surface_renderer: Option<Arc<SurfaceRenderer>>,
    pub volume_renderer: Option<Arc<VolumeRenderer>>,
    pub point_cloud_renderer: Option<Arc<PointCloudRenderer>>,
    pub sphere_rotation_y: f32,
    pub sphere_rotation_x: f32,
    pub sphere_auto_rotate: bool,
    pub sphere_zoom: f32,
    pub sphere_displacement_strength: f32,
    pub sphere_mode: u32,
    pub surface_displacement_strength: f32,
    pub surface_mode: u32,
    pub volume_opacity: f32,
    pub volume_step_count: u32,
    pub volume_transparency: bool,
    pub volume_attenuation: f32,
    pub volume_algorithm: u32,
    pub volume_isovalue: f32,
    pub volume_isorange: f32,
    pub volume_cmin: f32,
    pub volume_cmax: f32,
    pub point_cloud_size: f32,
    pub line_profile_dim_idx: usize,
    pub line_profile_slice_idx: usize,
    pub line_plot_all_series: bool,
    pub show_colorbar: bool,
    pub is_categorical: bool,
    pub rgb_composite_mode: bool,
    pub rgb_composite_channels: [usize; 3],
    pub composite_channel_configs: Vec<crate::data::slicing::ChannelColorConfig>,
    pub wgpu_render_state: Option<eframe::egui_wgpu::RenderState>,

    // Block-cache & Prefetcher State
    pub dataset_manager: crate::data::DatasetManager,
    pub block_cache: crate::data::BlockCache,
    pub block_prefetcher: crate::data::BlockPrefetcher,
    pub active_block_key: Option<crate::data::BlockCacheKey>,
    pub pending_target_step: Option<usize>,
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
    pub variables_overlay_width: f32,
    pub variable_search: String,
    pub show_bottom_bar: bool,
    pub show_hover_card: bool,
    pub settings_overlay_width: f32, // tracks prev-frame width to position Variable Controls to the right
    pub theme_preference: egui::ThemePreference,
    pub enforce_data_aspect_ratio: bool,

    // DimConfig
    pub dim_config: Vec<DimConfig>,               // one per dimension
    pub selected_dim_indices: Vec<usize>,         // collapsed index per dimension
    pub selected_dim_ranges: Vec<(usize, usize)>, // range per dimension
    pub spatial_dims: Vec<usize>,                 // dims assigned X,Y,Z
    pub animated_dim: Option<usize>,              // dim assigned Animated
    pub active_slice_request: Option<SliceRequest>,

    // Clipping & Color Range State
    pub nan_color: [f32; 4],
    pub use_nan_color: bool,
    pub lowclip_color: [f32; 4],
    pub use_lowclip: bool,
    pub highclip_color: [f32; 4],
    pub use_highclip: bool,
    pub lock_color_bounds: bool,
    pub color_range_min: f32,
    pub color_range_max: f32,
    pub global_data_min: f32,
    pub global_data_max: f32,
    pub active_scale_type: u32,
    pub scale_param: f32,
    pub custom_colorbar_label: Option<String>,

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
    pub export_toast: Option<crate::export::ExportToastNotification>,

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

impl Default for OctantApp {
    fn default() -> Self {
        let default_cache_mb = 1024; // Default 1GB cache size limit

        Self {
            selected_store_kind: StoreKind::RemoteZarr,
            store_target_input: "https://s3.bgc-jena.mpg.de:9000/esdl-esdc-v3.0.2/esdc-16d-2.5deg-46x72x1440-3.0.2.zarr".to_string(),
            active_dataset_metadata: None,
            cached_variable_tree: None,
            selected_variable_idx: 0,
            plotted_store_kind: StoreKind::RemoteZarr,
            plotted_store_target_input: "https://s3.bgc-jena.mpg.de:9000/esdl-esdc-v3.0.2/esdc-16d-2.5deg-46x72x1440-3.0.2.zarr".to_string(),
            plotted_dataset_metadata: None,
            plotted_variable_idx: 0,
            plotted_dim_config: Vec::new(),
            plotted_selected_dim_indices: Vec::new(),
            plotted_selected_dim_ranges: Vec::new(),
            plotted_spatial_dims: Vec::new(),
            plotted_animated_dim: None,
            current_plotted_var_key: None,
            multi_plotted_layers: Vec::new(),
            current_timestep: 0,
            active_plot_type: PlotType::Heatmap,
            active_colormap: 0,
            preview_colormap: None,
            status_message: "Ready. Select store and click Inspect Store Metadata.".to_string(),
            is_loading: false,
            matrix_data: None,
            active_pyramid: None,
            resampler: crate::data::ViewportResampler::default(),
            volume_data: None,
            renderer: None,
            line_renderer: None,
            sphere_renderer: None,
            surface_renderer: None,
            volume_renderer: None,
            point_cloud_renderer: None,
            sphere_rotation_y: 0.0,
            sphere_rotation_x: 0.25,
            sphere_auto_rotate: false,
            sphere_zoom: 2.5,
            sphere_displacement_strength: 0.3,
            sphere_mode: 0,
            surface_displacement_strength: 0.3,
            surface_mode: 0,
            volume_opacity: 3.0,
            volume_step_count: 64,
            volume_transparency: true,
            volume_attenuation: 0.0,
            volume_algorithm: 0,
            volume_isovalue: 50.0,
            volume_isorange: 5.0,
            volume_cmin: 5.0,
            volume_cmax: 100.0,
            point_cloud_size: 0.02,
            line_profile_dim_idx: 0,
            line_profile_slice_idx: 0,
            line_plot_all_series: false,
            show_colorbar: true,
            is_categorical: false,
            rgb_composite_mode: false,
            rgb_composite_channels: [0, 1, 2],
            composite_channel_configs: Vec::new(),
            wgpu_render_state: None,

            show_hero: true,
            hero_state: crate::ui::hero::HeroState::default(),

            dataset_manager: crate::data::DatasetManager::new(),
            block_cache: crate::data::BlockCache::new(default_cache_mb * 1024 * 1024),
            block_prefetcher: crate::data::BlockPrefetcher::new(),
            active_block_key: None,
            pending_target_step: None,
            max_cache_mb: default_cache_mb,
            block_window_size: 32,
            prefetch_threads: 16,

            metadata_rx: None,
            is_playing: false,
            playback_fps: 15.0,
            loop_playback: true,
            enable_prefetch: true,
            last_step_time: web_time::Instant::now(),

            show_catalog_window: false,
            show_about_window: false,
            show_icon_gallery_window: false,
            catalog_search_query: String::new(),
            catalog_category_filter: crate::catalog::CatalogCategoryFilter::All,

            show_left_panel: false,
            show_variables_overlay: false,
            show_settings_panel: false,
            show_variable_controls: false,
            show_bottom_bar: true,
            show_hover_card: true,
            settings_overlay_width: 0.0,
            variables_overlay_width: 340.0,
            variable_search: String::new(),

            theme_preference: egui::ThemePreference::System,
            enforce_data_aspect_ratio: true,
            dim_config: Vec::new(),
            selected_dim_indices: Vec::new(),
            selected_dim_ranges: Vec::new(),
            spatial_dims: Vec::new(),
            animated_dim: None,
            active_slice_request: None,

            nan_color: [0.0, 0.0, 0.0, 0.0],
            use_nan_color: false,
            lowclip_color: [0.0, 0.0, 1.0, 1.0],
            use_lowclip: false,
            highclip_color: [1.0, 0.0, 0.0, 1.0],
            use_highclip: false,
            lock_color_bounds: false,
            color_range_min: 0.0,
            color_range_max: 100.0,
            global_data_min: f32::INFINITY,
            global_data_max: f32::NEG_INFINITY,
            active_scale_type: 0,
            scale_param: 1.0,
            custom_colorbar_label: None,
            heatmap_zoom: 1.0,
            heatmap_pan: egui::Vec2::ZERO,
            line_zoom: 1.0,
            line_pan: egui::Vec2::ZERO,
            enable_pyramid_resampling: false,
            pyramid_aggregation_op: crate::data::AggregationOp::default(),
            export_settings: crate::export::ExportSettings::default(),
            show_export_modal: false,
            show_crop_overlay: false,
            roi_crop_box: crate::export::RoiCropBox::default(),
            pending_export: None,
            export_flash_timer: None,
            export_toast: None,

            show_coastlines: false,
            coastline_color: None,
            coastline_renderer: None,
            coastline_3d_renderer: None,
            coastline_current_lod: crate::plots::CoastlineLod::Lod110m,
            coastline_rx: None,
            coastline_is_loading: false,
            coastline_crop_to_data_domain: true,
            coastline_line_width: 1.0,
        }
    }
}
