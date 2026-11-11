//! How an overlay lines up with the base layer's grid.

use super::VariableSelection;
use crate::app::DimConfig;
use crate::data::{CoordValues, VariableInfo};
use crate::i18n::{PlotKey, tr};
use crate::plots::PlotType;

/// Where an overlay falls on the base layer's canvas.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Alignment {
    /// Same spatial dimensions, sizes, window and coordinates as the base:
    /// drawn cell for cell over it.
    SameGrid,
    /// Both on longitude/latitude, on different grids; the overlay spans
    /// `lon` x `lat` (placed over the base in a later step).
    Geo { lon: (f64, f64), lat: (f64, f64) },
    /// The same grid shape, but no coordinates tie it to the base.
    IndexOnly,
    /// Nothing places it on the base layer's canvas.
    Incompatible,
    /// A label image over a sphere or surface: label images draw only on
    /// the heatmap.
    LabelsHeatmapOnly,
}

impl Alignment {
    /// Whether the overlay is drawn over the base layer.
    pub fn is_drawn(self) -> bool {
        self == Self::SameGrid
    }

    /// Why an overlay with this alignment is not drawn; `None` when it is.
    pub fn reason(self) -> Option<&'static str> {
        match self {
            Self::SameGrid => None,
            Self::Geo { .. } => Some(tr(PlotKey::AlignGeo)),
            Self::IndexOnly => Some(tr(PlotKey::AlignIndexOnly)),
            Self::Incompatible => Some(tr(PlotKey::AlignIncompatible)),
            Self::LabelsHeatmapOnly => Some(tr(PlotKey::LabelsHeatmapOnly)),
        }
    }
}

/// How one spatial axis of the base matches the overlay's.
#[derive(PartialEq)]
enum AxisMatch {
    Same,
    /// Same name and size; coordinates not known on both sides yet.
    Unknown,
    Differs,
}

/// Classifies `overlay` against the `base` selection, which must be a 2D
/// grid (an X and a Y dimension, or a 1D discrete grid) drawn as a heatmap,
/// sphere or surface to place anything.
pub fn classify(base: &VariableSelection, overlay: &VariableSelection) -> Alignment {
    let (Some(base_var), Some(overlay_var)) = (base.variable_info(), overlay.variable_info())
    else {
        return Alignment::Incompatible;
    };
    if !base.plot_type.draws_overlays() || overlay.plot_type != PlotType::Heatmap {
        return Alignment::Incompatible;
    }
    // Label images have no sphere or surface mesh.
    if base.plot_type != PlotType::Heatmap && overlay_var.label_info().is_some() {
        return Alignment::LabelsHeatmapOnly;
    }
    if let Some(grid) = DimConfig::grid_dim(&base.dim_config) {
        return match axis_match(base, base_var, overlay, overlay_var, grid) {
            AxisMatch::Same => Alignment::SameGrid,
            AxisMatch::Unknown => Alignment::IndexOnly,
            AxisMatch::Differs => Alignment::Incompatible,
        };
    }
    if DimConfig::grid_dim(&overlay.dim_config).is_some() {
        return Alignment::Incompatible;
    }
    let (Some(x), Some(y)) = (
        DimConfig::x_dim(&base.dim_config),
        DimConfig::y_dim(&base.dim_config),
    ) else {
        return Alignment::Incompatible;
    };
    let axes = [x, y].map(|dim| axis_match(base, base_var, overlay, overlay_var, dim));
    if axes.iter().all(|m| *m == AxisMatch::Same) {
        return Alignment::SameGrid;
    }
    if let Some((lon, lat)) = geo_extent(overlay, overlay_var) {
        return Alignment::Geo { lon, lat };
    }
    if !axes.contains(&AxisMatch::Differs) || same_shape(base_var, overlay_var, [x, y]) {
        return Alignment::IndexOnly;
    }
    Alignment::Incompatible
}

