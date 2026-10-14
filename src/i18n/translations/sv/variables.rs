use crate::i18n::keys::VariablesKey;

pub fn text(key: VariablesKey) -> Option<&'static str> {
    Some(match key {
        VariablesKey::Group => "Grupp",
        VariablesKey::DataType => "Typ",
        VariablesKey::Shape => "Form",
        VariablesKey::Description => "Beskrivning",
        VariablesKey::SearchVariables => "Sök variabler...",
        VariablesKey::NoVariablesMatching => "Inga variabler matchar '{}'",
        VariablesKey::CloseVariablesWindow => "Stäng variabelfönstret",
        VariablesKey::NoMetadataLoaded => "Ingen lagringsmetadata inläst än.",
        VariablesKey::FetchStoreMetadata => "Hämta / läs in lagringsmetadata",
        VariablesKey::NoVariablesFound => "Inga variabler hittades i denna datasetlagring.",
        VariablesKey::RefreshStoreMetadata => "Uppdatera lagringsmetadata",
        VariablesKey::TruncatedVariables => {
            "Visar {} av {} variabler i denna mapp. Använd sökningen för att hitta alla."
        }
        VariablesKey::TruncatedFolders => {
            "Visar {} av {} mappar. Använd sökningen för att hitta alla."
        }
        VariablesKey::NoVariableSelected => "Ingen variabel vald.",
        VariablesKey::DimensionSliders => "Dimensionsreglage",
        VariablesKey::CloseDimensionPanel => "Stäng dimensionspanelen",
        VariablesKey::PlotData => "Plotta data",
        VariablesKey::PlotDataHover => "Hämta denna variabel och plotta den",
        VariablesKey::PlotDataOverlayHover => {
            "Hämta denna variabel och lägg till den som ett överlägg på diagrammet"
        }
        VariablesKey::AddOverlay => "Lägg till överlägg",
        VariablesKey::AddOverlayHover => {
            "När detta är på lägger Plotta data till variabeln som ett överlägg"
        }
        VariablesKey::InfoPath => "Sökväg:",
        VariablesKey::InfoUnits => "Enheter: {}",
        VariablesKey::InfoDescription => "Beskrivning: {}",
        VariablesKey::InfoShape => "Form: {}",
        VariablesKey::InfoDimensions => "Dimensioner: {}",
        VariablesKey::InfoTime => "Tid: {} -> {}",
        VariablesKey::InfoResolution => "Upplösning: {}",
        VariablesKey::InfoSize => "Storlek: {} MB",
        VariablesKey::InfoAttributes => "Attribut (.zattrs)",
        VariablesKey::Download => "Nedladdning",
        VariablesKey::DownloadCells => "{} ({} celler)",
        VariablesKey::LargeSelectionPyramid => {
            "Stort 2D-urval ({} celler, {} MB): automatisk aggregering i flerupplösta pyramider är aktiverad."
        }
        VariablesKey::LargeSelectionNoMeshes => {
            "3D-glob och 3D-yta är avaktiverade för detta stora urval ({} celler, {} MB). 2D-plan och 1D-linje är fortfarande helt tillgängliga."
        }
        VariablesKey::LargeVolumeDisabled => {
            "3D-volym och punktmoln är avaktiverade för detta urval: volymens storlek ({} MB) överskrider GPU:ns gräns på {} MB för lagringsbuffertar. 2D-plan, 1D-linje och 3D-glob är fortfarande tillgängliga."
        }
        VariablesKey::RoleNone => "Ingen",
        VariablesKey::RoleAnimated => "Animerad",
        VariablesKey::RoleGrid => "Rutnät",
        VariablesKey::Index => "Index:",
        VariablesKey::ChannelN => "Kanal {}",
    })
}
