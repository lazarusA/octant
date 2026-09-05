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

/// Automatically infers the supported `StoreKind` from a given target URI, file path, or directory.
///
/// Returns `Ok(StoreKind)` for supported formats and sources, or `Err("Type not supported")` otherwise.
pub fn infer_store_kind_from_target(target: &str) -> Result<crate::app::StoreKind, &'static str> {
    use crate::app::StoreKind;
    let target = target.trim();
    let target = target.strip_prefix("file://").unwrap_or(target);
    let trimmed = target.trim();
    if trimmed.is_empty() {
        return Err("Type not supported");
    }

    // 1. Procedural Known Sources
    if trimmed == "procedural://volume4d" {
        return Ok(StoreKind::ProceduralVolume4D);
    }
    if trimmed == "procedural://matrix"
        || trimmed == "procedural://random"
        || trimmed == "procedural://matrix2d"
    {
        return Ok(StoreKind::ProceduralRandom);
    }

    // 2. Remote URIs (HTTP / HTTPS / S3 / GS / AZ)
    if trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
        || trimmed.starts_with("s3://")
        || trimmed.starts_with("gs://")
        || trimmed.starts_with("az://")
    {
        let lower = trimmed.to_lowercase();
        if lower.contains("icechunk") || lower.contains("earthmover") {
            return Ok(StoreKind::RemoteIcechunk);
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
    let path_str = trimmed.to_lowercase();

    // Check by file extension
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    if let Some(ref ext_str) = ext {
        match ext_str.as_str() {
            "nc" | "nc4" | "cdf" | "netcdf" | "h5" | "hdf5" | "hdf" | "he5" => {
                return Ok(StoreKind::LocalNetCdf);
            }
            "zarr" | "zip" => {
                return Ok(StoreKind::LocalZarr);
            }
            _ => {
                if path.is_file() {
                    return Err("Type not supported");
                }
            }
        }
    }

    // Check directory on disk
    if path.is_dir() {
        if path_str.contains("icechunk")
            || path.join("icechunk.json").exists()
            || path.join("snapshots").exists()
            || path.join("manifests").exists()
            || path.join("config.json").exists()
        {
            return Ok(StoreKind::LocalIcechunk);
        }
        if path.join("zarr.json").exists()
            || path.join(".zmetadata").exists()
            || path.join(".zgroup").exists()
            || path.join(".zarray").exists()
            || path_str.ends_with(".zarr")
        {
            return Ok(StoreKind::LocalZarr);
        }
        return Ok(StoreKind::LocalZarr);
    }

    // Check string patterns for typed paths
    if path_str.contains("icechunk") {
        return Ok(StoreKind::LocalIcechunk);
    }
    if path_str.ends_with(".nc")
        || path_str.ends_with(".nc4")
        || path_str.ends_with(".cdf")
        || path_str.ends_with(".netcdf")
        || path_str.ends_with(".h5")
        || path_str.ends_with(".hdf5")
        || path_str.ends_with(".hdf")
        || path_str.ends_with(".he5")
    {
        return Ok(StoreKind::LocalNetCdf);
    }
    if path_str.ends_with(".zarr") || path_str.ends_with(".zip") {
        return Ok(StoreKind::LocalZarr);
    }

    Err("Type not supported")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::StoreKind;

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
            infer_store_kind_from_target(
                "https://earthmover-icechunk-era5.s3.us-east-1.amazonaws.com/era5_surface_aws"
            ),
            Ok(StoreKind::RemoteIcechunk)
        );
        assert_eq!(
            infer_store_kind_from_target("https://example.com/repo_icechunk"),
            Ok(StoreKind::RemoteIcechunk)
        );
        assert_eq!(
            infer_store_kind_from_target("s3://bucket/icechunk_store"),
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
