// Heatmaps: the data quad (`common/quad.wgsl`) colored through a colormap.

struct Uniforms {
    quad: QuadUniforms,
    color: ColorUniforms,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var<storage, read> data_buffer: array<f32>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if (outside_quad(in.uv)) {
        discard;
    }
    let q = uniforms.quad;
    let len = arrayLength(&data_buffer);
    var cell_index: u32;
    if (q.coord_mode == 4u || q.coord_mode == 5u) {
        let total_elements = max(q.width, 1u) * max(q.height, 1u);
        let lon = (in.uv.x - 0.5) * 6.2831853;
        let lat = (0.5 - in.uv.y) * 3.14159265;
        let nside = healpix_nside(total_elements);
        let pix = healpix_ang2pix(nside, lon, lat, q.coord_mode == 5u);
        cell_index = min(pix, min(total_elements, len) - 1u);
    } else {
        cell_index = quad_cell(in.uv, len);
    }

    let eval_color = evaluate_plot_color(data_buffer[cell_index], uniforms.color);
    if (eval_color.a <= 0.0) {
        discard;
    }
    return eval_color;
}
