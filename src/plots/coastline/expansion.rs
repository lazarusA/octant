//! Polyline expansion with on-the-fly antimeridian segment splitting.

/// Expands a stream of `[lon, lat]` points (separated by NaN sentinels) into
/// a flat line-list `[x0, y0, x1, y1, ...]`.
///
/// Segments that cross the $\pm 180^\circ$ antimeridian ($|\Delta lon| > 180^\circ$)
/// are dynamically split at the $180^\circ$ boundary on the fly.
pub fn expand_coastline_line_list(vertices: &[f32]) -> Vec<f32> {
    let mut expanded = Vec::with_capacity(vertices.len().saturating_mul(2));
    let mut prev_pt: Option<(f32, f32)> = None;

    for pair in vertices.chunks_exact(2) {
        let (lon, lat) = (pair[0], pair[1]);
        if !lon.is_finite() || !lat.is_finite() {
            prev_pt = None;
            continue;
        }

        if let Some(p0) = prev_pt {
            push_segment_with_antimeridian_split(&mut expanded, p0, (lon, lat));
        }
        prev_pt = Some((lon, lat));
    }

    expanded
}

/// Emits line-list segments, splitting on the fly if crossing the $\pm 180^\circ$ meridian.
fn push_segment_with_antimeridian_split(out: &mut Vec<f32>, p0: (f32, f32), p1: (f32, f32)) {
    let dlon = p1.0 - p0.0;
    if dlon.abs() <= 180.0 {
        out.extend_from_slice(&[p0.0, p0.1, p1.0, p1.1]);
        return;
    }

    // Antimeridian crossing detected: interpolate intersection at +/- 180
    if p0.0 > 0.0 && p1.0 < 0.0 {
        // West to East crossing (e.g., +179 -> -179 => total wrap distance is (180 - 179) + (179 - (-180)))
        let span = (180.0 - p0.0) + (p1.0 + 180.0);
        let t = if span.abs() > 1e-6 {
            (180.0 - p0.0) / span
        } else {
            0.5
        };
        let lat_mid = p0.1 + t * (p1.1 - p0.1);

        out.extend_from_slice(&[p0.0, p0.1, 180.0, lat_mid]);
        out.extend_from_slice(&[-180.0, lat_mid, p1.0, p1.1]);
    } else if p0.0 < 0.0 && p1.0 > 0.0 {
        // East to West crossing (e.g., -179 -> +179)
        let span = (p0.0 + 180.0) + (180.0 - p1.0);
        let t = if span.abs() > 1e-6 {
            (p0.0 + 180.0) / span
        } else {
            0.5
        };
        let lat_mid = p0.1 + t * (p1.1 - p0.1);

        out.extend_from_slice(&[p0.0, p0.1, -180.0, lat_mid]);
        out.extend_from_slice(&[180.0, lat_mid, p1.0, p1.1]);
    } else {
        out.extend_from_slice(&[p0.0, p0.1, p1.0, p1.1]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_segment_expansion() {
        let input = [
            10.0,
            20.0,
            11.0,
            21.0,
            f32::NAN,
            f32::NAN,
            30.0,
            40.0,
            31.0,
            41.0,
        ];
        let result = expand_coastline_line_list(&input);
        assert_eq!(result, vec![10.0, 20.0, 11.0, 21.0, 30.0, 40.0, 31.0, 41.0]);
    }

    #[test]
    fn test_antimeridian_split() {
        let input = [179.0, 10.0, -179.0, 20.0];
        let result = expand_coastline_line_list(&input);
        assert_eq!(result.len(), 8); // Two segments (4 vertices -> 8 floats)
        assert_eq!(result[0], 179.0);
        assert_eq!(result[1], 10.0);
        assert_eq!(result[2], 180.0);
        assert_eq!(result[3], 15.0); // Midpoint latitude
        assert_eq!(result[4], -180.0);
        assert_eq!(result[5], 15.0);
        assert_eq!(result[6], -179.0);
        assert_eq!(result[7], 20.0);
    }
}
