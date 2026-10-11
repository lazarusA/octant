//! Norwegian (Norsk) translations for Octant.

use crate::i18n::keys::Key;

pub fn translate(key: Key) -> Option<&'static str> {
    Some(match key {
        Key::Dataset => "Datasett",
        Key::Variables => "Variabler",
        Key::Dimensions => "Dimensjoner",
        Key::Plot => "Plott",
        Key::Colormaps => "Fargekart",
        Key::Settings => "Innstillinger",
        Key::About => "Om",

        Key::InspectMetadata => "Inspiser metadata",
        Key::ActiveDatasets => "Aktive datasett",
        Key::NoActiveDatasets => "Ingen aktive datasett. Åpne et datasett for å starte.",
        Key::CacheUsage => "Bruk av hurtigbuffer",
        Key::ClearCache => "Tøm hurtigbuffer",
        Key::OpenLocal => "Åpne lokal fil",
        Key::CloudStore => "Skylagring",
        Key::SampleDatasets => "Eksempeldatasett",
        Key::DropFilesHere => "Slipp filer eller mapper her",
        Key::SourceUrlOrPath => "Lagrings-URI eller lokal bane",
        Key::Inspect => "Inspiser",
        Key::LoadingMetadata => "Inspiserer metadata i bakgrunnen...",

        Key::Group => "Gruppe:",
        Key::DataType => "Type",
        Key::Shape => "Form",
        Key::Description => "Beskrivelse",
        Key::SearchVariables => "Søk etter variabler...",
        Key::NoVariablesMatching => "Ingen variabler samsvarer med søket",
        Key::ShowingItemsOfTotal => "Viser elementer av totalt",

        Key::Ready => "Klar",
        Key::Loading => "Laster...",
        Key::Close => "Lukk",
        Key::Cancel => "Avbryt",
        Key::Save => "Lagre",
        Key::Reset => "Tilbakestill",
        Key::Error => "Feil",

        Key::Play => "Spill av",
        Key::Pause => "Pause",
        Key::StepForward => "Trinn fremover",
        Key::StepBackward => "Trinn bakover",
        Key::Speed => "Hastighet",
        Key::Loop => "Gjenta",

        Key::AboutSubtitle => "N-Dimensjonal Datavisning",
        Key::AboutDescription => {
            "Octant er en interaktiv visning for N-dimensjonale datasett med innebygd støtte for lokal og skylagring, Zarr (v2/v3) og Icechunk. Bygget i Rust med GPU-akselerert rendering via WGPU. Kjører på macOS, Linux og Windows."
        }
        Key::AboutHyperslabTitle => "Hyperslab-snitt",
        Key::AboutHyperslabDesc => "Asynkron LRU-mellomlagring og multi-oppløselige pyramider.",
        Key::AboutStorageTitle => "Innebygd Zarr og Icechunk",
        Key::AboutStorageDesc => "Lokale, S3, GCS, Azure og HTTP-strømmingsgrensesnitt.",
        Key::AboutShadersTitle => "Maskinvareakselererte WGPU-shadere",
        Key::AboutShadersDesc => "2D-kart, 3D-sfærer, høydeflater, volumer og 1D-profiler.",
        Key::AboutLinksHeading => "Lenker og ressurser",
        Key::AboutGoodFaithNotice => {
            "Octant anerkjenner tredjepartsarbeid i god tro. Vennligst gi beskjed dersom noe mangler eller er uriktig tilskrevet."
        }
        Key::AboutBrowseIcons => "Bla gjennom innebygde vektorikoner",
    })
}
