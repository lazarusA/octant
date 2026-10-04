// Surface shading: world-space gradients, viewer-facing normals and
// Blinn-Phong lighting under a headlight.

// Unit-box to world scale: world = (p - 0.5) * world_scale().
fn world_scale() -> vec3<f32> {
    return max(vec3<f32>(uniforms.aspect_x, uniforms.aspect_y, uniforms.aspect_z), vec3<f32>(0.001));
}

// One voxel per axis, in unit-box units.
fn voxel_step() -> vec3<f32> {
    return 1.0 / volume_dims();
}

// Scalar driving surface normals: the trilinear value, or composite brightness.
fn sample_intensity(p: vec3<f32>) -> f32 {
    if (is_composite()) {
        return sample_composite(p).a;
    }
    let s = sample_scalar(p);
    return select(s, 0.0, is_missing(s));
}

// Tetrahedron offsets: four taps give a central-difference gradient, since the
// offsets sum to zero and their outer products to 4 I.
const TETRA = array<vec3<f32>, 4>(
    vec3<f32>(1.0, -1.0, -1.0),
    vec3<f32>(-1.0, -1.0, 1.0),
    vec3<f32>(-1.0, 1.0, -1.0),
    vec3<f32>(1.0, 1.0, 1.0),
);

// `p` moved inside the box far enough that taps `reach` voxels away stay on
// data: taps clamped at a face would skew normals right where DVR is seen.
fn stencil_center(p: vec3<f32>, reach: f32) -> vec3<f32> {
    let margin = min((reach + 0.5) * voxel_step(), vec3<f32>(0.5));
    return clamp(p, margin, 1.0 - margin);
}

// Gradient of the intensity at `p` per world unit, from taps one voxel away.
fn gradient_intensity(p: vec3<f32>) -> vec3<f32> {
    let h = voxel_step();
    let c = stencil_center(p, 1.0);
    var offsets = TETRA;
    var g = vec3<f32>(0.0);
    for (var i = 0; i < 4; i = i + 1) {
        g += offsets[i] * sample_intensity(c + offsets[i] * h);
    }
    return g / (4.0 * h * world_scale());
}

// Label foreground smoothed over half a voxel: the trilinear mask has
// voxel-sized facets, which this level set rounds off.
fn mask_smooth(p: vec3<f32>) -> f32 {
    let h = 0.5 * voxel_step();
    var offsets = TETRA;
    var sum = 0.0;
    for (var i = 0; i < 4; i = i + 1) {
        sum += mask_filtered(p + offsets[i] * h);
    }
    return 0.25 * sum;
}

// Gradient of the label mask at `p` per world unit, over two voxels so the
// normals follow the surface rather than its voxel facets.
fn gradient_mask(p: vec3<f32>) -> vec3<f32> {
    let h = 2.0 * voxel_step();
    let c = stencil_center(p, 2.0);
    var offsets = TETRA;
    var g = vec3<f32>(0.0);
    for (var i = 0; i < 4; i = i + 1) {
        g += offsets[i] * mask_smooth(c + offsets[i] * h);
    }
    return g / (4.0 * h * world_scale());
}

// Unit normal across gradient `g`, turned toward the viewer (surfaces are seen
// from whichever side the ray arrives).
fn facing_normal(g: vec3<f32>, view: vec3<f32>) -> vec3<f32> {
    let len = length(g);
    if (len < 1e-20) {
        return view;
    }
    let n = -g / len;
    return select(n, -n, dot(n, view) < 0.0);
}

fn blinnphong(n: vec3<f32>, view: vec3<f32>, light: vec3<f32>, color: vec3<f32>) -> vec3<f32> {
    let diffuse = max(dot(n, light), 0.0);
    let specular = pow(max(dot(n, normalize(light + view)), 0.0), uniforms.shininess);
    return color * (uniforms.ambient + uniforms.diffuse * diffuse)
        + uniforms.light_color * uniforms.specular * specular;
}

// Lit opaque surface color with normal `n`.
fn shade_surface(ray: Ray, n: vec3<f32>, color: vec3<f32>) -> vec4<f32> {
    return vec4<f32>(blinnphong(n, ray.view, ray.light, color), 1.0);
}

// Transparent sample color lit by its gradient (diffuse only: highlights on
// semi-transparent media read as hard glints), blended in by gradient strength
// `m / (m + 1)` (m: color ranges per world unit), so flat regions keep their
// colormap color and fronts and edges gain shape.
fn lit_sample(ray: Ray, p: vec3<f32>, color: vec3<f32>, range: f32) -> vec3<f32> {
    let g = gradient_intensity(p);
    let m = length(g) / max(range, 1e-30);
    let diffuse = max(dot(facing_normal(g, ray.view), ray.light), 0.0);
    let lit = color * (uniforms.ambient + uniforms.diffuse * diffuse);
    return mix(color, lit, m / (m + 1.0));
}
