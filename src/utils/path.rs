//! Path and filesystem utilities.

use std::path::{Path, PathBuf};

/// Expands leading `~` or `~/` in path strings to the user's home directory.
pub fn expand_tilde(path: impl AsRef<Path>) -> PathBuf {
    let path_ref = path.as_ref();
    let path_str = path_ref.to_string_lossy();

    if let Some(stripped) = path_str.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
            return PathBuf::from(home).join(stripped);
        }
    } else if path_str == "~"
        && let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))
    {
        return PathBuf::from(home);
    }

    path_ref.to_path_buf()
}

/// Expands leading `~` or `~/` in path string and returns a String.
pub fn expand_tilde_str(path: &str) -> String {
    expand_tilde(path).to_string_lossy().to_string()
}

const GEOTIFF_EXTS: &[&str] = &[".tif", ".tiff", ".geotif", ".geotiff", ".cog"];
const NETCDF_EXTS: &[&str] = &[
    ".nc", ".nc4", ".cdf", ".netcdf", ".h5", ".hdf5", ".hdf", ".he5",
];

fn has_ext(s: &str, exts: &[&str]) -> bool {
    exts.iter().any(|ext| s.ends_with(ext))
}

/// Automatically infers the supported `StoreKind` from a given target URI, file path, or directory.
///
/// Returns `Ok(StoreKind)` for supported formats and sources, or `Err("Type not supported")` otherwise.
pub fn infer_store_kind_from_target(target: &str) -> Result<crate::app::StoreKind, &'static str> {
    use crate::app::StoreKind;
    let trimmed = target.trim();
    let trimmed = trimmed.strip_prefix("file://").unwrap_or(trimmed).trim();
    if trimmed.is_empty() {
        return Err("Type not supported");
    }

    // 1. Procedural Known Sources
    if trimmed == "procedural://volume4d" {
        return Ok(StoreKind::ProceduralVolume4D);
    }
    if trimmed.starts_with("procedural://") {
        return Ok(StoreKind::ProceduralRandom);
    }

    // 2. Remote URIs (HTTP / HTTPS / S3 / GS / AZ / VCC / ICECHUNK+)
    let lower = trimmed.to_lowercase();
    if lower.starts_with("icechunk+")
        || lower.starts_with("vcc://")
        || lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("s3://")
        || lower.starts_with("gs://")
        || lower.starts_with("az://")
    {
        if lower.starts_with("icechunk+")
            || lower.starts_with("vcc://")
            || lower.contains("icechunk")
        {
            return Ok(StoreKind::RemoteIcechunk);
        }
        let path_part = lower.split('?').next().unwrap_or(&lower);
        if has_ext(path_part, GEOTIFF_EXTS) {
            return Ok(StoreKind::RemoteGeoTiff);
        }
        if has_ext(path_part, NETCDF_EXTS) {
            return Ok(StoreKind::LocalNetCdf);
        }
        return Ok(StoreKind::RemoteZarr);
    }

    // 3. Explicit Scheme Prefixes
    if trimmed.starts_with("netcdf://") || trimmed.starts_with("hdf5://") {
        return Ok(StoreKind::LocalNetCdf);
    }
    if trimmed.starts_with("icechunk://") {
        return Ok(StoreKind::LocalIcechunk);
    }
    if trimmed.starts_with("zarr://") {
        return Ok(StoreKind::LocalZarr);
    }

    // 4. Local Filesystem (File or Directory)
    let path = std::path::Path::new(trimmed);
    if path.is_dir() {
        if lower.contains("icechunk")
            || path.join("icechunk.json").exists()
            || path.join("snapshots").exists()
            || path.join("manifests").exists()
            || path.join("config.json").exists()
        {
            return Ok(StoreKind::LocalIcechunk);
        }
        return Ok(StoreKind::LocalZarr);
    }

    if has_ext(&lower, NETCDF_EXTS) {
        return Ok(StoreKind::LocalNetCdf);
    }
    if has_ext(&lower, GEOTIFF_EXTS) {
        return Ok(StoreKind::LocalGeoTiff);
    }
    if lower.ends_with(".zarr") || lower.ends_with(".zip") {
        return Ok(StoreKind::LocalZarr);
    }
    if lower.contains("icechunk") {
        return Ok(StoreKind::LocalIcechunk);
    }

    Err("Type not supported")
}

