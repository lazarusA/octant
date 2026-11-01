// A multi-channel composite blended once into the raymarcher's textures
// (`plots/channel_bake_volume.rs`), as `volume/encode.rs` `encode_rgba`
// encodes a CPU-blended one: each voxel's color with its brightest channel
// as alpha, a missing voxel (no drawn channel valid) filled with the average
// of its valid 26 neighbors (zeros without any) so trilinear filtering stays
// smooth, and its validity (255 drawn, 0 missing).

struct VolumeBake {
    w: u32,
    h: u32,
    d: u32,
    // `u32` words per validity row (4 voxels each, rows padded to 256 bytes
    // for the copy into the `R8Unorm` texture, which cannot be a storage one).
    row_words: u32,
};

@group(0) @binding(0)
var<uniform> vb: VolumeBake;

@group(0) @binding(1)
var texels: texture_storage_3d<rgba8unorm, write>;

@group(0) @binding(2)
var<storage, read_write> validity_words: array<u32>;

// Voxel `(x, y, z)`'s color in 0..255 with its brightest channel as alpha;
// alpha -1 when missing.
fn voxel(x: u32, y: u32, z: u32) -> vec4<f32> {
    let c = blend_cell((z * vb.h + y) * vb.w + x);
    if (c.a == 0.0) {
        return vec4<f32>(-1.0);
    }
    let rgb = round(c.rgb * 255.0);
    return vec4<f32>(rgb, max(rgb.r, max(rgb.g, rgb.b)));
}

// `Neighborhood::average`: the valid voxels among the 26 neighbors.
fn neighbor_average(x: u32, y: u32, z: u32) -> vec4<f32> {
    var sum = vec4<f32>(0.0);
    var count = 0.0;
    for (var nz = max(z, 1u) - 1u; nz < min(z + 2u, vb.d); nz = nz + 1u) {
        for (var ny = max(y, 1u) - 1u; ny < min(y + 2u, vb.h); ny = ny + 1u) {
            for (var nx = max(x, 1u) - 1u; nx < min(x + 2u, vb.w); nx = nx + 1u) {
                let c = voxel(nx, ny, nz);
                if (c.a >= 0.0) {
                    sum = sum + c;
                    count = count + 1.0;
                }
            }
        }
    }
    return select(sum, sum / count, count > 0.0);
}

// Four voxels along x per invocation: one validity word.
@compute @workgroup_size(8, 8, 1)
fn bake_volume(@builtin(global_invocation_id) id: vec3<u32>) {
    let x0 = id.x * 4u;
    let y = id.y;
    let z = id.z;
    if (x0 >= vb.w || y >= vb.h || z >= vb.d) {
        return;
    }
    var word = 0u;
    for (var k = 0u; k < 4u; k = k + 1u) {
        let x = x0 + k;
        if (x >= vb.w) {
            break;
        }
        var c = voxel(x, y, z);
        if (c.a >= 0.0) {
            word = word | (255u << (8u * k));
        } else {
            c = neighbor_average(x, y, z);
        }
        textureStore(texels, vec3<u32>(x, y, z), round(c) / 255.0);
    }
    validity_words[(z * vb.h + y) * vb.row_words + id.x] = word;
}
