//! Swedish (Svenska) translations for Octant.

use crate::i18n::keys::Key;

pub fn translate(key: Key) -> Option<&'static str> {
    Some(match key {
        Key::Dataset => "Dataset",
        Key::Variables => "Variabler",
        Key::Dimensions => "Dimensioner",
        Key::Plot => "Rita",
        Key::Colormaps => "Färgkartor",
        Key::Settings => "Inställningar",
        Key::About => "Om",

        Key::InspectMetadata => "Inspektera metadata",
        Key::ActiveDatasets => "Aktiva dataset",
        Key::NoActiveDatasets => "Inga aktiva dataset. Öppna ett dataset för att börja.",
        Key::CacheUsage => "Cacheanvändning",
        Key::ClearCache => "Rensa cache",
        Key::OpenLocal => "Öppna lokal fil",
        Key::CloudStore => "Molnlagring",
        Key::SampleDatasets => "Exempeldataset",
        Key::DropFilesHere => "Släpp filer eller mappar här",
        Key::SourceUrlOrPath => "Lagrings-URI eller lokal sökväg",
        Key::Inspect => "Inspektera",
        Key::LoadingMetadata => "Inspekterar lagringsmetadata i bakgrunden...",

        Key::Group => "Grupp:",
        Key::DataType => "Typ",
        Key::Shape => "Form",
        Key::Description => "Beskrivning",
        Key::SearchVariables => "Sök variabler...",
        Key::NoVariablesMatching => "Inga variabler matchar sökningen",
        Key::ShowingItemsOfTotal => "Visar element av totalt",

        Key::Ready => "Klar",
        Key::Loading => "Laddar...",
        Key::Close => "Stäng",
        Key::Cancel => "Avbryt",
        Key::Save => "Spara",
        Key::Reset => "Återställ",
        Key::Error => "Fel",

        Key::Play => "Spela",
        Key::Pause => "Pausa",
        Key::StepForward => "Stega framåt",
        Key::StepBackward => "Stega bakåt",
        Key::Speed => "Hastighet",
        Key::Loop => "Slinga",

        Key::AboutSubtitle => "N-Dimensionell Datavisare",
        Key::AboutDescription => {
            "Octant är en interaktiv visare för N-dimensionella dataset med inbyggt stöd för lokal och molnbaserad lagring, Zarr (v2/v3) och Icechunk. Byggd i Rust med GPU-accelererad rendering via WGPU. Körs på macOS, Linux och Windows."
        }
        Key::AboutHyperslabTitle => "Hyperslab-snitt",
        Key::AboutHyperslabDesc => "Asynkron LRU-cache och flerupplösta pyramider.",
        Key::AboutStorageTitle => "Inbyggt Zarr & Icechunk",
        Key::AboutStorageDesc => "Lokala, S3, GCS, Azure och HTTP-strömmande gränssnitt.",
        Key::AboutShadersTitle => "Hårdvaruaccelererade WGPU-shaders",
        Key::AboutShadersDesc => "2D-kartor, 3D-sfärer, höjdytsmodeller, volymer och 1D-profiler.",
        Key::AboutLinksHeading => "Länkar och resurser",
        Key::AboutGoodFaithNotice => {
            "Octant erkänner verk från tredje part i god tro. Om något saknas eller är felaktigt, meddela oss så att vi kan åtgärda det."
        }
        Key::AboutBrowseIcons => "Bläddra bland inbyggda vektorikoner",
    })
}
