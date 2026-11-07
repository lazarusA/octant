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

/// Every plot shader, by name.
const SHADERS: [(&str, &str); 16] = [
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
    ("channels", super::channels::CHANNELS_WGSL),
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
        "frame_cache_blit",
        concat!(
            include_str!("shaders/common/fullscreen.wgsl"),
            "\n",
            include_str!("shaders/frame_cache_blit.wgsl")
        ),
    ),
    ("channel_bake", super::channel_bake::SHADER),
    ("channel_bake_volume", super::channel_bake_volume::SHADER),
    (
        "line",
        crate::assemble_plot_shader!(include_str!("shaders/line.wgsl")),
    ),
    (
        "point_cloud",
        crate::assemble_plot_shader!(include_str!("shaders/point_cloud.wgsl")),
    ),
    ("coastline", super::coastline::renderer_2d::COASTLINE_WGSL),
    (
        "coastline_3d",
        concat!(
            include_str!("shaders/common/coords.wgsl"),
            "\n",
            include_str!("shaders/common/camera3d.wgsl"),
            "\n",
            include_str!("shaders/common/slice.wgsl"),
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

#[test]
fn test_all_plot_shaders_parse_cleanly() {
    for (name, source) in SHADERS {
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

/// Size of WGSL struct `name` in shader `shader` of `SHADERS`.
fn wgsl_size(shader: &str, name: &str) -> usize {
    use wgpu::naga;
    let Some((_, source)) = SHADERS.iter().find(|(n, _)| *n == shader) else {
        panic!("no shader {shader}");
    };
    let module = naga::front::wgsl::parse_str(source).expect("shader parses");
    let mut layouter = naga::proc::Layouter::default();
    layouter.update(module.to_ctx()).expect("layout");
    let Some((handle, _)) = module
        .types
        .iter()
        .find(|(_, ty)| ty.name.as_deref() == Some(name))
    else {
        panic!("no struct {name} in {shader}");
    };
    layouter[handle].size as usize
}

/// The 3D uniform blocks match their WGSL structs.
#[test]
fn uniform_layouts_match_wgsl() {
    use std::mem::size_of;
    let blocks = [
        (
            "surface",
            "Camera",
            size_of::<super::camera::CameraUniform>(),
        ),
        ("surface", "Uniforms", size_of::<super::Mesh3DUniforms>()),
        ("sphere", "Uniforms", size_of::<super::Mesh3DUniforms>()),
        (
            "point_cloud",
            "Uniforms",
            size_of::<super::point_cloud_types::PointCloudUniforms>(),
        ),
        (
            "coastline",
            "CoastlineUniforms",
            size_of::<super::coastline::types::CoastlineUniforms>(),
        ),
        (
            "coastline_3d",
            "Coastline3DUniforms",
            size_of::<super::coastline::types::Coastline3DUniforms>(),
        ),
        (
            "channels",
            "Channel",
            size_of::<super::channel_params::ChannelGpu>(),
        ),
        (
            "channels",
            "ChannelParams",
            size_of::<super::ChannelParams>(),
        ),
    ];
    for (shader, name, size) in blocks {
        assert_eq!(wgsl_size(shader, name), size, "{shader} {name}");
    }
}

/// Every entry point of every plot shader compiles for wgpu's GL backend
/// (GLSL ES 3.10, the first with storage buffers and compute), which
/// eframe falls back to without Vulkan or Metal: GLSL lacks some WGSL
/// (`textureLoad` of a depth texture), which only this would show.
#[test]
fn plot_shaders_compile_for_gl() {
    use naga::back::glsl;
    let options = glsl::Options {
        version: glsl::Version::Embedded {
            version: 310,
            is_webgl: false,
        },
        ..Default::default()
    };
    for (name, source) in SHADERS {
        let module = naga::front::wgsl::parse_str(source).expect("shader parses");
        let info = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .expect("shader validates");
        // Overrides take their defaults, as the volume's `ALGORITHM` does.
        let constants = naga::back::PipelineConstants::default();
        let (module, info) =
            naga::back::pipeline_constants::process_overrides(&module, &info, None, &constants)
                .expect("overrides resolve");
        for ep in &module.entry_points {
            let pipeline = glsl::PipelineOptions {
                shader_stage: ep.stage,
                entry_point: ep.name.clone(),
                multiview: None,
            };
            let mut out = String::new();
            let written = glsl::Writer::new(
                &mut out,
                &module,
                &info,
                &options,
                &pipeline,
                naga::proc::BoundsCheckPolicies::default(),
            )
            .and_then(|mut writer| writer.write().map(|_| ()));
            if let Err(e) = written {
                panic!("'{name}' {} does not compile for GL: {e}", ep.name);
            }
        }
    }
}

/// No shader bitcasts a literal to a float: WGSL forbids constants that
/// evaluate to NaN or infinity, which naga accepts but Safari hangs compiling
/// (`channel_blend.wgsl` `channel_nan` loads its bits from a `var` instead).
#[test]
fn no_float_bitcast_of_literals() {
    fn walk(dir: &std::path::Path, out: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for path in entries.flatten().map(|e| e.path()) {
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().is_some_and(|e| e == "wgsl")
                && let Ok(text) = std::fs::read_to_string(&path)
            {
                for (n, line) in text.lines().enumerate() {
                    let rest = line.split("bitcast<f32>(").skip(1);
                    if rest
                        .clone()
                        .any(|r| r.starts_with(|c: char| c.is_ascii_digit()))
                    {
                        out.push(format!("{}:{}", path.display(), n + 1));
                    }
                }
            }
        }
    }
    let mut found = Vec::new();
    walk(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/plots/shaders"),
        &mut found,
    );
    assert!(found.is_empty(), "float bitcasts of literals: {found:?}");
}
