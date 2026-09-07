struct Uniforms {
    pan: vec2<f32>,
    zoom: f32,
    coord_mode: u32,
    aspect_scale: vec2<f32>,
    width: u32,
    height: u32,
    tile_bounds: vec4<f32>,
    color: ColorUniforms,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var<storage, read> data_buffer: array<f32>;

@group(0) @binding(2)
var<storage, read> coord_x_buffer: array<f32>;

@group(0) @binding(3)
var<storage, read> coord_y_buffer: array<f32>;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

fn find_coord_cell_x(query_val: f32, len: u32) -> u32 {
    if (len <= 1u) {
        return 0u;
    }
    let max_idx = min(len - 1u, max(arrayLength(&coord_x_buffer), 1u) - 1u);
    if (max_idx == 0u) {
        return 0u;
    }
    let first = coord_x_buffer[0];
    let last = coord_x_buffer[max_idx];
    let is_descending = first > last;

    var low: u32 = 0u;
    var high: u32 = max_idx - 1u;

    if (is_descending) {
        while (low < high) {
            let mid = (low + high + 1u) / 2u;
            if (coord_x_buffer[mid] >= query_val) {
                low = mid;
            } else {
                high = mid - 1u;
            }
        }
    } else {
        while (low < high) {
            let mid = (low + high + 1u) / 2u;
            if (coord_x_buffer[mid] <= query_val) {
                low = mid;
            } else {
                high = mid - 1u;
            }
        }
    }

    let next = min(low + 1u, max_idx);
    if (abs(query_val - coord_x_buffer[low]) <= abs(query_val - coord_x_buffer[next])) {
        return low;
    } else {
        return next;
    }
}

fn find_coord_cell_y(query_val: f32, len: u32) -> u32 {
    if (len <= 1u) {
        return 0u;
    }
    let max_idx = min(len - 1u, max(arrayLength(&coord_y_buffer), 1u) - 1u);
    if (max_idx == 0u) {
        return 0u;
    }
    let first = coord_y_buffer[0];
    let last = coord_y_buffer[max_idx];
    let is_descending = first > last;

    var low: u32 = 0u;
    var high: u32 = max_idx - 1u;

    if (is_descending) {
        while (low < high) {
            let mid = (low + high + 1u) / 2u;
            if (coord_y_buffer[mid] >= query_val) {
                low = mid;
            } else {
                high = mid - 1u;
            }
        }
    } else {
        while (low < high) {
            let mid = (low + high + 1u) / 2u;
            if (coord_y_buffer[mid] <= query_val) {
                low = mid;
            } else {
                high = mid - 1u;
            }
        }
    }

    let next = min(low + 1u, max_idx);
    if (abs(query_val - coord_y_buffer[low]) <= abs(query_val - coord_y_buffer[next])) {
        return low;
    } else {
        return next;
    }
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Map quad vertices [-1..1] to tile sub-region [tile_bounds.x..tile_bounds.z] in data space
    let tile_u = mix(uniforms.tile_bounds.x, uniforms.tile_bounds.z, (model.position.x + 1.0) * 0.5);
    let tile_v = mix(uniforms.tile_bounds.y, uniforms.tile_bounds.w, (1.0 - model.position.y) * 0.5);

    let model_x = tile_u * 2.0 - 1.0;
    let model_y = 1.0 - tile_v * 2.0;
    let model_pos = vec2<f32>(model_x, model_y);

    let scaled_model_pos = model_pos * uniforms.aspect_scale;
    let transformed_pos = scaled_model_pos * uniforms.zoom + uniforms.pan;
    out.position = vec4<f32>(transformed_pos, 0.0, 1.0);
    out.uv = model.uv;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if (in.uv.x < 0.0 || in.uv.x > 1.0 || in.uv.y < 0.0 || in.uv.y > 1.0) {
        discard;
    }
    let w = max(uniforms.width, 1u);
    let h = max(uniforms.height, 1u);
    let max_idx = arrayLength(&data_buffer) - 1u;

    var gx: u32;
    var gy: u32;

    if (uniforms.coord_mode == 2u) {
        let max_cx = min(w - 1u, max(arrayLength(&coord_x_buffer), 1u) - 1u);
        let first_x = coord_x_buffer[0];
        let last_x = coord_x_buffer[max_cx];
        let target_x = mix(first_x, last_x, in.uv.x);
        gx = find_coord_cell_x(target_x, w);

        let max_cy = min(h - 1u, max(arrayLength(&coord_y_buffer), 1u) - 1u);
        let first_y = coord_y_buffer[0];
        let last_y = coord_y_buffer[max_cy];
        let target_y = mix(first_y, last_y, in.uv.y);
        gy = find_coord_cell_y(target_y, h);
    } else {
        gx = clamp(u32(in.uv.x * f32(w)), 0u, w - 1u);
        gy = clamp(u32(in.uv.y * f32(h)), 0u, h - 1u);
    }

    let cell_index = min(gy * w + gx, max_idx);
    let val = data_buffer[cell_index];

    let eval_color = evaluate_plot_color(val, uniforms.color);
    if (eval_color.a <= 0.0) {
        discard;
    }
    return eval_color;
}

