//! Types, bounds helpers, and uniform definitions for coastlines.

use bytemuck::{Pod, Zeroable};

// Force 4-byte alignment on the embedded byte blob so bytemuck::cast_slice
// can safely reinterpret it as &[f32] without panicking.
#[repr(C, align(4))]
pub(crate) struct Aligned4<T: ?Sized>(pub T);

/// Coastline level of detail.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CoastlineLod {
    /// 1:110 000 000 — ~5 K vertex pairs, 41 KB. Suitable for world-scale views.
    #[default]
    Lod110m,
    /// 1:50 000 000 — ~60 K vertex pairs, 483 KB. Good for continental views.
    Lod50m,
    /// 1:10 000 000 — ~411 K vertex pairs, 3.2 MB. Regional / zoomed-in views.
    Lod10m,
}

impl CoastlineLod {
    /// Chooses the appropriate LOD based on the current zoom level.
    pub fn from_zoom(zoom: f32) -> Self {
        if zoom < 4.0 {
            Self::Lod110m
        } else if zoom < 20.0 {
            Self::Lod50m
        } else {
            Self::Lod10m
        }
    }

    /// Returns the asset filename for this LOD.
    pub const fn filename(self) -> &'static str {
        match self {
            Self::Lod110m => "coastline_110m.bin",
            Self::Lod50m => "coastline_50m.bin",
            Self::Lod10m => "coastline_10m.bin",
        }
    }
}

pub type CoastlineFetchResult = Result<(CoastlineLod, Vec<f32>), String>;
pub type CoastlineReceiver = std::sync::mpsc::Receiver<CoastlineFetchResult>;
pub type CoastlineSender = std::sync::mpsc::Sender<CoastlineFetchResult>;

/// Holds coastline vertex data, either borrowed from static storage or owned.
#[derive(Clone)]
pub enum CoastlineBuffer {
    /// Points into the `'static` embedded binary — zero allocation.
    Static(&'static [f32]),
    /// Heap-allocated data loaded from disk or network.
    Owned(std::sync::Arc<[f32]>),
}

impl CoastlineBuffer {
    /// Returns a slice of `[lon, lat]` `f32` pairs.
    pub fn as_slice(&self) -> &[f32] {
        match self {
            Self::Static(s) => s,
            Self::Owned(v) => v.as_ref(),
        }
    }
}

// ---------------------------------------------------------------------------
// Uniform buffer layouts
// ---------------------------------------------------------------------------

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CoastlineUniforms {
    pub pan: [f32; 2],
    pub zoom: f32,
    pub crop_to_domain: u32,
    pub aspect_scale: [f32; 2],
    pub line_width: f32,
    pub _pad2: u32,
    pub line_color: [f32; 4],
    pub lon_min: f32,
    pub lon_max: f32,
    pub lat_min: f32,
    pub lat_max: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Coastline3DUniforms {
    pub rotation_y: f32,
    pub rotation_x: f32,
    pub aspect_ratio: f32,
    pub zoom: f32,
    pub displacement_strength: f32,
    pub plot_kind: u32,
    pub plot_mode: u32,
    pub line_width: f32,
    pub width: u32,
    pub height: u32,
    pub coord_mode: u32,
    pub crop_to_domain: u32,
    pub lon_bounds: [f32; 2],
    pub lat_bounds: [f32; 2],
    pub color: [f32; 4],
    pub _pad_color: [u32; 2],
    pub color_range: [f32; 2],
    pub _pad: [u32; 2],
    pub _pad_tail: [u32; 2],
}

#[derive(Copy, Clone, Debug)]
pub struct Coastline3DParams {
    pub rotation_y: f32,
    pub rotation_x: f32,
    pub aspect_ratio: f32,
    pub zoom: f32,
    pub displacement_strength: f32,
    pub plot_kind: u32,
    pub plot_mode: u32,
    pub line_width: f32,
    pub coord_mode: u32,
    pub crop_to_domain: u32,
    pub lon_bounds: [f32; 2],
    pub lat_bounds: [f32; 2],
    pub color: [f32; 4],
    pub color_range: [f32; 2],
}

// ---------------------------------------------------------------------------
// Coordinate-alignment helpers
// ---------------------------------------------------------------------------

/// Extracts the dataset's geographic bounds from a [`crate::data::CoordinateGrid`].
pub fn dataset_geo_bounds(grid: &crate::data::CoordinateGrid) -> (f32, f32, f32, f32) {
    use crate::data::CoordinateGrid;
    match grid {
        CoordinateGrid::GlobalRegular => (-180.0, 180.0, -90.0, 90.0),
        CoordinateGrid::RegionalRegular {
            lon_bounds,
            lat_bounds,
        }
        | CoordinateGrid::Irregular1D {
            lon_bounds,
            lat_bounds,
            ..
        }
        | CoordinateGrid::Curvilinear2D {
            lon_bounds,
            lat_bounds,
            ..
        } => (lon_bounds.0, lon_bounds.1, lat_bounds.0, lat_bounds.1),
    }
}
