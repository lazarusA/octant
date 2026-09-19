//! Coordinate bounds and dimension extraction for GeoTIFF rasters.

use async_tiff::ImageFileDirectory;
use std::collections::HashMap;

/// Spatial coordinate information extracted from an IFD.
#[derive(Debug, Clone, Default)]
pub struct GeoSpatialBounds {
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
    pub is_geographic: bool,
}

impl GeoSpatialBounds {
    /// Extract spatial bounds from an IFD's GeoTIFF tags.
    pub fn from_ifd(ifd: &ImageFileDirectory) -> Self {
        let width = ifd.image_width() as f64;
        let height = ifd.image_height() as f64;

        let is_geographic = ifd
            .geo_key_directory()
            .and_then(|g| g.geographic_type)
            .is_some();

        if let (Some(scale), Some(tiepoint)) = (ifd.model_pixel_scale(), ifd.model_tiepoint())
            && scale.len() >= 2
            && tiepoint.len() >= 6
        {
            let (i, j, origin_x, origin_y) = (tiepoint[0], tiepoint[1], tiepoint[3], tiepoint[4]);
            let (scale_x, scale_y) = (scale[0], scale[1]);

            let x0 = origin_x - i * scale_x;
            let x1 = x0 + width * scale_x;

            let y0 = origin_y + j * scale_y;
            let y1 = y0 - height * scale_y;

            let min_x = x0.min(x1);
            let max_x = x0.max(x1);
            let min_y = y0.min(y1);
            let max_y = y0.max(y1);

            return Self {
                min_x,
                max_x,
                min_y,
                max_y,
                is_geographic,
            };
        }

        if let Some(m) = ifd.model_transformation()
            && m.len() >= 16
        {
            let x0 = m[3];
            let y0 = m[7];
            let x1 = m[0] * width + m[1] * height + m[3];
            let y1 = m[4] * width + m[5] * height + m[7];

            return Self {
                min_x: x0.min(x1),
                max_x: x0.max(x1),
                min_y: y0.min(y1),
                max_y: y0.max(y1),
                is_geographic,
            };
        }

        // Fallback to raster pixel index coordinates
        Self {
            min_x: 0.0,
            max_x: width,
            min_y: 0.0,
            max_y: height,
            is_geographic: false,
        }
    }

    /// Generates dimension coordinate entries for `DatasetMetadata`.
    pub fn populate_dimension_coordinates(
        &self,
        var_name: &str,
        dimension_coordinates: &mut HashMap<String, Vec<String>>,
    ) {
        let x_coords = vec![self.min_x.to_string(), self.max_x.to_string()];
        let y_coords = vec![self.min_y.to_string(), self.max_y.to_string()];

        dimension_coordinates.insert("x".to_string(), x_coords.clone());
        dimension_coordinates.insert("y".to_string(), y_coords.clone());
        dimension_coordinates.insert(format!("{var_name}/x"), x_coords.clone());
        dimension_coordinates.insert(format!("{var_name}/y"), y_coords.clone());

        if self.is_geographic {
            dimension_coordinates.insert("lon".to_string(), x_coords.clone());
            dimension_coordinates.insert("lat".to_string(), y_coords.clone());
            dimension_coordinates.insert(format!("{var_name}/lon"), x_coords);
            dimension_coordinates.insert(format!("{var_name}/lat"), y_coords);
        }
    }

    /// Compute coordinate vector for a sliced window `[col_start..col_end]` along X.
    pub fn compute_x_coords(&self, width: usize, col_start: usize, col_end: usize) -> Vec<f64> {
        let count = col_end.saturating_sub(col_start).max(1);
        let total_w = width.max(1) as f64;
        let mut result = Vec::with_capacity(count);

        for c in col_start..col_end {
            let t = c as f64 / total_w;
            result.push(self.min_x + t * (self.max_x - self.min_x));
        }
        result
    }

    /// Compute coordinate vector for a sliced window `[row_start..row_end]` along Y.
    pub fn compute_y_coords(&self, height: usize, row_start: usize, row_end: usize) -> Vec<f64> {
        let count = row_end.saturating_sub(row_start).max(1);
        let total_h = height.max(1) as f64;
        let mut result = Vec::with_capacity(count);

        for r in row_start..row_end {
            let t = r as f64 / total_h;
            result.push(self.max_y - t * (self.max_y - self.min_y));
        }
        result
    }
}
