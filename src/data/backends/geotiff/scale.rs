//! Per-band scale and offset from the GDAL metadata tag (`<Item role="scale">`,
//! `<Item role="offset">`), which turn stored integers into physical values:
//! `value = stored * scale + offset`. Blocks hold the physical values; the
//! band's `SCALE` and `OFFSET` attributes (`bands::add_gdal_attributes`) say
//! how they were stored.

use async_tiff::ImageFileDirectory;

use super::bands::{GdalItem, parse_gdal_metadata};

/// One band's linear transform; never the identity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BandScale {
    pub scale: f64,
    pub offset: f64,
}

impl BandScale {
    /// Band `sample`'s transform among `items`, or `None` when it has none
    /// (or the identity).
    fn of(items: &[GdalItem], sample: usize) -> Option<Self> {
        let value = |role: &str| {
            items
                .iter()
                .find(|it| it.sample == Some(sample) && it.role.as_deref() == Some(role))
                .and_then(|it| it.value.trim().parse::<f64>().ok())
                .filter(|v| v.is_finite())
        };
        let scale = value("scale").filter(|&s| s != 0.0).unwrap_or(1.0);
        let offset = value("offset").unwrap_or(0.0);
        (scale != 1.0 || offset != 0.0).then_some(Self { scale, offset })
    }

    /// Applies the transform in place; missing values (NaN) stay missing.
    pub fn apply(&self, values: &mut [f32]) {
        let (s, o) = (self.scale as f32, self.offset as f32);
        for v in values.iter_mut() {
            *v = *v * s + o;
        }
    }
}

/// The transforms of an IFD's bands, by band index.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BandScales(Vec<Option<BandScale>>);

impl BandScales {
    /// The transforms of IFD `idx` of `ifds`: its own, else, for an overview
    /// of IFD 0 (GDAL writes the metadata tag on the full image only), IFD 0's.
    pub fn of_ifd_in(ifds: &[ImageFileDirectory], idx: usize) -> Self {
        let own = ifds.get(idx).map(Self::from_ifd).unwrap_or_default();
        if idx == 0 || !own.is_empty() {
            return own;
        }
        if crate::data::multiscale::geotiff::is_overview_of(ifds, idx) {
            ifds.first().map(Self::from_ifd).unwrap_or_default()
        } else {
            own
        }
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
        let scales: Vec<_> = (0..samples).map(|b| BandScale::of(items, b)).collect();
        Self(if scales.iter().any(Option::is_some) {
            scales
        } else {
            Vec::new()
        })
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Band `band`'s transform.
    pub fn band(&self, band: usize) -> Option<BandScale> {
        self.0.get(band).copied().flatten()
    }
}
