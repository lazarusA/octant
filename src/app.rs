use crate::cache::{SliceCacheKey, SliceLruCache, SlicePrefetcher};
use crate::matrix_data::MatrixData;
use crate::renderer::{MatrixCallback, MatrixRenderer};
use crate::stores::{
    icechunk_local::IcechunkLocalStore, icechunk_remote::IcechunkRemoteStore,
    zarr_local::ZarrLocalStore, zarr_remote::ZarrRemoteStore, DataStore, DatasetMetadata, VariableInfo,
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
    pub active_colormap: u32,
    pub status_message: String,
    pub is_loading: bool,
    pub matrix_data: Option<MatrixData>,
    pub renderer: Option<Arc<MatrixRenderer>>,
    pub wgpu_render_state: Option<eframe::egui_wgpu::RenderState>,

    // LRU Cache & Prefetcher State
    pub lru_cache: SliceLruCache,
    pub prefetcher: SlicePrefetcher,
    pub max_cache_mb: usize,
    pub prefetch_lookahead: usize,

    // Animation & Playback Controls
    pub is_fetching_slice: bool,
    pub is_playing: bool,
    pub playback_fps: f32,
    pub loop_playback: bool,
    pub last_step_time: std::time::Instant,
}

impl OctantApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let wgpu_render_state = cc.wgpu_render_state.clone();
        let default_cache_mb = 1024; // Default 1GB cache size limit

        let mut app = Self {
            selected_store_kind: StoreKind::RemoteZarr,
            store_target_input: "https://s3.bgc-jena.mpg.de:9000/esdl-esdc-v3.0.2/esdc-16d-2.5deg-46x72x1440-3.0.2.zarr".to_string(),
            active_dataset_metadata: None,
            selected_variable_idx: 0,
            current_timestep: 0,
            active_colormap: 0,
            status_message: "Ready. Select store and click Inspect Store Metadata.".to_string(),
            is_loading: false,
            matrix_data: None,
            renderer: None,
            wgpu_render_state,

            lru_cache: SliceLruCache::new(default_cache_mb * 1024 * 1024),
            prefetcher: SlicePrefetcher::new(),
            max_cache_mb: default_cache_mb,
            prefetch_lookahead: 24,

            is_fetching_slice: false,
            is_playing: false,
            playback_fps: 15.0,
            loop_playback: true,
            last_step_time: std::time::Instant::now(),
        };

        // Perform initial store inspection and load
        app.inspect_active_store();
        app
    }

    pub fn inspect_active_store(&mut self) {
        self.is_loading = true;
        self.status_message = format!("Inspecting {:?} metadata...", self.selected_store_kind);

        let store: Box<dyn DataStore> = match self.selected_store_kind {
            StoreKind::RemoteZarr => Box::new(ZarrRemoteStore::new(&self.store_target_input)),
            StoreKind::LocalZarr => Box::new(ZarrLocalStore::new(&self.store_target_input)),
            StoreKind::RemoteIcechunk => Box::new(IcechunkRemoteStore::new(&self.store_target_input)),
            StoreKind::LocalIcechunk => Box::new(IcechunkLocalStore::new(&self.store_target_input)),
            StoreKind::ProceduralRandom => {
                let random_matrix = MatrixData::create_random_matrix(64, 64).unwrap();
                self.active_dataset_metadata = Some(DatasetMetadata {
                    name: "Procedural Test Store".to_string(),
                    store_type: "Random Procedural".to_string(),
                    variables: vec![VariableInfo {
                        name: "random_matrix".to_string(),
                        data_type: "float32".to_string(),
                        shape: vec![64, 64],
                        dimension_names: vec!["y".to_string(), "x".to_string()],
                        chunk_shape: vec![64, 64],
                        file_size: crate::utils::calculate_variable_size_bytes(&[64, 64], "float32"),
                    }],
                });
                self.selected_variable_idx = 0;
                self.rebuild_pipeline_with_matrix_data(random_matrix);
                self.status_message = "Loaded procedural test matrix (64x64).".to_string();
                self.is_loading = false;
                return;
            }
        };

        match store.inspect() {
            Ok(metadata) => {
                self.status_message = format!(
                    "Inspected '{}' (Found {} variables)",
                    metadata.name,
                    metadata.variables.len()
                );
                self.active_dataset_metadata = Some(metadata);
                self.selected_variable_idx = 0;
                self.load_selected_variable_slice();
            }
            Err(err) => {
                self.status_message = format!("Store inspect error: {}", err);
            }
        }
        self.is_loading = false;
    }

    pub fn load_selected_variable_slice(&mut self) {
        let (var_name, timestep, max_steps, chunk_time_size) = {
            if let Some(metadata) = &self.active_dataset_metadata {
                if let Some(var_info) = metadata.variables.get(self.selected_variable_idx) {
                    let max_steps = if var_info.shape.is_empty() {
                        1
                    } else {
                        var_info.shape[0] as usize
                    };
                    let chunk_time_size = if !var_info.chunk_shape.is_empty() {
                        var_info.chunk_shape[0] as usize
                    } else {
                        46
                    };
                    (var_info.name.clone(), self.current_timestep, max_steps, chunk_time_size)
                } else {
                    return;
                }
            } else {
                return;
            }
        };

        let cache_key = SliceCacheKey {
            store_kind: self.selected_store_kind,
            store_target: self.store_target_input.clone(),
            variable_name: var_name.clone(),
            timestep,
        };

        // 1. Check LRU Cache HIT
        if let Some(slice) = self.lru_cache.get(&cache_key) {
            let mdata = MatrixData {
                width: slice.width,
                height: slice.height,
                values: slice.values.clone(),
                dataset_name: format!("{} ({})", slice.dataset_name, slice.variable_name),
                max_timesteps: slice.max_timesteps,
            };
            self.rebuild_pipeline_with_matrix_data(mdata);
            self.is_fetching_slice = false;
            self.status_message = format!("🚀 Cache HIT for '{}' (step {})", slice.variable_name, timestep + 1);

            // Trigger chunk-aligned prefetching ahead
            let total_steps = max_steps.max(slice.max_timesteps);
            self.prefetcher.prefetch_chunk_aligned(
                self.selected_store_kind,
                &self.store_target_input,
                &var_name,
                timestep,
                total_steps,
                chunk_time_size,
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

        // 2. Cache MISS: Dispatch Non-Blocking Async Background Request!
        self.is_fetching_slice = true;
        self.status_message = format!("⏳ Downloading slice for '{}' (step {})...", var_name, timestep + 1);

        self.prefetcher.request_slice(cache_key, &self.lru_cache);

        self.prefetcher.prefetch_chunk_aligned(
            self.selected_store_kind,
            &self.store_target_input,
            &var_name,
            timestep,
            max_steps,
            chunk_time_size,
            self.prefetch_lookahead,
            &self.lru_cache,
        );
    }

    pub fn rebuild_pipeline_with_matrix_data(&mut self, data: MatrixData) {
        if let Some(wgpu_render_state) = &self.wgpu_render_state {
            let renderer = MatrixRenderer::new(
                &wgpu_render_state.device,
                wgpu_render_state.target_format,
                &data.values,
                data.width,
                data.height,
            );
            self.renderer = Some(Arc::new(renderer));
        }
        self.matrix_data = Some(data);
    }
}

impl eframe::App for OctantApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1. Drain completed background prefetch results into LRU cache
        let active_var_name = self
            .active_dataset_metadata
            .as_ref()
            .and_then(|m| m.variables.get(self.selected_variable_idx))
            .map(|v| v.name.clone())
            .unwrap_or_default();

        let completed_prefetches = self.prefetcher.poll_results();
        for res in completed_prefetches {
            if let Ok(slice) = res.result {
                let is_active_step = res.key.store_kind == self.selected_store_kind
                    && res.key.store_target == self.store_target_input
                    && res.key.variable_name == active_var_name
                    && res.key.timestep == self.current_timestep;

                self.lru_cache.put(res.key, slice.clone());

                if is_active_step {
                    let mdata = MatrixData {
                        width: slice.width,
                        height: slice.height,
                        values: slice.values,
                        dataset_name: format!("{} ({})", slice.dataset_name, slice.variable_name),
                        max_timesteps: slice.max_timesteps,
                    };
                    self.rebuild_pipeline_with_matrix_data(mdata);
                    self.is_fetching_slice = false;
                    self.status_message = format!("⚡ Loaded slice for '{}' (step {})", active_var_name, self.current_timestep + 1);
                }
            }
        }


        // 2. Playback Animation Timer Loop
        if self.is_playing {
            let now = std::time::Instant::now();
            let frame_dur = std::time::Duration::from_secs_f32(1.0 / self.playback_fps.max(1.0));
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
            ctx.request_repaint_after(frame_dur);
        } else {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }

        // 3. Render Top Navigation Bar & Bottom Playback Toolbar
        crate::ui::top_bar::show_top_bar(self, ctx);
        crate::ui::bottom_bar::show_bottom_bar(self, ctx);

        // 4. Centered Drawing Canvas Area with Aspect Data Ratio
        egui::CentralPanel::default().show(ctx, |ui| {
            let available_rect = ui.available_rect_before_wrap();

            // Enforce aspect data ratio (matrix.width / matrix.height)
            let rect = if let Some(matrix) = &self.matrix_data {
                let data_aspect = (matrix.width as f32 / matrix.height as f32).max(0.01);
                let avail_w = available_rect.width();
                let avail_h = available_rect.height();
                let avail_aspect = avail_w / avail_h.max(1.0);

                let (plot_w, plot_h) = if avail_aspect > data_aspect {
                    (avail_h * data_aspect, avail_h)
                } else {
                    (avail_w, avail_w / data_aspect)
                };

                egui::Rect::from_center_size(available_rect.center(), egui::vec2(plot_w, plot_h))
            } else {
                available_rect
            };

            let (rect, _) = ui.allocate_exact_size(rect.size(), egui::Sense::drag());

            if let Some(renderer) = &self.renderer {
                let callback = eframe::egui_wgpu::Callback::new_paint_callback(
                    rect,
                    MatrixCallback {
                        renderer: renderer.clone(),
                        colormap: self.active_colormap,
                        rect,
                    },
                );

                ui.painter().add(callback);
            }
        });
    }
}


