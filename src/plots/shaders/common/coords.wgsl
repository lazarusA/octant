// Shared 1D coordinate buffer bindings and calculation utilities for Octant WGSL shaders

@group(0) @binding(2)
var<storage, read> coord_x_buffer: array<f32>;

@group(0) @binding(3)
var<storage, read> coord_y_buffer: array<f32>;

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

fn get_cell_normalized_bounds_x(cell_idx: u32, grid_w: u32, coord_mode: u32) -> vec2<f32> {
    if (coord_mode != 2u || grid_w <= 1u) {
        let u0 = f32(cell_idx) / f32(grid_w);
        let u1 = f32(cell_idx + 1u) / f32(grid_w);
        return vec2<f32>(u0, u1);
    }
    let max_cx = min(grid_w - 1u, max(arrayLength(&coord_x_buffer), 1u) - 1u);
    if (max_cx == 0u) {
        return vec2<f32>(0.0, 1.0);
    }
    let first_x = coord_x_buffer[0];
    let last_x = coord_x_buffer[max_cx];
    let span_x = last_x - first_x;
    if (abs(span_x) < 1e-6) {
        let u0 = f32(cell_idx) / f32(grid_w);
        let u1 = f32(cell_idx + 1u) / f32(grid_w);
        return vec2<f32>(u0, u1);
    }

    let idx = min(cell_idx, max_cx);
    var b0: f32;
    if (idx == 0u) {
        b0 = first_x - 0.5 * (coord_x_buffer[min(1u, max_cx)] - first_x);
    } else {
        b0 = 0.5 * (coord_x_buffer[idx - 1u] + coord_x_buffer[idx]);
    }

    var b1: f32;
    if (idx >= max_cx) {
        let prev_idx = select(max_cx - 1u, 0u, max_cx == 0u);
        b1 = last_x + 0.5 * (last_x - coord_x_buffer[prev_idx]);
    } else {
        b1 = 0.5 * (coord_x_buffer[idx] + coord_x_buffer[idx + 1u]);
    }

    let u0 = (b0 - first_x) / span_x;
    let u1 = (b1 - first_x) / span_x;
    return vec2<f32>(u0, u1);
}

fn get_cell_normalized_bounds_y(cell_idx: u32, grid_h: u32, coord_mode: u32) -> vec2<f32> {
    if (coord_mode != 2u || grid_h <= 1u) {
        let v0 = f32(cell_idx) / f32(grid_h);
        let v1 = f32(cell_idx + 1u) / f32(grid_h);
        return vec2<f32>(v0, v1);
    }
    let max_cy = min(grid_h - 1u, max(arrayLength(&coord_y_buffer), 1u) - 1u);
    if (max_cy == 0u) {
        return vec2<f32>(0.0, 1.0);
    }
    let first_y = coord_y_buffer[0];
    let last_y = coord_y_buffer[max_cy];
    let span_y = last_y - first_y;
    if (abs(span_y) < 1e-6) {
        let v0 = f32(cell_idx) / f32(grid_h);
        let v1 = f32(cell_idx + 1u) / f32(grid_h);
        return vec2<f32>(v0, v1);
    }

    let idx = min(cell_idx, max_cy);
    var b0: f32;
    if (idx == 0u) {
        b0 = first_y - 0.5 * (coord_y_buffer[min(1u, max_cy)] - first_y);
    } else {
        b0 = 0.5 * (coord_y_buffer[idx - 1u] + coord_y_buffer[idx]);
    }

    var b1: f32;
    if (idx >= max_cy) {
        let prev_idx = select(max_cy - 1u, 0u, max_cy == 0u);
        b1 = last_y + 0.5 * (last_y - coord_y_buffer[prev_idx]);
    } else {
        b1 = 0.5 * (coord_y_buffer[idx] + coord_y_buffer[idx + 1u]);
    }

    let v0 = (b0 - first_y) / span_y;
    let v1 = (b1 - first_y) / span_y;
    return vec2<f32>(v0, v1);
}
