//! A GeoTIFF's overviews as multiscale stacks.
//!
//! GeoTIFF inspection names IFD 0's variables `band_K` and `raster`, and
//! those of IFD `N` `overview_N/band_K` and `overview_N/raster`. Each base
//! variable and its namesakes in the reduced-resolution IFDs form a stack.
//! An overview covers the base image's extent, so its factor along each
//! dimension is the base length over its own, with no offset.

use crate::data::metadata::VariableInfo;

use super::types::{MultiscaleKind, ScaleLevel};

/// `NewSubfileType` bit: a reduced-resolution version of another image.
const REDUCED_RESOLUTION: u32 = 1;
/// `NewSubfileType` bit: a transparency mask for another image.
const MASK: u32 = 4;

/// What decides whether an IFD is an overview of IFD 0.
#[derive(Debug, Clone, Copy)]
pub struct IfdShape {
    pub new_subfile_type: Option<u32>,
    pub width: u64,
    pub height: u64,
    pub samples: u64,
}

/// Whether `ifd` is a reduced-resolution overview of `base`: flagged as
/// one by `NewSubfileType` and not a mask; files without the tag count an
/// IFD with the base's samples and a smaller size in both directions.
pub fn is_overview(base: &IfdShape, ifd: &IfdShape) -> bool {
    let smaller = ifd.width <= base.width
        && ifd.height <= base.height
        && (ifd.width, ifd.height) != (base.width, base.height);
    match ifd.new_subfile_type {
        Some(t) => t & REDUCED_RESOLUTION != 0 && t & MASK == 0 && smaller,
        None => smaller && ifd.samples == base.samples,
    }
}

/// The variable prefix of IFD `ifd_idx` (empty for IFD 0).
pub fn ifd_prefix(ifd_idx: usize) -> String {
    if ifd_idx == 0 {
        String::new()
    } else {
        format!("overview_{ifd_idx}/")
    }
}

/// Links every IFD-0 variable with its namesakes in the `overviews` IFDs.
pub fn link_overviews(variables: &mut [VariableInfo], overviews: &[usize]) {
    if overviews.is_empty() {
        return;
    }
    let bases: Vec<usize> = (0..variables.len())
        .filter(|&i| !variables[i].name.contains('/'))
        .collect();
    for base in bases {
        let levels = base_levels(variables, base, overviews);
        let v = &variables[base];
        let (name, dims) = (v.name.clone(), v.dimension_names.clone());
        super::try_link(
            name,
            dims,
            levels,
            MultiscaleKind::GeoTiffOverviews,
            variables,
        );
    }
}

/// The levels of IFD-0 variable `base`: itself, then each overview holding
/// a variable of the same name and rank, largest first.
fn base_levels(variables: &[VariableInfo], base: usize, overviews: &[usize]) -> Vec<ScaleLevel> {
    let v = &variables[base];
    let mut found: Vec<&VariableInfo> = overviews
        .iter()
        .filter_map(|&ifd| {
            let path = format!("{}{}", ifd_prefix(ifd), v.name);
            variables.iter().find(|o| o.name == path)
        })
        .filter(|o| o.shape.len() == v.shape.len())
        .collect();
    found.sort_by_key(|o| std::cmp::Reverse(o.shape.iter().product::<u64>()));
    std::iter::once(v)
        .chain(found)
        .map(|o| ScaleLevel {
            path: o.name.clone(),
            shape: o.shape.clone(),
            factor: v
                .shape
                .iter()
                .zip(&o.shape)
                .map(|(&b, &l)| {
                    if l == 0 {
                        f64::NAN
                    } else {
                        b as f64 / l as f64
                    }
                })
                .collect(),
            offset: vec![0.0; v.shape.len()],
        })
        .collect()
}
