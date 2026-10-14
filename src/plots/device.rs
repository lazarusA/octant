//! GPU device setup shared by the native and web runners.

use eframe::egui_wgpu::{WgpuConfiguration, WgpuSetup};
use std::sync::Arc;

/// Features renderers use when the adapter offers them (the volume raymarcher
/// filters `R32Float` in hardware with `FLOAT32_FILTERABLE`).
const OPTIONAL_FEATURES: wgpu::Features = wgpu::Features::FLOAT32_FILTERABLE;

/// egui's default wgpu setup, also requesting the [`OPTIONAL_FEATURES`] the
/// adapter supports, its full 3D texture size and its storage buffer sizes.
pub fn wgpu_configuration() -> WgpuConfiguration {
    let mut config = WgpuConfiguration::default();
    if let WgpuSetup::CreateNew(setup) = &mut config.wgpu_setup {
        let base = Arc::clone(&setup.device_descriptor);
        setup.device_descriptor = Arc::new(move |adapter| {
            let mut descriptor = base(adapter);
            descriptor.required_features |= adapter.features() & OPTIONAL_FEATURES;
            let adapter_limits = adapter.limits();
            descriptor.required_limits.max_texture_dimension_3d =
                adapter_limits.max_texture_dimension_3d;
            // Large images upload whole instead of through a pyramid.
            descriptor.required_limits =
                super::limits::raised(descriptor.required_limits.clone(), &adapter_limits);
            descriptor
        });
    }
    config
}
