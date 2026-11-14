//! The horizontal grids of a GRIB file and the variables describing them: regular lat/lon
//! grids by `latitude`/`longitude`, projected ones (Lambert conformal, polar stereographic,
//! Mercator) by `y`/`x` in metres and a CF grid mapping carrying the grid's proj string.
//! Values stay in stored order, which the coordinates follow.

use std::collections::HashMap;
use std::sync::Arc;

use gribberish::message_metadata::MessageMetadata;

use super::entry::{Entry, attrs};

/// One grid, shared by the messages on it.
pub struct Grid {
    pub ny: usize,
    pub nx: usize,
    /// Its first point (latitude, longitude), which tells grids of one shape and projection
    /// apart without computing every coordinate.
    start: (f64, f64),
    y: Vec<f64>,
    x: Vec<f64>,
    proj: String,
    projected: bool,
}

impl Grid {
    fn of(meta: &MessageMetadata) -> Self {
        let (x, y) = meta.xy();
        let x = match meta.is_regular_grid {
            true => continuous_longitudes(x),
            false => x,
        };
        Self {
            ny: meta.grid_shape.0,
            nx: meta.grid_shape.1,
            start: meta.projector.latlng_start(),
            y,
            x,
            proj: meta.proj.clone(),
            projected: !meta.is_regular_grid,
        }
    }

    /// Whether `meta`'s message lies on this grid.
    fn holds(&self, meta: &MessageMetadata) -> bool {
        (self.ny, self.nx) == meta.grid_shape
            && self.proj == meta.proj
            && self.start == meta.projector.latlng_start()
    }

    /// Its dimension names, `suffix` telling grids of one file apart (`""` for the first).
    pub fn dims(&self, suffix: &str) -> (String, String) {
        match self.projected {
            true => (format!("y{suffix}"), format!("x{suffix}")),
            false => (format!("latitude{suffix}"), format!("longitude{suffix}")),
        }
    }

    /// The grid mapping variable of a projected grid.
    pub fn crs_name(&self, suffix: &str) -> Option<String> {
        self.projected.then(|| format!("crs{suffix}"))
    }

    /// Its coordinate variables and, when projected, its grid mapping.
    pub fn entries(&self, suffix: &str) -> Vec<Entry> {
        let (ydim, xdim) = self.dims(suffix);
        let units = |unit, standard| attrs(&[("units", unit), ("standard_name", standard)]);
        let (yattrs, xattrs) = match self.projected {
            true => (
                units("m", "projection_y_coordinate"),
                units("m", "projection_x_coordinate"),
            ),
            false => (
                units("degrees_north", "latitude"),
                units("degrees_east", "longitude"),
            ),
        };
        let mut entries = vec![
            Entry::coordinate(&ydim, Arc::from(self.y.as_slice()), yattrs),
            Entry::coordinate(&xdim, Arc::from(self.x.as_slice()), xattrs),
        ];
        if let Some(name) = self.crs_name(suffix) {
            entries.push(Entry::mapping(&name, mapping_attributes(&self.proj)));
        }
        entries
    }
}

/// Longitudes without the wrap gribberish puts in them (it reads them into 0..360, so a
/// grid from 180 runs 180..359.75, 0..179.75): continuous along the row. A row crossing
/// 0/360 (first longitude past its last, ECMWF's 180 to 179.75) has no monotonic form within
/// 0..360, so it is shifted by -360 (-180..179.75); every other row keeps the file's values
/// (0..360, 260..310). A wrapped row looks descending and is mirrored to display.
pub fn continuous_longitudes(lons: Vec<f64>) -> Vec<f64> {
    let mut shift = 0.0;
    let mut out: Vec<f64> = Vec::with_capacity(lons.len());
    for lon in lons {
        let mut v = lon + shift;
        if let Some(&prev) = out.last() {
            let turn = match v - prev {
                d if d < -180.0 => 360.0,
                d if d > 180.0 => -360.0,
                _ => 0.0,
            };
            (shift, v) = (shift + turn, v + turn);
        }
        out.push(v);
    }
    if out.iter().any(|&v| v > 360.0) {
        out.iter_mut().for_each(|v| *v -= 360.0);
    }
    out
}

/// A CF grid mapping for proj string `proj`: its name and the string itself.
fn mapping_attributes(proj: &str) -> HashMap<String, String> {
    let name = match proj
        .split_whitespace()
        .find_map(|t| t.strip_prefix("+proj="))
    {
        Some("lcc") => "lambert_conformal_conic",
        Some("stere") => "polar_stereographic",
        Some("merc") => "mercator",
        _ => "unknown",
    };
    // gribberish leaves the `+` off some parameters (`lon_0=262.5`), which proj readers need.
    let proj4: Vec<String> = (proj.split_whitespace())
        .map(|t| match t.starts_with('+') {
            true => t.to_owned(),
            false => format!("+{t}"),
        })
        .collect();
    attrs(&[("grid_mapping_name", name), ("proj4", &proj4.join(" "))])
}

/// The grids of a file, each once.
#[derive(Default)]
pub struct Grids(pub Vec<Grid>);

impl Grids {
    /// The index of the grid `meta`'s message lies on, added when new.
    pub fn index_of(&mut self, meta: &MessageMetadata) -> usize {
        match self.0.iter().position(|g| g.holds(meta)) {
            Some(i) => i,
            None => {
                self.0.push(Grid::of(meta));
                self.0.len() - 1
            }
        }
    }

    /// The suffix of grid `i`'s names: none for the first, `_1`, `_2`, ... after.
    pub fn suffix(i: usize) -> String {
        match i {
            0 => String::new(),
            i => format!("_{i}"),
        }
    }
}
