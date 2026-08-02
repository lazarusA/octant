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
            prefetch_lookahead: 24, // 24-48 target slice prefetch range

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
        let (var_name, timestep, max_steps) = {
            if let Some(metadata) = &self.active_dataset_metadata {
                if let Some(var_info) = metadata.variables.get(self.selected_variable_idx) {
                    let max_steps = if var_info.shape.is_empty() {
                        1
                    } else {
                        var_info.shape[0] as usize
                    };
                    (var_info.name.clone(), self.current_timestep, max_steps)
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
            self.status_message = format!("🚀 Cache HIT for '{}' (step {})", slice.variable_name, timestep + 1);

            // Trigger background prefetching for upcoming timesteps
            let total_steps = max_steps.max(slice.max_timesteps);
            self.prefetcher.prefetch_ahead(
                self.selected_store_kind,
                &self.store_target_input,
                &var_name,
                timestep,
                total_steps,
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
            return;
        }

        // 2. Cache MISS: Fetch synchronously and put into LRU cache
        let store: Box<dyn DataStore> = match self.selected_store_kind {
            StoreKind::RemoteZarr => Box::new(ZarrRemoteStore::new(&self.store_target_input)),
            StoreKind::LocalZarr => Box::new(ZarrLocalStore::new(&self.store_target_input)),
            StoreKind::RemoteIcechunk => Box::new(IcechunkRemoteStore::new(&self.store_target_input)),
            StoreKind::LocalIcechunk => Box::new(IcechunkLocalStore::new(&self.store_target_input)),
            StoreKind::ProceduralRandom => unreachable!(),
        };

        match store.fetch_slice(&var_name, timestep) {
            Ok(slice) => {
                let mdata = MatrixData {
                    width: slice.width,
                    height: slice.height,
                    values: slice.values.clone(),
                    dataset_name: format!("{} ({})", slice.dataset_name, slice.variable_name),
                    max_timesteps: slice.max_timesteps,
                };
                self.rebuild_pipeline_with_matrix_data(mdata);
                self.status_message = format!("⚡ Fetched & Cached slice for '{}' (step {})", slice.variable_name, timestep + 1);

                let max_timesteps = slice.max_timesteps;
                self.lru_cache.put(cache_key, slice);

                // Trigger background prefetching for upcoming timesteps
                let total_steps = max_steps.max(max_timesteps);
                self.prefetcher.prefetch_ahead(
                    self.selected_store_kind,
                    &self.store_target_input,
                    &var_name,
                    timestep,
                    total_steps,
                    self.prefetch_lookahead,
                    &self.lru_cache,
                );
            }
            Err(err) => {
                self.status_message = format!("Slice fetch error: {}", err);
            }
        }
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
        let completed_prefetches = self.prefetcher.poll_results();
        for res in completed_prefetches {
            if let Ok(slice) = res.result {
                self.lru_cache.put(res.key, slice);
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

        // 3. Side Control Panel UI
        egui::SidePanel::left("octant_controls")
            .resizable(false)
            .default_width(330.0)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.heading("Octant Engine");
                ui.small("Multiscale Cloud & Local Tensor Visualizer");
                ui.separator();

                ui.label(egui::RichText::new("Data Store Provider:").strong());
                let old_store_kind = self.selected_store_kind;
                egui::ComboBox::from_id_salt("store_kind_select")
                    .selected_text(match self.selected_store_kind {
                        StoreKind::RemoteZarr => "🌐 Remote Zarr (HTTP/S3)",
                        StoreKind::LocalZarr => "📁 Local Zarr (FileSystem)",
                        StoreKind::RemoteIcechunk => "🧊 Remote Icechunk (HTTP/S3)",
                        StoreKind::LocalIcechunk => "🧊 Local Icechunk (FileSystem)",
                        StoreKind::ProceduralRandom => "🎲 Procedural Random Test",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.selected_store_kind, StoreKind::RemoteZarr, "🌐 Remote Zarr (HTTP/S3)");
                        ui.selectable_value(&mut self.selected_store_kind, StoreKind::LocalZarr, "📁 Local Zarr (FileSystem)");
                        ui.selectable_value(&mut self.selected_store_kind, StoreKind::RemoteIcechunk, "🧊 Remote Icechunk (HTTP/S3)");
                        ui.selectable_value(&mut self.selected_store_kind, StoreKind::LocalIcechunk, "🧊 Local Icechunk (FileSystem)");
                        ui.selectable_value(&mut self.selected_store_kind, StoreKind::ProceduralRandom, "🎲 Procedural Random Test");
                    });

                if old_store_kind != self.selected_store_kind {
                    match self.selected_store_kind {
                        StoreKind::RemoteZarr => {
                            self.store_target_input = "https://s3.bgc-jena.mpg.de:9000/esdl-esdc-v3.0.2/esdc-16d-2.5deg-46x72x1440-3.0.2.zarr".to_string();
                        }
                        StoreKind::LocalZarr => {
                            self.store_target_input = "./data/sample_dataset.zarr".to_string();
                        }
                        StoreKind::RemoteIcechunk => {
                            self.store_target_input = "https://s3.amazonaws.com/icechunk-demo/repository".to_string();
                        }
                        StoreKind::LocalIcechunk => {
                            self.store_target_input = "./data/icechunk_repo".to_string();
                        }
                        StoreKind::ProceduralRandom => {
                            self.store_target_input = "procedural://random".to_string();
                        }
                    }
                    self.inspect_active_store();
                }

                ui.add_space(6.0);
                ui.label(egui::RichText::new("Store Path / URL Target:").strong());
                ui.text_edit_singleline(&mut self.store_target_input);
                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    if ui.button("🔍 Inspect Store Metadata").clicked() {
                        self.inspect_active_store();
                    }
                });
                ui.add_space(8.0);

                // Variable Selection & Dynamic Metadata Inspection
                let mut var_changed = false;
                ui.group(|ui| {
                    ui.label(egui::RichText::new("Extracted Dataset Metadata").strong());
                    if let Some(metadata) = &self.active_dataset_metadata {
                        ui.label(format!("Provider: {}", metadata.store_type));
                        ui.label(format!("Store: {}", metadata.name));
                        ui.label(format!("Variables Found: {}", metadata.variables.len()));

                        if !metadata.variables.is_empty() {
                            ui.add_space(4.0);
                            ui.label(egui::RichText::new("Select Active Variable:").strong());
                            let old_var_idx = self.selected_variable_idx;

                            let current_var_name = metadata
                                .variables
                                .get(self.selected_variable_idx)
                                .map(|v| v.name.as_str())
                                .unwrap_or("Select Variable");

                            egui::ComboBox::from_id_salt("var_extracted_select")
                                .selected_text(current_var_name)
                                .show_ui(ui, |ui| {
                                    for (idx, var_info) in metadata.variables.iter().enumerate() {
                                        ui.selectable_value(&mut self.selected_variable_idx, idx, &var_info.name);
                                    }
                                });

                            if old_var_idx != self.selected_variable_idx {
                                var_changed = true;
                            }

                            if let Some(var_info) = metadata.variables.get(self.selected_variable_idx) {
                                ui.add_space(4.0);
                                ui.small(format!("DType: {}", var_info.data_type));
                                ui.small(format!("Shape: {:?}", var_info.shape));
                                ui.small(format!("Dimensions: {:?}", var_info.dimension_names));
                                ui.small(format!("Chunks: {:?}", var_info.chunk_shape));
                                let size_mb = var_info.file_size as f64 / (1024.0 * 1024.0);
                                ui.small(format!("File Size: {} bytes ({:.2} MB)", var_info.file_size, size_mb));
                            }
                        }
                    } else {
                        ui.label("No store metadata inspected.");
                    }
                    ui.add_space(4.0);
                    ui.small(&self.status_message);
                });

                if var_changed {
                    self.load_selected_variable_slice();
                }
                ui.add_space(8.0);

                // Colormap selection
                ui.label(egui::RichText::new("GPU Colormap Routine:").strong());
                egui::ComboBox::from_id_salt("cmap_select")
                    .selected_text(match self.active_colormap {
                        0 => "Viridis (Thermal)",
                        1 => "Plasma (Spectral)",
                        2 => "Inferno (Radiance)",
                        _ => "Magma",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.active_colormap, 0, "Viridis (Thermal)");
                        ui.selectable_value(&mut self.active_colormap, 1, "Plasma (Spectral)");
                        ui.selectable_value(&mut self.active_colormap, 2, "Inferno (Radiance)");
                        ui.selectable_value(&mut self.active_colormap, 3, "Magma");
                    });
                ui.add_space(10.0);

                // Playback & Animation Control Group
                let max_steps = self.matrix_data.as_ref().map(|h| h.max_timesteps).unwrap_or(1);
                ui.group(|ui| {
                    ui.label(egui::RichText::new("🎬 Animation & Playback Controls").strong());
                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        let play_btn_text = if self.is_playing { "⏸ Pause" } else { "▶ Play" };
                        if ui.button(egui::RichText::new(play_btn_text).strong()).clicked() {
                            self.is_playing = !self.is_playing;
                            self.last_step_time = std::time::Instant::now();
                        }

                        if ui.button("◀ Prev").clicked() {
                            if self.current_timestep > 0 {
                                self.current_timestep -= 1;
                            } else if max_steps > 0 {
                                self.current_timestep = max_steps - 1;
                            }
                            self.load_selected_variable_slice();
                        }

                        if ui.button("Next ▶").clicked() {
                            if max_steps > 0 {
                                self.current_timestep = (self.current_timestep + 1) % max_steps;
                            }
                            self.load_selected_variable_slice();
                        }

                        ui.checkbox(&mut self.loop_playback, "🔄 Loop");
                    });

                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label("Playback Speed:");
                        ui.add(egui::Slider::new(&mut self.playback_fps, 1.0..=60.0).suffix(" FPS"));
                    });

                    ui.add_space(4.0);
                    let slider_max = max_steps.saturating_sub(1);
                    ui.label(format!("Timestep Index: {} / {}", self.current_timestep + 1, max_steps));
                    let slider_res = ui.add(
                        egui::Slider::new(&mut self.current_timestep, 0..=slider_max)
                            .text("Time Slice"),
                    );
                    if slider_res.drag_stopped() || slider_res.changed() && !self.is_playing {
                        self.load_selected_variable_slice();
                    }
                });
                ui.add_space(10.0);

                // LRU Cache & Prefetcher Statistics Group
                ui.group(|ui| {
                    ui.label(egui::RichText::new("🧠 LRU Slice Cache & Prefetcher").strong());
                    ui.add_space(4.0);

                    let current_bytes = self.lru_cache.current_bytes();
                    let current_mb = current_bytes as f64 / (1024.0 * 1024.0);
                    let max_bytes = self.lru_cache.max_bytes();
                    let fraction = (current_bytes as f32 / max_bytes as f32).clamp(0.0, 1.0);

                    ui.label(egui::RichText::new("Memory Usage (1GB Default Limit):").small());
                    ui.add(
                        egui::ProgressBar::new(fraction).text(format!(
                            "{:.2} MB / {} MB ({:.1}%)",
                            current_mb,
                            self.max_cache_mb,
                            fraction * 100.0
                        )),
                    );

                    ui.add_space(4.0);
                    ui.small(format!(
                        "Cached Slices: {} | Target Lookahead: {} slices",
                        self.lru_cache.cached_count(),
                        self.prefetch_lookahead
                    ));

                    ui.small(format!(
                        "Hits: {} | Misses: {} (Hit Rate: {:.1}%)",
                        self.lru_cache.hits(),
                        self.lru_cache.misses(),
                        self.lru_cache.hit_rate()
                    ));

                    let pending = self.prefetcher.pending_count();
                    if pending > 0 {
                        ui.small(format!("🟢 Background Prefetching: {} slices queued", pending));
                    } else {
                        ui.small("⚪ Buffer Warm / Prefetch Idle");
                    }

                    ui.add_space(6.0);
                    ui.collapsing("⚙ Cache Settings", |ui| {
                        let old_mb = self.max_cache_mb;
                        ui.add(
                            egui::Slider::new(&mut self.max_cache_mb, 256..=4096)
                                .text("Max Capacity (MB)"),
                        );
                        if old_mb != self.max_cache_mb {
                            self.lru_cache.set_max_bytes(self.max_cache_mb * 1024 * 1024);
                        }

                        ui.add(
                            egui::Slider::new(&mut self.prefetch_lookahead, 12..=48)
                                .text("Prefetch Lookahead Slices"),
                        );

                        if ui.button("🗑 Flush & Clear Cache").clicked() {
                            self.lru_cache.clear();
                        }
                    });
                });
            });

        // 4. Interactive Heatmap Matrix Canvas Area
        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = ui.available_rect_before_wrap();
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

            // Overlay Heatmap & Playback Information Card
            if let Some(matrix) = &self.matrix_data {
                let overlay_rect = egui::Rect::from_min_size(
                    rect.min + egui::vec2(16.0, 16.0),
                    egui::vec2(380.0, 80.0),
                );
                ui.put(overlay_rect, |ui: &mut egui::Ui| {
                    egui::Frame::popup(ui.style())
                        .fill(egui::Color32::from_black_alpha(210))
                        .rounding(6.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(&matrix.dataset_name)
                                        .strong()
                                        .color(egui::Color32::WHITE),
                                );
                                if self.is_playing {
                                    ui.label(
                                        egui::RichText::new("[▶ PLAYING]")
                                            .small()
                                            .color(egui::Color32::GREEN),
                                    );
                                } else {
                                    ui.label(
                                        egui::RichText::new("[⏸ PAUSED]")
                                            .small()
                                            .color(egui::Color32::LIGHT_GRAY),
                                    );
                                }
                            });

                            ui.small(format!(
                                "Grid: {}x{} cells | Colormap: {} | Timestep: {}/{}",
                                matrix.width,
                                matrix.height,
                                match self.active_colormap {
                                    0 => "Viridis",
                                    1 => "Plasma",
                                    2 => "Inferno",
                                    _ => "Magma",
                                },
                                self.current_timestep + 1,
                                matrix.max_timesteps
                            ));

                            ui.small(format!(
                                "LRU Cache: {} slices ({:.1} MB) | Hit Rate: {:.1}%",
                                self.lru_cache.cached_count(),
                                self.lru_cache.current_bytes() as f64 / (1024.0 * 1024.0),
                                self.lru_cache.hit_rate()
                            ));
                        })
                        .response
                });
            }
        });
    }
}
