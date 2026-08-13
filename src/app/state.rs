use crate::data::DatasetMetadata;
use crate::data::matrix_data::MatrixData;
use crate::data::slice_request::SliceRequest;
use crate::plots::{
    LineRenderer, MatrixRenderer, PlotType, PointCloudRenderer, SphereRenderer, SurfaceRenderer,
    VolumeRenderer,
};
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum StoreKind {
    RemoteZarr,
    LocalZarr,
    RemoteIcechunk,
    LocalIcechunk,
    ProceduralRandom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpatialRole {
    None,
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationRole {
    None,
    Animated,
}

#[derive(Debug, Clone)]
pub struct DimConfig {
    pub spatial: SpatialRole,
    pub animation: AnimationRole,
    pub active: bool, // expanded (range) or collapsed (index)
}

/// Represents a single plotted variable layer for current and future multi-variable visual overlays
/// (e.g. vector fields (u, v), multi-channel RGB composite layers, dual-curve plots).
#[derive(Debug, Clone)]
pub struct PlottedVariableState {
    pub store_kind: StoreKind,
    pub store_target_input: String,
    pub dataset_metadata: Option<DatasetMetadata>,
    pub variable_idx: usize,
    pub dim_config: Vec<DimConfig>,
    pub selected_dim_indices: Vec<usize>,
    pub selected_dim_ranges: Vec<(usize, usize)>,
    pub spatial_dims: Vec<usize>,
    pub animated_dim: Option<usize>,
}

impl PlottedVariableState {
    pub fn from_app(app: &OctantApp) -> Self {
        Self {
            store_kind: app.plotted_store_kind,
            store_target_input: app.plotted_store_target_input.clone(),
            dataset_metadata: app.plotted_dataset_metadata.clone(),
            variable_idx: app.plotted_variable_idx,
            dim_config: app.plotted_dim_config.clone(),
            selected_dim_indices: app.plotted_selected_dim_indices.clone(),
            selected_dim_ranges: app.plotted_selected_dim_ranges.clone(),
            spatial_dims: app.plotted_spatial_dims.clone(),
            animated_dim: app.plotted_animated_dim,
        }
    }
}

pub struct OctantApp {
    pub selected_store_kind: StoreKind,
    pub store_target_input: String,
    pub active_dataset_metadata: Option<DatasetMetadata>,
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
    /// Placeholder list for future multi-variable layer overlays (e.g. vector fields, RGB composites)
    pub multi_plotted_layers: Vec<PlottedVariableState>,
    pub current_timestep: usize,
    pub active_plot_type: PlotType,
    pub active_colormap: u32,
    pub preview_colormap: Option<u32>,
    pub status_message: String,
    pub is_loading: bool,
    pub matrix_data: Option<MatrixData>,
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
    pub wgpu_render_state: Option<eframe::egui_wgpu::RenderState>,

    // Block-cache & Prefetcher State
    pub dataset_manager: crate::data::DatasetManager,
    pub block_cache: crate::data::BlockCache,
    pub block_prefetcher: crate::data::BlockPrefetcher,
    pub active_block_key: Option<crate::data::BlockCacheKey>,
    pub pending_target_step: Option<usize>,
    pub max_cache_mb: usize,
    pub block_window_size: usize,

    // Animation & Playback Controls
    pub metadata_rx: Option<std::sync::mpsc::Receiver<Result<DatasetMetadata, String>>>,
    pub is_playing: bool,
    pub playback_fps: f32,
    pub loop_playback: bool,
    pub last_step_time: std::time::Instant,

    // Catalog State
    pub show_catalog_window: bool,
    pub catalog_search_query: String,
    pub catalog_category_filter: crate::catalog::CatalogCategoryFilter,

    // Panel Visibility State
    pub show_left_panel: bool,
    pub show_variables_overlay: bool,
    pub show_settings_panel: bool,
    pub show_variable_controls: bool,
    pub variables_overlay_width: f32,
    pub variable_search: String,
    pub show_bottom_bar: bool,
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

    // 2D Flatmap Heatmap & 1D Line Plot Viewport Zoom / Pan State
    pub heatmap_zoom: f32,
    pub heatmap_pan: egui::Vec2,
    pub line_zoom: f32,
    pub line_pan: egui::Vec2,
}

impl Default for OctantApp {
    fn default() -> Self {
        let default_cache_mb = 1024; // Default 1GB cache size limit

        Self {
            selected_store_kind: StoreKind::RemoteZarr,
            store_target_input: "https://s3.bgc-jena.mpg.de:9000/esdl-esdc-v3.0.2/esdc-16d-2.5deg-46x72x1440-3.0.2.zarr".to_string(),
            active_dataset_metadata: None,
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
            multi_plotted_layers: Vec::new(),
            current_timestep: 0,
            active_plot_type: PlotType::Heatmap,
            active_colormap: 0,
            preview_colormap: None,
            status_message: "Ready. Select store and click Inspect Store Metadata.".to_string(),
            is_loading: false,
            matrix_data: None,
            volume_data: None,
            renderer: None,
            line_renderer: None,
            sphere_renderer: None,
            surface_renderer: None,
            volume_renderer: None,
            point_cloud_renderer: None,
            sphere_rotation_y: 0.0,
            sphere_rotation_x: 0.25,
            sphere_auto_rotate: true,
            sphere_zoom: 2.5,
            sphere_displacement_strength: 0.3,
            sphere_mode: 0,
            surface_displacement_strength: 0.3,
            surface_mode: 0,
            volume_opacity: 3.0,
            volume_step_count: 64,
            volume_algorithm: 1,
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
            wgpu_render_state: None,

            dataset_manager: crate::data::DatasetManager::new(),
            block_cache: crate::data::BlockCache::new(default_cache_mb * 1024 * 1024),
            block_prefetcher: crate::data::BlockPrefetcher::new(),
            active_block_key: None,
            pending_target_step: None,
            max_cache_mb: default_cache_mb,
            block_window_size: 32,

            metadata_rx: None,
            is_playing: false,
            playback_fps: 15.0,
            loop_playback: true,
            last_step_time: std::time::Instant::now(),

            show_catalog_window: false,
            catalog_search_query: String::new(),
            catalog_category_filter: crate::catalog::CatalogCategoryFilter::All,

            show_left_panel: true,
            show_variables_overlay: true,
            show_settings_panel: false,
            show_variable_controls: false,
            show_bottom_bar: true,
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
            heatmap_zoom: 1.0,
            heatmap_pan: egui::Vec2::ZERO,
            line_zoom: 1.0,
            line_pan: egui::Vec2::ZERO,
        }
    }
}

impl OctantApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            wgpu_render_state: cc.wgpu_render_state.clone(),
            ..Default::default()
        }
    }

    pub fn reset_heatmap_view(&mut self) {
        self.heatmap_zoom = 1.0;
        self.heatmap_pan = egui::Vec2::ZERO;
    }

    pub fn reset_line_view(&mut self) {
        self.line_zoom = 1.0;
        self.line_pan = egui::Vec2::ZERO;
    }

    /// Placeholder method to add a secondary dimensionally-compatible variable layer
    /// for multi-variable plotting (e.g., vector fields, RGB composites, dual-curves).
    pub fn add_plotted_layer(&mut self, layer: PlottedVariableState) -> Result<(), String> {
        if let (Some(existing_meta), Some(new_meta)) =
            (&self.plotted_dataset_metadata, &layer.dataset_metadata)
        {
            let existing_var = existing_meta.variables.get(self.plotted_variable_idx);
            let new_var = new_meta.variables.get(layer.variable_idx);

            if let (Some(v_a), Some(v_b)) = (existing_var, new_var) {
                check_dimensional_compatibility(v_a, v_b)?;
            }
        }
        self.multi_plotted_layers.push(layer);
        Ok(())
    }

    /// Clears secondary multi-variable layers.
    pub fn clear_plotted_layers(&mut self) {
        self.multi_plotted_layers.clear();
    }
}

/// Helper function to verify dimensional compatibility between two variables
/// (matching rank, shapes, or spatial extent) for multi-layer plotting.
pub fn check_dimensional_compatibility(
    var_a: &crate::data::VariableInfo,
    var_b: &crate::data::VariableInfo,
) -> Result<(), String> {
    if var_a.shape.len() != var_b.shape.len() {
        return Err(format!(
            "Rank mismatch: '{}' (rank {}) vs '{}' (rank {})",
            var_a.name,
            var_a.shape.len(),
            var_b.name,
            var_b.shape.len()
        ));
    }
    if var_a.shape != var_b.shape {
        return Err(format!(
            "Shape mismatch: '{}' ({:?}) vs '{}' ({:?})",
            var_a.name, var_a.shape, var_b.name, var_b.shape
        ));
    }
    Ok(())
}
