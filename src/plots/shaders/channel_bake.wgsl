// A multi-channel composite blended once into a plot's own data buffer
// (`plots/channel_bake.rs`): each cell's packed color (`channel_value`), so
// the surface, sphere and point cloud draw it as a CPU-blended composite.

struct Bake {
    cells: u32,
    // Invocations per dispatch row (`x` workgroups of 64).
    row: u32,
    // A thinned grid (`data/render/decimate.rs`): `width` x `height` a
    // plane, one cell in `stride` of the planes' `full_width` x
    // `full_height` along x and y, every plane (stride 1: the cells are the
    // planes').
    stride: u32,
    width: u32,
    full_width: u32,
    full_height: u32,
    height: u32,
    _pad: u32,
};

@group(0) @binding(0)
var<uniform> bake: Bake;

@group(0) @binding(1)
var<storage, read_write> baked: array<f32>;

// `decimate.rs` `sample_index`: full index `floor((2i + 1) n / 2w)` of
// thinned index `i` of `w` drawn over `n` cells by stride `k`, in u32
// without overflow: with `n = k w - r` (`r < k`) it is
// `floor((2i + 1) k / 2 - (2i + 1) r / 2w)`.
fn sample_index(i: u32, k: u32, w: u32, n: u32) -> u32 {
    let a = (2u * i + 1u) * k;
    let b = (2u * i + 1u) * (k * w - n);
    var x = (a >> 1u) - b / (2u * w);
    // The half `a` drops against the remainder of `b / 2w`.
    if ((a & 1u) * w < b % (2u * w)) {
        x = x - 1u;
    }
    return min(x, n - 1u);
}

@compute @workgroup_size(64)
fn bake_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.y * bake.row + id.x;
    if (i >= bake.cells) {
        return;
    }
    var cell = i;
    if (bake.stride > 1u) {
        let plane = bake.width * bake.height;
        let x = sample_index(i % bake.width, bake.stride, bake.width, bake.full_width);
        let y = sample_index((i % plane) / bake.width, bake.stride, bake.height, bake.full_height);
        cell = ((i / plane) * bake.full_height + y) * bake.full_width + x;
    }
    baked[i] = channel_value(cell, bitcast<f32>(0x7FC00000u));
}
