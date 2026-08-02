pub struct MatrixData {
    pub width: usize,
    pub height: usize,
    pub values: Vec<f32>,
    pub dataset_name: String,
    pub max_timesteps: usize,
}

impl MatrixData {
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

        Ok(Self {
            width,
            height,
            values: raw_data,
            dataset_name: format!("Random Matrix ({}x{})", width, height),
            max_timesteps: 1,
        })
    }
}
