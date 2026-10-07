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
        let (width, height) = (ifd.image_width() as f64, ifd.image_height() as f64);
        let is_geographic = ifd
            .geo_key_directory()
            .and_then(|g| g.geographic_type)
            .is_some();

        if let (Some(scale), Some(tiepoint)) = (ifd.model_pixel_scale(), ifd.model_tiepoint())
            && scale.len() >= 2
            && tiepoint.len() >= 6
        {
            let (i, j, origin_x, origin_y) = (tiepoint[0], tiepoint[1], tiepoint[3], tiepoint[4]);
            let (x0, x1) = (
                origin_x - i * scale[0],
                origin_x - i * scale[0] + width * scale[0],
            );
            let (y0, y1) = (
                origin_y + j * scale[1],
                origin_y + j * scale[1] - height * scale[1],
            );
            return Self {
                min_x: x0.min(x1),
                max_x: x0.max(x1),
                min_y: y0.min(y1),
                max_y: y0.max(y1),
                is_geographic,
            };
        }

        if let Some(m) = ifd.model_transformation()
            && m.len() >= 16
        {
            let (x0, y0) = (m[3], m[7]);
            let (x1, y1) = (
                m[0] * width + m[1] * height + m[3],
                m[4] * width + m[5] * height + m[7],
            );
            return Self {
                min_x: x0.min(x1),
                max_x: x0.max(x1),
                min_y: y0.min(y1),
                max_y: y0.max(y1),
                is_geographic,
            };
        }

        Self {
            min_x: 0.0,
            max_x: width,
            min_y: 0.0,
            max_y: height,
            is_geographic: false,
        }
    }

    /// Generates dimension coordinate entries for `DatasetMetadata`, including the band
    /// names of a multi-band variable when it has them.
    pub fn populate_dimension_coordinates(
        &self,
        var_name: &str,
        band_labels: Option<&[String]>,
        coords: &mut HashMap<String, Vec<String>>,
    ) {
        if let Some(labels) = band_labels {
            coords.insert("band".into(), labels.to_vec());
            coords.insert(format!("{var_name}/band"), labels.to_vec());
        }
        let (xc, yc) = (
            vec![self.min_x.to_string(), self.max_x.to_string()],
            vec![self.min_y.to_string(), self.max_y.to_string()],
        );
        coords.insert("x".into(), xc.clone());
        coords.insert("y".into(), yc.clone());
        coords.insert(format!("{var_name}/x"), xc.clone());
        coords.insert(format!("{var_name}/y"), yc.clone());
        if self.is_geographic {
            coords.insert("lon".into(), xc.clone());
            coords.insert("lat".into(), yc.clone());
            coords.insert(format!("{var_name}/lon"), xc);
            coords.insert(format!("{var_name}/lat"), yc);
        }
    }

    /// Compute coordinate vector for a sliced window `[col_start..col_end]` along X.
    pub fn compute_x_coords(&self, width: usize, col_start: usize, col_end: usize) -> Vec<f64> {
        let total = width.max(1) as f64;
        (col_start..col_end)
            .map(|c| self.min_x + (c as f64 / total) * (self.max_x - self.min_x))
            .collect()
    }

    /// Compute coordinate vector for a sliced window `[row_start..row_end]` along Y.
    pub fn compute_y_coords(&self, height: usize, row_start: usize, row_end: usize) -> Vec<f64> {
        let total = height.max(1) as f64;
        (row_start..row_end)
            .map(|r| self.max_y - (r as f64 / total) * (self.max_y - self.min_y))
            .collect()
    }
}