/// An iterator over ancestor directory or group paths in a hierarchical dataset.
///
/// Yields progressively shorter parent path slices, concluding with the root group (`""`).
#[derive(Debug, Clone)]
pub struct AncestorPaths<'a> {
    current: Option<&'a str>,
}

impl<'a> AncestorPaths<'a> {
    /// Constructs a new `AncestorPaths` iterator from a dataset or node path.
    #[inline]
    pub fn new(path: &'a str) -> Self {
        let clean = path.trim_matches('/');
        Self {
            current: if clean.is_empty() { None } else { Some(clean) },
        }
    }
}

impl<'a> Iterator for AncestorPaths<'a> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let cur = self.current?;
        if let Some(idx) = cur.rfind('/') {
            let parent = &cur[..idx];
            self.current = Some(parent);
            Some(parent)
        } else {
            self.current = None;
            Some("")
        }
    }
}

/// Helper function to construct a zero-allocation `AncestorPaths` iterator.
#[inline]
pub fn ancestor_paths(path: &str) -> AncestorPaths<'_> {
    AncestorPaths::new(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::StoreKind;

    #[test]
    fn test_ancestor_paths() {
        let ancestors: Vec<&str> = ancestor_paths("/atmosphere/model_run_1/temp").collect();
        assert_eq!(ancestors, vec!["atmosphere/model_run_1", "atmosphere", ""]);

        let ancestors_single: Vec<&str> = ancestor_paths("temperature").collect();
        assert_eq!(ancestors_single, vec![""]);

        let ancestors_root: Vec<&str> = ancestor_paths("/").collect();
        assert!(ancestors_root.is_empty());
    }

    #[test]
    fn test_expand_tilde() {
        let expanded = expand_tilde("~/test_dir/file.nc");
        assert!(!expanded.to_string_lossy().starts_with("~/"));

        let unexpanded = expand_tilde("./data/sample.nc");
        assert_eq!(unexpanded.to_string_lossy(), "./data/sample.nc");
    }

    #[test]
    fn test_infer_store_kind_from_target_procedural() {
        assert_eq!(
            infer_store_kind_from_target("procedural://volume4d"),
            Ok(StoreKind::ProceduralVolume4D)
        );
        assert_eq!(
            infer_store_kind_from_target("procedural://matrix"),
            Ok(StoreKind::ProceduralRandom)
        );
        assert_eq!(
            infer_store_kind_from_target("procedural://random"),
            Ok(StoreKind::ProceduralRandom)
        );
        assert_eq!(
            infer_store_kind_from_target("procedural://matrix2d"),
            Ok(StoreKind::ProceduralRandom)
        );
        assert_eq!(
            infer_store_kind_from_target("procedural://irregular"),
            Ok(StoreKind::ProceduralRandom)
        );
    }

    #[test]
    fn test_infer_store_kind_from_target_remote() {
        assert_eq!(
            infer_store_kind_from_target("https://s3.bgc-jena.mpg.de/esdl.zarr"),
            Ok(StoreKind::RemoteZarr)
        );
        assert_eq!(
            infer_store_kind_from_target("s3://bucket/dataset.zarr"),
            Ok(StoreKind::RemoteZarr)
        );
        assert_eq!(
            infer_store_kind_from_target("https://example.com/repo_icechunk"),
            Ok(StoreKind::RemoteIcechunk)
        );
        assert_eq!(
            infer_store_kind_from_target("s3://bucket/icechunk_store"),
            Ok(StoreKind::RemoteIcechunk)
        );
        assert_eq!(
            infer_store_kind_from_target("icechunk+s3://my-bucket/dataset"),
            Ok(StoreKind::RemoteIcechunk)
        );
        assert_eq!(
            infer_store_kind_from_target("icechunk+https://example.com/repo"),
            Ok(StoreKind::RemoteIcechunk)
        );
        assert_eq!(
            infer_store_kind_from_target("vcc://container-bucket/key"),
            Ok(StoreKind::RemoteIcechunk)
        );
    }

    #[test]
    fn test_infer_store_kind_from_target_supported_and_unsupported() {
        assert_eq!(
            infer_store_kind_from_target("/data/sample.nc"),
            Ok(StoreKind::LocalNetCdf)
        );
        assert_eq!(
            infer_store_kind_from_target("/data/sample.nc4"),
            Ok(StoreKind::LocalNetCdf)
        );
        assert_eq!(
            infer_store_kind_from_target("/data/sample.h5"),
            Ok(StoreKind::LocalNetCdf)
        );
        assert_eq!(
            infer_store_kind_from_target("/data/sample.hdf5"),
            Ok(StoreKind::LocalNetCdf)
        );
        assert_eq!(
            infer_store_kind_from_target("/data/sample.cdf"),
            Ok(StoreKind::LocalNetCdf)
        );
        assert_eq!(
            infer_store_kind_from_target("netcdf:///path/to/data"),
            Ok(StoreKind::LocalNetCdf)
        );
        assert_eq!(
            infer_store_kind_from_target("/path/to/dataset.zarr"),
            Ok(StoreKind::LocalZarr)
        );
        assert_eq!(
            infer_store_kind_from_target("/path/to/icechunk_repo"),
            Ok(StoreKind::LocalIcechunk)
        );
        assert_eq!(
            infer_store_kind_from_target("file:///data/sample.nc"),
            Ok(StoreKind::LocalNetCdf)
        );
        assert_eq!(
            infer_store_kind_from_target("file:///data/dataset.zarr"),
            Ok(StoreKind::LocalZarr)
        );
        assert_eq!(
            infer_store_kind_from_target("/data/elevation.tif"),
            Ok(StoreKind::LocalGeoTiff)
        );
        assert_eq!(
            infer_store_kind_from_target("/data/satellite.tiff"),
            Ok(StoreKind::LocalGeoTiff)
        );
        assert_eq!(
            infer_store_kind_from_target("/data/imagery.cog"),
            Ok(StoreKind::LocalGeoTiff)
        );
        assert_eq!(
            infer_store_kind_from_target(
                "https://sentinel-cogs.s3.us-west-2.amazonaws.com/sentinel-s2-l2a-cogs/36/Q/WD/2020/7/S2A_36QWD_20200701_0_L2A/TCI.tif"
            ),
            Ok(StoreKind::RemoteGeoTiff)
        );
        assert_eq!(
            infer_store_kind_from_target("https://example.com/data/elevation.tif?download=1"),
            Ok(StoreKind::RemoteGeoTiff)
        );
        assert_eq!(
            infer_store_kind_from_target("s3://bucket/sentinel/raster.cog"),
            Ok(StoreKind::RemoteGeoTiff)
        );
        assert_eq!(
            infer_store_kind_from_target("https://example.com/model/output.nc"),
            Ok(StoreKind::LocalNetCdf)
        );

        // Unsupported types
        assert_eq!(
            infer_store_kind_from_target("sample.csv"),
            Err("Type not supported")
        );
        assert_eq!(
            infer_store_kind_from_target("image.png"),
            Err("Type not supported")
        );
        assert_eq!(
            infer_store_kind_from_target("document.pdf"),
            Err("Type not supported")
        );
        assert_eq!(infer_store_kind_from_target(""), Err("Type not supported"));
    }
}
