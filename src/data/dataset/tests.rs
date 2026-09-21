use super::{DataSource, DataSourceKind};
use crate::app::StoreKind;

#[test]
fn test_store_kind_data_source_kind_bijective_roundtrip() {
    let pairs = [
        (StoreKind::RemoteZarr, DataSourceKind::RemoteZarr),
        (StoreKind::LocalZarr, DataSourceKind::LocalZarr),
        (StoreKind::RemoteIcechunk, DataSourceKind::RemoteIcechunk),
        (StoreKind::LocalIcechunk, DataSourceKind::LocalIcechunk),
        (StoreKind::RemoteGeoTiff, DataSourceKind::RemoteGeoTiff),
        (StoreKind::LocalGeoTiff, DataSourceKind::LocalGeoTiff),
        (StoreKind::LocalNetCdf, DataSourceKind::NetCdf),
        (StoreKind::ProceduralVolume4D, DataSourceKind::Procedural),
    ];

    for (sk, dsk) in pairs {
        assert_eq!(sk.to_data_source_kind(), dsk);
        assert_eq!(StoreKind::from_data_source_kind(&dsk), sk);
    }
}

#[test]
fn test_data_source_instantiation() {
    let ds = DataSource::new(
        "test_cog",
        DataSourceKind::RemoteGeoTiff,
        "https://example.com/raster.cog",
        "Example COG",
    );
    assert_eq!(ds.id, "test_cog");
    assert_eq!(ds.kind, DataSourceKind::RemoteGeoTiff);
    assert_eq!(ds.uri, "https://example.com/raster.cog");
    assert_eq!(ds.display_name, "Example COG");
}
