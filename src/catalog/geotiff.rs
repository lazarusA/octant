//! Cloud-Optimized GeoTIFFs: those the browser can read (their servers allow
//! cross-origin requests) and, in `GEOTIFF_DESKTOP_CATALOG`, those only the
//! desktop app can open (their servers send no CORS headers).

use super::types::CatalogEntry;
use crate::app::StoreKind;

/// OpenLandMap's analysis-ready COGs.
macro_rules! olm {
    ($file:literal) => {
        concat!("https://s3.openlandmap.org/arco/", $file)
    };
}

pub static GEOTIFF_CATALOG: &[CatalogEntry] = &[
    CatalogEntry {
        key: "sentinel-2-tci",
        label: "Sentinel-2 L2A True Color (TCI)",
        subtitle: "Cloud-Optimized GeoTIFF 10m RGB surface reflectance on AWS S3",
        store: "https://sentinel-cogs.s3.us-west-2.amazonaws.com/sentinel-s2-l2a-cogs/36/Q/WD/2020/7/S2A_36QWD_20200701_0_L2A/TCI.tif",
        store_kind: StoreKind::RemoteGeoTiff,
    },
    CatalogEntry {
        key: "sentinel-2-b04",
        label: "Sentinel-2 L2A Red (B04)",
        subtitle: "Cloud-Optimized GeoTIFF 10m Red Band on AWS S3",
        store: "https://sentinel-cogs.s3.us-west-2.amazonaws.com/sentinel-s2-l2a-cogs/36/Q/WD/2020/7/S2A_36QWD_20200701_0_L2A/B04.tif",
        store_kind: StoreKind::RemoteGeoTiff,
    },
    CatalogEntry {
        key: "glc-fcs30d-1985",
        label: "GLC_FCS30D Land Cover 1985",
        subtitle: "Global 30 m land cover classes, 1.3 million columns with 11 overviews (OpenLandMap)",
        store: olm!("lc_glc.fcs30d_c_30m_s_19850101_19851231_go_epsg.4326_v20231026.tif"),
        store_kind: StoreKind::RemoteGeoTiff,
    },
    CatalogEntry {
        key: "glc-fcs30d-2022",
        label: "GLC_FCS30D Land Cover 2022",
        subtitle: "The same global 30 m land cover classes, 37 years later (OpenLandMap)",
        store: olm!("lc_glc.fcs30d_c_30m_s_20220101_20221231_go_epsg.4326_v20231026.tif"),
        store_kind: StoreKind::RemoteGeoTiff,
    },
    CatalogEntry {
        key: "olm-lst-day-2021",
        label: "MODIS Daytime Land Surface Temperature 2021",
        subtitle: "Global 1 km annual median of MOD11A2 daytime temperature (OpenLandMap)",
        store: olm!("lst_mod11a2.daytime_p50_1km_s_20210101_20211231_go_epsg.4326_v1.2.tif"),
        store_kind: StoreKind::RemoteGeoTiff,
    },
    CatalogEntry {
        key: "olm-dtm-30m",
        label: "Ensemble Digital Terrain Model 30 m",
        subtitle: "Global bare-earth elevation in decimeters, 10th percentile of the ensemble (OpenLandMap)",
        store: olm!(
            "dtm.bareearth_ensemble_p10_30m_s_20180101_20181231_go_epsg.4326_v20230211.tif"
        ),
        store_kind: StoreKind::RemoteGeoTiff,
    },
];

/// COGs whose servers send no CORS headers: the browser cannot read them.
pub static GEOTIFF_DESKTOP_CATALOG: &[CatalogEntry] = &[
    CatalogEntry {
        key: "esa-worldcover-2021-n48e009",
        label: "ESA WorldCover 2021 (48-51 N, 9-12 E)",
        subtitle: "10 m land cover classes over central Germany, CC-BY 4.0",
        store: "https://esa-worldcover.s3.eu-central-1.amazonaws.com/v200/2021/map/ESA_WorldCover_10m_2021_v200_N48E009_Map.tif",
        store_kind: StoreKind::RemoteGeoTiff,
    },
    CatalogEntry {
        key: "copernicus-dem-30m-n50e011",
        label: "Copernicus DEM GLO-30 (50 N, 11 E)",
        subtitle: "30 m surface elevation over Thuringia, one degree tile",
        store: "https://copernicus-dem-30m.s3.amazonaws.com/Copernicus_DSM_COG_10_N50_00_E011_00_DEM/Copernicus_DSM_COG_10_N50_00_E011_00_DEM.tif",
        store_kind: StoreKind::RemoteGeoTiff,
    },
    CatalogEntry {
        key: "copernicus-dem-90m-n50e011",
        label: "Copernicus DEM GLO-90 (50 N, 11 E)",
        subtitle: "90 m surface elevation over Thuringia, one degree tile",
        store: "https://copernicus-dem-90m.s3.amazonaws.com/Copernicus_DSM_COG_30_N50_00_E011_00_DEM/Copernicus_DSM_COG_30_N50_00_E011_00_DEM.tif",
        store_kind: StoreKind::RemoteGeoTiff,
    },
];

impl CatalogEntry {
    /// Whether only the desktop app can open this entry (it is one of
    /// `GEOTIFF_DESKTOP_CATALOG`'s), found by its key in a set built once,
    /// since every card asks each frame.
    pub fn desktop_only(&self) -> bool {
        static KEYS: std::sync::OnceLock<std::collections::HashSet<&'static str>> =
            std::sync::OnceLock::new();
        KEYS.get_or_init(|| GEOTIFF_DESKTOP_CATALOG.iter().map(|e| e.key).collect())
            .contains(self.key)
    }
}
