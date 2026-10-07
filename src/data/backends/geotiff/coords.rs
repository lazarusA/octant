//! Coordinate bounds and dimension extraction for GeoTIFF rasters.

use async_tiff::ImageFileDirectory;

use crate::data::CoordValues;
use std::collections::HashMap;

/// `GTRasterTypeGeoKey` value of rasters whose tie points mark pixel centers.
const RASTER_PIXEL_IS_POINT: u16 = 2;

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
    /// Rows run south to north (row 0 at `min_y`).
    pub y_ascending: bool,
    /// The model transformation rotates or shears the grid, so rows and columns follow no
    /// single x or y spacing.
    pub rotated: bool,
}

impl GeoSpatialBounds {
    /// Extract spatial bounds from an IFD's GeoTIFF tags.
    pub fn from_ifd(ifd: &ImageFileDirectory) -> Self {
        let size = (ifd.image_width() as f64, ifd.image_height() as f64);
        let keys = ifd.geo_key_directory();
        let is_geographic = keys.is_some_and(|g| g.geographic_type.is_some());
        let pixel_is_point = keys.is_some_and(|g| g.raster_type == Some(RASTER_PIXEL_IS_POINT));
        let bounds = match (ifd.model_pixel_scale(), ifd.model_tiepoint()) {
            (Some(scale), Some(tiepoint)) if scale.len() >= 2 && tiepoint.len() >= 6 => {
                Some(Self::from_tiepoint(scale, tiepoint, size, pixel_is_point))
            }
            _ => ifd
                .model_transformation()
                .filter(|m| m.len() >= 16)
                .map(|m| Self::from_transformation(m, size, pixel_is_point)),
        };
        match bounds {
            Some(b) => Self { is_geographic, ..b },
            None => Self {
                max_x: size.0,
                max_y: size.1,
                ..Self::default()
            },
        }
    }

    /// Bounds from `ModelPixelScale` and `ModelTiepoint`: a positive y scale (the norm)
    /// runs rows north to south. A tie point on a pixel center starts half a pixel early.
    fn from_tiepoint(
        scale: &[f64],
        tiepoint: &[f64],
        (width, height): (f64, f64),
        point: bool,
    ) -> Self {
        let center = if point { 0.5 } else { 0.0 };
        let (i, j) = (tiepoint[0] + center, tiepoint[1] + center);
        let x0 = tiepoint[3] - i * scale[0];
        let y0 = tiepoint[4] + j * scale[1];
        Self::georeferenced(
            (x0, x0 + width * scale[0]),
            (y0, y0 - height * scale[1]),
            scale[1] < 0.0,
            false,
        )
    }

    /// Bounds from a 4x4 `ModelTransformation`: rows run south to north when `m[5]` is
    /// positive, and `m[1]`/`m[4]` rotate the grid.
    fn from_transformation(m: &[f64], (width, height): (f64, f64), point: bool) -> Self {
        let c = if point { -0.5 } else { 0.0 };
        let at = |col: f64, row: f64| {
            (
                m[0] * (col + c) + m[1] * (row + c) + m[3],
                m[4] * (col + c) + m[5] * (row + c) + m[7],
            )
        };
        let ((x0, y0), (x1, y1)) = (at(0.0, 0.0), at(width, height));
        Self::georeferenced((x0, x1), (y0, y1), m[5] > 0.0, m[1] != 0.0 || m[4] != 0.0)
    }

    fn georeferenced(
        (x0, x1): (f64, f64),
        (y0, y1): (f64, f64),
        y_ascending: bool,
        rotated: bool,
    ) -> Self {
        Self {
            min_x: x0.min(x1),
            max_x: x0.max(x1),
            min_y: y0.min(y1),
            max_y: y0.max(y1),
            is_geographic: false,
            georeferenced: true,
            y_ascending,
            rotated,
        }
    }

    /// Whether `axes` gives pixel indices: without georeferencing, or on a rotated grid.
    fn index_axes(&self) -> bool {
        !self.georeferenced || self.rotated
    }

    /// The x coordinate of each of `width` columns: pixel centers west to east.
    pub fn x_axis(&self, width: usize) -> CoordValues {
        if self.index_axes() {
            return index_axis(width);
        }
        let dx = (self.max_x - self.min_x) / width.max(1) as f64;
        CoordValues::Regular {
            start: self.min_x + 0.5 * dx,
            step: dx,
            len: width,
        }
    }

    /// The y coordinate of each of `height` rows: pixel centers in row order.
    pub fn y_axis(&self, height: usize) -> CoordValues {
        if self.index_axes() {
            return index_axis(height);
        }
        let dy = (self.max_y - self.min_y) / height.max(1) as f64;
        let (start, step) = if self.y_ascending {
            (self.min_y + 0.5 * dy, dy)
        } else {
            (self.max_y - 0.5 * dy, -dy)
        };
        CoordValues::Regular {
            start,
            step,
            len: height,
        }
    }

    /// The `(x, y)` coordinates of a `width x height` raster: georeferenced pixel centers,
    /// or pixel indices without georeferencing or on a rotated grid.
    pub fn axes(&self, width: usize, height: usize) -> (CoordValues, CoordValues) {
        (self.x_axis(width), self.y_axis(height))
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

    /// The coordinates of columns `col_start..col_end`, matching [`Self::axes`].
    pub fn compute_x_coords(&self, width: usize, col_start: usize, col_end: usize) -> Vec<f64> {
        let x = self.x_axis(width);
        (col_start..col_end).filter_map(|c| x.number(c)).collect()
    }

    /// The coordinates of rows `row_start..row_end`, matching [`Self::axes`].
    pub fn compute_y_coords(&self, height: usize, row_start: usize, row_end: usize) -> Vec<f64> {
        let y = self.y_axis(height);
        (row_start..row_end).filter_map(|r| y.number(r)).collect()
    }
}

/// Pixel indices `0..len`.
fn index_axis(len: usize) -> CoordValues {
    CoordValues::Regular {
        start: 0.0,
        step: 1.0,
        len,
    }
}
