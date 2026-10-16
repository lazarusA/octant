use crate::i18n::keys::VariablesKey;

pub fn text(key: VariablesKey) -> Option<&'static str> {
    Some(match key {
        VariablesKey::Group => "Gruppo",
        VariablesKey::DataType => "Tipo",
        VariablesKey::Shape => "Forma",
        VariablesKey::Description => "Descrizione",
        VariablesKey::SearchVariables => "Cerca variabili...",
        VariablesKey::NoVariablesMatching => "Nessuna variabile corrisponde a '{}'",
        VariablesKey::CloseVariablesWindow => "Chiudi la finestra delle variabili",
        VariablesKey::NoMetadataLoaded => "Nessun metadato dell'archivio ancora caricato.",
        VariablesKey::FetchStoreMetadata => "Recupera / carica i metadati dell'archivio",
        VariablesKey::NoVariablesFound => "Nessuna variabile trovata in questo archivio.",
        VariablesKey::RefreshStoreMetadata => "Aggiorna i metadati dell'archivio",
        VariablesKey::TruncatedVariables => {
            "Mostrate {} di {} variabili in questa cartella. Usa la ricerca per trovarle tutte."
        }
        VariablesKey::TruncatedFolders => {
            "Mostrate {} di {} cartelle. Usa la ricerca per trovarle tutte."
        }
        VariablesKey::NoVariableSelected => "Nessuna variabile selezionata.",
        VariablesKey::DimensionSliders => "Cursori delle dimensioni",
        VariablesKey::CloseDimensionPanel => "Chiudi il pannello delle dimensioni",
        VariablesKey::PlotData => "Traccia dati",
        VariablesKey::PlotDataHover => "Recupera questa variabile e tracciala",
        VariablesKey::PlotDataOverlayHover => {
            "Recupera questa variabile e aggiungila come sovrapposizione al grafico"
        }
        VariablesKey::AddOverlay => "Sovrapponi",
        VariablesKey::AddOverlayHover => {
            "Se attivo, 'Traccia dati' aggiunge questa variabile come sovrapposizione"
        }
        VariablesKey::InfoPath => "Percorso:",
        VariablesKey::InfoUnits => "Unità: {}",
        VariablesKey::InfoDescription => "Descrizione: {}",
        VariablesKey::InfoShape => "Forma: {}",
        VariablesKey::InfoDimensions => "Dimensioni: {}",
        VariablesKey::InfoTime => "Tempo: {} -> {}",
        VariablesKey::InfoResolution => "Risoluzione: {}",
        VariablesKey::InfoSize => "Dimensione: {} MB",
        VariablesKey::InfoAttributes => "Attributi (.zattrs)",
        VariablesKey::Download => "Scarica",
        VariablesKey::DownloadCells => "{} ({} celle)",
        VariablesKey::LargeSelectionPyramid => {
            "Selezione 2D grande ({} celle, {} MB): l'aggregazione automatica a piramide multirisoluzione è attiva."
        }
        VariablesKey::LargeSelectionNoMeshes => {
            "Le mesh 3D Globo e 3D Superficie sono disattivate per questa selezione grande ({} celle, {} MB). I grafici Piano 2D e Linea 1D restano pienamente attivi."
        }
        VariablesKey::LargeVolumeDisabled => {
            "Volume 3D e Nuvola di punti sono disattivati per questa selezione: la dimensione del volume ({} MB) supera il limite di {} MB del buffer di archiviazione della GPU. Piano 2D, Linea 1D e Globo 3D restano attivi."
        }
        VariablesKey::RoleNone => "Nessuna",
        VariablesKey::RoleAnimated => "Animata",
        VariablesKey::RoleGrid => "Griglia",
        VariablesKey::Index => "Indice:",
        VariablesKey::ChannelN => "Canale {}",
        VariablesKey::LevelN => "Livello {}",
        VariablesKey::LevelAuto => "Auto",
        VariablesKey::LevelOf => "Livello {} di {}",
        VariablesKey::AutoLevelOf => "Auto, livello {} di {}",
    })
}
