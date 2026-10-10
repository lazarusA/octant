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
        for &v in slab_values {
            if v.is_finite() && !v.is_nan() {
                if self.min_val.is_nan() || v < self.min_val {
                    self.min_val = v;
                }
                if self.max_val.is_nan() || v > self.max_val {
                    self.max_val = v;
                }
            }
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
}
