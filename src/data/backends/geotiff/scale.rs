//! Per-band scale and offset from the GDAL metadata tag (`<Item role="scale">`,
//! `<Item role="offset">`), which turn stored integers into physical values
//! through the backends' shared `DataCalibration`. Blocks hold the physical
//! values; the band's `SCALE` and `OFFSET` attributes
//! (`bands::add_gdal_attributes`) say how they were stored.

use async_tiff::ImageFileDirectory;

use super::bands::{GdalItem, parse_gdal_metadata};
use crate::data::DataCalibration;

/// The transforms of an IFD's bands, by band index (empty without any).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BandScales(Vec<Option<DataCalibration>>);

impl BandScales {
    /// The transforms of every IFD: its own, else, for an overview of IFD 0
    /// (GDAL writes the metadata tag on the full image only), IFD 0's.
    pub fn of_ifds(ifds: &[ImageFileDirectory]) -> Vec<Self> {
        let own: Vec<Self> = ifds.iter().map(Self::from_ifd).collect();
        let pick = |i: usize| {
            let inherit = i > 0
                && own.get(i).is_some_and(|s| s.0.is_empty())
                && crate::data::multiscale::geotiff::is_overview_of(ifds, i);
            own.get(if inherit { 0 } else { i })
                .cloned()
                .unwrap_or_default()
        };
        (0..ifds.len()).map(pick).collect()
    }

    fn from_ifd(ifd: &ImageFileDirectory) -> Self {
        let items = ifd
            .gdal_metadata()
            .map(parse_gdal_metadata)
            .unwrap_or_default();
        Self::from_items(&items, ifd.samples_per_pixel() as usize)
    }

    /// The transforms of `samples` bands among `items`.
    pub fn from_items(items: &[GdalItem], samples: usize) -> Self {
        let scales: Vec<_> = (0..samples).map(|b| band(items, b)).collect();
        Self(if scales.iter().any(Option::is_some) {
            scales
        } else {
            Vec::new()
        })
    }

    /// Band `band`'s transform.
    pub fn band(&self, band: usize) -> Option<&DataCalibration> {
        self.0.get(band)?.as_ref()
    }
}

/// Band `sample`'s scale and offset among `items`; `None` without either
/// (identity values count as none).
fn band(items: &[GdalItem], sample: usize) -> Option<DataCalibration> {
    let value = |role: &str| {
        items
            .iter()
            .find(|it| it.sample == Some(sample) && it.role.as_deref() == Some(role))
            .and_then(|it| it.value.trim().parse::<f64>().ok())
            .filter(|v| v.is_finite())
    };
    let scale_factor = value("scale").filter(|&s| s != 0.0 && s != 1.0);
    let add_offset = value("offset").filter(|&o| o != 0.0);
    (scale_factor.is_some() || add_offset.is_some()).then_some(DataCalibration {
        scale_factor,
        add_offset,
        ..Default::default()
    })
}
