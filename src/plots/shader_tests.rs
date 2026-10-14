//! Every plot pipeline builds and every plot shader parses and validates.

/// Builds the 3D renderers (all their pipelines, including the transparent
/// ones) and fails on any wgpu validation error. Skipped without a GPU.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn mesh_and_point_cloud_pipelines_validate() {
    let (Some((device, _queue)), Some(rt)) = (
        super::test_gpu::device(),
        tokio::runtime::Runtime::new().ok(),
    ) else {
        eprintln!("SKIPPED mesh_and_point_cloud_pipelines_validate: no GPU device");
        return;
    };
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let data = [0.0f32; 4];
    let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let surface = super::SurfaceRenderer::new_surface(&device, format, &data, 2, 2);
    let sphere = super::SphereRenderer::new_sphere(&device, format, &data, 2, 2);
    let points = super::PointCloudRenderer::new(&device, format, &data, 2, 2);
    // OIT pipelines are built on first use; build them here too.
    for mesh in [&surface, &sphere] {
        let _oit = super::oit::OitState::new(&device, format, |variant| {
            mesh.oit_pipeline(&device, variant)
        });
    }
    let _oit = super::oit::OitState::new(&device, format, |variant| {
        points.oit_pipeline(&device, variant)
    });
    if let Some(error) = rt.block_on(scope.pop()) {
        panic!("pipeline validation failed: {error}");
    }
}

#[test]
fn test_all_plot_shaders_parse_cleanly() {
    let shaders = [
        (
            "sphere",
            crate::assemble_plot_with_coords_shader!(include_str!("shaders/sphere.wgsl")),
        ),
        (
            "surface",
            crate::assemble_plot_with_coords_shader!(include_str!("shaders/surface.wgsl")),
        ),
        ("heatmap", super::heatmap::HEATMAP_WGSL),
        ("labels", super::labels::LABELS_WGSL),
        (
            "volume_hardware_filter",
            crate::plots::volume::pipeline::SHADER_HARDWARE_FILTER,
        ),
        (
            "volume_manual_filter",
            crate::plots::volume::pipeline::SHADER_MANUAL_FILTER,
        ),
        (
            "volume_blit",
            concat!(
                include_str!("shaders/common/fullscreen.wgsl"),
                "\n",
                include_str!("shaders/volume/blit.wgsl")
            ),
        ),
        (
            "oit_composite",
            concat!(
                include_str!("shaders/common/fullscreen.wgsl"),
                "\n",
                include_str!("shaders/oit/composite.wgsl")
            ),
        ),
        (
            "line",
            crate::assemble_plot_shader!(include_str!("shaders/line.wgsl")),
        ),
        (
            "point_cloud",
            crate::assemble_plot_shader!(include_str!("shaders/point_cloud.wgsl")),
        ),
        ("coastline", include_str!("shaders/coastline.wgsl")),
        (
            "coastline_3d",
            concat!(
                include_str!("shaders/common/coords.wgsl"),
                "\n",
                include_str!("shaders/common/camera3d.wgsl"),
                "\n",
                include_str!("shaders/common/healpix_scheme.wgsl"),
                "\n",
                include_str!("shaders/common/healpix_math.wgsl"),
                "\n",
                include_str!("shaders/common/projections.wgsl"),
                "\n",
                include_str!("shaders/common/height.wgsl"),
                "\n",
                include_str!("shaders/coastline_3d.wgsl"),
            ),
        ),
    ];

    for (name, source) in shaders {
        let mut validator = wgpu::naga::valid::Validator::new(
            wgpu::naga::valid::ValidationFlags::all(),
            wgpu::naga::valid::Capabilities::all(),
        );
        let module = wgpu::naga::front::wgsl::parse_str(source)
            .unwrap_or_else(|e| panic!("Failed to parse WGSL shader '{name}': {e}"));
        validator
            .validate(&module)
            .unwrap_or_else(|e| panic!("Failed to validate WGSL shader '{name}': {e}"));
    }
}
