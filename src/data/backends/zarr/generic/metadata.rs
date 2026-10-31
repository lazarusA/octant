//! What a `GenericZarrBlockStore` reads of its metadata: the root's
//! consolidated metadata and the store's variables.

use super::GenericZarrBlockStore;
use crate::data::VariableInfo;
use crate::data::backends::coord_bounds::discover::RootListing;
use crate::data::backends::zarr::storage::root_consolidated;
use crate::data::blocks::BlockStoreError;
use crate::data::ome::MetaReader;
use zarrs::metadata_ext::group::consolidated_metadata::ConsolidatedMetadataMetadata;

impl GenericZarrBlockStore {
    /// The root's consolidated metadata, read once. A failed read is asked
    /// again next time, and a browser store fills in later, so there only a
    /// found one is kept.
    pub(in crate::data::backends::zarr) fn consolidated(
        &self,
    ) -> Option<&ConsolidatedMetadataMetadata> {
        if let Some(known) = self.consolidated.get() {
            return known.as_ref();
        }
        let found = root_consolidated(&self.storage, &self.meta).ok()?;
        if found.is_none() && cfg!(target_arch = "wasm32") {
            return None;
        }
        self.consolidated.get_or_init(|| found).as_ref()
    }

    /// [`Self::consolidated`] as the coordinate search takes it: `None`
    /// while the root is unread (a failed read), so the search reads it.
    pub(super) fn root_listing(&self) -> RootListing<'_> {
        self.consolidated();
        self.consolidated.get().map(Option::as_ref)
    }

    /// The store's OME-Zarr variables with their discovery, else its plain
    /// Zarr variables.
    pub(super) fn store_variables(
        &self,
    ) -> Result<(Vec<VariableInfo>, Option<crate::data::ome::Discovery>), BlockStoreError> {
        // The one read of the root: its attributes go to `meta` for the walks.
        let consolidated = self.consolidated();
        let storage = || self.storage.clone();
        if let Some(mut found) = crate::data::ome::discover_direct(storage(), &self.meta) {
            return Ok((std::mem::take(&mut found.variables), Some(found)));
        }
        let url = self.source_url.trim_end_matches('/');
        let reader = MetaReader::direct(storage()).with_cache(&self.meta);
        let root_attrs = reader.group_attributes("").ok().flatten();
        let mut vars = crate::utils::extract_store_variables_consolidated(
            storage(),
            url,
            consolidated,
            root_attrs.as_deref(),
        )
        .map_err(|e| e.to_string())?;
        crate::data::multiscale::geozarr::link_pyramids_in(storage(), &self.meta, &mut vars);
        crate::data::crs::attach_zarr_grid_mappings_in(storage(), &self.meta, &mut vars);
        Ok((vars, None))
    }
}
