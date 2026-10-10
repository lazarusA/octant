//! `OctantApp` defaults: the state of a fresh launch.

use super::app_state::OctantApp;
use crate::app::layers::LayerStack;
use crate::app::layers::VariableSelection;

impl Default for OctantApp {
    fn default() -> Self {
        let default_cache_mb = 1024; // Default 1GB cache size limit

        Self {
            selected: VariableSelection::default(),
            cached_variable_tree: None,
            cached_search: None,
            layers: LayerStack::default(),
            preview_colormap: None,
            colormaps: super::ColormapState::default(),
            status_message: "Ready. Select store and click Inspect Store Metadata.".to_string(),
            is_loading: false,
            nav: super::NavigationState::default(),
            plot_configs: super::PlotConfigs::default(),
            wgpu_render_state: None,

            dataset_manager: crate::data::DatasetManager::new(),
            block_cache: crate::data::BlockCache::new(default_cache_mb * 1024 * 1024),
            block_prefetcher: crate::data::BlockPrefetcher::new(),
            coordinate_loader: crate::data::blocks::CoordinateLoader::default(),
            coordinates_revision: 0,
            max_cache_mb: default_cache_mb,
            block_window_size: 32,
            prefetch_threads: 16,

            playback: super::PlaybackState::default(),
            metadata_rx: None,

            layout: super::UiLayoutState::default(),

            enable_pyramid_resampling: false,
            pyramid_aggregation_op: crate::data::AggregationOp::default(),
            export_settings: crate::export::ExportSettings::default(),
            show_export_modal: false,
            show_crop_overlay: false,
            roi_crop_box: crate::export::RoiCropBox::default(),
            pending_export: None,
            export_flash_timer: None,
            toasts: crate::ui::toast::Toasts::default(),

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
