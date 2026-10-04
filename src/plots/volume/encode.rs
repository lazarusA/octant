//! CPU encoding of volume planes for the GPU textures.
//!
//! Missing voxels (NaN, infinities, |v| > 1e30) get the average of their valid
//! 26 neighbors and a zero validity byte. Trilinear filtering next to missing
//! data then blends real values only, and the filtered validity's 0.5 level is
//! a smooth, sub-voxel boundary instead of a voxel staircase.

use std::ops::Range;

/// How voxel values are stored on the GPU.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VolumeEncoding {
    /// Raw `f32` scalars (`R32Float`).
    Scalar,
    /// RGB composite colors packed into `f32` as `r | g << 8 | b << 16` (`Rgba8Unorm`).
    PackedRgb,
}

/// Whether `v` is drawable; matches the shaders' missing-value test.
pub fn is_valid(v: f32) -> bool {
    v.is_finite() && v.abs() <= 1e30
}

/// Volume extent in voxels (x fastest, then y, then z).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Dims {
    pub w: usize,
    pub h: usize,
    pub d: usize,
}

impl Dims {
    pub fn plane(&self) -> usize {
        self.w * self.h
    }
}

/// Encoded planes `z.start..z.end`, ready for `write_texture`.
pub struct Encoded<T> {
    pub z: Range<usize>,
    pub texels: Vec<T>,
    /// 255 where the voxel holds data, 0 where it is missing.
    pub validity: Vec<u8>,
    /// Missing voxels in each encoded plane.
    pub invalid_per_plane: Vec<u32>,
}

/// Encodes scalar planes `z` of `values`.
pub fn encode_scalar(values: &[f32], dims: Dims, z: Range<usize>) -> Encoded<f32> {
    encode::<1, f32>(values, dims, z, |v| is_valid(v).then_some([v]), |[v]| v)
}

/// Encodes packed RGB composite planes `z` of `values`; alpha is the brightest
/// channel, as the shaders unpacked it before.
pub fn encode_rgba(values: &[f32], dims: Dims, z: Range<usize>) -> Encoded<[u8; 4]> {
    encode::<4, [u8; 4]>(values, dims, z, unpack_rgb, |c| {
        c.map(|x| x.round().clamp(0.0, 255.0) as u8)
    })
}

fn unpack_rgb(v: f32) -> Option<[f32; 4]> {
    if !is_valid(v) {
        return None;
    }
    let packed = v as u32;
    let [r, g, b] = [packed & 0xFF, (packed >> 8) & 0xFF, (packed >> 16) & 0xFF].map(|c| c as f32);
    Some([r, g, b, r.max(g).max(b)])
}

fn encode<const C: usize, T>(
    values: &[f32],
    dims: Dims,
    z: Range<usize>,
    decode: impl Fn(f32) -> Option<[f32; C]>,
    store: impl Fn([f32; C]) -> T,
) -> Encoded<T> {
    let plane = dims.plane();
    let z = z.start.min(dims.d)..z.end.min(dims.d);
    let voxels = z.len() * plane;
    let mut out = Encoded {
        z: z.clone(),
        texels: Vec::with_capacity(voxels),
        validity: Vec::with_capacity(voxels),
        invalid_per_plane: Vec::with_capacity(z.len()),
    };
    if values.len() < dims.d * plane {
        return out;
    }
    let sampler = Neighborhood {
        values,
        dims,
        decode: &decode,
    };
    for zi in z {
        let mut invalid = 0u32;
        // A plane with no data around it fills with zeros without searching.
        let isolated = !sampler.planes_have_data(zi);
        for yi in 0..dims.h {
            for xi in 0..dims.w {
                let raw = values[zi * plane + yi * dims.w + xi];
                if let Some(c) = decode(raw) {
                    out.texels.push(store(c));
                    out.validity.push(255);
                } else {
                    let fill = if isolated {
                        [0.0; C]
                    } else {
                        sampler.average(xi, yi, zi)
                    };
                    out.texels.push(store(fill));
                    out.validity.push(0);
                    invalid += 1;
                }
            }
        }
        out.invalid_per_plane.push(invalid);
    }
    out
}

struct Neighborhood<'a, const C: usize, F> {
    values: &'a [f32],
    dims: Dims,
    decode: &'a F,
}

impl<const C: usize, F: Fn(f32) -> Option<[f32; C]>> Neighborhood<'_, C, F> {
    /// Whether plane `zi` or a plane next to it holds any valid voxel.
    fn planes_have_data(&self, zi: usize) -> bool {
        let plane = self.dims.plane();
        let z0 = zi.saturating_sub(1);
        let z1 = (zi + 2).min(self.dims.d);
        self.values[z0 * plane..z1 * plane]
            .iter()
            .any(|&v| (self.decode)(v).is_some())
    }

    /// Average of the valid voxels among the 26 neighbors of (x, y, z), or zeros.
    fn average(&self, x: usize, y: usize, z: usize) -> [f32; C] {
        let Dims { w, h, d } = self.dims;
        let mut sum = [0.0f32; C];
        let mut count = 0.0f32;
        for nz in z.saturating_sub(1)..(z + 2).min(d) {
            for ny in y.saturating_sub(1)..(y + 2).min(h) {
                for nx in x.saturating_sub(1)..(x + 2).min(w) {
                    let v = self.values[nz * w * h + ny * w + nx];
                    if let Some(c) = (self.decode)(v) {
                        for (s, ci) in sum.iter_mut().zip(c) {
                            *s += ci;
                        }
                        count += 1.0;
                    }
                }
            }
        }
        if count > 0.0 {
            sum.map(|s| s / count)
        } else {
            sum
        }
    }
}
