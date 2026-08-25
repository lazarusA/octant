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
    ProceduralVolume4D,
    ProceduralRandom,
}

impl StoreKind {
    pub fn to_data_source_kind(self) -> crate::data::DataSourceKind {
        match self {
            StoreKind::RemoteZarr => crate::data::DataSourceKind::RemoteZarr,
            StoreKind::LocalZarr => crate::data::DataSourceKind::LocalZarr,
            StoreKind::RemoteIcechunk => crate::data::DataSourceKind::RemoteIcechunk,
            StoreKind::LocalIcechunk => crate::data::DataSourceKind::LocalIcechunk,
            StoreKind::ProceduralVolume4D | StoreKind::ProceduralRandom => {
                crate::data::DataSourceKind::Procedural
            }
        }
    }

    pub fn from_data_source_kind(kind: &crate::data::DataSourceKind) -> Self {
        match kind {
            crate::data::DataSourceKind::RemoteZarr => StoreKind::RemoteZarr,
            crate::data::DataSourceKind::LocalZarr => StoreKind::LocalZarr,
            crate::data::DataSourceKind::RemoteIcechunk => StoreKind::RemoteIcechunk,
            crate::data::DataSourceKind::LocalIcechunk => StoreKind::LocalIcechunk,
            crate::data::DataSourceKind::Procedural => StoreKind::ProceduralVolume4D,
            _ => StoreKind::ProceduralRandom,
        }
    }

    pub fn make_source_id(kind: StoreKind, target: &str) -> String {
        format!("{:?}:{}", kind, target)
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DimConfig {
    pub spatial: SpatialRole,
    pub animation: AnimationRole,
    pub active: bool, // true = range (expanded), false = single index (collapsed)
    pub index: usize, // selected single step / index
    pub range: (usize, usize), // selected (start, end) range
}

impl Default for DimConfig {
    fn default() -> Self {
        Self {
            spatial: SpatialRole::None,
            animation: AnimationRole::None,
            active: false,
            index: 0,
            range: (0, 0),
        }
    }
}

impl DimConfig {
    pub fn new(
        spatial: SpatialRole,
        animation: AnimationRole,
        active: bool,
        index: usize,
        range: (usize, usize),
    ) -> Self {
        Self {
            spatial,
            animation,
            active,
            index,
            range,
        }
    }

    pub fn x_dim(configs: &[DimConfig]) -> Option<usize> {
        configs.iter().position(|c| c.spatial == SpatialRole::X)
    }

    pub fn y_dim(configs: &[DimConfig]) -> Option<usize> {
        configs.iter().position(|c| c.spatial == SpatialRole::Y)
    }

    pub fn z_dim(configs: &[DimConfig]) -> Option<usize> {
        configs.iter().position(|c| c.spatial == SpatialRole::Z)
    }

    pub fn animated_dim(configs: &[DimConfig]) -> Option<usize> {
        configs
            .iter()
            .position(|c| c.animation == AnimationRole::Animated)
    }

    pub fn spatial_dims(configs: &[DimConfig]) -> Vec<usize> {
        let mut list = Vec::new();
        if let Some(i) = Self::x_dim(configs) {
            list.push(i);
        }
        if let Some(i) = Self::y_dim(configs) {
            list.push(i);
        }
        if let Some(i) = Self::z_dim(configs) {
            list.push(i);
        }
        list
    }
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
    pub enable_prefetch: bool,
    pub last_step_time: std::time::Instant,

    // Catalog State
    pub show_catalog_window: bool,
    pub show_about_window: bool,
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

    // Canvas Save & Video Recording State
    pub capture_config: crate::app::capture::CaptureConfig,
    pub capture_ring: Option<crate::app::capture_ring::CaptureRing>,
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
            sphere_auto_rotate: true,
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

            metadata_rx: None,
            is_playing: false,
            playback_fps: 15.0,
            loop_playback: true,
            enable_prefetch: true,
            last_step_time: std::time::Instant::now(),

            show_catalog_window: false,
            show_about_window: false,
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
            capture_config: crate::app::capture::CaptureConfig::default(),
            capture_ring: None,
        }
    }
}

