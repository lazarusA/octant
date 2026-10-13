//! Bioimaging OME-Zarr datasets: images, labels, plates and containers of
//! every OME-Zarr version, from public archives (IDR, the BioImage Archive's
//! EMPIAR and NGFF challenge conversions, JAX, Janelia OpenOrganelle). The
//! archives publish them under CC-BY 4.0.

use super::types::CatalogEntry;
use crate::app::StoreKind;

/// IDR's OME-Zarr archive.
macro_rules! idr {
    ($path:literal) => {
        concat!("https://uk1s3.embassy.ebi.ac.uk/idr/zarr", $path)
    };
}

pub static MICROSCOPY_CATALOG: &[CatalogEntry] = &[
    CatalogEntry {
        key: "idr0062-nuclei-labels",
        label: "IDR 6001240: Nuclei with Labels",
        subtitle: "3D nuclei and lamina, 2 channels in micrometers, with a segmentation label image (OME-Zarr 0.4)",
        store: idr!("/v0.4/idr0062A/6001240.zarr"),
        store_kind: StoreKind::RemoteZarr,
    },
    CatalogEntry {
        key: "idr0062-nuclei-v05",
        label: "IDR 6001240: Sharded Zarr v3",
        subtitle: "The same nuclei and labels as OME-Zarr 0.5 with sharded Zarr v3 arrays",
        store: idr!("/v0.5/idr0062A/6001240_labels.zarr"),
        store_kind: StoreKind::RemoteZarr,
    },
    CatalogEntry {
        key: "idr0101-time-lapse",
        label: "IDR 13457537: Translated Time-Lapse",
        subtitle: "18 time points, 6 channels, 12 planes, with a translation offset per level",
        store: idr!("/v0.4/idr0101A/13457537.zarr"),
        store_kind: StoreKind::RemoteZarr,
    },
    CatalogEntry {
        key: "idr0052-mitosis",
        label: "IDR 5514375: Mitosis Time-Lapse",
        subtitle: "40 time points, 3 channels, 31 planes; opens at its default time point",
        store: idr!("/v0.4/idr0052A/5514375.zarr"),
        store_kind: StoreKind::RemoteZarr,
    },
    CatalogEntry {
        key: "idr0048-brain",
        label: "IDR 9846152: Multiphoton Brain",
        subtitle: "3 channels, 91 planes of 13350 x 19120 pixels in 9 resolution levels",
        store: idr!("/v0.4/idr0048A/9846152.zarr"),
        store_kind: StoreKind::RemoteZarr,
    },
    CatalogEntry {
        key: "idr0083-slide",
        label: "IDR 9822152: Whole Slide",
        subtitle: "93184 x 144384 greyscale slide in 11 levels, big-endian uint16",
        store: idr!("/v0.4/idr0083A/9822152.zarr"),
        store_kind: StoreKind::RemoteZarr,
    },
    CatalogEntry {
        key: "idr0076-cytometry",
        label: "IDR 10501752: Imaging Mass Cytometry",
        subtitle: "50 float64 channels, most hidden by default, each with its own window",
        store: idr!("/v0.4/idr0076A/10501752.zarr"),
        store_kind: StoreKind::RemoteZarr,
    },
    CatalogEntry {
        key: "idr0001-plate",
        label: "IDR 2551: Screening Plate",
        subtitle: "High-content screening plate: 96 wells, 6 fields each, 6 acquisitions",
        store: idr!("/v0.4/idr0001A/2551.zarr"),
        store_kind: StoreKind::RemoteZarr,
    },
    CatalogEntry {
        key: "idr-ngff-v01",
        label: "IDR 9836842: OME-NGFF 0.1",
        subtitle: "Legacy metadata without axes or coordinate transformations",
        store: idr!("/v0.1/9836842.zarr"),
        store_kind: StoreKind::RemoteZarr,
    },
    CatalogEntry {
        key: "idr0044-light-sheet",
        label: "Light-Sheet Mouse Development",
        subtitle: "532 time points of a 2-channel 988 x 2048 x 2169 volume (idr0044, OME-Zarr 0.5)",
        store: "https://livingobjects.ebi.ac.uk/idr/share/ome2024-ngff-challenge/idr0044/4007801.zarr",
        store_kind: StoreKind::RemoteZarr,
    },
    CatalogEntry {
        key: "empiar-10392-plasmodium",
        label: "Plasmodium Schizont (EMPIAR-10392)",
        subtitle: "Volume electron microscopy in angstroms, float32, greyscale (OME-Zarr 0.5)",
        store: "https://livingobjects.ebi.ac.uk/bioimaging-01-pub/ebi-ngff-challenge-2024/fb416517-e36d-4bdb-98f9-47b644138d47.zarr",
        store_kind: StoreKind::RemoteZarr,
    },
    CatalogEntry {
        key: "jax-histopathology",
        label: "JAX Mouse Histopathology",
        subtitle: "RGB slide of 51200 x 67200 pixels in a bioformats2raw container (OME-Zarr 0.5)",
        store: "https://storage.googleapis.com/jax-public-ngff/public/41908.zarr",
        store_kind: StoreKind::RemoteZarr,
    },
    CatalogEntry {
        key: "janelia-hela-fibsem",
        label: "HeLa Cell FIB-SEM (jrc_hela-2)",
        subtitle: "Janelia OpenOrganelle electron microscopy in nanometers, 6 levels",
        store: "https://janelia-cosem-datasets.s3.amazonaws.com/jrc_hela-2/jrc_hela-2.zarr/recon-1/em/fibsem-uint8",
        store_kind: StoreKind::RemoteZarr,
    },
];
