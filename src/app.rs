use crate::cache::{SliceCacheKey, SliceLruCache, SlicePrefetcher};
use crate::data::matrix_data::MatrixData;
use crate::plots::{
    MatrixCallback, MatrixRenderer, PlotType, PointCloudCallback, PointCloudRenderer,
    SphereCallback, SphereRenderer, SurfaceCallback, SurfaceRenderer, VolumeCallback,
    VolumeRenderer,
};
use crate::stores::{
    DataStore, DatasetMetadata, VariableInfo, icechunk_local::IcechunkLocalStore,
    icechunk_remote::IcechunkRemoteStore, zarr_local::ZarrLocalStore, zarr_remote::ZarrRemoteStore,
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

pub struct OctantApp {
    pub selected_store_kind: StoreKind,
    pub store_target_input: String,
    pub active_dataset_metadata: Option<DatasetMetadata>,
    pub selected_variable_idx: usize,
    pub current_timestep: usize,
    pub active_plot_type: PlotType,
    pub active_colormap: u32,
    pub preview_colormap: Option<u32>,
    pub status_message: String,
    pub is_loading: bool,
    pub matrix_data: Option<MatrixData>,
    pub renderer: Option<Arc<MatrixRenderer>>,
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
    pub show_colorbar: bool,
    pub is_categorical: bool,
    pub wgpu_render_state: Option<eframe::egui_wgpu::RenderState>,

    // LRU Cache & Prefetcher State
    pub lru_cache: SliceLruCache,
    pub prefetcher: SlicePrefetcher,
    pub max_cache_mb: usize,
    pub prefetch_lookahead: usize,

    // Animation & Playback Controls
    pub is_fetching_slice: bool,
    pub active_requested_key: Option<SliceCacheKey>,
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
    pub show_right_panel: bool,
    pub show_bottom_bar: bool,
    pub theme_preference: egui::ThemePreference,
    pub selected_dim_indices: Vec<usize>,
    pub selected_dim_ranges: Vec<(usize, usize)>,

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
}

impl OctantApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let wgpu_render_state = cc.wgpu_render_state.clone();
        let default_cache_mb = 1024; // Default 1GB cache size limit

        let app = Self {
            selected_store_kind: StoreKind::RemoteZarr,
            store_target_input: "https://s3.bgc-jena.mpg.de:9000/esdl-esdc-v3.0.2/esdc-16d-2.5deg-46x72x1440-3.0.2.zarr".to_string(),
            active_dataset_metadata: None,
            selected_variable_idx: 0,
            current_timestep: 0,
            active_plot_type: PlotType::Heatmap,
            active_colormap: 0,
            preview_colormap: None,
            status_message: "Ready. Select store and click Inspect Store Metadata.".to_string(),
            is_loading: false,
            matrix_data: None,
            renderer: None,
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
            show_colorbar: true,
            is_categorical: false,
            wgpu_render_state,

            lru_cache: SliceLruCache::new(default_cache_mb * 1024 * 1024),
            prefetcher: SlicePrefetcher::new(),
            max_cache_mb: default_cache_mb,
            prefetch_lookahead: 24,

            is_fetching_slice: false,
            active_requested_key: None,
            metadata_rx: None,
            is_playing: false,
            playback_fps: 15.0,
            loop_playback: true,
            last_step_time: std::time::Instant::now(),

            show_catalog_window: false,
            catalog_search_query: String::new(),
            catalog_category_filter: crate::catalog::CatalogCategoryFilter::All,

            show_left_panel: true,
            show_right_panel: true,
            show_bottom_bar: true,
            theme_preference: egui::ThemePreference::System,
            selected_dim_indices: Vec::new(),
            selected_dim_ranges: Vec::new(),

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
        };

        // App starts clean without auto-fetching. User clicks "Fetch Store Metadata" when ready.
        app
    }

    pub fn inspect_active_store(&mut self) {
        self.is_loading = true;
        self.status_message = format!("Inspecting {:?} metadata...", self.selected_store_kind);

        let store_kind = self.selected_store_kind;
        let target_input = self.store_target_input.clone();

        let (tx, rx) = std::sync::mpsc::channel();
        self.metadata_rx = Some(rx);

        std::thread::spawn(move || {
            let store: Box<dyn DataStore> = match store_kind {
                StoreKind::RemoteZarr => Box::new(ZarrRemoteStore::new(&target_input)),
                StoreKind::LocalZarr => Box::new(ZarrLocalStore::new(&target_input)),
                StoreKind::RemoteIcechunk => Box::new(IcechunkRemoteStore::new(&target_input)),
                StoreKind::LocalIcechunk => Box::new(IcechunkLocalStore::new(&target_input)),
                StoreKind::ProceduralRandom => {
                    let meta = DatasetMetadata {
                        name: "Procedural Test Store".to_string(),
                        store_type: "Random Procedural".to_string(),
                        variables: vec![VariableInfo {
                            name: "random_matrix".to_string(),
                            data_type: "float32".to_string(),
                            shape: vec![64, 64],
                            dimension_names: vec!["y".to_string(), "x".to_string()],
                            chunk_shape: vec![64, 64],
                            file_size: crate::utils::calculate_variable_size_bytes(
                                &[64, 64],
                                "float32",
                            ),
                            ..Default::default()
                        }],
                        dimension_coordinates: std::collections::HashMap::new(),
                    };
                    let _ = tx.send(Ok(meta));
                    return;
                }
            };

            let res = store.inspect().map_err(|e| e.to_string());
            let _ = tx.send(res);
        });
    }

    pub fn load_selected_variable_slice(&mut self) {
        let (var_name, max_steps, chunk_time_size, slice_bytes_hint) = {
            if let Some(metadata) = &self.active_dataset_metadata {
                if let Some(var_info) = metadata.variables.get(self.selected_variable_idx) {
                    let max_steps = if var_info.shape.len() <= 2 {
                        if let Some(first_dim) = var_info.dimension_names.first() {
                            let name = first_dim.to_lowercase();
                            if name == "time" || name == "t" || name.contains("step") {
                                var_info.shape.first().copied().unwrap_or(1) as usize
                            } else {
                                1
                            }
                        } else {
                            1
                        }
                    } else {
                        var_info.shape.first().copied().unwrap_or(1) as usize
                    };
                    let chunk_time_size = if !var_info.chunk_shape.is_empty() {
                        var_info.chunk_shape[0] as usize
                    } else {
                        46
                    };

                    let slice_bytes = if var_info.shape.len() >= 2 {
                        let w = var_info.shape[var_info.shape.len() - 1] as usize;
                        let h = var_info.shape[var_info.shape.len() - 2] as usize;
                        w * h * std::mem::size_of::<f32>()
                    } else {
                        64 * 64 * std::mem::size_of::<f32>()
                    };

                    (
                        var_info.name.clone(),
                        max_steps,
                        chunk_time_size,
                        slice_bytes,
                    )
                } else {
                    return;
                }
            } else {
                return;
            }
        };

        // Enforce bounds on current_timestep for new variable
        if self.current_timestep >= max_steps {
            self.current_timestep = 0;
        }

        let cache_key = SliceCacheKey {
            store_kind: self.selected_store_kind,
            store_target: self.store_target_input.clone(),
            variable_name: var_name.clone(),
            timestep: self.current_timestep,
        };

        self.active_requested_key = Some(cache_key.clone());

        // 1. Check LRU Cache HIT
        if let Some(slice) = self.lru_cache.get(&cache_key) {
            let mdata = MatrixData::new(
                slice.width,
                slice.height,
                slice.values.clone(),
                slice.min_val,
                slice.max_val,
                format!("{} ({})", slice.dataset_name, slice.variable_name),
                slice.max_timesteps,
            );
            self.rebuild_pipeline_with_matrix_data(mdata);
            self.is_fetching_slice = false;
            self.status_message = format!(
                "🚀 Cache HIT for '{}' (step {})",
                slice.variable_name,
                self.current_timestep + 1
            );

            // Trigger chunk-aligned prefetching ahead
            let total_steps = max_steps.max(slice.max_timesteps);
            self.prefetcher.prefetch_chunk_aligned(
                self.selected_store_kind,
                &self.store_target_input,
                &var_name,
                self.current_timestep,
                total_steps,
                chunk_time_size,
                slice_bytes_hint,
                self.prefetch_lookahead,
                &self.lru_cache,
            );
            return;
        }

        // Procedural random bypass
        if self.selected_store_kind == StoreKind::ProceduralRandom {
            if let Ok(data) = MatrixData::create_random_matrix(64, 64) {
                self.rebuild_pipeline_with_matrix_data(data);
            }
            self.is_fetching_slice = false;
            return;
        }

        // 2. Cache MISS: Dispatch Non-Blocking Async Background Request for active slice ONLY!
        self.is_fetching_slice = true;
        self.status_message = format!(
            "⏳ Downloading slice for '{}' (step {})...",
            var_name,
            self.current_timestep + 1
        );

        // Fetch active slice first so 2D map renders immediately on screen
        self.prefetcher.request_slice(cache_key, &self.lru_cache);
    }

    pub fn trigger_background_prefetch(&mut self) {
        if let Some(metadata) = &self.active_dataset_metadata
            && let Some(var_info) = metadata.variables.get(self.selected_variable_idx)
        {
            let max_steps = if var_info.shape.len() <= 2 {
                1
            } else {
                var_info.shape.first().copied().unwrap_or(1) as usize
            };
            if max_steps <= 1 {
                return;
            }
            let chunk_time_size = if !var_info.chunk_shape.is_empty() {
                var_info.chunk_shape[0] as usize
            } else {
                46
            };
            let slice_bytes = if var_info.shape.len() >= 2 {
                let w = var_info.shape[var_info.shape.len() - 1] as usize;
                let h = var_info.shape[var_info.shape.len() - 2] as usize;
                w * h * std::mem::size_of::<f32>()
            } else {
                64 * 64 * std::mem::size_of::<f32>()
            };

            self.prefetcher.prefetch_chunk_aligned(
                self.selected_store_kind,
                &self.store_target_input,
                &var_info.name,
                self.current_timestep,
                max_steps,
                chunk_time_size,
                slice_bytes,
                self.prefetch_lookahead,
                &self.lru_cache,
            );
        }
    }

    pub fn rebuild_pipeline_with_matrix_data(&mut self, data: MatrixData) {
        if let Some(wgpu_render_state) = &self.wgpu_render_state {
            let same_dimensions = self
                .matrix_data
                .as_ref()
                .is_some_and(|m| m.width == data.width && m.height == data.height);

            if same_dimensions
                && self.renderer.is_some()
                && self.sphere_renderer.is_some()
                && self.surface_renderer.is_some()
                && self.volume_renderer.is_some()
                && self.point_cloud_renderer.is_some()
            {
                if let Some(renderer) = &self.renderer {
                    renderer.update_data(&wgpu_render_state.queue, &data.values);
                }
                if let Some(sphere_renderer) = &self.sphere_renderer {
                    sphere_renderer.update_data(&wgpu_render_state.queue, &data.values);
                }
                if let Some(surface_renderer) = &self.surface_renderer {
                    surface_renderer.update_data(&wgpu_render_state.queue, &data.values);
                }
                if let Some(volume_renderer) = &mut self.volume_renderer
                    && let Some(r) = Arc::get_mut(volume_renderer)
                {
                    r.update_data(&wgpu_render_state.queue, &data.values)
                }
                if let Some(point_cloud_renderer) = &mut self.point_cloud_renderer
                    && let Some(r) = Arc::get_mut(point_cloud_renderer)
                {
                    r.update_data(&wgpu_render_state.queue, &data.values)
                }
            } else {
                let renderer = MatrixRenderer::new(
                    &wgpu_render_state.device,
                    wgpu_render_state.target_format,
                    &data.values,
                    data.width,
                    data.height,
                );
                let sphere_renderer = SphereRenderer::new(
                    &wgpu_render_state.device,
                    wgpu_render_state.target_format,
                    &data.values,
                    data.width,
                    data.height,
                );
                let surface_renderer = SurfaceRenderer::new(
                    &wgpu_render_state.device,
                    wgpu_render_state.target_format,
                    &data.values,
                    data.width,
                    data.height,
                );
                let volume_renderer = VolumeRenderer::new(
                    &wgpu_render_state.device,
                    wgpu_render_state.target_format,
                    &data.values,
                    data.width as u32,
                    data.height as u32,
                );
                let point_cloud_renderer = PointCloudRenderer::new(
                    &wgpu_render_state.device,
                    wgpu_render_state.target_format,
                    &data.values,
                    data.width as u32,
                    data.height as u32,
                );
                self.renderer = Some(Arc::new(renderer));
                self.sphere_renderer = Some(Arc::new(sphere_renderer));
                self.surface_renderer = Some(Arc::new(surface_renderer));
                self.volume_renderer = Some(Arc::new(volume_renderer));
                self.point_cloud_renderer = Some(Arc::new(point_cloud_renderer));
            }
        }

        self.global_data_min = self.global_data_min.min(data.min_val);
        self.global_data_max = self.global_data_max.max(data.max_val);

        if !self.lock_color_bounds {
            self.color_range_min = data.min_val;
            self.color_range_max = data.max_val;
            self.volume_cmin = data.min_val;
            self.volume_cmax = data.max_val;
        }

        self.matrix_data = Some(data);
    }

    pub fn get_color_params(&self) -> crate::plots::common::PlotColorParams {
        let effective_colormap = self.preview_colormap.unwrap_or(self.active_colormap);

        let (is_cat, num_cats) = if self.is_categorical {
            if let Some(mdata) = &self.matrix_data {
                if let Some(unique) = mdata.detect_unique_values() {
                    (1, unique.len() as u32)
                } else {
                    (1, 10)
                }
            } else {
                (1, 10)
            }
        } else {
            (0, 10)
        };

        crate::plots::common::PlotColorParams {
            colormap: effective_colormap,
            cmin: self.color_range_min,
            cmax: self.color_range_max,
            use_nan_color: if self.use_nan_color { 1 } else { 0 },
            use_lowclip: if self.use_lowclip { 1 } else { 0 },
            use_highclip: if self.use_highclip { 1 } else { 0 },
            scale_type: self.active_scale_type,
            scale_param: self.scale_param,
            is_categorical: is_cat,
            num_categories: num_cats,
            _pad0: 0,
            _pad1: 0,
            nan_color: self.nan_color,
            lowclip_color: self.lowclip_color,
            highclip_color: self.highclip_color,
        }
    }

    pub fn get_3d_aspect_ratio(&self) -> (f32, f32, f32) {
        let (w, h, max_t) = self.matrix_data.as_ref().map_or((64, 64, 64), |m| {
            (m.width as u32, m.height as u32, m.max_timesteps as u32)
        });

        let (shape_d, shape_h, shape_w) = if let Some(meta) = &self.active_dataset_metadata {
            if let Some(v) = meta.variables.get(self.selected_variable_idx) {
                if v.shape.len() >= 3 {
                    (v.shape[0] as u32, v.shape[1] as u32, v.shape[2] as u32)
                } else {
                    (max_t, h, w)
                }
            } else {
                (max_t, h, w)
            }
        } else {
            (max_t, h, w)
        };

        let width = shape_w.max(w);
        let height = shape_h.max(h);
        let depth = shape_d.max(max_t);

        let max_spatial = (width.max(height)) as f32;
        let aspect_x = width as f32 / max_spatial;
        let aspect_y = height as f32 / max_spatial;
        let aspect_z = ((depth as f32 / max_spatial) * 0.12).clamp(0.4, 1.0);

        (aspect_x, aspect_y, aspect_z)
    }
}

