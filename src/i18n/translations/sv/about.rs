use crate::i18n::keys::AboutKey;

pub fn text(key: AboutKey) -> Option<&'static str> {
    Some(match key {
        AboutKey::WindowTitle => "Om Octant",
        AboutKey::IconsWindowTitle => "Inbyggda vektorikoner",
        AboutKey::IconsTab => "Vektorikoner",
        AboutKey::Subtitle => "N-dimensionell datavisare",
        AboutKey::Description => {
            "Octant är en interaktiv visare för N-dimensionella dataset med inbyggt stöd för lokal och molnbaserad objektlagring, Zarr (v2/v3) och Icechunk. Byggd i Rust med GPU-accelererad rendering via WGPU. Octant körs inbyggt på macOS, Linux och Windows."
        }
        AboutKey::HyperslabTitle => "Hyperslab-snitt",
        AboutKey::HyperslabDesc => "Asynkron LRU-cache för chunkar och flerupplösta pyramider.",
        AboutKey::StorageTitle => "Inbyggt stöd för Zarr och Icechunk",
        AboutKey::StorageDesc => "Lokala, S3, GCS, Azure och HTTP-strömmande gränssnitt.",
        AboutKey::ShadersTitle => "Hårdvaruaccelererade WGPU-shaders",
        AboutKey::ShadersDesc => "2D-kartor, 3D-sfärer, höjdytor, volymer och 1D-profiler.",
        AboutKey::LinksHeading => "Länkar och resurser",
        AboutKey::DocsLink => "Octant-dokumentation",
        AboutKey::GoodFaithNotice => {
            "Octant erkänner verk från tredje part i god tro. Om du tror att något saknas eller är felaktigt attribuerat, meddela oss så rättar vi det eller tar bort det."
        }
        AboutKey::BrowseIcons => "Bläddra bland inbyggda vektorikoner",
        AboutKey::BrowseIconsHover => "Utforska procedurella vektorikoner i en popup",
        AboutKey::Licensed => "Licensierad under MIT eller Apache-2.0",
        AboutKey::OctantTeaser => {
            "Tecknen för x, y och z (+/-, +/-, +/-) delar rummet i åtta oktanter. Utforska dina data på samma sätt: välj axlarna, skär ut ett skikt och se på en region i taget."
        }
        AboutKey::OctantHover => "Klicka för att gå till nästa oktant",
        AboutKey::ColormapCredits => "Erkännanden för färgkartor",
        AboutKey::ColormapCreditsNote => {
            "Medföljande färgkartor behåller sina upphovspersoners licenser."
        }
        AboutKey::LicenseTexts => "Licenstexter",
        AboutKey::ThirdPartyLicenses => "Licenser från tredje part",
        AboutKey::ThirdPartyNote => {
            "Octant innehåller crates med öppen källkod, typsnitt och C-bibliotek (netCDF, HDF5), var och en under sin egen licens."
        }
        AboutKey::ViewOnGithub => "Visa på GitHub",
        AboutKey::IconsSearchHint => "Sök ikoner...",
        AboutKey::IconsAll => "Alla",
        AboutKey::IconsCopied => "Kopierade {}",
        AboutKey::IconsNoMatch => "Inga matchande vektorikoner hittades.",
        AboutKey::IconsCategory => "Kategori: {}",
        AboutKey::IconsClickToCopy => "Klicka för att kopiera enum-identifieraren",
        AboutKey::IconsSizesTones => "Storlekar och toner",
        AboutKey::ClearSearchHover => "Rensa sökning",
        AboutKey::FontLicensesNote => {
            "Noto Sans SC, JP och KR (kinesisk, japansk och koreansk text):"
        }
    })
}
