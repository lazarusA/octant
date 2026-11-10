//! Array discovery heuristics across group hierarchies and consolidated metadata.

use zarrs::array::Array;
use zarrs::metadata_ext::group::consolidated_metadata::ConsolidatedMetadataMetadata;
use zarrs::storage::{ReadableStorage, ReadableWritableListableStorage};

type CoordArray = Array<dyn zarrs::storage::ReadableStorageTraits>;

/// What the caller knows of the root's consolidated metadata: `None` not
/// read yet, `Some(None)` read and absent, `Some(Some(_))` its listing.
pub type RootListing<'a> = Option<Option<&'a ConsolidatedMetadataMetadata>>;

/// Finds and opens an array matching any candidate path across root, group
/// children, and consolidated metadata. With the store's `consolidated`
/// metadata only candidates it lists are opened: it lists every node, so
/// the store is asked nothing (each probe a request on a remote store), and
/// the root's child groups are searched only for a root variable (`root`),
/// so a sibling group's coordinate is never taken for a grouped variable's.
/// Without it, a listable store's child groups are probed for any variable,
/// as before. A root known to have none is not read again. `ReadFailed` when
/// nothing was found but a probe failed to read (a timeout, a server error).
pub fn discover_coord_array(
    store: ReadableWritableListableStorage,
    consolidated: RootListing<'_>,
    candidates: &[String],
    root: bool,
) -> Result<Option<CoordArray>, ReadFailed> {
    let readable_store: ReadableStorage = store.clone();
    if let Some(Some(metadata)) = consolidated {
        let found = listed_array(&readable_store, metadata, candidates);
        return Ok(found.or_else(|| {
            let in_children = root.then(|| child_candidates(metadata, candidates))?;
            listed_array(&readable_store, metadata, &in_children)
        }));
    }
    let mut failed = false;
    let found = probed(&store, candidates, |key| {
        probe(&readable_store, key).unwrap_or_else(|ReadFailed| {
            failed = true;
            None
        })
    });
    let listed = || {
        let group = zarrs::group::Group::open(readable_store.clone(), "/").ok()?;
        let metadata = group.consolidated_metadata()?.metadata;
        listed_array(&readable_store, &metadata, candidates)
    };
    match found.or_else(|| consolidated.is_none().then(listed).flatten()) {
        Some(found) => Ok(Some(found)),
        None if failed => Err(ReadFailed),
        None => Ok(None),
    }
}

/// The first of `candidates` that `try_probe` opens at the root, else in one of the
/// root's child groups (paths among the candidates are not probed twice).
fn probed(
    store: &ReadableWritableListableStorage,
    candidates: &[String],
    mut try_probe: impl FnMut(&str) -> Option<CoordArray>,
) -> Option<CoordArray> {
    if let Some(found) = candidates.iter().find_map(|key| try_probe(key)) {
        return Some(found);
    }
    let root = zarrs::node::NodePath::new("/").ok()?;
    let children = zarrs::node::get_child_nodes(store, &root, false).ok()?;
    children.iter().find_map(|child| {
        let group = child.path().as_str().trim_start_matches('/');
        let in_group = |key: &String| {
            let path = format!("{group}/{key}");
            (!candidates.contains(&path)).then(|| try_probe(&path))?
        };
        (!group.is_empty()).then(|| candidates.iter().find_map(in_group))?
    })
}

/// A coordinate's metadata or values could not be read (a timeout, a server error,
/// a chunk that failed): unlike a missing coordinate, not remembered, so the next
/// lookup reads it again.
#[derive(Debug)]
pub struct ReadFailed;

/// Whether a storage error is the server refusing a key, which S3 answers for missing
/// keys where it may not list: a missing key, not a failed read.
/// shortcut: object_store reports a 403 only in its error text, so a store wording it
/// otherwise has its refusals read again on every lookup; match the error kind once
/// zarrs keeps it.
fn refused(e: &zarrs::storage::StorageError) -> bool {
    e.to_string().contains("lacked the necessary privileges")
}

/// The array at `key`, its metadata read once: a missing array costs no
/// more requests, a failed v3 read one more for the v2 metadata. Only
/// metadata zarrs rejects is opened again, normalized, and never in place of
/// another array (that open falls back to the root's). A read that failed, not
/// refused (`refused`), is `ReadFailed`.
fn probe(store: &ReadableStorage, key: &str) -> Result<Option<CoordArray>, ReadFailed> {
    use zarrs::array::ArrayCreateError;
    use zarrs::storage::StorageError;
    let path = format!("/{key}");
    let unreadable = |e: &StorageError| !matches!(e, StorageError::InvalidMetadata(..));
    let mut v3_failed = false;
    let opened = match Array::open(store.clone(), &path) {
        // The v3 read failed (S3 answers 403 for a missing key where it may
        // not list): the v2 metadata may still be there.
        Err(ArrayCreateError::StorageError(e)) if unreadable(&e) => {
            v3_failed = !refused(&e);
            let v2 = zarrs::config::MetadataRetrieveVersion::V2;
            Array::open_opt(store.clone(), &path, &v2)
        }
        opened => opened,
    };
    match opened {
        Ok(array) => Ok(Some(array)),
        Err(ArrayCreateError::MissingMetadata) if v3_failed => Err(ReadFailed),
        Err(ArrayCreateError::MissingMetadata) => Ok(None),
        Err(ArrayCreateError::StorageError(e)) if unreadable(&e) => match refused(&e) {
            true => Ok(None),
            false => Err(ReadFailed),
        },
        Err(_) => {
            let normalized =
                crate::utils::metadata::open_or_instantiate_array_normalized(store.clone(), &path);
            Ok(normalized
                .ok()
                .filter(|array| array.path().as_str() == path))
        }
    }
}

/// `candidates` (aliases among them) within each group at the root that
/// `metadata` lists, in path order, as the probe looks in the root's child
/// groups.
fn child_candidates(metadata: &ConsolidatedMetadataMetadata, candidates: &[String]) -> Vec<String> {
    let mut children: Vec<&str> = (metadata.iter())
        .filter(|(_, node)| matches!(node, zarrs::node::NodeMetadata::Group(_)))
        .map(|(path, _)| path.trim_matches('/'))
        .filter(|path| !path.is_empty() && !path.contains('/'))
        .collect();
    children.sort_unstable();
    (children.iter())
        .flat_map(|child| candidates.iter().map(move |name| format!("{child}/{name}")))
        .collect()
}

/// The first of `candidates` that `metadata` lists, opened from it.
fn listed_array(
    store: &ReadableStorage,
    metadata: &ConsolidatedMetadataMetadata,
    candidates: &[String],
) -> Option<CoordArray> {
    candidates.iter().find_map(|path| {
        let key = path.trim_start_matches('/');
        let node_meta = metadata.get(key).or_else(|| metadata.get(path))?;
        crate::utils::metadata::instantiate_array_from_node_metadata(store.clone(), path, node_meta)
    })
}
