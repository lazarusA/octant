//! `local://<n>/<name>[/<key>]` URLs: files and folders the browser was
//! given (picked or dropped), named so the web's readers fetch them like
//! remote objects (`http::fetch_url_bytes` and friends read them from the
//! files instead).

/// The scheme of local sources.
pub const SCHEME: &str = "local://";

/// The URL of source `id`, a file or folder named `name`.
pub fn url_of(id: u32, name: &str) -> String {
    format!("{SCHEME}{id}/{}", name.replace('/', "_"))
}

/// The source id of `url` and the key under its root (`""` for the root).
pub fn parse(url: &str) -> Option<(u32, &str)> {
    let rest = url.strip_prefix(SCHEME)?;
    let (id, rest) = rest.split_once('/').unwrap_or((rest, ""));
    let key = rest.split_once('/').map_or("", |(_, key)| key);
    Some((id.parse().ok()?, key.trim_end_matches('/')))
}

/// The source id of local `url`; none for other URLs.
pub fn id_of(url: &str) -> Option<u32> {
    parse(url).map(|(id, _)| id)
}

/// The path segments of `key`, none when it names nothing under the root
/// (empty, `.` or `..` segments).
pub fn segments(key: &str) -> Option<Vec<&str>> {
    let parts: Vec<&str> = key.split('/').collect();
    let valid = parts
        .iter()
        .all(|s| !s.is_empty() && *s != "." && *s != "..");
    valid.then_some(parts)
}

/// The folders directly under folder `dir` (`""` the root) among file `keys`.
pub fn subfolders<'a>(
    keys: impl Iterator<Item = &'a str>,
    dir: &str,
) -> std::collections::BTreeSet<&'a str> {
    keys.filter_map(|key| {
        let rest = match dir {
            "" => key,
            _ => key.strip_prefix(dir)?.strip_prefix('/')?,
        };
        rest.split_once('/').map(|(folder, _)| folder)
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::{id_of, parse, segments, subfolders, url_of};

    #[test]
    fn a_plate_and_its_fields_share_one_source() {
        let plate = url_of(3, "plate.zarr");
        assert_eq!(id_of(&plate), Some(3));
        assert_eq!(id_of(&format!("{plate}/B/3/0")), Some(3));
        assert_eq!(id_of("https://host/3/plate.zarr"), None);
    }

    #[test]
    fn subfolders_are_the_next_segment_under_a_folder() {
        let keys = [
            ".zgroup",
            "u/.zarray",
            "u/0.0",
            "g/.zgroup",
            "g/v/.zarray",
            "gv/x",
        ];
        let root: Vec<_> = subfolders(keys.into_iter(), "").into_iter().collect();
        assert_eq!(root, ["g", "gv", "u"]);
        let g: Vec<_> = subfolders(keys.into_iter(), "g").into_iter().collect();
        assert_eq!(g, ["v"]);
        assert!(subfolders(keys.into_iter(), "u").is_empty());
    }

    #[test]
    fn urls_name_a_source_and_a_key_under_it() {
        let root = url_of(3, "my store.zarr");
        assert_eq!(root, "local://3/my store.zarr");
        assert_eq!(parse(&root), Some((3, "")));
        assert_eq!(parse(&format!("{root}/")), Some((3, "")));
        assert_eq!(
            parse(&format!("{root}/a/zarr.json")),
            Some((3, "a/zarr.json"))
        );
        assert_eq!(parse("local://x/a"), None);
        assert_eq!(parse("https://3/a"), None);
    }

    #[test]
    fn keys_never_leave_the_root() {
        assert_eq!(segments("a/c/0/1"), Some(vec!["a", "c", "0", "1"]));
        assert_eq!(segments(""), None);
        assert_eq!(segments("a/../b"), None);
        assert_eq!(segments("./a"), None);
        assert_eq!(segments("a//b"), None);
    }
}
