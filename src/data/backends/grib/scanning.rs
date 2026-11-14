//! Grids whose adjacent rows scan in opposite directions (GRIB2 scanning mode flag `0x10`,
//! NOAA's NBM): gribberish reads the flag but decodes their values in stored order, every
//! second row reversed, which shows the field overlaid with its mirror image in stripes.

use gribberish::templates::grid_definition::{
    lambert_conformal_template::LambertConformalTemplate, latlng_template::LatLngTemplate,
    mercator_template::MercatorTemplate, polar_stereographic_template::PolarStereographicTemplate,
    tables::ScanningMode,
};

use super::sections::section_at;

/// Whether the rows of `message`'s grid alternate direction, by the scanning mode of the
/// grid templates gribberish decodes (lat/lon, Mercator, polar stereographic, Lambert).
/// Asked only of messages the scan described, whose grid section gribberish has already
/// read through these templates, so its bytes are all there.
pub fn alternates(message: &[u8]) -> bool {
    let Some(grid) = section_at(message, 3)
        .and_then(|(at, len)| message.get(at..at.checked_add(len)?))
        .map(<[u8]>::to_vec)
    else {
        return false;
    };
    let template = grid.get(12..14).map(|t| u16::from_be_bytes([t[0], t[1]]));
    let flags = match template {
        Some(0) => LatLngTemplate::new(grid).scanning_mode_flags(),
        Some(10) => MercatorTemplate::new(grid).scanning_mode_flags(),
        Some(20) => PolarStereographicTemplate::new(grid).scanning_mode_flags(),
        Some(30) => LambertConformalTemplate::new(grid).scanning_mode_flags(),
        _ => return false,
    };
    flags[3] == ScanningMode::AdjecentOppositeDirection
}

/// Puts the rows (`nx` values each) that scan backwards the way the first one scans.
pub fn unreverse(values: &mut [f64], nx: usize) {
    if nx == 0 {
        return;
    }
    for row in values.chunks_exact_mut(nx).skip(1).step_by(2) {
        row.reverse();
    }
}
