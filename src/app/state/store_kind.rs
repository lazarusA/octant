//! Data store kinds and URI target inference for Octant.

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum StoreKind {
    RemoteZarr,
    LocalZarr,
    RemoteIcechunk,
    LocalIcechunk,
    RemoteGeoTiff,
    LocalGeoTiff,
    LocalNetCdf,
    ProceduralVolume4D,
    ProceduralRandom,
}

impl StoreKind {
    pub fn to_data_source_kind(self) -> crate::data::DataSourceKind {
        match self {
            StoreKind::RemoteZarr => crate::data::DataSourceKind::RemoteZarr,
            StoreKind::LocalZarr => crate::data::DataSourceKind::LocalZarr,
            StoreKind::RemoteIcechunk => crate::data::DataSourceKind::RemoteIcechunk,
            StoreKind::LocalIcechunk => crate::data::DataSourceKind::LocalIcechunk,
            StoreKind::RemoteGeoTiff => crate::data::DataSourceKind::RemoteGeoTiff,
            StoreKind::LocalGeoTiff => crate::data::DataSourceKind::LocalGeoTiff,
            StoreKind::LocalNetCdf => crate::data::DataSourceKind::NetCdf,
            StoreKind::ProceduralVolume4D | StoreKind::ProceduralRandom => {
                crate::data::DataSourceKind::Procedural
            }
        }
    }

    pub fn from_data_source_kind(kind: &crate::data::DataSourceKind) -> Self {
        match kind {
            crate::data::DataSourceKind::RemoteZarr => StoreKind::RemoteZarr,
            crate::data::DataSourceKind::LocalZarr => StoreKind::LocalZarr,
            crate::data::DataSourceKind::RemoteIcechunk => StoreKind::RemoteIcechunk,
            crate::data::DataSourceKind::LocalIcechunk => StoreKind::LocalIcechunk,
            crate::data::DataSourceKind::RemoteGeoTiff => StoreKind::RemoteGeoTiff,
            crate::data::DataSourceKind::LocalGeoTiff => StoreKind::LocalGeoTiff,
            crate::data::DataSourceKind::NetCdf => StoreKind::LocalNetCdf,
            crate::data::DataSourceKind::Procedural => StoreKind::ProceduralVolume4D,
            _ => StoreKind::ProceduralRandom,
        }
    }

    pub fn make_source_id(kind: StoreKind, target: &str) -> String {
        format!("{:?}:{}", kind, target)
    }

    /// Resolves the effective store kind from an optional explicit UI selection,
    /// an inferred target kind, and the current active store kind.
    /// Automatically upgrades generic Zarr selections if the target specifically matches Icechunk or NetCDF.
    pub fn resolve_with_inferred(
        explicit: Option<StoreKind>,
        target: &str,
        current: StoreKind,
    ) -> StoreKind {
        let inferred = crate::utils::infer_store_kind_from_target(target).ok();
        match (explicit, inferred) {
            (Some(kind), Some(inf)) => {
                if (kind == StoreKind::RemoteZarr && inf == StoreKind::RemoteIcechunk)
                    || (kind == StoreKind::RemoteZarr && inf == StoreKind::RemoteGeoTiff)
                    || (kind == StoreKind::RemoteZarr && inf == StoreKind::LocalGeoTiff)
                    || (kind == StoreKind::RemoteZarr && inf == StoreKind::LocalNetCdf)
                    || (kind == StoreKind::LocalZarr && inf == StoreKind::LocalIcechunk)
                    || (kind == StoreKind::LocalZarr && inf == StoreKind::LocalNetCdf)
                    || (kind == StoreKind::LocalZarr && inf == StoreKind::LocalGeoTiff)
                    || (kind == StoreKind::LocalZarr && inf == StoreKind::RemoteGeoTiff)
                {
                    inf
                } else {
                    kind
                }
            }
            (Some(kind), None) => kind,
            (None, Some(inf)) => inf,
            (None, None) => current,
        }
    }
}
