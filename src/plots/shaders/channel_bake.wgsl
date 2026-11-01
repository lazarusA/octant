// A multi-channel composite blended once into a plot's own data buffer
// (`plots/channel_bake.rs`): each cell's packed color (`channel_value`), so
// the surface, sphere and point cloud draw it as a CPU-blended composite.

struct Bake {
    cells: u32,
    // Invocations per dispatch row (`x` workgroups of 64).
    row: u32,
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
    baked[i] = channel_value(i, bitcast<f32>(0x7FC00000u));
}
