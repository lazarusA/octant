//! The uniform blocks the plots upload match their WGSL structs.

use super::shader_tests::SHADERS;

/// Size of WGSL struct `name` in shader `shader` of `SHADERS`, and its
/// members' names and offsets.
fn wgsl_layout(shader: &str, name: &str) -> (usize, Vec<(String, usize)>) {
    use wgpu::naga;
    let Some((_, source)) = SHADERS.iter().find(|(n, _)| *n == shader) else {
        panic!("no shader {shader}");
    };
    let module = naga::front::wgsl::parse_str(source).expect("shader parses");
    let mut layouter = naga::proc::Layouter::default();
    layouter.update(module.to_ctx()).expect("layout");
    let Some((handle, ty)) = module
        .types
        .iter()
        .find(|(_, ty)| ty.name.as_deref() == Some(name))
    else {
        panic!("no struct {name} in {shader}");
    };
    let members = match &ty.inner {
        naga::TypeInner::Struct { members, .. } => members
            .iter()
            .map(|m| (m.name.clone().unwrap_or_default(), m.offset as usize))
            .collect(),
        _ => Vec::new(),
    };
    (layouter[handle].size as usize, members)
}

fn wgsl_size(shader: &str, name: &str) -> usize {
    wgsl_layout(shader, name).0
}

/// The heightfield uniforms match field by field, not only in size: two
/// swapped fields of the same size would pass a size check.
#[test]
fn mesh3d_uniform_offsets_match_wgsl() {
    use super::Mesh3DUniforms as U;
    use std::mem::offset_of;
    let rust = [
        ("camera", offset_of!(U, camera)),
        ("slice", offset_of!(U, slice)),
        (
            "displacement_strength",
            offset_of!(U, displacement_strength),
        ),
        ("mode", offset_of!(U, mode)),
        ("width", offset_of!(U, width)),
        ("height", offset_of!(U, height)),
        ("coord_mode", offset_of!(U, coord_mode)),
        ("flip_height", offset_of!(U, flip_height)),
        ("lon_bounds", offset_of!(U, lon_bounds)),
        ("lat_bounds", offset_of!(U, lat_bounds)),
        ("_pad", offset_of!(U, _pad)),
        ("color", offset_of!(U, color)),
    ]
    .map(|(n, o)| (n.to_string(), o));
    for shader in ["surface", "sphere"] {
        let (_, wgsl) = wgsl_layout(shader, "Uniforms");
        assert_eq!(wgsl, rust, "{shader} Uniforms");
    }
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
