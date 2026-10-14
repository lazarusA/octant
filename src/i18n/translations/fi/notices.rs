use crate::i18n::keys::NoticesKey;

pub fn text(key: NoticesKey) -> Option<&'static str> {
    Some(match key {
        NoticesKey::DismissHover => "Sulje",
        NoticesKey::CopyDetails => "Kopioi tiedot",
        NoticesKey::RevealInFinder => "Näytä Finderissa",
        NoticesKey::OpenFolder => "Avaa kansio",
        NoticesKey::NoVariablesFound => "Muuttujia ei löytynyt",
        NoticesKey::CouldntOpenDataset => "Aineiston avaaminen epäonnistui",
        NoticesKey::UnsupportedFile => "Tiedostoa ei tueta",
        NoticesKey::CouldntLoadData => "Datan lataaminen epäonnistui",
        NoticesKey::CoordinatesUnavailable => "Koordinaatteja ei saatavilla",
        NoticesKey::CoordinatesFailedDetail => {
            "Muuttujan '{}' koordinaattien lataus epäonnistui ({}); akseleilla näytetään indeksit"
        }
        NoticesKey::CoastlinesUnavailable => "Rantaviivoja ei saatavilla",
        NoticesKey::VolumeTooLargeForGpu => "Tilavuus on liian suuri GPU:lle",
        NoticesKey::VolumeBufferTooLargeDetail => {
            "Tilavuusdatan puskurin koko ({} alkiota / {} tavua) ylittää GPU:n tallennuspuskurin enimmäiskoon ({})"
        }
        NoticesKey::VolumeTextureTooLargeDetail => {
            "Tilavuus {}x{}x{} ylittää GPU:n 3D-tekstuurirajan ({})"
        }
        NoticesKey::VolumeUploadSkipped => "Tilavuuden lähetys ohitettiin",
        NoticesKey::VolumeUploadMismatchDetail => "{} arvoa {}x{}x{}-tekstuurille",
        NoticesKey::DataTooLargeForGpuBuffer => "Data on liian suuri GPU-puskurille",
        NoticesKey::BufferTooLargeDetail => {
            "{}: datan koko ({} tavua) ylittää GPU-puskurin kapasiteetin ({} tavua), kirjoitus ohitetaan"
        }
        NoticesKey::BufferRangeTooLargeDetail => {
            "{}: alue alkiosta {} (+{} alkiota) ylittää GPU-puskurin kapasiteetin ({} tavua), kirjoitus ohitetaan"
        }
        NoticesKey::LinePlotTooLargeForGpu => "Viivakuvaaja on liian suuri GPU:lle",
        NoticesKey::LinePayloadTooLargeDetail => {
            "Viivat vievät {} tavua; tämä GPU sitoo enintään {}."
        }
        NoticesKey::FontLoadFailed => "Fonttia ei voitu ladata",
        NoticesKey::FontLoadFailedDetail => "Teksti ({}) voi näkyä laatikkoina: {}",
        NoticesKey::DataFontMissing => "Osalle aineiston tekstistä ei löydy fonttia",
        NoticesKey::DataFontMissingDetail => {
            "Aineiston teksti ({}) voi näkyä laatikkoina. Asenna fontti, kuten Noto Sans CJK."
        }
        NoticesKey::OmeMetadataNonConforming => "OME-Zarr-metatiedot eivät noudata määritystä",
        NoticesKey::LabelIdsBeyond32Bits => {
            "Yli 32-bittiset tunnisteet jakavat värin pienempien kanssa"
        }
        NoticesKey::OmeMetadataNonConformingDetail => {
            "Version {} metatiedot kohteessa {} eivät noudata määritystä; näytetään se, mikä voitiin lukea."
        }
    })
}
