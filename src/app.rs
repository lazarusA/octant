use crate::matrix_data::MatrixData;
use crate::renderer::{MatrixCallback, MatrixRenderer};
use std::sync::Arc;

pub struct OctantApp {
    pub cloud_store_url: String,
    pub selected_variable: String,
    pub current_timestep: usize,
    pub active_colormap: u32,
    pub status_message: String,
    pub is_loading: bool,
    pub matrix_data: Option<MatrixData>,
    pub renderer: Option<Arc<MatrixRenderer>>,
    pub wgpu_render_state: Option<eframe::egui_wgpu::RenderState>,
}

impl OctantApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let wgpu_render_state = cc.wgpu_render_state.clone();

        let initial_data = MatrixData::create_random_matrix(64, 64);
        let mut app = Self {
            cloud_store_url: "https://s3.bgc-jena.mpg.de:9000/esdl-esdc-v3.0.2/esdc-16d-2.5deg-46x72x1440-3.0.2.zarr".to_string(),
            selected_variable: "random_matrix".to_string(),
            current_timestep: 0,
            active_colormap: 0,
            status_message: "Loaded 64x64 test matrix.".to_string(),
            is_loading: false,
            matrix_data: None,
            renderer: None,
            wgpu_render_state,
        };

        if let Ok(data) = initial_data {
            app.rebuild_pipeline_with_data(data);
        }

        app
    }

    pub fn rebuild_pipeline_with_data(&mut self, data: MatrixData) {
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

    pub fn load_remote_esdc(&mut self) {
        self.is_loading = true;
        self.status_message = "Fetching remote Zarr array metadata over HTTP...".to_string();

        let url = self.cloud_store_url.clone();
        let timestep = self.current_timestep;

        match MatrixData::fetch_remote_esdc_temperature(&url, timestep) {
            Ok(data) => {
                self.status_message = format!("Loaded dataset '{}' ({} x {})", data.dataset_name, data.width, data.height);
                self.rebuild_pipeline_with_data(data);
            }
            Err(err) => {
                self.status_message = format!("Error loading remote Zarr: {}", err);
            }
        }
        self.is_loading = false;
    }
}

impl eframe::App for OctantApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        // 1. Draw Side Control Panel
        egui::SidePanel::left("octant_controls")
            .resizable(false)
            .default_width(310.0)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.heading("Octant Engine");
                ui.small("Multiscale Cloud Tensor Visualizer");
                ui.separator();

                ui.label(egui::RichText::new("Cloud Data Store Target:").strong());
                ui.text_edit_singleline(&mut self.cloud_store_url);
                ui.add_space(4.0);

                ui.label(egui::RichText::new("Active Variable:").strong());
                let old_var = self.selected_variable.clone();
                egui::ComboBox::from_id_salt("var_select")
                    .selected_text(&self.selected_variable)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.selected_variable,
                            "random_matrix".to_string(),
                            "Random Test Matrix (64x64)",
                        );
                        ui.selectable_value(
                            &mut self.selected_variable,
                            "air_temperature_2m".to_string(),
                            "air_temperature_2m (ESDC v3.0.2)",
                        );
                        ui.selectable_value(
                            &mut self.selected_variable,
                            "sample_heatmap".to_string(),
                            "Sample 2D Gaussian Matrix (32x32)",
                        );
                    });

                if old_var != self.selected_variable {
                    match self.selected_variable.as_str() {
                        "random_matrix" => {
                            if let Ok(data) = MatrixData::create_random_matrix(64, 64) {
                                self.rebuild_pipeline_with_data(data);
                                self.status_message = "Loaded 64x64 random test matrix.".to_string();
                            }
                        }
                        "sample_heatmap" => {
                            if let Ok(data) = MatrixData::create_sample_heatmap(32, 32) {
                                self.rebuild_pipeline_with_data(data);
                                self.status_message = "Loaded Gaussian matrix (32x32).".to_string();
                            }
                        }
                        "air_temperature_2m" => {
                            self.load_remote_esdc();
                        }
                        _ => {}
                    }
                }
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    if ui.button("🎲 Test Random Matrix (64x64)").clicked() {
                        self.selected_variable = "random_matrix".to_string();
                        if let Ok(data) = MatrixData::create_random_matrix(64, 64) {
                            self.rebuild_pipeline_with_data(data);
                            self.status_message = "Loaded new 64x64 random test matrix.".to_string();
                        }
                    }
                    if ui.button("📥 Fetch Remote ESDC").clicked() {
                        self.selected_variable = "air_temperature_2m".to_string();
                        self.load_remote_esdc();
                    }
                });

                ui.add_space(8.0);

                ui.group(|ui| {
                    ui.label(egui::RichText::new("Zarr Array Storage Info").strong());
                    if let Some(matrix) = &self.matrix_data {
                        ui.label(format!("Dataset: {}", matrix.dataset_name));
                        ui.label(format!("Tensor Shape: {:?}", matrix.shape));
                        ui.label(format!("Active Slice: Step {} / {}", matrix.current_timestep + 1, matrix.max_timesteps));
                        ui.label(format!("Grid Resolution: {} (Lon) x {} (Lat)", matrix.width, matrix.height));
                        ui.label(format!("Slice Elements: {}", matrix.values.len()));
                    } else {
                        ui.label("No matrix dataset loaded.");
                    }
                    ui.add_space(4.0);
                    ui.small(&self.status_message);
                });
                ui.add_space(8.0);

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

                let max_steps = self.matrix_data.as_ref().map(|h| h.max_timesteps.saturating_sub(1)).unwrap_or(365);
                ui.label(egui::RichText::new("Dimension Coordinate Slices:").strong());

                let slider_res = ui.add(egui::Slider::new(&mut self.current_timestep, 0..=max_steps).text("Time Step"));
                if slider_res.drag_stopped() && self.selected_variable == "air_temperature_2m" {
                    self.load_remote_esdc();
                }
            });

        // 2. Interactive Heatmap Matrix Canvas Area
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

            // Overlay Heatmap Information Banner
            if let Some(matrix) = &self.matrix_data {
                let overlay_rect = egui::Rect::from_min_size(
                    rect.min + egui::vec2(16.0, 16.0),
                    egui::vec2(320.0, 60.0),
                );
                ui.put(
                    overlay_rect,
                    |ui: &mut egui::Ui| {
                        egui::Frame::popup(ui.style())
                            .fill(egui::Color32::from_black_alpha(200))
                            .rounding(6.0)
                            .show(ui, |ui| {
                                ui.label(egui::RichText::new(&matrix.dataset_name).strong().color(egui::Color32::WHITE));
                                ui.small(format!(
                                    "Grid: {}x{} cells | Colormap: {}",
                                    matrix.width,
                                    matrix.height,
                                    match self.active_colormap {
                                        0 => "Viridis",
                                        1 => "Plasma",
                                        2 => "Inferno",
                                        _ => "Magma",
                                    }
                                ));
                            }).response
                    },
                );
            }
        });
    }
}
