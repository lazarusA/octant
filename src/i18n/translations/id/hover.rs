use crate::i18n::keys::HoverKey;

pub fn text(key: HoverKey) -> Option<&'static str> {
    Some(match key {
        HoverKey::NoData => "Tidak ada data",
        HoverKey::MoreFields => "+{} lainnya",
        HoverKey::ChannelN => "Kanal {}",
        HoverKey::SeriesLabel => "seri",
        HoverKey::FallbackVariable => "variabel",
        HoverKey::FallbackOverlay => "overlay",
        HoverKey::DataValue => "Nilai Data",
        HoverKey::LongitudeDegrees => "Bujur (derajat)",
        HoverKey::LatitudeDegrees => "Lintang (derajat)",
        HoverKey::AlongAxisNamed => "Sepanjang {} ({})",
        HoverKey::AlongAxis => "Sepanjang {}",
        HoverKey::AxisXLongitude => "X / Bujur",
        HoverKey::AxisYLatitude => "Y / Lintang",
        HoverKey::AxisZDepth => "Z / Kedalaman",
        HoverKey::DropDataset => {
            "Lepas dataset untuk divisualisasikan (.nc, .h5, .zarr, .icechunk, .tif)"
        }
        HoverKey::LabelN => "label {}",
        HoverKey::Level => "level",
        HoverKey::LevelOfTotal => "{} dari {}",
    })
}
