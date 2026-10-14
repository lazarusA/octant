//! How much data one GPU storage buffer holds: the device's own limit, read
//! once when the app starts (`set_from_device`; `device.rs` requests the
//! adapter's), WebGPU's 128 MiB default until then (tests, no GPU). In the
//! browser it is capped by a memory budget, since WebAssembly memory (4 GB at
//! most) also holds the block cache and the layers' data. It decides whether
//! a 2D heatmap uploads whole or through a pyramid; volumes, point clouds and
//! meshes, built several at once, keep `common::MAX_GPU_STORAGE_BUFFER_BYTES`.

use std::sync::atomic::{AtomicU64, Ordering};

/// WebGPU's default for one storage binding.
const DEFAULT_STORAGE_BYTES: u64 = 128 * 1024 * 1024;

/// The most data one layer uploads whole in the browser.
#[cfg(target_arch = "wasm32")]
const WEB_BUDGET_BYTES: u64 = 512 * 1024 * 1024;

/// Shaders index values with `u32`.
const INDEX_LIMIT_BYTES: u64 = u32::MAX as u64 * 4;

static STORAGE_BYTES: AtomicU64 = AtomicU64::new(DEFAULT_STORAGE_BYTES);

/// Records the device's limits (`storage_bytes_of`).
pub fn set_from_device(limits: &wgpu::Limits) {
    STORAGE_BYTES.store(storage_bytes_of(limits), Ordering::Relaxed);
}

/// The bytes one storage buffer of a device with `limits` holds: one
/// storage binding, within one buffer.
pub fn storage_bytes_of(limits: &wgpu::Limits) -> u64 {
    limits
        .max_storage_buffer_binding_size
        .min(limits.max_buffer_size)
}

/// The bytes one storage buffer holds (in the browser, within its budget).
pub fn max_storage_bytes() -> usize {
    let bytes = STORAGE_BYTES.load(Ordering::Relaxed).min(INDEX_LIMIT_BYTES);
    #[cfg(target_arch = "wasm32")]
    let bytes = bytes.min(WEB_BUDGET_BYTES);
    usize::try_from(bytes).unwrap_or(usize::MAX)
}

/// The `f32` values one storage buffer holds.
pub fn max_storage_elements() -> usize {
    max_storage_bytes() / std::mem::size_of::<f32>()
}

/// [`max_storage_bytes`] in whole MiB, for messages.
pub fn max_storage_mb() -> usize {
    max_storage_bytes() / (1024 * 1024)
}

/// Device limits asking for the adapter's storage binding and buffer sizes
/// where they exceed `limits` (never lowering them).
pub fn raised(mut limits: wgpu::Limits, adapter: &wgpu::Limits) -> wgpu::Limits {
    limits.max_storage_buffer_binding_size = limits
        .max_storage_buffer_binding_size
        .max(adapter.max_storage_buffer_binding_size);
    limits.max_buffer_size = limits.max_buffer_size.max(adapter.max_buffer_size);
    limits
}
