use crate::i18n::keys::AboutKey;

pub fn text(key: AboutKey) -> Option<&'static str> {
    Some(match key {
        AboutKey::WindowTitle => "Om Octant",
        AboutKey::IconsWindowTitle => "Innebygde vektorikoner",
        AboutKey::IconsTab => "Vektorikoner",
        AboutKey::Subtitle => "Utforsker for N-dimensjonale data",
        AboutKey::Description => {
            "Octant er en interaktiv visning for N-dimensjonale datasett med innebygd støtte for lokal lagring og objektlagring i skyen, Zarr (v2/v3) og Icechunk. Bygget i Rust med GPU-akselerert rendering via WGPU. Octant kjører nativt på macOS, Linux og Windows."
        }
        AboutKey::HyperslabTitle => "Hyperslab-snitt",
        AboutKey::HyperslabDesc => "Asynkron LRU-hurtigbuffer for blokker og flernivåpyramider.",
        AboutKey::StorageTitle => "Innebygd Zarr og Icechunk",
        AboutKey::StorageDesc => "Lokale, S3, GCS, Azure og HTTP-strømmingsgrensesnitt.",
        AboutKey::ShadersTitle => "Maskinvareakselererte WGPU-shadere",
        AboutKey::ShadersDesc => "2D-kart, 3D-sfærer, høydeflater, volumer og 1D-profiler.",
        AboutKey::LinksHeading => "Lenker og ressurser",
        AboutKey::DocsLink => "Octant-dokumentasjon",
        AboutKey::GoodFaithNotice => {
            "Octant anerkjenner tredjepartsarbeid i god tro. Hvis du mener at noe mangler eller er feil tilskrevet, gi oss beskjed, så retter eller fjerner vi det."
        }
        AboutKey::BrowseIcons => "Bla gjennom innebygde vektorikoner",
        AboutKey::BrowseIconsHover => "Utforsk prosedyriske vektorikoner i et vindu",
        AboutKey::Licensed => "Lisensiert under MIT eller Apache-2.0",
        AboutKey::OctantTeaser => {
            "Fortegnene til x, y og z (+/-, +/-, +/-) deler rommet i åtte oktanter. Utforsk dataene dine på samme måte: velg aksene, skjær ut et snitt og se på én region om gangen."
        }
        AboutKey::OctantHover => "Klikk for å gå til neste oktant",
        AboutKey::ColormapCredits => "Fargekart: kreditering",
        AboutKey::ColormapCreditsNote => {
            "De medfølgende fargekartene beholder lisensene til opphavspersonene sine."
        }
        AboutKey::LicenseTexts => "Lisenstekster",
        AboutKey::ThirdPartyLicenses => "Tredjepartslisenser",
        AboutKey::ThirdPartyNote => {
            "Octant inneholder åpen kildekode-crates, skrifttyper og C-biblioteker (netCDF, HDF5), hver under sin egen lisens."
        }
        AboutKey::ViewOnGithub => "Vis på GitHub",
        AboutKey::IconsSearchHint => "Søk etter ikoner...",
        AboutKey::IconsAll => "Alle",
        AboutKey::IconsCopied => "Kopierte {}",
        AboutKey::IconsNoMatch => "Fant ingen samsvarende vektorikoner.",
        AboutKey::IconsCategory => "Kategori: {}",
        AboutKey::IconsClickToCopy => "Klikk for å kopiere enum-identifikatoren",
        AboutKey::IconsSizesTones => "Størrelser og toner",
        AboutKey::ClearSearchHover => "Tøm søk",
        AboutKey::FontLicensesNote => {
            "Noto Sans SC, JP og KR (kinesisk, japansk og koreansk tekst):"
        }
    })
}
