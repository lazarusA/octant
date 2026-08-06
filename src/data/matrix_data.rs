pub struct MatrixData {
    pub width: usize,
    pub height: usize,
    pub values: Vec<f32>,
    pub min_val: f32,
    pub max_val: f32,
    pub dataset_name: String,
    pub max_timesteps: usize,
    pub unique_values: Option<Vec<f32>>,
}

impl MatrixData {
    pub fn new(
        width: usize,
        height: usize,
        values: Vec<f32>,
        min_val: f32,
        max_val: f32,
        dataset_name: String,
        max_timesteps: usize,
    ) -> Self {
        let unique_values = Self::compute_unique_values(&values);
        Self {
            width,
            height,
            values,
            min_val,
            max_val,
            dataset_name,
            max_timesteps,
            unique_values,
        }
    }

    /// Generates a random 2D scalar field for visualization
    pub fn create_random_matrix(
        width: usize,
        height: usize,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let (raw_data, min_v, max_v) =
            super::procedural::generate_procedural_matrix(width, height, 0);

        Ok(Self::new(
            width,
            height,
            raw_data,
            min_v,
            max_v,
            format!("Random Matrix ({}x{})", width, height),
            1,
        ))
    }

    /// Fast strided unique value detector (samples up to 2000 elements with early exit when > 20 unique values).
    pub fn compute_unique_values(values: &[f32]) -> Option<Vec<f32>> {
        if values.is_empty() {
            return None;
        }

        let step = (values.len() / 2000).max(1);
        let mut unique: Vec<f32> = Vec::with_capacity(24);

        for &val in values.iter().step_by(step) {
            if val.is_nan() || val.is_infinite() {
                continue;
            }
            if !unique.iter().any(|&u| (u - val).abs() < 1e-5) {
                unique.push(val);
                if unique.len() > 20 {
                    return None; // Exceeds 20 categories -> Continuous field
                }
            }
        }

        if unique.len() >= 2 {
            unique.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            Some(unique)
        } else {
            None
        }
    }

    /// Returns pre-cached unique values in O(1) constant time per frame.
    pub fn detect_unique_values(&self) -> Option<Vec<f32>> {
        self.unique_values.clone()
    }

    /// Extracts a 1D line profile along dimension axis (`dim_axis`: 0 = along X/width, 1 = along Y/height).
    pub fn extract_1d_line_profile(&self, dim_axis: usize, slice_idx: usize) -> Vec<f32> {
        if self.values.is_empty() || self.width == 0 || self.height == 0 {
            return Vec::new();
        }

        if dim_axis == 0 {
            // Along X (width): extract row at `slice_idx`
            let row = slice_idx.min(self.height.saturating_sub(1));
            let start = row * self.width;
            let end = (start + self.width).min(self.values.len());
            self.values[start..end].to_vec()
        } else {
            // Along Y (height): extract column at `slice_idx`
            let col = slice_idx.min(self.width.saturating_sub(1));
            (0..self.height)
                .map(|y| {
                    let idx = y * self.width + col;
                    self.values.get(idx).copied().unwrap_or(f32::NAN)
                })
                .collect()
        }
    }

    /// Extracts all 1D line profiles along dimension axis (`dim_axis`: 0 = all rows along X, 1 = all columns along Y).
    pub fn extract_all_1d_line_profiles(&self, dim_axis: usize) -> Vec<Vec<f32>> {
        if dim_axis == 0 {
            (0..self.height)
                .map(|y| self.extract_1d_line_profile(0, y))
                .collect()
        } else {
            (0..self.width)
                .map(|x| self.extract_1d_line_profile(1, x))
                .collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MatrixData;

    #[test]
    fn extracts_row_and_column_profiles() {
        let data = MatrixData::new(
            3,
            2,
            vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
            1.0,
            6.0,
            "test".to_string(),
            1,
        );

        assert_eq!(data.extract_1d_line_profile(0, 0), vec![1.0, 2.0, 3.0]);
        assert_eq!(data.extract_1d_line_profile(1, 1), vec![2.0, 5.0]);
        assert_eq!(
            data.extract_all_1d_line_profiles(0),
            vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]]
        );
        assert_eq!(
            data.extract_all_1d_line_profiles(1),
            vec![vec![1.0, 4.0], vec![2.0, 5.0], vec![3.0, 6.0]]
        );
    }
}
