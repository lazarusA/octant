//! Coordinate bounds and dimension extraction for GeoTIFF rasters.

use async_tiff::ImageFileDirectory;

use crate::data::CoordValues;
use std::collections::HashMap;

/// Spatial coordinate information extracted from an IFD.
#[derive(Debug, Clone, Default)]
pub struct GeoSpatialBounds {
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
    pub is_geographic: bool,
    /// The bounds come from GeoTIFF tags; otherwise they are the pixel grid itself.
    pub georeferenced: bool,
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
                georeferenced: true,
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
                georeferenced: true,
            };
        }

        Self {
            min_x: 0.0,
            max_x: width,
            min_y: 0.0,
            max_y: height,
            is_geographic: false,
            georeferenced: false,
        }
    }

    /// The `(x, y)` coordinates of a `width x height` raster: georeferenced pixel centers
    /// (rows from north to south), or pixel indices without georeferencing.
    pub fn axes(&self, width: usize, height: usize) -> (CoordValues, CoordValues) {
        if !self.georeferenced {
            let index = |len| CoordValues::Regular {
                start: 0.0,
                step: 1.0,
                len,
            };
            return (index(width), index(height));
        }
        let dx = (self.max_x - self.min_x) / width.max(1) as f64;
        let dy = (self.max_y - self.min_y) / height.max(1) as f64;
        let x = CoordValues::Regular {
            start: self.min_x + 0.5 * dx,
            step: dx,
            len: width,
        };
        let y = CoordValues::Regular {
            start: self.max_y - 0.5 * dy,
            step: -dy,
            len: height,
        };
        (x, y)
    }

    /// Generates dimension coordinate entries for `DatasetMetadata`, including the band
    /// names of a multi-band variable when it has them.
    pub fn populate_dimension_coordinates(
        &self,
        var_name: &str,
        (width, height): (usize, usize),
        band_labels: Option<&[String]>,
        coords: &mut HashMap<String, CoordValues>,
    ) {
        // Unscoped keys keep the full-resolution image's axes; overviews only add scoped ones.
        let mut insert = |name: &str, values: &CoordValues| {
            coords
                .entry(name.to_string())
                .or_insert_with(|| values.clone());
            coords.insert(format!("{var_name}/{name}"), values.clone());
        };
        if let Some(labels) = band_labels.and_then(|l| CoordValues::from_labels(l.to_vec())) {
            insert("band", &labels);
        }
        let (xc, yc) = self.axes(width, height);
        insert("x", &xc);
        insert("y", &yc);
        if self.is_geographic {
            insert("lon", &xc);
            insert("lat", &yc);
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
