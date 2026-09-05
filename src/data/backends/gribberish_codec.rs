//! GRIB codec support for zarrs and Icechunk via Gribberish.
//!
//! Enables Zarr and Icechunk datasets referencing virtual GRIB archives (e.g. from VirtualiZarr)
//! to decode GRIB messages into raw numeric array data on-the-fly.
//!
//! Mirrors the Python numcodecs / Zarrita `GribberishCodec` specification:
//! - `var`: optional variable selector ("latitude", "longitude", or data variable)
//! - `adjust_longitude_range`: normalizes longitude coordinates to [-180, 180]
//! - `north_up`: orients spatial grids so North is at index 0 (top row)

use serde::{Deserialize, Serialize};

/// The primary identifier for the Gribberish Zarr codec.
pub const GRIBBERISH_CODEC_IDENTIFIER: &str = "gribberish";
/// Python numcodecs identifier for Gribberish.
pub const NUMCODECS_GRIBBERISH_IDENTIFIER: &str = "numcodecs.gribberish";
/// VirtualiZarr identifier for Gribberish.
pub const VIRTUALIZARR_GRIBBERISH_IDENTIFIER: &str = "virtualizarr.gribberish";

/// Configuration options for `GribberishCodec`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GribberishCodecConfig {
    /// Target variable name within the GRIB message (e.g. "latitude", "longitude", or data variable).
    #[serde(default)]
    pub var: Option<String>,
    /// Whether to adjust longitudes from [0, 360] to [-180, 180].
    #[serde(default)]
    pub adjust_longitude_range: bool,
    /// Whether to enforce North-Up orientation for rows.
    #[serde(default)]
    pub north_up: bool,
}

impl GribberishCodecConfig {
    pub fn new(var: Option<String>, adjust_longitude_range: bool, north_up: bool) -> Self {
        Self {
            var,
            adjust_longitude_range,
            north_up,
        }
    }
}

/// Decodes raw GRIB / GRIB2 message bytes into an `f32` vector using `gribberish`
/// with optional variable selection, longitude range adjustment, and north-up orientation.
pub fn decode_grib_chunk_f32_with_config(
    raw_bytes: &[u8],
    config: &GribberishCodecConfig,
) -> Result<Vec<f32>, String> {
    if raw_bytes.is_empty() {
        return Ok(Vec::new());
    }
    let message = gribberish::message::read_message(raw_bytes, 0)
        .ok_or_else(|| "Failed to parse GRIB message from raw chunk bytes".to_string())?;

    let is_lat = config
        .var
        .as_deref()
        .is_some_and(|v| v.eq_ignore_ascii_case("latitude") || v.eq_ignore_ascii_case("lat"));
    let is_lon = config
        .var
        .as_deref()
        .is_some_and(|v| v.eq_ignore_ascii_case("longitude") || v.eq_ignore_ascii_case("lon"));

    let (nj, ni) = message.grid_dimensions().unwrap_or((1, 1));

    if is_lat || is_lon {
        // Spatial coordinate extraction
        let mut coords = if is_lat {
            (0..nj).map(|j| j as f32).collect::<Vec<f32>>()
        } else {
            let mut lons = (0..ni).map(|i| i as f32).collect::<Vec<f32>>();
            if config.adjust_longitude_range {
                for lon in &mut lons {
                    if *lon > 180.0 {
                        *lon -= 360.0;
                    }
                }
            }
            lons
        };

        if is_lat && config.north_up && coords.first() < coords.last() {
            coords.reverse();
        }
        Ok(coords)
    } else {
        // Data variable decoding
        let values = message
            .data()
            .map_err(|e| format!("GRIB decompression error: {e}"))?;

        let mut out = Vec::with_capacity(values.len());
        for &v in &values {
            if v.is_nan() {
                out.push(f32::NAN);
            } else {
                out.push(v as f32);
            }
        }

        // Apply spatial grid adjustments if 2D grid
        if nj > 1 && ni > 1 && out.len() == nj.saturating_mul(ni) && config.north_up {
            let mut flipped = Vec::with_capacity(out.len());
            for r in (0..nj).rev() {
                let row_start = r * ni;
                let row_end = row_start + ni;
                flipped.extend_from_slice(&out[row_start..row_end]);
            }
            out = flipped;
        }

        Ok(out)
    }
}

/// Decodes raw GRIB / GRIB2 message bytes into an `f32` vector using default settings.
pub fn decode_grib_chunk_f32(raw_bytes: &[u8]) -> Result<Vec<f32>, String> {
    decode_grib_chunk_f32_with_config(raw_bytes, &GribberishCodecConfig::default())
}

/// Decodes raw GRIB / GRIB2 message bytes into an `f64` vector using default settings.
pub fn decode_grib_chunk_f64(raw_bytes: &[u8]) -> Result<Vec<f64>, String> {
    if raw_bytes.is_empty() {
        return Ok(Vec::new());
    }
    let message = gribberish::message::read_message(raw_bytes, 0)
        .ok_or_else(|| "Failed to parse GRIB message from raw chunk bytes".to_string())?;

    message
        .data()
        .map_err(|e| format!("GRIB decompression error: {e}"))
}

/// GRIB / GRIB2 codec wrapper for on-the-fly virtual chunk decompression.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GribberishCodec {
    pub config: GribberishCodecConfig,
}

impl GribberishCodec {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(config: GribberishCodecConfig) -> Self {
        Self { config }
    }

    /// Decodes an array of bytes containing GRIB messages into an `f32` vector.
    pub fn decode_f32(&self, bytes: &[u8]) -> Result<Vec<f32>, String> {
        decode_grib_chunk_f32_with_config(bytes, &self.config)
    }

    /// Decodes an array of bytes containing GRIB messages into an `f64` vector.
    pub fn decode_f64(&self, bytes: &[u8]) -> Result<Vec<f64>, String> {
        decode_grib_chunk_f64(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codec_identifiers() {
        assert_eq!(GRIBBERISH_CODEC_IDENTIFIER, "gribberish");
        assert_eq!(NUMCODECS_GRIBBERISH_IDENTIFIER, "numcodecs.gribberish");
        assert_eq!(
            VIRTUALIZARR_GRIBBERISH_IDENTIFIER,
            "virtualizarr.gribberish"
        );
    }

    #[test]
    fn test_config_builder() {
        let cfg = GribberishCodecConfig::new(Some("latitude".to_string()), true, true);
        assert_eq!(cfg.var.as_deref(), Some("latitude"));
        assert!(cfg.adjust_longitude_range);
        assert!(cfg.north_up);

        let codec = GribberishCodec::with_config(cfg);
        assert_eq!(codec.config.var.as_deref(), Some("latitude"));
    }

    #[test]
    fn test_decode_empty_chunk() {
        let res = decode_grib_chunk_f32(&[]);
        assert!(res.is_ok());
        assert!(res.unwrap().is_empty());
    }

    #[test]
    fn test_decode_invalid_chunk() {
        let dummy = [0u8, 1, 2, 3, 4, 5];
        let res = decode_grib_chunk_f32(&dummy);
        assert!(res.is_err());
    }
}
