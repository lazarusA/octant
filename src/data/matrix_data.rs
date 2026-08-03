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
    pub fn create_random_matrix(width: usize, height: usize) -> Result<Self, Box<dyn std::error::Error>> {
        let mut raw_data = Vec::with_capacity(width * height);
        for y in 0..height {
            for x in 0..width {
                let fx = x as f32 / width as f32;
                let fy = y as f32 / height as f32;
                let wave1 = (fx * 14.2 + fy * 9.7).sin() * 0.5 + 0.5;
                let wave2 = ((fx * 28.4 - fy * 19.1).cos() * 0.5 + 0.5) * 0.5;
                let hash = (((x * 1597 + y * 28491) % 1000) as f32 / 1000.0) * 0.25;
                let val = ((wave1 * 0.5 + wave2 + hash) * 100.0).clamp(0.0, 100.0);
                raw_data.push(val);
            }
        }

        Ok(Self::new(
            width,
            height,
            raw_data,
            0.0,
            100.0,
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
}
