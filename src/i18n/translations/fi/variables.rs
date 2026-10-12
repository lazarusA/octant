use crate::i18n::keys::VariablesKey;

pub fn text(key: VariablesKey) -> Option<&'static str> {
    Some(match key {
        VariablesKey::Group => "Ryhmä",
        VariablesKey::DataType => "Tyyppi",
        VariablesKey::Shape => "Muoto",
        VariablesKey::Description => "Kuvaus",
        VariablesKey::SearchVariables => "Etsi muuttujia...",
        VariablesKey::NoVariablesMatching => "Haulla '{}' ei löytynyt muuttujia",
        VariablesKey::CloseVariablesWindow => "Sulje muuttujaikkuna",
        VariablesKey::NoMetadataLoaded => "Säilön metatietoja ei ole vielä ladattu.",
        VariablesKey::FetchStoreMetadata => "Hae / lataa säilön metatiedot",
        VariablesKey::NoVariablesFound => "Tästä aineistosäilöstä ei löytynyt muuttujia.",
        VariablesKey::RefreshStoreMetadata => "Päivitä säilön metatiedot",
        VariablesKey::TruncatedVariables => {
            "Näytetään {} / {} muuttujaa tässä kansiossa. Löydä kaikki haulla."
        }
        VariablesKey::TruncatedFolders => "Näytetään {} / {} kansiota. Löydä kaikki haulla.",
        VariablesKey::NoVariableSelected => "Muuttujaa ei ole valittu.",
        VariablesKey::DimensionSliders => "Ulottuvuuksien liukusäätimet",
        VariablesKey::CloseDimensionPanel => "Sulje ulottuvuuspaneeli",
        VariablesKey::PlotData => "Piirrä data",
        VariablesKey::PlotDataHover => "Hae tämä muuttuja ja piirrä se",
        VariablesKey::PlotDataOverlayHover => {
            "Hae tämä muuttuja ja lisää se päällystasoksi kuvaajan päälle"
        }
        VariablesKey::AddOverlay => "Lisää päällystaso",
        VariablesKey::AddOverlayHover => {
            "Kun päällä, 'Piirrä data' lisää tämän muuttujan päällystasoksi"
        }
        VariablesKey::InfoPath => "Polku:",
        VariablesKey::InfoUnits => "Yksiköt: {}",
        VariablesKey::InfoDescription => "Kuvaus: {}",
        VariablesKey::InfoShape => "Muoto: {}",
        VariablesKey::InfoDimensions => "Ulottuvuudet: {}",
        VariablesKey::InfoTime => "Aika: {} -> {}",
        VariablesKey::InfoResolution => "Resoluutio: {}",
        VariablesKey::InfoSize => "Koko: {} MB",
        VariablesKey::InfoAttributes => "Attribuutit (.zattrs)",
        VariablesKey::Download => "Lataus",
        VariablesKey::DownloadCells => "{} ({} solua)",
        VariablesKey::LargeSelectionPyramid => {
            "Suuri 2D-valinta ({} solua, {} MB): automaattinen moniresoluutiopyramidi on käytössä."
        }
        VariablesKey::LargeSelectionNoMeshes => {
            "3D-pallon ja 3D-pinnan verkot on poistettu käytöstä tälle suurelle valinnalle ({} solua, {} MB). 2D-taso- ja 1D-viivakuvaajat toimivat täysin."
        }
        VariablesKey::LargeVolumeDisabled => {
            "3D-tilavuus ja pistepilvi on poistettu käytöstä tälle valinnalle: tilavuuden koko ({} MB) ylittää 128 MB:n GPU-tallennuspuskurin rajan. 2D-taso, 1D-viiva ja 3D-pallo ovat käytössä."
        }
        VariablesKey::RoleNone => "Ei",
        VariablesKey::RoleAnimated => "Animoitu",
        VariablesKey::RoleGrid => "Ruudukko",
        VariablesKey::Index => "Indeksi:",
        VariablesKey::ChannelN => "Kanava {}",
    })
}
