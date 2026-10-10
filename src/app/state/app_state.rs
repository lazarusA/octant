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
    pub preview_colormap: Option<u32>,
    pub colormaps: super::ColormapState,
    pub status_message: String,
    pub is_loading: bool,
    pub nav: super::NavigationState,
    pub plot_configs: super::PlotConfigs,
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
    pub playback: super::PlaybackState,
    pub metadata_rx: Option<
        std::sync::mpsc::Receiver<Result<(DatasetMetadata, crate::data::StoreHandle), String>>,
    >,

    pub layout: super::UiLayoutState,

    // 2D Flatmap Heatmap & 1D Line Plot Viewport Zoom / Pan State
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
