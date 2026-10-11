//! Snapshot ID and branch reference discovery for remote Icechunk repositories.

#[cfg(target_arch = "wasm32")]
use icechunk_format::format_constants::SpecVersionBin;
#[cfg(target_arch = "wasm32")]
use icechunk_format::repo_info::RepoInfo;

#[cfg(target_arch = "wasm32")]
use super::header::decompress_icechunk_file;
#[cfg(target_arch = "wasm32")]
use crate::data::backends::http::fetch_url_bytes;

/// Resolves the latest snapshot ID and format spec version for an Icechunk repository URL.
#[cfg(target_arch = "wasm32")]
pub async fn resolve_snapshot_id(clean_url: &str) -> Result<(String, SpecVersionBin), String> {
    // 1. Attempt Icechunk V2 discovery via RepoInfo (/repo)
    let repo_url = format!("{clean_url}/repo");
    if let Ok(raw_repo_bytes) = fetch_url_bytes(&repo_url).await
        && let Ok((spec_version, decomp_repo)) = decompress_icechunk_file(&raw_repo_bytes)
        && let Ok(repo_info) = RepoInfo::from_buffer(decomp_repo)
    {
        if let Ok(branches) = repo_info.branches() {
            for (b_name, snap_id) in branches {
                if b_name == "main" || b_name == "master" {
                    return Ok((snap_id.to_string(), spec_version));
                }
            }
        }
        if let Ok(mut tags) = repo_info.tags()
            && let Some((_tag_name, snap_id)) = tags.next()
        {
            return Ok((snap_id.to_string(), spec_version));
        }
    }

    Err(format!(
        "Failed to find Icechunk branch reference at '{clean_url}/repo'. Ensure the server allows CORS."
    ))
}
