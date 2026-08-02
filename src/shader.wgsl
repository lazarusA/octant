struct Uniforms {
    colormap: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) val: f32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) val: f32,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(model.position, 0.0, 1.0);
    out.uv = model.uv;
    out.val = model.val;
    return out;
}

// 0: Viridis scientific colormap
fn colormap_viridis(t: f32) -> vec3<f32> {
    let x = clamp(t, 0.0, 1.0);
    let c0 = vec3<f32>(0.267, 0.004, 0.329);
    let c1 = vec3<f32>(0.231, 0.322, 0.545);
    let c2 = vec3<f32>(0.129, 0.569, 0.551);
    let c3 = vec3<f32>(0.369, 0.788, 0.384);
    let c4 = vec3<f32>(0.992, 0.906, 0.145);
    
    if (x < 0.25)       { return mix(c0, c1, x / 0.25); }
    else if (x < 0.50)  { return mix(c1, c2, (x - 0.25) / 0.25); }
    else if (x < 0.75)  { return mix(c2, c3, (x - 0.50) / 0.25); }
    else                { return mix(c3, c4, (x - 0.75) / 0.25); }
}

// 1: Plasma scientific colormap
fn colormap_plasma(t: f32) -> vec3<f32> {
    let x = clamp(t, 0.0, 1.0);
    let c0 = vec3<f32>(0.051, 0.031, 0.529);
    let c1 = vec3<f32>(0.416, 0.000, 0.659);
    let c2 = vec3<f32>(0.694, 0.165, 0.565);
    let c3 = vec3<f32>(0.882, 0.392, 0.384);
    let c4 = vec3<f32>(0.941, 0.976, 0.129);
    
    if (x < 0.25)       { return mix(c0, c1, x / 0.25); }
    else if (x < 0.50)  { return mix(c1, c2, (x - 0.25) / 0.25); }
    else if (x < 0.75)  { return mix(c2, c3, (x - 0.50) / 0.25); }
    else                { return mix(c3, c4, (x - 0.75) / 0.25); }
}

// 2: Inferno scientific colormap
fn colormap_inferno(t: f32) -> vec3<f32> {
    let x = clamp(t, 0.0, 1.0);
    let c0 = vec3<f32>(0.000, 0.000, 0.016);
    let c1 = vec3<f32>(0.341, 0.062, 0.429);
    let c2 = vec3<f32>(0.733, 0.216, 0.330);
    let c3 = vec3<f32>(0.976, 0.557, 0.035);
    let c4 = vec3<f32>(0.988, 1.000, 0.643);
    
    if (x < 0.25)       { return mix(c0, c1, x / 0.25); }
    else if (x < 0.50)  { return mix(c1, c2, (x - 0.25) / 0.25); }
    else if (x < 0.75)  { return mix(c2, c3, (x - 0.50) / 0.25); }
    else                { return mix(c3, c4, (x - 0.75) / 0.25); }
}

// 3: Magma scientific colormap
fn colormap_magma(t: f32) -> vec3<f32> {
    let x = clamp(t, 0.0, 1.0);
    let c0 = vec3<f32>(0.000, 0.000, 0.016);
    let c1 = vec3<f32>(0.318, 0.071, 0.486);
    let c2 = vec3<f32>(0.714, 0.212, 0.475);
    let c3 = vec3<f32>(0.984, 0.533, 0.380);
    let c4 = vec3<f32>(0.988, 0.992, 0.749);
    
    if (x < 0.25)       { return mix(c0, c1, x / 0.25); }
    else if (x < 0.50)  { return mix(c1, c2, (x - 0.25) / 0.25); }
    else if (x < 0.75)  { return mix(c2, c3, (x - 0.50) / 0.25); }
    else                { return mix(c3, c4, (x - 0.75) / 0.25); }
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let norm = clamp(in.val / 100.0, 0.0, 1.0);
    var color: vec3<f32>;
    if (uniforms.colormap == 0u) {
        color = colormap_viridis(norm);
    } else if (uniforms.colormap == 1u) {
        color = colormap_plasma(norm);
    } else if (uniforms.colormap == 2u) {
        color = colormap_inferno(norm);
    } else {
        color = colormap_magma(norm);
    }
    return vec4<f32>(color, 1.0);
}
