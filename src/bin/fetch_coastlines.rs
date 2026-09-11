//! # fetch_coastlines
//!
//! A one-shot data preparation tool that downloads Natural Earth coastline GeoJSON
//! files from the official `nvkelso/natural-earth-vector` GitHub repository and
//! converts them into compact binary vertex buffers ready for GPU upload.
//!
//! ## Output format
//!
//! Each `.bin` file is a tightly-packed sequence of `f32` pairs `[lon, lat]` in
//! WGS-84 geographic coordinates (degrees). Line-string segments are separated by
//! a sentinel pair `[f32::NAN, f32::NAN]`, which the WGSL shader uses to emit
//! degenerate triangles that the GPU clips silently (no visible gap artifact when
//! using `LineStrip` topology with primitive restart).
//!
//! ## Usage
//!
//! ```sh
//! cargo run --bin fetch_coastlines
//! ```
//!
//! Output files written to `assets/coastlines/`:
//! - `coastline_110m.bin`  (~40 KB,  ~10 K vertex pairs)
//! - `coastline_50m.bin`   (~150 KB, ~60 K vertex pairs)
//! - `coastline_10m.bin`   (~1 MB,   ~500 K vertex pairs)

use std::{
    fs,
    io::{self, Write},
    path::Path,
    time::Instant,
};

// ---------------------------------------------------------------------------
// GeoJSON subset types — only the fields we actually read
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
struct FeatureCollection {
    features: Vec<Feature>,
}

#[derive(serde::Deserialize)]
struct Feature {
    geometry: Option<Geometry>,
}

#[derive(serde::Deserialize)]
struct Geometry {
    #[serde(rename = "type")]
    kind: String,
    /// `LineString`      → `[[lon, lat], ...]`
    /// `MultiLineString` → `[[[lon, lat], ...], ...]`
    coordinates: serde_json::Value,
}

// ---------------------------------------------------------------------------
// Scale descriptors
// ---------------------------------------------------------------------------

struct Scale {
    name: &'static str,
    url: &'static str,
}

const SCALES: &[Scale] = &[
    Scale {
        name: "110m",
        url: "https://raw.githubusercontent.com/nvkelso/natural-earth-vector/master/geojson/ne_110m_coastline.geojson",
    },
    Scale {
        name: "50m",
        url: "https://raw.githubusercontent.com/nvkelso/natural-earth-vector/master/geojson/ne_50m_coastline.geojson",
    },
    Scale {
        name: "10m",
        url: "https://raw.githubusercontent.com/nvkelso/natural-earth-vector/master/geojson/ne_10m_coastline.geojson",
    },
];

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    let out_dir = Path::new("assets/coastlines");
    if let Err(e) = fs::create_dir_all(out_dir) {
        eprintln!("Failed to create '{}': {e}", out_dir.display());
        std::process::exit(1);
    }

    for scale in SCALES {
        let t0 = Instant::now();
        print!("Fetching {} coastline ... ", scale.name);
        io::stdout().flush().ok();

        match fetch_and_convert(scale, out_dir) {
            Ok((pair_count, byte_len)) => {
                println!(
                    "done in {:.1}s  —  {pair_count} vertex pairs, {:.1} KB on disk",
                    t0.elapsed().as_secs_f32(),
                    byte_len as f32 / 1024.0,
                );
            }
            Err(e) => eprintln!("ERROR: {e}"),
        }
    }
}

// ---------------------------------------------------------------------------
// Fetch + convert pipeline
// ---------------------------------------------------------------------------

/// Downloads the GeoJSON for one scale, extracts all line vertices, and writes
/// the flat `f32` binary to `<out_dir>/coastline_<name>.bin`.
///
/// Returns `(vertex_pair_count, written_bytes)` on success.
fn fetch_and_convert(
    scale: &Scale,
    out_dir: &Path,
) -> Result<(usize, usize), Box<dyn std::error::Error>> {
    // 1. Download ---------------------------------------------------------------
    let body = reqwest::blocking::get(scale.url)?
        .error_for_status()?
        .text()?;

    // 2. Parse ------------------------------------------------------------------
    let collection: FeatureCollection = serde_json::from_str(&body)?;

    // 3. Extract vertices -------------------------------------------------------
    let mut verts: Vec<f32> = Vec::with_capacity(1 << 16);
    let mut pair_count = 0usize;

    for feature in &collection.features {
        let Some(geom) = &feature.geometry else {
            continue;
        };

        match geom.kind.as_str() {
            "LineString" => {
                push_line_string(&geom.coordinates, &mut verts, &mut pair_count);
                // NaN sentinel marks the end of this segment
                verts.push(f32::NAN);
                verts.push(f32::NAN);
            }
            "MultiLineString" => {
                let Some(rings) = geom.coordinates.as_array() else {
                    continue;
                };
                for ring in rings {
                    push_line_string(ring, &mut verts, &mut pair_count);
                    verts.push(f32::NAN);
                    verts.push(f32::NAN);
                }
            }
            other => {
                eprintln!("  warning: skipping unexpected geometry type '{other}'");
            }
        }
    }

    // 4. Write binary -----------------------------------------------------------
    let out_path = out_dir.join(format!("coastline_{}.bin", scale.name));
    let raw_bytes: &[u8] = bytemuck::cast_slice(&verts);
    fs::write(&out_path, raw_bytes)?;

    Ok((pair_count, raw_bytes.len()))
}

// ---------------------------------------------------------------------------
// Geometry helpers
// ---------------------------------------------------------------------------

/// Appends `[lon, lat]` `f32` pairs from a GeoJSON `LineString` coordinate
/// array into `verts`. Skips malformed points without panicking.
fn push_line_string(coords: &serde_json::Value, verts: &mut Vec<f32>, pair_count: &mut usize) {
    let Some(points) = coords.as_array() else {
        return;
    };
    for pt in points {
        let Some(xy) = pt.as_array() else { continue };
        let lon = xy.first().and_then(|v| v.as_f64());
        let lat = xy.get(1).and_then(|v| v.as_f64());
        let (Some(lon), Some(lat)) = (lon, lat) else {
            continue;
        };
        verts.push(lon as f32);
        verts.push(lat as f32);
        *pair_count += 1;
    }
}
