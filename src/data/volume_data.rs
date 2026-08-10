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
}
