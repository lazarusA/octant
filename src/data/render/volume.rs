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
