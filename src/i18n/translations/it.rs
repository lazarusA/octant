//! Italian (Italiano) translations for Octant.

use crate::i18n::keys::Key;

pub fn translate(key: Key) -> Option<&'static str> {
    Some(match key {
        Key::Dataset => "Set di dati",
        Key::Variables => "Variabili",
        Key::Dimensions => "Dimensioni",
        Key::Plot => "Grafico",
        Key::Colormaps => "Mappe di colore",
        Key::Settings => "Impostazioni",
        Key::About => "Informazioni",

        Key::InspectMetadata => "Ispeziona metadati",
        Key::ActiveDatasets => "Set di dati attivi",
        Key::NoActiveDatasets => "Nessun set attivo. Apri un set di dati per iniziare.",
        Key::CacheUsage => "Utilizzo cache",
        Key::ClearCache => "Svuota cache",
        Key::OpenLocal => "Apri file locale",
        Key::CloudStore => "Archivio oggetti cloud",
        Key::SampleDatasets => "Set di dati di esempio",
        Key::DropFilesHere => "Trascina file o cartelle qui",
        Key::SourceUrlOrPath => "URI archivio o percorso locale",
        Key::Inspect => "Ispeziona",
        Key::LoadingMetadata => "Ispezione metadati in corso...",

        Key::Group => "Gruppo:",
        Key::DataType => "Tipo",
        Key::Shape => "Forma",
        Key::Description => "Descrizione",
        Key::SearchVariables => "Cerca variabili...",
        Key::NoVariablesMatching => "Nessuna variabile corrispondente",
        Key::ShowingItemsOfTotal => "Elementi mostrati sul totale",

        Key::Ready => "Pronto",
        Key::Loading => "Caricamento...",
        Key::Close => "Chiudi",
        Key::Cancel => "Annulla",
        Key::Save => "Salva",
        Key::Reset => "Reimposta",
        Key::Error => "Errore",

        Key::Play => "Riproduci",
        Key::Pause => "Pausa",
        Key::StepForward => "Passo avanti",
        Key::StepBackward => "Passo indietro",
        Key::Speed => "Velocità",
        Key::Loop => "Ripeti",

        Key::AboutSubtitle => "Esploratore di dati N-dimensionali",
        Key::AboutDescription => {
            "Octant è un visualizzatore interattivo per dataset N-dimensionali con supporto nativo per archiviazione locale e cloud, Zarr (v2/v3) e Icechunk. Sviluppato in Rust con rendering accelerato da GPU tramite WGPU. Funziona su macOS, Linux e Windows."
        }
        Key::AboutHyperslabTitle => "Sezionamento iperslab",
        Key::AboutHyperslabDesc => "Cache LRU asincrona e piramidi a risoluzione multipla.",
        Key::AboutStorageTitle => "Nativo per Zarr e Icechunk",
        Key::AboutStorageDesc => "Backend locali, S3, GCS, Azure e streaming HTTP.",
        Key::AboutShadersTitle => "Shader WGPU accelerati via hardware",
        Key::AboutShadersDesc => {
            "Mappe 2D, sfere 3D, superfici di elevazione, volumi e profili 1D."
        }
        Key::AboutLinksHeading => "Collegamenti e risorse",
        Key::AboutGoodFaithNotice => {
            "Octant riconosce i lavori di terze parti in buona fede. Per correzioni o omissioni, vi invitiamo a contattarci."
        }
        Key::AboutBrowseIcons => "Esplora icone vettoriali native",
    })
}