/// Matches base dimension `dim` with the overlay's dimension of the same name.
fn axis_match(
    base: &VariableSelection,
    base_var: &VariableInfo,
    overlay: &VariableSelection,
    overlay_var: &VariableInfo,
    dim: usize,
) -> AxisMatch {
    let Some(name) = base_var.dimension_names.get(dim) else {
        return AxisMatch::Differs;
    };
    let Some(odim) = dim_named(overlay_var, name) else {
        return AxisMatch::Differs;
    };
    let len = base_var.shape.get(dim).copied();
    if len.is_none()
        || len != overlay_var.shape.get(odim).copied()
        || base.dim_config.get(dim).map(|c| c.range)
            != overlay.dim_config.get(odim).map(|c| c.range)
    {
        return AxisMatch::Differs;
    }
    let len = len.unwrap_or(0) as usize;
    // The overlay's coordinates are keyed by its own spelling of the name.
    let overlay_name = overlay_var
        .dimension_names
        .get(odim)
        .map_or(name.as_str(), String::as_str);
    match (
        dim_coords(base, base_var, name),
        dim_coords(overlay, overlay_var, overlay_name),
    ) {
        (Some(a), Some(b)) if coords_match(a, b, len) => AxisMatch::Same,
        (Some(_), Some(_)) => AxisMatch::Differs,
        _ if same_dataset(base, overlay) => AxisMatch::Same,
        _ if healpix_match(base_var, overlay_var, name) => AxisMatch::Same,
        _ => AxisMatch::Unknown,
    }
}

/// The coordinates of `var`'s dimension `name`, once they arrived.
fn dim_coords<'a>(
    selection: &'a VariableSelection,
    var: &VariableInfo,
    name: &str,
) -> Option<&'a CoordValues> {
    selection
        .metadata
        .as_ref()?
        .get_dim_coords(Some(&var.name), name)
}

/// Whether `a` and `b` hold the same `len` coordinates (numbers within a
/// relative 1e-6, so f32 and f64 copies of one grid match).
fn coords_match(a: &CoordValues, b: &CoordValues, len: usize) -> bool {
    if a == b {
        return true;
    }
    if !a.matches(len) || !b.matches(len) {
        return false;
    }
    if let (Some(la), Some(lb)) = (a.labels(), b.labels()) {
        return la == lb;
    }
    (0..len).all(|i| match (a.number_for(i, len), b.number_for(i, len)) {
        (Some(x), Some(y)) => (x - y).abs() <= 1e-6 * x.abs().max(y.abs()).max(1.0),
        _ => false,
    })
}

/// The overlay's longitude and latitude extents, when both are numbers.
fn geo_extent(overlay: &VariableSelection, var: &VariableInfo) -> Option<((f64, f64), (f64, f64))> {
    use crate::data::coordinates::naming::{is_spatial_x_name, is_spatial_y_name};
    let meta = overlay.metadata.as_ref()?;
    let bounds = |is_axis: fn(&str) -> bool| {
        let name = var.dimension_names.iter().find(|n| is_axis(n))?;
        meta.get_coord_bounds_for_var(Some(&var.name), name)
    };
    Some((bounds(is_spatial_x_name)?, bounds(is_spatial_y_name)?))
}

/// Whether the overlay's last two dimensions have the sizes of the base's
/// `[x, y]` dimensions, in either order.
fn same_shape(base: &VariableInfo, overlay: &VariableInfo, [x, y]: [usize; 2]) -> bool {
    let size = |var: &VariableInfo, d: usize| var.shape.get(d).copied();
    let rank = overlay.shape.len();
    let (Some(a), Some(b)) = (rank.checked_sub(2), rank.checked_sub(1)) else {
        return false;
    };
    let base_xy = (size(base, x), size(base, y));
    let last = (size(overlay, a), size(overlay, b));
    base_xy == (last.1, last.0) || base_xy == last
}

fn healpix_match(base_var: &VariableInfo, overlay_var: &VariableInfo, name: &str) -> bool {
    use crate::data::coordinates::naming::is_healpix_dim_name;
    use crate::data::coordinates::ordering::detect_healpix_ordering_with_dggs;
    if !is_healpix_dim_name(name) {
        return false;
    }
    let base_order = detect_healpix_ordering_with_dggs(&base_var.attributes, None);
    let overlay_order = detect_healpix_ordering_with_dggs(&overlay_var.attributes, None);
    base_order == overlay_order
}

fn same_dataset(a: &VariableSelection, b: &VariableSelection) -> bool {
    a.store_kind == b.store_kind && a.store_target == b.store_target
}

/// The dimension of `var` named `name` (ASCII case-insensitive), or matching
/// HEALPix discrete cell dimension naming if `name` is a HEALPix dimension.
pub(crate) fn dim_named(var: &VariableInfo, name: &str) -> Option<usize> {
    var.dimension_names
        .iter()
        .position(|n| n.eq_ignore_ascii_case(name))
        .or_else(|| {
            crate::data::coordinates::naming::is_healpix_dim_name(name)
                .then(|| {
                    var.dimension_names
                        .iter()
                        .position(|n| crate::data::coordinates::naming::is_healpix_dim_name(n))
                })
                .flatten()
        })
}
