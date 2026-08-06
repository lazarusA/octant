/// Generates a synthetic 2D scalar field matrix for testing and fallback rendering.
pub fn generate_procedural_matrix(
    width: usize,
    height: usize,
    timestep: usize,
) -> (Vec<f32>, f32, f32) {
    let mut raw_data = Vec::with_capacity(width * height);
    let mut min_val = f32::INFINITY;
    let mut max_val = f32::NEG_INFINITY;

    for y in 0..height {
        for x in 0..width {
            let fx = x as f32 / width as f32;
            let fy = y as f32 / height as f32;
            let t_shift = (timestep % 365) as f32 * 0.05;
            let wave1 = ((fx * 8.0 + t_shift).sin() * (fy * 8.0).cos() * 0.5 + 0.5) * 80.0;
            let wave2 = (((x * 23 + y * 47) % 100) as f32) * 0.2;
            let val = (wave1 + wave2).clamp(0.0, 100.0);

            if val < min_val {
                min_val = val;
            }
            if val > max_val {
                max_val = val;
            }

            raw_data.push(val);
        }
    }

    if min_val > max_val {
        min_val = 0.0;
        max_val = 100.0;
    }

    (raw_data, min_val, max_val)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_procedural_matrix_bounds() {
        let (data, min_v, max_v) = generate_procedural_matrix(32, 32, 0);
        assert_eq!(data.len(), 32 * 32);
        assert!(min_v >= 0.0);
        assert!(max_v <= 100.0);
    }
}
