// Multi-channel composites blended on the GPU: each drawn channel's raw
// value, stretched, through its display curve and in its tint, added up as
// `ChannelAccumulator` does on the CPU (`data/slicing/composite`).
//
// Group 2 holds the channel parameters and the planes, one `plane_len` run
// of raw values per channel of the block (`plots/channels.rs`
// `ChannelPlanes`), bound the same way by every pipeline that blends.

// One drawn channel, mirrored by `ChannelGpu` in `channel_params.rs`.
struct Channel {
    plane: u32,
    // 0 linear, 1 polynomial, 2 logarithmic, 3 exponential.
    family: u32,
    inverted: u32,
    // Raw values read as int8 bit patterns (`Stretch::is_i8`).
    is_i8: u32,
    // Tint in 0..255 (alpha unused).
    tint: vec4<f32>,
    // `(raw - offset) * scale`, then onto 0..1.
    scale: f32,
    offset: f32,
    _pad0: f32,
    _pad1: f32,
    // `[k, 255^k, 1 - exp(-255^k)]` (`ChannelCurve::gpu_terms`).
    curve: vec4<f32>,
};

struct ChannelParams {
    count: u32,
    plane_len: u32,
    use_nan: u32,
    _pad: u32,
    nan_color: vec4<f32>,
    channels: array<Channel, 16>,
};

@group(2) @binding(0)
var<uniform> channel_params: ChannelParams;

@group(2) @binding(1)
var<storage, read> channel_planes: array<f32>;

fn channel_is_nan(v: f32) -> bool {
    let bits = bitcast<u32>(v);
    return (bits & 0x7F800000u) == 0x7F800000u && (bits & 0x007FFFFFu) != 0u;
}

// `ChannelCurve::map`: the windowed intensity through the family's curve,
// then inverted.
fn channel_curve(ch: Channel, norm: f32) -> f32 {
    let n = clamp(norm, 0.0, 1.0);
    var c = n;
    if (ch.family == 1u) {
        c = select(pow(n, ch.curve.x), 0.0, n <= 0.0);
    } else if (ch.family == 2u) {
        c = log(max(255.0 * n, 1.0)) / log(255.0);
    } else if (ch.family == 3u) {
        let p = select(pow(255.0 * n, ch.curve.x), 0.0, n <= 0.0);
        c = exp(p - ch.curve.y) * (1.0 - exp(-p)) / ch.curve.z;
    }
    c = clamp(c, 0.0, 1.0);
    return select(c, 1.0 - c, ch.inverted == 1u);
}

// Cell `cell`'s blended color, rgb in 0..1 on whole 0..255 levels as
// `pack_rgb` keeps them; alpha 0 when no drawn channel is valid there.
fn blend_cell(cell: u32) -> vec4<f32> {
    var rgb = vec3<f32>(0.0);
    var valid = false;
    for (var i = 0u; i < channel_params.count; i = i + 1u) {
        let ch = channel_params.channels[i];
        var v = channel_planes[ch.plane * channel_params.plane_len + cell];
        if (channel_is_nan(v)) {
            continue;
        }
        if (ch.is_i8 == 1u) {
            // `raw as i8 as u8`.
            let t = trunc(v);
            v = select(t, t + 256.0, t < 0.0);
        }
        let norm = clamp((v - ch.offset) * ch.scale, 0.0, 1.0);
        rgb = rgb + channel_curve(ch, norm) * ch.tint.rgb;
        valid = true;
    }
    let level = floor(clamp(rgb, vec3<f32>(0.0), vec3<f32>(255.0))) / 255.0;
    return vec4<f32>(level, select(0.0, 1.0, valid));
}

// What a 2D plot draws at `cell` whose data is `raw`: with drawn channels,
// their blend packed as `pack_rgb` packs CPU-blended composites (NaN where
// no channel is valid, `pack_or_nan`), so heights, colors and missing data
// read it as they read those; else `raw`.
fn channel_value(cell: u32, raw: f32) -> f32 {
    if (channel_params.count == 0u) {
        return raw;
    }
    let color = blend_cell(cell);
    if (color.a == 0.0) {
        return bitcast<f32>(0x7FC00000u);
    }
    let level = vec3<u32>(round(color.rgb * 255.0));
    return f32(level.r | (level.g << 8u) | (level.b << 16u));
}
