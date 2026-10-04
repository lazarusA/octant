//! 3D volume data container for volumetric visualization.

#[derive(Clone, Debug)]
pub struct VolumeData {
    pub width: usize,
    pub height: usize,
    pub depth: usize,
    pub values: Vec<f32>,
    pub min_val: f32,
    pub max_val: f32,
    pub dataset_name: String,
}

impl VolumeData {
    pub fn new(
        width: usize,
        height: usize,
        depth: usize,
        values: Vec<f32>,
        min_val: f32,
        max_val: f32,
        dataset_name: String,
    ) -> Self {
        Self {
            width,
            height,
            depth,
            values,
            min_val,
            max_val,
            dataset_name,
        }
    }

    pub fn new_procedural(
        width: usize,
        height: usize,
        depth: usize,
        timestep: usize,
        max_timesteps: usize,
    ) -> Self {
        let (values, min_val, max_val) = crate::data::procedural::generate_procedural_volume_3d(
            width,
            height,
            depth,
            timestep,
            max_timesteps,
        );
        Self::new(
            width,
            height,
            depth,
            values,
            min_val,
            max_val,
            format!("Procedural Volume [t={timestep}]"),
        )
    }

    /// In-place update of a 3D sub-volume slab (e.g. progressive chunk arrivals along Z or XY).
    pub fn update_subvolume(
        &mut self,
        dest_pos: [usize; 3],  // [dest_x, dest_y, dest_z]
        slab_size: [usize; 3], // [slab_w, slab_h, slab_d]
        slab_values: &[f32],
    ) {
        let [dest_x, dest_y, dest_z] = dest_pos;
        let [slab_w, slab_h, slab_d] = slab_size;

        if self.width == slab_w && self.height == slab_h && dest_x == 0 && dest_y == 0 {
            // Fast contiguous copy along Z
            let plane_elements = self.width * self.height;
            let start_idx = dest_z * plane_elements;
            let copy_elements = slab_d * plane_elements;
            let end_idx = (start_idx + copy_elements).min(self.values.len());
            let copy_len = (end_idx.saturating_sub(start_idx)).min(slab_values.len());
            if start_idx < self.values.len() && copy_len > 0 {
                self.values[start_idx..start_idx + copy_len]
                    .copy_from_slice(&slab_values[..copy_len]);
            }
        } else {
            // General row-by-row sub-volume copy
            for z in 0..slab_d {
                let target_z = dest_z + z;
                if target_z >= self.depth {
                    break;
                }
                for y in 0..slab_h {
                    let target_y = dest_y + y;
                    if target_y >= self.height {
                        break;
                    }
                    let target_x = dest_x.min(self.width);
                    let row_w = slab_w.min(self.width.saturating_sub(target_x));
                    if row_w == 0 {
                        continue;
                    }

                    let dest_idx =
                        target_z * (self.width * self.height) + target_y * self.width + target_x;
                    let src_idx = z * (slab_w * slab_h) + y * slab_w;
                    if dest_idx + row_w <= self.values.len() && src_idx + row_w <= slab_values.len()
                    {
                        self.values[dest_idx..dest_idx + row_w]
                            .copy_from_slice(&slab_values[src_idx..src_idx + row_w]);
                    }
                }
            }
        }

        // Update min_val and max_val with finite values from the new slab
        if let Some((lo, hi)) = finite_range(slab_values) {
            self.min_val = if self.min_val.is_nan() {
                lo
            } else {
                self.min_val.min(lo)
            };
            self.max_val = if self.max_val.is_nan() {
                hi
            } else {
                self.max_val.max(hi)
            };
        }
    }

    /// Element range of the whole Z planes `[dest_z, dest_z + slab_d)` in `values`,
    /// clamped to the volume: the contiguous span a slab written by
    /// [`Self::update_subvolume`] touches, for partial GPU uploads.
    pub fn plane_range(&self, dest_z: usize, slab_d: usize) -> std::ops::Range<usize> {
        let plane = self.width.saturating_mul(self.height);
        let z0 = dest_z.min(self.depth);
        let z1 = dest_z.saturating_add(slab_d).min(self.depth);
        let len = self.values.len();
        z0.saturating_mul(plane).min(len)..z1.saturating_mul(plane).min(len)
    }

    /// Extracts a single 1D ray along Z (depth) for a given (x, y) spatial pixel coordinate.
    pub fn extract_z_line_profile(&self, target_x: usize, target_y: usize) -> Vec<f32> {
        let (nx, ny, nz) = (self.width, self.height, self.depth);
        let x = target_x.min(nx.saturating_sub(1));
        let y = target_y.min(ny.saturating_sub(1));
        let mut profile = Vec::with_capacity(nz);
        for z in 0..nz {
            let idx = z * (nx * ny) + y * nx + x;
            profile.push(self.values.get(idx).copied().unwrap_or(f32::NAN));
        }
        profile
    }

    /// Extracts all valid finite rays along Z (depth) flattened into a contiguous payload.
    /// Returns `(payload, profile_length, valid_lines_count)`.
    pub fn extract_all_z_lines_payload(&self) -> (Vec<f32>, u32, u32) {
        let (nx, ny, nz) = (self.width, self.height, self.depth);
        let num_pixels = nx * ny;
        let mut payload = Vec::with_capacity(num_pixels * nz);
        let mut valid_lines = 0u32;
        for y in 0..ny {
            for x in 0..nx {
                let mut has_valid = false;
                for z in 0..nz {
                    let idx = z * (nx * ny) + y * nx + x;
                    if let Some(&v) = self.values.get(idx)
                        && !v.is_nan()
                        && v.is_finite()
                    {
                        has_valid = true;
                        break;
                    }
                }
                if has_valid {
                    valid_lines += 1;
                    for z in 0..nz {
                        let idx = z * (nx * ny) + y * nx + x;
                        payload.push(self.values.get(idx).copied().unwrap_or(f32::NAN));
                    }
                }
            }
        }
        (payload, nz as u32, valid_lines)
    }
}

/// Min and max of the finite values in `values`, or `None` when there are
/// none (parallel on native: slabs arrive on every animation step).
pub fn finite_range(values: &[f32]) -> Option<(f32, f32)> {
    let fold = |(lo, hi): (f32, f32), &v: &f32| {
        if v.is_finite() {
            (lo.min(v), hi.max(v))
        } else {
            (lo, hi)
        }
    };
    let empty = (f32::INFINITY, f32::NEG_INFINITY);
    #[cfg(not(target_arch = "wasm32"))]
    let (lo, hi) = {
        use rayon::prelude::*;
        values
            .par_chunks(1 << 16)
            .map(|c| c.iter().fold(empty, fold))
            .reduce(|| empty, |a, b| (a.0.min(b.0), a.1.max(b.1)))
    };
    #[cfg(target_arch = "wasm32")]
    let (lo, hi) = values.iter().fold(empty, fold);
    (lo <= hi).then_some((lo, hi))
}
