use crate::i18n::keys::NoticesKey;

pub fn text(key: NoticesKey) -> Option<&'static str> {
    Some(match key {
        NoticesKey::DismissHover => "Lukk",
        NoticesKey::CopyDetails => "Kopier detaljer",
        NoticesKey::RevealInFinder => "Vis i Finder",
        NoticesKey::OpenFolder => "Åpne mappe",
        NoticesKey::NoVariablesFound => "Fant ingen variabler",
        NoticesKey::CouldntOpenDataset => "Kunne ikke åpne datasettet",
        NoticesKey::UnsupportedFile => "Filen støttes ikke",
        NoticesKey::CouldntLoadData => "Kunne ikke laste inn data",
        NoticesKey::CoordinatesUnavailable => "Koordinater utilgjengelige",
        NoticesKey::CoordinatesFailedDetail => {
            "Koordinatene til '{}' kunne ikke lastes inn ({}); aksene viser indekser"
        }
        NoticesKey::CoastlinesUnavailable => "Kystlinjer utilgjengelige",
        NoticesKey::VolumeTooLargeForGpu => "Volumet er for stort for GPU-en",
        NoticesKey::VolumeBufferTooLargeDetail => {
            "Størrelsen på volumbufferen ({} elementer / {} byte) overskrider GPU-ens maksimale lagringsbuffer ({})"
        }
        NoticesKey::VolumeTextureTooLargeDetail => {
            "Volumet {}x{}x{} overskrider GPU-ens grense for 3D-teksturer ({})"
        }
        NoticesKey::VolumeUploadSkipped => "Opplasting av volum hoppet over",
        NoticesKey::VolumeUploadMismatchDetail => "{} verdier for en {}x{}x{}-tekstur",
        NoticesKey::DataTooLargeForGpuBuffer => "Dataene er for store for GPU-bufferen",
        NoticesKey::BufferTooLargeDetail => {
            "{}: datastørrelsen ({} byte) overskrider kapasiteten til GPU-bufferen ({} byte), skriving hoppes over"
        }
        NoticesKey::BufferRangeTooLargeDetail => {
            "{}: området ved element {} (+{} elementer) overskrider kapasiteten til GPU-bufferen ({} byte), skriving hoppes over"
        }
        NoticesKey::LinePlotTooLargeForGpu => "Linjeplottet er for stort for GPU-en",
        NoticesKey::LinePayloadTooLargeDetail => {
            "Linjene tar {} byte; denne GPU-en binder høyst {}."
        }
        NoticesKey::FontLoadFailed => "Kunne ikke laste inn en skrift",
        NoticesKey::FontLoadFailedDetail => "Tekst på {} kan vises som bokser: {}",
        NoticesKey::OmeMetadataNonConforming => "OME-Zarr-metadata følger ikke spesifikasjonen",
        NoticesKey::LabelIdsBeyond32Bits => "Etikett-id over 32 biter deler farge med mindre id",
        NoticesKey::OmeMetadataNonConformingDetail => {
            "Metadata i versjon {} for {} følger ikke spesifikasjonen; det som kunne leses, vises."
        }
        NoticesKey::PlateWellStopped => "Brønn {}: lesingen ble avbrutt",
        NoticesKey::PlateWellNotJson => "Brønn {}: metadataene er ikke gyldig JSON",
        NoticesKey::PlateWellNotOme => "Brønn {}: ikke en OME-Zarr-brønn",
        NoticesKey::PlateWellNoMetadata => "Brønn {}: ingen brønnmetadata",
    })
}
