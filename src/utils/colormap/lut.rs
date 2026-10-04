//! 256-entry RGBA8 lookup tables: the single representation sampled by both the
//! CPU (colorbar, hover card, swatches) and the GPU (`colormaps/mod.wgsl`).

use egui::Color32;

pub const LUT_SIZE: usize = 256;

/// One colormap row; RGBA8 so rows upload directly into the `Rgba8Unorm` GPU atlas.
pub type Lut = [[u8; 4]; LUT_SIZE];

/// Resamples color stops into a LUT. Continuous maps interpolate linearly in sRGB
/// between evenly spaced stops; discrete maps use hard steps.
pub fn resample(stops: &[[u8; 3]], discrete: bool) -> Box<Lut> {
    let mut lut = Box::new([[0, 0, 0, 255]; LUT_SIZE]);
    let n = stops.len();
    if n == 0 {
        return lut;
    }
    for (i, slot) in lut.iter_mut().enumerate() {
        let rgb = if discrete || n == 1 {
            stops[(i * n / LUT_SIZE).min(n - 1)]
        } else {
            let pos = i as f32 / (LUT_SIZE - 1) as f32 * (n - 1) as f32;
            let j = (pos.floor() as usize).min(n - 2);
            mix_u8(stops[j], stops[j + 1], pos - j as f32)
        };
        *slot = [rgb[0], rgb[1], rgb[2], 255];
    }
    lut
}

/// Continuous version of a palette: its colors blended in Oklab (evenly
/// spaced, endpoints kept exactly). Falls back to linear sRGB blending.
pub fn smooth(stops: &[[u8; 3]]) -> Box<Lut> {
    use colorgrad::{BlendMode, Color, GradientBuilder, LinearGradient};
    let colors: Vec<Color> = stops
        .iter()
        .map(|c| Color::from_rgba8(c[0], c[1], c[2], 255))
        .collect();
    match GradientBuilder::new()
        .colors(&colors)
        .mode(BlendMode::Oklab)
        .build::<LinearGradient>()
    {
        Ok(gradient) => bake(&gradient),
        Err(_) => resample(stops, false),
    }
}

/// Samples a `colorgrad` gradient across its whole domain into a LUT.
pub fn bake(gradient: &dyn colorgrad::Gradient) -> Box<Lut> {
    let (dmin, dmax) = gradient.domain();
    let mut lut = Box::new([[0, 0, 0, 255]; LUT_SIZE]);
    for (i, slot) in lut.iter_mut().enumerate() {
        let t = dmin + (dmax - dmin) * i as f32 / (LUT_SIZE - 1) as f32;
        let [r, g, b, _] = gradient.at(t).to_rgba8();
        *slot = [r, g, b, 255];
    }
    lut
}

fn mix_u8(a: [u8; 3], b: [u8; 3], f: f32) -> [u8; 3] {
    std::array::from_fn(|k| {
        let (a, b) = (f32::from(a[k]), f32::from(b[k]));
        (a + (b - a) * f).round().clamp(0.0, 255.0) as u8
    })
}

/// Flips the colormap parameter when the map is shown reversed.
#[inline]
pub fn orient(t: f32, reversed: bool) -> f32 {
    if reversed { 1.0 - t } else { t }
}

/// Samples a LUT at `t` in [0, 1] exactly like WGSL `sample_colormap`: two
/// neighbouring texels blended linearly, or the nearest texel for stepped
/// (categorical) maps so a sample between two steps never mixes two colors.
pub fn sample_lut(lut: &Lut, t: f32, nearest: bool) -> Color32 {
    // NaN samples the start; infinities clamp to the ends (as WGSL `clamp`).
    let x = if t.is_nan() { 0.0 } else { t.clamp(0.0, 1.0) } * (LUT_SIZE - 1) as f32;
    if nearest {
        let [r, g, b, _] = lut[((x + 0.5).floor() as usize).min(LUT_SIZE - 1)];
        return Color32::from_rgb(r, g, b);
    }
    let i0 = (x.floor() as usize).min(LUT_SIZE - 2);
    let f = x - i0 as f32;
    let (c0, c1) = (lut[i0], lut[i0 + 1]);
    let ch = |k: usize| {
        let (a, b) = (f32::from(c0[k]), f32::from(c1[k]));
        (a + (b - a) * f).round().clamp(0.0, 255.0) as u8
    };
    Color32::from_rgb(ch(0), ch(1), ch(2))
}