impl eframe::App for OctantApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        // Reset hover preview at start of frame
        self.preview_colormap = None;

        // 0. Poll completed background metadata inspection
        let mut metadata_done = false;
        if let Some(rx) = &self.metadata_rx {
            if let Ok(result) = rx.try_recv() {
                metadata_done = true;
                self.is_loading = false;
                match result {
                    Ok(metadata) => {
                        self.status_message = format!(
                            "Inspected '{}' (Found {} variables)",
                            metadata.name,
                            metadata.variables.len()
                        );
                        if let Some(first_var) = metadata.variables.first() {
                            self.selected_dim_indices = vec![0; first_var.shape.len()];
                            self.selected_dim_ranges = first_var
                                .shape
                                .iter()
                                .map(|&s| (0, (s as usize).saturating_sub(1)))
                                .collect();
                        }

                        self.active_dataset_metadata = Some(metadata);
                        self.selected_variable_idx = 0;
                        self.show_right_panel = true;
                        self.load_selected_variable_slice();
                    }
                    Err(err) => {
                        self.status_message = format!("Store inspect error: {}", err);
                    }
                }
            } else {
                ctx.request_repaint();
            }
        }
        if metadata_done {
            self.metadata_rx = None;
        }

        // 1. Drain completed background prefetch results into LRU cache
        let completed_prefetches = self.prefetcher.poll_results();
        for res in completed_prefetches {
            if let Ok(slice) = res.result {
                let is_active_target = self.active_requested_key.as_ref() == Some(&res.key);

                self.lru_cache.put(res.key, slice.clone());

                if is_active_target {
                    let mdata = MatrixData::new(
                        slice.width,
                        slice.height,
                        slice.values,
                        slice.min_val,
                        slice.max_val,
                        format!("{} ({})", slice.dataset_name, slice.variable_name),
                        slice.max_timesteps,
                    );
                    self.rebuild_pipeline_with_matrix_data(mdata);
                    self.is_fetching_slice = false;
                    self.status_message = format!(
                        "⚡ Loaded slice for '{}' (step {})",
                        slice.variable_name,
                        slice.current_timestep + 1
                    );
                }
            }
        }

        // Continuously replenish prefetch buffer when active slice is rendered
        if !self.is_fetching_slice {
            self.trigger_background_prefetch();
        }

        // 2. Playback Animation Timer Loop
        if self.is_playing {
            let now = std::time::Instant::now();
            let frame_dur = std::time::Duration::from_secs_f32(1.0 / self.playback_fps.max(1.0));

            // Only advance playback when current requested slice is already loaded & rendered
            if !self.is_fetching_slice {
                if now.duration_since(self.last_step_time) >= frame_dur {
                    self.last_step_time = now;
                    let max_steps = self
                        .matrix_data
                        .as_ref()
                        .map(|h| h.max_timesteps)
                        .unwrap_or(1);

                    if max_steps > 1 {
                        if self.current_timestep + 1 < max_steps {
                            self.current_timestep += 1;
                        } else if self.loop_playback {
                            self.current_timestep = 0;
                        } else {
                            self.is_playing = false;
                        }
                        self.load_selected_variable_slice();
                    }
                }
            } else {
                // Keep timer fresh while waiting for slice download
                self.last_step_time = now;
            }
            ctx.request_repaint_after(frame_dur);
        } else {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }

        // 3. Render Left Store Panel, Top Navigation Bar, Plot Controls, Bottom Playback Toolbar & Right Selection Panel
        crate::ui::store::show_left_panel(self, ui);
        crate::ui::top_bar::show_top_bar(self, ui);
        crate::ui::bottom_bar::show_plot_controls_bar(self, ui);
        crate::ui::bottom_bar::show_bottom_bar(self, ui);
        crate::ui::catalog::show_catalog_window(self, &ctx);
        crate::ui::colorbar::show_colorbar_overlay(self, &ctx);
        crate::ui::variables_panel::show_right_panel(self, &ctx);

        // 4. Drawing Canvas Area with Aspect Data Ratio
        {
            let canvas_rect = ui.available_rect_before_wrap();
            let response = ui.allocate_rect(canvas_rect, egui::Sense::drag());

            let canvas_bg = ui.style().visuals.panel_fill;
            ui.painter().rect_filled(canvas_rect, 0.0, canvas_bg);

            // Enforce aspect data ratio (matrix.width / matrix.height)
            let plot_rect = if let Some(matrix) = &self.matrix_data {
                let data_aspect = (matrix.width as f32 / matrix.height as f32).max(0.01);
                let avail_w = canvas_rect.width();
                let avail_h = canvas_rect.height();
                let avail_aspect = avail_w / avail_h.max(1.0);

                let (plot_w, plot_h) = if avail_aspect > data_aspect {
                    (avail_h * data_aspect, avail_h)
                } else {
                    (avail_w, avail_w / data_aspect)
                };

                egui::Rect::from_center_size(canvas_rect.center(), egui::vec2(plot_w, plot_h))
            } else {
                canvas_rect
            };

            if response.dragged() {
                let delta = response.drag_delta();
                self.sphere_rotation_y += delta.x * 0.008;
                self.sphere_rotation_x = (self.sphere_rotation_x + delta.y * 0.008).clamp(
                    -std::f32::consts::FRAC_PI_2 + 0.05,
                    std::f32::consts::FRAC_PI_2 - 0.05,
                );
            }

            if response.hovered() {
                let scroll = ui.input(|i| i.smooth_scroll_delta.y);
                if scroll != 0.0 {
                    self.sphere_zoom = (self.sphere_zoom - scroll * 0.003).clamp(1.1, 8.0);
                    ui.ctx().request_repaint();
                }
            }

            if self.sphere_auto_rotate
                && (self.active_plot_type == PlotType::Sphere
                    || self.active_plot_type == PlotType::Surface
                    || self.active_plot_type == PlotType::Volume
                    || self.active_plot_type == PlotType::PointCloud)
            {
                self.sphere_rotation_y += ui.ctx().input(|i| i.stable_dt).min(0.1) * 0.15;
                ui.ctx().request_repaint();
            }

            match self.active_plot_type {
                PlotType::Sphere => {
                    if let Some(sphere_renderer) = &self.sphere_renderer {
                        let callback = eframe::egui_wgpu::Callback::new_paint_callback(
                            plot_rect,
                            SphereCallback {
                                renderer: sphere_renderer.clone(),
                                color_params: self.get_color_params(),
                                rotation_y: self.sphere_rotation_y,
                                rotation_x: self.sphere_rotation_x,
                                zoom: self.sphere_zoom,
                                displacement_strength: self.sphere_displacement_strength,
                                sphere_mode: self.sphere_mode,
                                rect: plot_rect,
                            },
                        );
                        ui.painter().add(callback);
                    }
                }
                PlotType::Surface => {
                    if let Some(surface_renderer) = &self.surface_renderer {
                        let callback = eframe::egui_wgpu::Callback::new_paint_callback(
                            plot_rect,
                            SurfaceCallback {
                                renderer: surface_renderer.clone(),
                                color_params: self.get_color_params(),
                                rotation_y: self.sphere_rotation_y,
                                rotation_x: self.sphere_rotation_x,
                                zoom: self.sphere_zoom,
                                displacement_strength: self.surface_displacement_strength,
                                surface_mode: self.surface_mode,
                                rect: plot_rect,
                            },
                        );
                        ui.painter().add(callback);
                    }
                }
                PlotType::Volume => {
                    if let Some(volume_renderer) = &self.volume_renderer {
                        let (width, height) = self
                            .matrix_data
                            .as_ref()
                            .map_or((64, 64), |m| (m.width as u32, m.height as u32));
                        let (aspect_x, aspect_y, aspect_z) = self.get_3d_aspect_ratio();

                        let callback = eframe::egui_wgpu::Callback::new_paint_callback(
                            plot_rect,
                            VolumeCallback {
                                renderer: volume_renderer.clone(),
                                color_params: self.get_color_params(),
                                rot_y: self.sphere_rotation_y,
                                rot_x: self.sphere_rotation_x,
                                aspect_x,
                                aspect_y,
                                aspect_z,
                                zoom: self.sphere_zoom,
                                opacity_scale: self.volume_opacity,
                                step_count: self.volume_step_count,
                                width,
                                height,
                                algorithm: self.volume_algorithm,
                                isovalue: self.volume_isovalue,
                                isorange: self.volume_isorange,
                                rect: plot_rect,
                            },
                        );
                        ui.painter().add(callback);
                    }
                }
                PlotType::PointCloud => {
                    if let Some(point_cloud_renderer) = &self.point_cloud_renderer {
                        let (width, height) = self
                            .matrix_data
                            .as_ref()
                            .map_or((64, 64), |m| (m.width as u32, m.height as u32));
                        let (aspect_x, aspect_y, aspect_z) = self.get_3d_aspect_ratio();

                        let callback = eframe::egui_wgpu::Callback::new_paint_callback(
                            plot_rect,
                            PointCloudCallback {
                                renderer: point_cloud_renderer.clone(),
                                color_params: self.get_color_params(),
                                rot_y: self.sphere_rotation_y,
                                rot_x: self.sphere_rotation_x,
                                aspect_x,
                                aspect_y,
                                aspect_z,
                                zoom: self.sphere_zoom,
                                point_size: self.point_cloud_size,
                                width,
                                height,
                                rect: plot_rect,
                            },
                        );
                        ui.painter().add(callback);
                    }
                }
                _ => {
                    if let Some(renderer) = &self.renderer {
                        let callback = eframe::egui_wgpu::Callback::new_paint_callback(
                            plot_rect,
                            MatrixCallback {
                                renderer: renderer.clone(),
                                color_params: self.get_color_params(),
                                rect: plot_rect,
                            },
                        );
                        ui.painter().add(callback);
                    }
                }
            }

            // Render high-performance Hover Pixel Info Tooltip & Canvas Reticle
            crate::ui::hover_tooltip::show_hover_tooltip(self, &ctx, ui, &response, plot_rect);
        }
    }
}
