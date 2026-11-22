//! A bricked volume's textures and buffers (`bricks.rs`): creating them and
//! writing a box of texels.

/// A 3D texture of `size` texels of `format`, sampled and written to.
pub(super) fn texture_3d(
    device: &wgpu::Device,
    label: &str,
    [width, height, depth]: [u32; 3],
    format: wgpu::TextureFormat,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: depth,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D3,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

/// A buffer of `size` bytes for `usage`.
pub(super) fn buffer(
    device: &wgpu::Device,
    label: &str,
    size: u64,
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage,
        mapped_at_creation: false,
    })
}

/// Writes `bytes` (`texel` bytes each, x fastest) over the box `origin`,
/// `size` of `texture`.
pub(super) fn write_box(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    origin: [u32; 3],
    size: [u32; 3],
    texel: u32,
    bytes: &[u8],
) {
    if size.contains(&0) {
        return;
    }
    // A short or long write is a wgpu validation error: logged and skipped.
    let expected = size
        .iter()
        .try_fold(u64::from(texel), |a, &n| a.checked_mul(u64::from(n)));
    if expected != Some(bytes.len() as u64) {
        log::warn!(
            "Brick write of {} bytes skipped: the box {size:?} takes {expected:?}",
            bytes.len()
        );
        return;
    }
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d {
                x: origin[0],
                y: origin[1],
                z: origin[2],
            },
            aspect: wgpu::TextureAspect::All,
        },
        bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(size[0] * texel),
            rows_per_image: Some(size[1]),
        },
        wgpu::Extent3d {
            width: size[0],
            height: size[1],
            depth_or_array_layers: size[2],
        },
    );
}
