use crate::i18n::keys::NoticesKey;

pub fn text(key: NoticesKey) -> Option<&'static str> {
    Some(match key {
        NoticesKey::DismissHover => "Stäng",
        NoticesKey::CopyDetails => "Kopiera detaljer",
        NoticesKey::RevealInFinder => "Visa i Finder",
        NoticesKey::OpenFolder => "Öppna mapp",
        NoticesKey::NoVariablesFound => "Inga variabler hittades",
        NoticesKey::CouldntOpenDataset => "Kunde inte öppna dataset",
        NoticesKey::UnsupportedFile => "Filen stöds inte",
        NoticesKey::CouldntLoadData => "Kunde inte läsa in data",
        NoticesKey::CoordinatesUnavailable => "Koordinater saknas",
        NoticesKey::CoordinatesFailedDetail => {
            "Koordinaterna för '{}' kunde inte läsas in ({}); axlarna visar index"
        }
        NoticesKey::CoastlinesUnavailable => "Kustlinjer saknas",
        NoticesKey::VolumeTooLargeForGpu => "Volymen är för stor för GPU:n",
        NoticesKey::VolumeBufferTooLargeDetail => {
            "Volymdatabuffertens storlek ({} element / {} byte) överskrider GPU:ns maxgräns för lagringsbuffertar ({})"
        }
        NoticesKey::VolumeTextureTooLargeDetail => {
            "Volymen {}x{}x{} överskrider GPU:ns gräns för 3D-texturer ({})"
        }
        NoticesKey::VolumeUploadSkipped => "Volymuppladdning hoppades över",
        NoticesKey::VolumeUploadMismatchDetail => "{} värden för en {}x{}x{}-textur",
        NoticesKey::DataTooLargeForGpuBuffer => "Data för stora för GPU-bufferten",
        NoticesKey::BufferTooLargeDetail => {
            "{}: datastorleken ({} byte) överskrider GPU-buffertens kapacitet ({} byte), skrivningen hoppas över"
        }
        NoticesKey::BufferRangeTooLargeDetail => {
            "{}: intervallet vid element {} (+{} element) överskrider GPU-buffertens kapacitet ({} byte), skrivningen hoppas över"
        }
        NoticesKey::LinePlotTooLargeForGpu => "Linjediagrammet är för stort för GPU:n",
        NoticesKey::LinePayloadTooLargeDetail => "Linjerna tar {} byte; denna GPU binder högst {}.",
        NoticesKey::FontLoadFailed => "Kunde inte läsa in ett typsnitt",
        NoticesKey::FontLoadFailedDetail => "Text på {} kan visas som rutor: {}",
        NoticesKey::DataFontMissing => "Inget typsnitt för en del av datasetets text",
        NoticesKey::DataFontMissingDetail => {
            "Text på {} i detta dataset kan visas som rutor. Installera ett typsnitt som Noto Sans CJK."
        }
        NoticesKey::OmeMetadataNonConforming => "OME-Zarr-metadata följer inte specifikationen",
        NoticesKey::LabelIdsBeyond32Bits => "Etikett-id över 32 bitar delar färg med mindre id",
        NoticesKey::OmeMetadataNonConformingDetail => {
            "Metadata i version {} för {} följer inte specifikationen; det som kunde läsas visas."
        }
    })
}