impl OctantApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // if let Some(wgpu_state) = &cc.wgpu_render_state {
        //     crate::utils::diagnostics::log_gpu_diagnostics(wgpu_state);
        // }

        let mut app = Self {
            wgpu_render_state: cc.wgpu_render_state.clone(),
            ..Default::default()
        };

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(target) = std::env::args().nth(1) {
            app.submit_or_activate_source(&target, None);
        }

        app
    }

    pub fn reset_heatmap_view(&mut self) {
        self.heatmap_zoom = 1.0;
        self.heatmap_pan = egui::Vec2::ZERO;
    }

    pub fn reset_line_view(&mut self) {
        self.line_zoom = 1.0;
        self.line_pan = egui::Vec2::ZERO;
    }

    /// Returns the default automatic label for the active plotted variable (including unit if available).
    pub fn default_colorbar_label(&self) -> String {
        if let Some(meta) = &self.plotted_dataset_metadata {
            meta.variables
                .get(self.plotted_variable_idx)
                .map(|v| {
                    if let Some(unit) = v.attributes.get("units").or(v.units.as_ref()) {
                        format!("{} ({})", v.name, unit)
                    } else {
                        v.name.clone()
                    }
                })
                .unwrap_or_else(|| "Scalar Field".to_string())
        } else {
            "Scalar Field".to_string()
        }
    }

    /// Returns the effective colorbar label (custom overridden label if set, otherwise default).
    pub fn colorbar_label(&self) -> String {
        if let Some(ref custom) = self.custom_colorbar_label {
            custom.clone()
        } else {
            self.default_colorbar_label()
        }
    }

    /// Resets custom colorbar label back to default.
    pub fn reset_colorbar_label(&mut self) {
        self.custom_colorbar_label = None;
    }

    /// Resets color range min and max to the current dataset/matrix slice bounds and unlocks bounds.
    pub fn reset_color_range(&mut self) {
        let is_3d = self.active_plot_type == crate::plots::PlotType::Volume
            || self.active_plot_type == crate::plots::PlotType::PointCloud;

        if is_3d && let Some(vdata) = &self.volume_data {
            self.color_range_min = vdata.min_val;
            self.color_range_max = vdata.max_val;
            self.volume_cmin = vdata.min_val;
            self.volume_cmax = vdata.max_val;
        } else if let Some(mdata) = &self.matrix_data {
            self.color_range_min = mdata.min_val;
            self.color_range_max = mdata.max_val;
            self.volume_cmin = mdata.min_val;
            self.volume_cmax = mdata.max_val;
        } else if let Some(vdata) = &self.volume_data {
            self.color_range_min = vdata.min_val;
            self.color_range_max = vdata.max_val;
            self.volume_cmin = vdata.min_val;
            self.volume_cmax = vdata.max_val;
        } else {
            self.color_range_min = 0.0;
            self.color_range_max = 100.0;
            self.volume_cmin = 0.0;
            self.volume_cmax = 100.0;
        }
        self.lock_color_bounds = false;
    }

    /// Resolves active 3D volume/point-cloud dimensions with fallback to matrix data or default.
    #[inline]
    pub fn get_volume_dimensions(&self) -> (u32, u32) {
        self.volume_data
            .as_ref()
            .map(|v| (v.width as u32, v.height as u32))
            .unwrap_or_else(|| {
                self.matrix_data
                    .as_ref()
                    .map_or((64, 64), |m| (m.width as u32, m.height as u32))
            })
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

impl OctantApp {
    /// Returns the source_id string for the currently plotted store.
    pub fn plotted_source_id(&self) -> String {
        StoreKind::make_source_id(self.plotted_store_kind, &self.plotted_store_target_input)
    }

    /// Returns the source_id string for the currently selected (UI active) store.
    pub fn selected_source_id(&self) -> String {
        StoreKind::make_source_id(self.selected_store_kind, &self.store_target_input)
    }

    /// Synchronizes all plotted configuration fields from the current UI selection.
    pub fn sync_plotted_state_from_selected(&mut self) {
        self.plotted_store_kind = self.selected_store_kind;
        self.plotted_store_target_input = self.store_target_input.clone();
        self.plotted_dataset_metadata = self.active_dataset_metadata.clone();
        self.plotted_variable_idx = self.selected_variable_idx;
        self.plotted_dim_config = self.dim_config.clone();
        self.plotted_selected_dim_indices = self.selected_dim_indices.clone();
        self.plotted_selected_dim_ranges = self.selected_dim_ranges.clone();
        self.plotted_spatial_dims = self.spatial_dims.clone();
        self.plotted_animated_dim = self.animated_dim;
        self.reset_variable_bounds();
    }

    /// Returns VariableInfo for the currently plotted variable, if available.
    pub fn plotted_variable_info(&self) -> Option<&crate::data::VariableInfo> {
        self.plotted_dataset_metadata
            .as_ref()
            .and_then(|m| m.variables.get(self.plotted_variable_idx))
    }

    /// Returns VariableInfo for the currently selected variable, if available.
    pub fn selected_variable_info(&self) -> Option<&crate::data::VariableInfo> {
        self.active_dataset_metadata
            .as_ref()
            .and_then(|m| m.variables.get(self.selected_variable_idx))
    }

    /// Returns the chunk size along `dim` for the currently plotted variable (defaults to 1).
    pub fn plotted_chunk_size(&self, dim: usize) -> usize {
        self.plotted_variable_info()
            .and_then(|v| v.chunk_shape.get(dim))
            .copied()
            .unwrap_or(1) as usize
    }

    /// Returns the total extent along `dim` for the currently plotted variable (defaults to 1).
    pub fn plotted_dim_size(&self, dim: usize) -> usize {
        self.plotted_variable_info()
            .and_then(|v| v.shape.get(dim))
            .copied()
            .unwrap_or(1) as usize
    }

    /// Checks if a dataset matching `target` (by URI, ID, or display name) is already in `dataset_manager`.
    /// If found and it has metadata, activates it and opens the variables overlay.
    pub fn try_activate_dataset(&mut self, target: &str) -> bool {
        let input_target = target.trim();
        if input_target.is_empty() {
            return false;
        }

        let existing = self
            .dataset_manager
            .iter()
            .find(|d| {
                d.source.uri == input_target
                    || d.id == input_target
                    || d.source.display_name == input_target
            })
            .cloned();

        if let Some(dataset) = existing {
            self.store_target_input = dataset.source.uri.clone();
            self.selected_store_kind = StoreKind::from_data_source_kind(&dataset.source.kind);
            if let Some(meta) = dataset.metadata {
                self.status_message = format!(
                    "Activated dataset '{}' (Found {} variables)",
                    meta.name,
                    meta.variables.len()
                );
                self.show_variables_overlay = true;
                self.active_dataset_metadata = Some(meta);
                self.selected_variable_idx = 0;
                return true;
            }
        }
        false
    }

    /// Triggers immediate lossless screenshot capture of the canvas framing region.
    pub fn trigger_save_screenshot(&mut self) {
        self.capture_config.pending_save = true;
    }

    /// Starts recording canvas interactions into an MP4 video.
    pub fn start_recording(&mut self) {
        if self.capture_config.export_state.is_some() {
            self.cancel_deterministic_export();
        }
        self.capture_config.is_recording = true;
        self.capture_config.pending_frame_capture = false;
        self.capture_config.recording_start_time = Some(std::time::Instant::now());
        self.capture_config.recorded_frames.clear();
        self.capture_config.last_recording_time = std::time::Instant::now();
        self.status_message =
            "⏺ Recording canvas interaction... Click Stop Recording to export MP4.".to_string();
    }

    /// Stops video recording and initiates background MP4 encoding and muxing.
    pub fn stop_recording(&mut self) {
        if !self.capture_config.is_recording {
            return;
        }
        self.capture_config.is_recording = false;
        self.capture_config.pending_frame_capture = false;
        self.capture_config.recording_start_time = None;
        let frames = std::mem::take(&mut self.capture_config.recorded_frames);
        let (width, height) = self.capture_config.recorded_frame_size;
        let fps = self.capture_config.recording_fps;
        let output_path = self.capture_config.generate_filepath(true);

        if frames.is_empty() || width == 0 || height == 0 {
            self.status_message = "Recording stopped (No frames captured)".to_string();
            return;
        }

        let filename = output_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("octant_recording.mp4")
            .to_string();

        self.capture_config.save_notification = Some((
            format!(
                "🎬 Saved video: {} ({} frames @ {:.0} fps)",
                filename,
                frames.len(),
                fps
            ),
            output_path.clone(),
            std::time::Instant::now(),
        ));

        self.status_message = format!(
            "🎬 Encoding MP4 video ({} frames @ {:.0} fps) to {}...",
            frames.len(),
            fps,
            filename
        );

        rayon::spawn(move || {
            match crate::utils::video::encode_rgba_frames_to_mp4(
                &frames,
                width,
                height,
                fps,
                &output_path,
            ) {
                Ok(()) => {
                    log::info!("Successfully saved recording to {}", output_path.display());
                }
                Err(err) => {
                    log::error!("Failed to encode MP4 recording: {}", err);
                }
            }
        });
    }

    /// Starts deterministic frame-by-frame animation export with optional camera orbit and zoom.
    pub fn start_deterministic_export(&mut self) {
        if self.capture_config.is_recording {
            self.stop_recording();
        }
        self.capture_config.pending_frame_capture = false;

        let total_timesteps = self.animated_dim_extent();
        let total_frames = match self.capture_config.motion_mode {
            crate::app::capture::MotionTrajectory::TimestepOnly => total_timesteps.max(2),
            _ => self.capture_config.export_total_frames.max(10),
        };

        let output_path = match self.capture_config.export_format {
            crate::app::capture::ExportFormat::Mp4Video => {
                self.capture_config.generate_filepath(true)
            }
            crate::app::capture::ExportFormat::PngImageSequence => {
                let default_dir = self.capture_config.resolve_output_dir(true);
                default_dir.join(format!(
                    "octant_sequence_{}",
                    crate::app::capture::CaptureConfig::timestamp_suffix()
                ))
            }
        };

        self.is_playing = false; // Pause interactive playback during export

        self.capture_config.export_state = Some(crate::app::capture::DeterministicExportState {
            is_active: true,
            current_frame: 0,
            total_frames,
            motion_mode: self.capture_config.motion_mode,
            zoom_mode: self.capture_config.zoom_mode,
            export_format: self.capture_config.export_format,
            export_fps: self.capture_config.recording_fps,
            output_path,
            captured_frames: Vec::with_capacity(total_frames),
            frame_size: (0, 0),
            initial_timestep: self.current_timestep,
            initial_rotation_y: self.sphere_rotation_y,
            initial_rotation_x: self.sphere_rotation_x,
            initial_zoom_3d: self.sphere_zoom,
            initial_zoom_2d: self.heatmap_zoom,
        });

        self.status_message = format!(
            "🎬 Starting deterministic animation export ({} frames)...",
            total_frames
        );
    }

    /// Restores initial camera, zoom, and timestep state from an export run.
    pub fn restore_initial_export_state(
        &mut self,
        state: &crate::app::capture::DeterministicExportState,
    ) {
        self.current_timestep = state.initial_timestep;
        self.sphere_rotation_y = state.initial_rotation_y;
        self.sphere_rotation_x = state.initial_rotation_x;
        self.sphere_zoom = state.initial_zoom_3d;
        self.heatmap_zoom = state.initial_zoom_2d;
        self.load_selected_variable_block();
    }

    /// Cancels in-progress deterministic animation export and restores camera state.
    pub fn cancel_deterministic_export(&mut self) {
        self.capture_config.pending_frame_capture = false;
        if let Some(state) = self.capture_config.export_state.take() {
            self.restore_initial_export_state(&state);
            self.status_message = "Animation export cancelled".to_string();
        }
    }

    /// Finishes deterministic animation export, restores camera state, and triggers MP4 encoding or PNG sequence saving.
    pub fn finish_deterministic_export(&mut self) {
        self.capture_config.pending_frame_capture = false;
        if let Some(state) = self.capture_config.export_state.take() {
            self.restore_initial_export_state(&state);

            let frames = state.captured_frames;
            let (width, height) = state.frame_size;
            let fps = state.export_fps;
            let output_path = state.output_path;
            let export_format = state.export_format;

            if frames.is_empty() || width == 0 || height == 0 {
                self.status_message = "Animation export completed (no frames)".to_string();
                return;
            }

            let filename = output_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("octant_export")
                .to_string();

            match export_format {
                crate::app::capture::ExportFormat::Mp4Video => {
                    self.capture_config.save_notification = Some((
                        format!(
                            "🎬 Saved animation video: {} ({} frames @ {:.0} fps)",
                            filename,
                            frames.len(),
                            fps
                        ),
                        output_path.clone(),
                        std::time::Instant::now(),
                    ));

                    self.status_message = format!(
                        "🎬 Encoding MP4 video ({} frames @ {:.0} fps) to {}...",
                        frames.len(),
                        fps,
                        filename
                    );

                    rayon::spawn(move || {
                        match crate::utils::video::encode_rgba_frames_to_mp4(
                            &frames,
                            width,
                            height,
                            fps,
                            &output_path,
                        ) {
                            Ok(()) => {
                                log::info!(
                                    "Successfully encoded animation video to {}",
                                    output_path.display()
                                );
                            }
                            Err(err) => {
                                log::error!("Failed to encode MP4 animation: {}", err);
                            }
                        }
                    });
                }
                crate::app::capture::ExportFormat::PngImageSequence => {
                    self.capture_config.save_notification = Some((
                        format!(
                            "📁 Saved PNG sequence: {} ({} frames)",
                            filename,
                            frames.len()
                        ),
                        output_path.clone(),
                        std::time::Instant::now(),
                    ));

                    self.status_message = format!(
                        "📁 Saving {} Display P3 PNG frames to {}...",
                        frames.len(),
                        filename
                    );

                    rayon::spawn(move || {
                        let _ = std::fs::create_dir_all(&output_path);
                        for (i, frame_bytes) in frames.iter().enumerate() {
                            let frame_path = output_path.join(format!("frame_{:04}.png", i));
                            if let Some(img) =
                                image::RgbaImage::from_raw(width, height, frame_bytes.to_vec())
                            {
                                let _ = crate::utils::png::save_display_p3_png(&img, &frame_path);
                            }
                        }
                        log::info!(
                            "Successfully saved PNG sequence to {}",
                            output_path.display()
                        );
                    });
                }
            }
        }
    }
}
