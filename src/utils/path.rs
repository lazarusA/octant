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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_tilde() {
        let expanded = expand_tilde("~/test_dir/file.nc");
        assert!(!expanded.to_string_lossy().starts_with("~/"));

        let unexpanded = expand_tilde("./data/sample.nc");
        assert_eq!(unexpanded.to_string_lossy(), "./data/sample.nc");
    }
}
