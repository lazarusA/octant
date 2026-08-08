//! Builds the `ReadableWritableListableStorage` handle the block-cache path
//! needs (`BlockPrefetcher::request_block`), per `StoreKind`.
//!
//! Kept separate from `BlockPrefetcher` itself so the prefetcher stays
//! backend-agnostic (it just takes a storage handle) while this module owns
//! the "which backend am I" branching — the same shape as
//! `inspect_active_store`'s `Box<dyn DataStore>` match, but producing a
//! `ReadableWritableListableStorage` instead.
//!
//! STATUS: only `StoreKind::RemoteZarr` is implemented (backed by
//! `build_sync_store`, HTTP object_store). `LocalZarr`, `RemoteIcechunk`, and
//! `LocalIcechunk` return an explicit "not yet implemented" error rather than
//! guessing at filesystem/Icechunk store construction — fill these in once
//! their store builders are available, following the same pattern.

use std::error::Error;

use zarrs::storage::ReadableWritableListableStorage;

use crate::app::StoreKind;
use crate::utils::zarr::build_sync_store;

/// Builds a synchronous Zarr storage handle for the given store kind/target.
///
/// `ProceduralRandom` has no backing store at all — callers should branch on
/// `StoreKind::ProceduralRandom` before reaching the block-cache path
/// entirely, the same way `load_selected_variable_slice` already bypasses
/// the cache for it.
pub fn build_storage_for(
    store_kind: StoreKind,
    target: &str,
) -> Result<ReadableWritableListableStorage, Box<dyn Error>> {
    match store_kind {
        StoreKind::RemoteZarr => build_sync_store(target),

        StoreKind::LocalZarr => Err(format!(
            "block cache: StoreKind::LocalZarr storage construction not yet implemented (target: {target})"
        )
        .into()),

        StoreKind::RemoteIcechunk => Err(format!(
            "block cache: StoreKind::RemoteIcechunk storage construction not yet implemented (target: {target})"
        )
        .into()),

        StoreKind::LocalIcechunk => Err(format!(
            "block cache: StoreKind::LocalIcechunk storage construction not yet implemented (target: {target})"
        )
        .into()),

        StoreKind::ProceduralRandom => Err(
            "block cache: ProceduralRandom has no backing store; callers should bypass the block cache for this kind".into(),
        ),
    }
}
