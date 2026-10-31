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
/// A root known to have none is not read again.
pub fn discover_coord_array(
    store: ReadableWritableListableStorage,
    consolidated: RootListing<'_>,
    candidates: &[String],
    root: bool,
) -> Option<CoordArray> {
    let readable_store: ReadableStorage = store.clone();
    if let Some(Some(metadata)) = consolidated {
        let found = listed_array(&readable_store, metadata, candidates);
        return found.or_else(|| {
            let in_children = root.then(|| child_candidates(metadata, candidates))?;
            listed_array(&readable_store, metadata, &in_children)
        });
    }
    if let Some(found) = (candidates.iter()).find_map(|key| probe(&readable_store, key)) {
        return Some(found);
    }

    // Discover child group nodes from root if still not found
    if let Ok(root_path) = zarrs::node::NodePath::new("/")
        && let Ok(children) = zarrs::node::get_child_nodes(&store, &root_path, false)
    {
        for child in children {
            let child_group = child.path().as_str().trim_start_matches('/');
            if child_group.is_empty() {
                continue;
            }
            // Paths among the candidates were probed already.
            let in_child = |key: &String| {
                let path = format!("{child_group}/{key}");
                (!candidates.contains(&path)).then(|| probe(&readable_store, &path))?
            };
            if let Some(found) = candidates.iter().find_map(in_child) {
                return Some(found);
            }
        }
    }

    if consolidated.is_some() {
        return None;
    }
    let group = zarrs::group::Group::open(readable_store.clone(), "/").ok()?;
    let metadata = group.consolidated_metadata()?.metadata;
    listed_array(&readable_store, &metadata, candidates)
}

/// The array at `key`, its metadata read once: a missing array or an
/// unreadable store costs no more requests. Only metadata zarrs rejects is
/// opened again, normalized, and never in place of another array (that open
/// falls back to the root's).
fn probe(store: &ReadableStorage, key: &str) -> Option<CoordArray> {
    use zarrs::array::ArrayCreateError;
    use zarrs::storage::StorageError;
    let path = format!("/{key}");
    match Array::open(store.clone(), &path) {
        Ok(array) => Some(array),
        Err(ArrayCreateError::MissingMetadata) => None,
        Err(ArrayCreateError::StorageError(e))
            if !matches!(e, StorageError::InvalidMetadata(..)) =>
        {
            None
        }
        Err(_) => {
            let array =
                crate::utils::metadata::open_or_instantiate_array_normalized(store.clone(), &path)
                    .ok()?;
            (array.path().as_str() == path).then_some(array)
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
