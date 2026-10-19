//! Soil property maps as Cloud-Optimized GeoTIFFs: ISRIC SoilGrids 2.0
//! (CC-BY 4.0) and OpenLandMap-soildb (2025, CC-BY 4.0), whose servers allow
//! cross-origin requests, plus OpenLandMap's earlier soil classes (CC-BY-SA
//! 4.0). Values are as stored: SoilGrids keeps pH times 10 and organic carbon
//! in dg/kg, OpenLandMap-soildb records its scale in GDAL metadata.

use super::types::CatalogEntry;
use crate::app::StoreKind;

/// SoilGrids 2.0 maps aggregated to coarser grids.
macro_rules! soilgrids {
    ($path:literal) => {
        concat!(
            "https://files.isric.org/soilgrids/latest/data_aggregated/",
            $path
        )
    };
}

/// OpenLandMap-soildb's global mosaics.
macro_rules! soildb {
    ($path:literal) => {
        concat!("https://s3.opengeohub.org/global-soil/", $path)
    };
}

pub static SOIL_CATALOG: &[CatalogEntry] = &[
    CatalogEntry {
        key: "soilgrids-ph-1km",
        label: "SoilGrids 2.0 pH (H2O) 0-5 cm",
        subtitle: "Global 1 km topsoil pH times 10, Homolosine projection (ISRIC, CC-BY 4.0)",
        store: soilgrids!("1000m/phh2o/phh2o_0-5cm_mean_1000.tif"),
        store_kind: StoreKind::RemoteGeoTiff,
    },
    CatalogEntry {
        key: "soilgrids-soc-5km",
        label: "SoilGrids 2.0 Organic Carbon 0-5 cm",
        subtitle: "Global 5 km topsoil organic carbon in dg/kg, Homolosine projection (ISRIC, CC-BY 4.0)",
        store: soilgrids!("5000m/soc/soc_0-5cm_mean_5000.tif"),
        store_kind: StoreKind::RemoteGeoTiff,
    },
    CatalogEntry {
        key: "olm-soildb-soc-120m",
        label: "OpenLandMap-soildb Organic Carbon Density 0-30 cm",
        subtitle: "Global 120 m, 2020-2022, mg/cm3 times 10, 8 overviews (2025 release, CC-BY 4.0)",
        store: soildb!(
            "global_soil_props_v20250204_mosaics/oc_iso.10694.1995.mg.cm3_m_120m_b0cm..30cm_20200101_20221231_g_epsg.4326_v20250204.tif"
        ),
        store_kind: StoreKind::RemoteGeoTiff,
    },
    CatalogEntry {
        key: "olm-soildb-ph-120m",
        label: "OpenLandMap-soildb pH (H2O) 0-30 cm",
        subtitle: "Global 120 m, 2020-2022, pH times 10 (2025 release, CC-BY 4.0)",
        store: soildb!(
            "global_soil_props_v20250204_mosaics/ph.h2o_iso.10390.2021.index_m_120m_b0cm..30cm_20200101_20221231_g_epsg.4326_v20250204.tif"
        ),
        store_kind: StoreKind::RemoteGeoTiff,
    },
    CatalogEntry {
        key: "olm-soildb-clay-120m",
        label: "OpenLandMap-soildb Clay Content 0-30 cm",
        subtitle: "Global 120 m, 2020-2022, percent of the fine earth (2025 release, CC-BY 4.0)",
        store: soildb!(
            "global_soil_props_v20250523/clay.tot_iso.11277.2020.wpct_m_120m_b0cm..30cm_20200101_20221231_g_epsg.4326_v20250523.tif"
        ),
        store_kind: StoreKind::RemoteGeoTiff,
    },
    CatalogEntry {
        key: "olm-soil-greatgroups",
        label: "USDA Soil Great Groups",
        subtitle: "Global 250 m soil classes, the most probable great group (OpenLandMap, CC-BY-SA 4.0)",
        store: "https://s3.openlandmap.org/arco/grtgroup_usda.soiltax_c_250m_s_19500101_20171231_go_espg.4326_v0.2.tif",
        store_kind: StoreKind::RemoteGeoTiff,
    },
];
