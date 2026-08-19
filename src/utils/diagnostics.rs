/// GPU and Renderer Diagnostic Utilities
use eframe::egui_wgpu::RenderState;

/// Logs GPU adapter info and device limits for debugging backend and memory constraints.
pub fn log_gpu_diagnostics(wgpu_state: &RenderState) {
    let info = wgpu_state.adapter.get_info();
    log::info!(
        "🎮 GPU initialized: '{}' ({:?}, backend: {:?}, driver: '{}')",
        info.name,
        info.device_type,
        info.backend,
        info.driver_info
    );

    let limits = wgpu_state.device.limits();
    let max_buf_mb = limits.max_buffer_size / (1024 * 1024);
    let max_storage_mb = limits.max_storage_buffer_binding_size / (1024 * 1024);
    let max_tex_2d = limits.max_texture_dimension_2d;

    log::info!(
        "📊 WGPU device limits: max_buffer_size = {} MB, max_storage_buffer = {} MB, max_texture_2d = {}px",
        max_buf_mb,
        max_storage_mb,
        max_tex_2d
    );
}

/// Logs 2D pyramid LOD sub-tile upload diagnostics.
pub fn log_lod_tile_upload(
    width: usize,
    height: usize,
    num_cells: usize,
    size_mb: f64,
    tile_bounds: [f32; 4],
) {
    log::debug!(
        "🖼️ [Heatmap] Uploading LOD tile: {}x{} ({} cells, {:.2} MB), tile_bounds: [{:.3}, {:.3}, {:.3}, {:.3}]",
        width,
        height,
        num_cells,
        size_mb,
        tile_bounds[0],
        tile_bounds[1],
        tile_bounds[2],
        tile_bounds[3]
    );
}
