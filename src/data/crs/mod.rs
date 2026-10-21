//! Coordinate reference systems: what a variable's metadata says its x/y
//! coordinates are (`CrsRef`, found by `discover` from its attributes) and
//! what that resolves to (`resolve`: a proj string read by `proj4rs`).
//!
//! Backends put what they know into the variable's attributes (GeoTIFF its
//! GeoKeys' EPSG code as `proj:code`, CF `grid_mapping` variables their
//! `crs_wkt`/`spatial_ref`/`epsg_code` through `attach_grid_mappings`, Zarr
//! groups their `proj:*` keys by inheritance); `assign` then reads them once
//! per dataset, in `StoreHandle::inspect`.

#[cfg(test)]
mod network_tests;
mod resolve;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod zarr_tests;

#[cfg(target_arch = "wasm32")]
pub use resolve::set_wake;
pub use resolve::{Crs, Resolution, resolve};

use std::collections::HashMap;
use std::sync::Arc;

use crate::data::coordinates::naming::contains_ascii_case_insensitive;
use crate::data::metadata::{DatasetMetadata, VariableInfo};

/// What a variable's metadata says its CRS is.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CrsRef {
    /// lon/lat axes without an explicit CRS.
    Geographic,
    Epsg(u32),
    /// WKT1 or WKT2.
    Wkt(Arc<str>),
}

/// The attributes naming a CRS, read in this order: EPSG codes first (canonical
/// and short), then WKT.
const EPSG_KEYS: [&str; 3] = ["proj:code", "proj:epsg", "epsg_code"];
const WKT_KEYS: [&str; 3] = ["proj:wkt2", "crs_wkt", "spatial_ref"];

/// The CRS `attrs` name, else `Geographic` when `dims` hold a longitude and
/// a latitude, else `None` (never assumed to be degrees).
pub fn discover(attrs: &HashMap<String, String>, dims: &[String]) -> Option<CrsRef> {
    let epsg = EPSG_KEYS.iter().find_map(|k| epsg_code(attrs.get(*k)?));
    if let Some(code) = epsg {
        return Some(CrsRef::Epsg(code));
    }
    let wkt = WKT_KEYS
        .iter()
        .find_map(|k| attrs.get(*k).filter(|w| is_wkt(w)));
    if let Some(wkt) = wkt {
        return Some(wkt_epsg(wkt).map_or_else(|| CrsRef::Wkt(wkt.as_str().into()), CrsRef::Epsg));
    }
    let named = |part| {
        dims.iter()
            .any(|d| contains_ascii_case_insensitive(d, part))
    };
    (named("lon") && named("lat")).then_some(CrsRef::Geographic)
}

/// The EPSG code in `text`: `EPSG:n`, a bare `n`, or an OGC URL ending in
/// `/EPSG/0/n`.
fn epsg_code(text: &str) -> Option<u32> {
    let text = text.trim();
    let code = text
        .rsplit_once("EPSG/0/")
        .map(|(_, c)| c)
        .or_else(|| text.strip_prefix("EPSG:"))
        .or_else(|| text.strip_prefix("epsg:"))
        .unwrap_or(text);
    code.trim_end_matches('/').parse().ok()
}

/// The EPSG code a WKT gives its whole CRS: its last `AUTHORITY["EPSG","n"]`
/// (WKT1) or `ID["EPSG",n]` (WKT2), when that sits directly in the outermost
/// brackets (the ones inside name its datum, units and so on).
fn wkt_epsg(wkt: &str) -> Option<u32> {
    let at = ["AUTHORITY[\"EPSG\",", "ID[\"EPSG\","]
        .iter()
        .filter_map(|tag| wkt.rfind(tag).map(|i| (i, i + tag.len())))
        .max()?;
    let depth = wkt[..at.0].chars().fold(0i32, |d, c| match c {
        '[' => d + 1,
        ']' => d - 1,
        _ => d,
    });
    let digits = wkt[at.1..].trim_start_matches('"');
    let end = digits.find(|c: char| !c.is_ascii_digit())?;
    (depth == 1).then(|| digits[..end].parse().ok()).flatten()
}

fn is_wkt(text: &str) -> bool {
    text.trim_start()
        .starts_with(|c: char| c.is_ascii_uppercase())
        && text.contains('[')
}

/// The name a WKT gives its CRS: the first quoted string (`PROJCS["WGS 84 /
/// UTM zone 36N", ...`).
pub fn wkt_name(wkt: &str) -> Option<&str> {
    let (_, rest) = wkt.split_once("[\"")?;
    rest.split_once('"').map(|(name, _)| name)
}

