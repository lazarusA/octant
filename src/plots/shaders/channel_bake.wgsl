// A multi-channel composite blended once into a plot's own data buffer
// (`plots/channel_bake.rs`): each cell's packed color (`channel_value`), so
// the surface, sphere and point cloud draw it as a CPU-blended composite.

struct Bake {
    cells: u32,
    // Invocations per dispatch row (`x` workgroups of 64).
    row: u32,
    // A thinned grid (`data/render/decimate.rs`): its width, one cell in
    // `stride` of the planes' `full_width` x `full_height` (stride 1: the
    // cells are the planes').
    stride: u32,
    width: u32,
    full_width: u32,
    full_height: u32,
    _pad0: u32,
    _pad1: u32,
};

@group(0) @binding(0)
var<uniform> bake: Bake;

@group(0) @binding(1)
var<storage, read_write> baked: array<f32>;

@compute @workgroup_size(64)
fn bake_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.y * bake.row + id.x;
    if (i >= bake.cells) {
        return;
    }
    var cell = i;
    if (bake.stride > 1u) {
        // `sample_index`: the middle cell of each stride.
        let half = bake.stride / 2u;
        let x = min((i % bake.width) * bake.stride + half, bake.full_width - 1u);
        let y = min((i / bake.width) * bake.stride + half, bake.full_height - 1u);
        cell = y * bake.full_width + x;
    }
    baked[i] = channel_value(cell, bitcast<f32>(0x7FC00000u));
}
