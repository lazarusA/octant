//! Bioimaging OME-Zarr datasets: images, labels, plates and containers of
//! every OME-Zarr version, from public archives (IDR, the BioImage Archive's
//! EMPIAR and NGFF challenge conversions, JAX, Janelia OpenOrganelle). The
//! archives publish them under CC-BY 4.0. Also large multiscale volumes for
//! testing volume LOD: CT and micro-CT from the OpenSciVis collection's
//! OME-Zarr conversions, cryo-ET and fluorescence stacks on public object
//! stores; linked only, under each store's own terms.

use super::types::CatalogEntry;
use crate::app::StoreKind;

/// IDR's OME-Zarr archive.
macro_rules! idr {
    ($path:literal) => {
        concat!("https://uk1s3.embassy.ebi.ac.uk/idr/zarr", $path)
    };
}

impl CatalogEntry {
    /// Whether this entry is one of `MICROSCOPY_CATALOG`'s OME-Zarr datasets,
    /// found by its key (keys are unique across the catalog) in a set built
    /// once, since every card asks each frame.
    pub fn is_ome(&self) -> bool {
        static KEYS: std::sync::OnceLock<std::collections::HashSet<&'static str>> =
            std::sync::OnceLock::new();
        KEYS.get_or_init(|| MICROSCOPY_CATALOG.iter().map(|e| e.key).collect())
            .contains(self.key)
    }
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
        key: "n4bi-zebrafish-multiview",
        label: "Zebrafish Light-Sheet Multiview",
        subtitle: "8 views of 342 x 2048 x 2048 as named series listed on the OME group (OME-Zarr 0.5)",
        store: "https://radosgw.public.os.wwu.de/n4bi-goe/Zebrafish-XSPIM-multiview.ome.zarr",
        store_kind: StoreKind::RemoteZarr,
    },
    CatalogEntry {
        key: "muenster-plate-labels",
        label: "JSC13 Screening Plate with Labels",
        subtitle: "132 wells of 2 fields, each with a colored label image and properties (OME-Zarr 0.5)",
        store: "https://radosgw.public.os2.wwu.de/ngff/1904_resaved.zarr",
        store_kind: StoreKind::RemoteZarr,
    },
    CatalogEntry {
        key: "ssbd-time-lapse-minutes",
        label: "SSBD Time-Lapse in Minutes",
        subtitle: "271 time points in minutes, 2 channels, 43 planes (RIKEN SSBD, OME-Zarr 0.5)",
        store: "https://dmss3gw.riken.jp/ssbd/zarr/v0.5/8b0e0f7f-0ed4-487a-b8d2-cd6744d43c3e.zarr",
        store_kind: StoreKind::RemoteZarr,
    },
    CatalogEntry {
        key: "janelia-hela-fibsem",
        label: "HeLa Cell FIB-SEM (jrc_hela-2)",
        subtitle: "Janelia OpenOrganelle electron microscopy in nanometers, 6 levels",
        store: "https://janelia-cosem-datasets.s3.amazonaws.com/jrc_hela-2/jrc_hela-2.zarr/recon-1/em/fibsem-uint8",
        store_kind: StoreKind::RemoteZarr,
    },
    CatalogEntry {
        key: "scivis-woodbranch",
        label: "Woodbranch (Micro-CT)",
        subtitle: "2048^3 uint16 wood micro-CT, 16 GB in a multiscale pyramid (OpenSciVis)",
        store: "https://ome-zarr-scivis.s3.us-east-1.amazonaws.com/v0.5/96x2/woodbranch.ome.zarr",
        store_kind: StoreKind::RemoteZarr,
    },
    CatalogEntry {
        key: "microct-beechnut",
        label: "Beechnut (Micro-CT)",
        subtitle: "1546 x 1024 x 1024 uint16 micro-CT in 5 levels, Gaussian scale space",
        store: "https://d39zu0xtgv0613.cloudfront.net/beechnut.ome.zarr",
        store_kind: StoreKind::RemoteZarr,
    },
    CatalogEntry {
        key: "scivis-kingsnake",
        label: "Kingsnake (CT)",
        subtitle: "1024 x 1024 x 795 uint8 CT of a kingsnake egg (OpenSciVis)",
        store: "https://ome-zarr-scivis.s3.us-east-1.amazonaws.com/v0.5/96x2/kingsnake.ome.zarr",
        store_kind: StoreKind::RemoteZarr,
    },
    CatalogEntry {
        key: "scivis-pawpawsaurus",
        label: "Pawpawsaurus Skull (CT)",
        subtitle: "958 x 646 x 1088 uint16 CT of a fossil skull (OpenSciVis)",
        store: "https://ome-zarr-scivis.s3.us-east-1.amazonaws.com/v0.5/96x2/pawpawsaurus.ome.zarr",
        store_kind: StoreKind::RemoteZarr,
    },
    CatalogEntry {
        key: "scivis-pancreas",
        label: "Pancreas (CT)",
        subtitle: "240 x 512 x 512 int16 CT in Hounsfield units (OpenSciVis)",
        store: "https://ome-zarr-scivis.s3.us-east-1.amazonaws.com/v0.5/96x2/pancreas.ome.zarr",
        store_kind: StoreKind::RemoteZarr,
    },
    CatalogEntry {
        key: "scivis-backpack",
        label: "Backpack (CT)",
        subtitle: "512 x 512 x 373 uint16 baggage CT scan (OpenSciVis)",
        store: "https://ome-zarr-scivis.s3.us-east-1.amazonaws.com/v0.5/96x2/backpack.ome.zarr",
        store_kind: StoreKind::RemoteZarr,
    },
    CatalogEntry {
        key: "cryoet-vibrio",
        label: "Vibrio cholerae (Cryo-ET)",
        subtitle: "1023 x 1440 x 400 float32 cryo-electron tomogram",
        store: "https://d39zu0xtgv0613.cloudfront.net/sma2022-07-13-10.zarr",
        store_kind: StoreKind::RemoteZarr,
    },
    CatalogEntry {
        key: "bia-zebrafish-im4",
        label: "Zebrafish Embryo IM4",
        subtitle: "4 channels, 103 planes of 512 x 512 uint16 fluorescence (BioImage Archive S-BSST410)",
        store: "https://uk1s3.embassy.ebi.ac.uk/bia-integrator-data/S-BSST410/IM4/IM4.zarr/0",
        store_kind: StoreKind::RemoteZarr,
    },
    CatalogEntry {
        key: "bia-arabidopsis-im500",
        label: "Arabidopsis FLC smFISH IM500",
        subtitle: "2 channels, 46 planes of 1024 x 512 uint8 smFISH (BioImage Archive S-BIAD425)",
        store: "https://uk1s3.embassy.ebi.ac.uk/bia-integrator-data/S-BIAD425/IM500/IM500.zarr/0",
        store_kind: StoreKind::RemoteZarr,
    },
    CatalogEntry {
        key: "idr0047-yeast",
        label: "Yeast smFISH (IDR0047)",
        subtitle: "4 channels, 25 planes of 2048 x 2048 uint16 fluorescence",
        store: "https://d39zu0xtgv0613.cloudfront.net/multichannel/4496763.zarr/4496763.zarr",
        store_kind: StoreKind::RemoteZarr,
    },
    CatalogEntry {
        key: "efish-drosophila",
        label: "Drosophila Brain (Fly-eFISH)",
        subtitle: "4 channels, 752 planes of 1920 x 1920 uint16 expansion FISH",
        store: "https://d39zu0xtgv0613.cloudfront.net/Fly-eFISH/NP01_1_1_SS00790_AstA546_CCHa1_647_1x_LOL.chunked.zarr",
        store_kind: StoreKind::RemoteZarr,
    },
];