/// The name `var`'s metadata gives its CRS: a WKT's, else a GeoTIFF citation.
pub fn named(var: &VariableInfo) -> Option<&str> {
    let wkt = WKT_KEYS
        .iter()
        .find_map(|k| wkt_name(var.attributes.get(*k)?));
    wkt.or_else(|| var.attributes.get("crs_citation").map(String::as_str))
}

/// The paths `var`'s CF grid mapping (`grid_mapping`, the first name of its
/// `"crs: x y"` form) may sit at: its group's, then the root's.
fn mapping_paths(var: &VariableInfo) -> Vec<String> {
    let Some(name) = var.attributes.get("grid_mapping") else {
        return Vec::new();
    };
    let name = name.split([':', ' ']).next().unwrap_or_default().trim();
    let scoped = var.group_path().map(|g| format!("{g}/{name}"));
    scoped.into_iter().chain([name.to_string()]).collect()
}

/// Copies the CRS keys of each variable's CF grid mapping onto the variable
/// where it has none of its own; `mapping` reads a mapping variable's
/// attributes by path (`mapping_paths`).
pub fn attach_grid_mappings(
    variables: &mut [VariableInfo],
    mapping: impl Fn(&str) -> Option<HashMap<String, String>>,
) {
    for var in variables {
        let Some(source) = mapping_paths(var).iter().find_map(|p| mapping(p)) else {
            continue;
        };
        fill_crs_keys(&mut var.attributes, |k| source.get(k).cloned());
    }
}

/// Sets each CRS key `get` gives that `attrs` lacks.
fn fill_crs_keys(attrs: &mut HashMap<String, String>, get: impl Fn(&str) -> Option<String>) {
    for key in WKT_KEYS.iter().chain(&EPSG_KEYS) {
        if !attrs.contains_key(*key)
            && let Some(value) = get(key)
        {
            attrs.insert((*key).to_string(), value);
        }
    }
}

/// [`attach_grid_mappings`] reading Zarr nodes' attributes through `reader`.
pub fn attach_zarr_grid_mappings(
    reader: &crate::data::ome::MetaReader<'_>,
    variables: &mut [VariableInfo],
) {
    use crate::utils::metadata::cf::attr_string;
    attach_grid_mappings(variables, |path| {
        let attrs = reader.group_attributes(path).ok().flatten()?;
        Some(
            attrs
                .iter()
                .map(|(k, v)| (k.clone(), attr_string(v)))
                .collect(),
        )
    });
}

/// Copies the CRS keys of each variable's ancestor groups (nearest first)
/// onto the variable where it has none of its own, for paths that build
/// variables from their arrays alone (the web's Icechunk).
pub fn inherit_zarr_group_crs(
    reader: &crate::data::ome::MetaReader<'_>,
    variables: &mut [VariableInfo],
) {
    use crate::utils::metadata::cf::attr_string;
    for var in variables {
        for group in crate::utils::path::ancestor_paths(&var.name) {
            let Ok(Some(attrs)) = reader.group_attributes(group) else {
                continue;
            };
            fill_crs_keys(&mut var.attributes, |k| attrs.get(k).map(attr_string));
        }
    }
}

/// [`attach_zarr_grid_mappings`] over `storage`, every mapping read
/// concurrently on the desktop.
pub fn attach_zarr_grid_mappings_in(
    storage: Arc<dyn zarrs::storage::ReadableStorageTraits>,
    variables: &mut [VariableInfo],
) {
    #[cfg(not(target_arch = "wasm32"))]
    crate::data::ome::direct::walk_concurrent(storage, 3, |reader| {
        let pending = (variables.iter())
            .flat_map(mapping_paths)
            .any(|p| reader.group_attributes(&p).is_err());
        (!pending).then(|| attach_zarr_grid_mappings(reader, variables))
    });
    #[cfg(target_arch = "wasm32")]
    attach_zarr_grid_mappings(&crate::data::ome::MetaReader::direct(storage), variables);
}

/// Sets every variable's CRS from its attributes; levels of a multiscale
/// stack without one of their own (or only guessed from their names) take
/// level 0's (GeoTIFF overviews carry no GeoKeys).
pub fn assign(meta: &mut DatasetMetadata) {
    for var in &mut meta.variables {
        var.crs = discover(&var.attributes, &var.dimension_names);
    }
    let by_name: HashMap<&str, &VariableInfo> = (meta.variables.iter())
        .map(|v| (v.name.as_str(), v))
        .collect();
    let base_crs: Vec<Option<CrsRef>> = (meta.variables.iter())
        .map(|v| {
            let guessed = matches!(v.crs, None | Some(CrsRef::Geographic));
            let base = &v.multiscale.as_ref().filter(|_| guessed)?.set;
            by_name.get(base.base().path.as_str())?.crs.clone()
        })
        .collect();
    for (var, base) in meta.variables.iter_mut().zip(base_crs) {
        if base.is_some() {
            var.crs = base;
        }
    }
}
