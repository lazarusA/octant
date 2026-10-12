use crate::i18n::keys::VariablesKey;

pub fn text(key: VariablesKey) -> Option<&'static str> {
    Some(match key {
        VariablesKey::Group => "Gruppe",
        VariablesKey::DataType => "Type",
        VariablesKey::Shape => "Form",
        VariablesKey::Description => "Beskrivelse",
        VariablesKey::SearchVariables => "Søk etter variabler...",
        VariablesKey::NoVariablesMatching => "Ingen variabler samsvarer med '{}'",
        VariablesKey::CloseVariablesWindow => "Lukk variabelvinduet",
        VariablesKey::NoMetadataLoaded => "Ingen metadata er lastet inn ennå.",
        VariablesKey::FetchStoreMetadata => "Hent / last inn metadata",
        VariablesKey::NoVariablesFound => "Fant ingen variabler i dette datasettet.",
        VariablesKey::RefreshStoreMetadata => "Oppdater metadata",
        VariablesKey::TruncatedVariables => {
            "Viser {} av {} variabler i denne mappen. Bruk søk for å finne alle."
        }
        VariablesKey::TruncatedFolders => "Viser {} av {} mapper. Bruk søk for å finne alle.",
        VariablesKey::NoVariableSelected => "Ingen variabel valgt.",
        VariablesKey::DimensionSliders => "Dimensjonsglidere",
        VariablesKey::CloseDimensionPanel => "Lukk dimensjonspanelet",
        VariablesKey::PlotData => "Plott data",
        VariablesKey::PlotDataHover => "Hent denne variabelen og plott den",
        VariablesKey::PlotDataOverlayHover => {
            "Hent denne variabelen og legg den til som overlegg over plottet"
        }
        VariablesKey::AddOverlay => "Legg til overlegg",
        VariablesKey::AddOverlayHover => {
            "Når denne er på, legger Plott data til variabelen som overlegg"
        }
        VariablesKey::InfoPath => "Bane:",
        VariablesKey::InfoUnits => "Enheter: {}",
        VariablesKey::InfoDescription => "Beskrivelse: {}",
        VariablesKey::InfoShape => "Form: {}",
        VariablesKey::InfoDimensions => "Dimensjoner: {}",
        VariablesKey::InfoTime => "Tid: {} -> {}",
        VariablesKey::InfoResolution => "Oppløsning: {}",
        VariablesKey::InfoSize => "Størrelse: {} MB",
        VariablesKey::InfoAttributes => "Attributter (.zattrs)",
        VariablesKey::Download => "Nedlasting",
        VariablesKey::DownloadCells => "{} ({} celler)",
        VariablesKey::LargeSelectionPyramid => {
            "Stort 2D-utvalg ({} celler, {} MB): automatisk aggregering med flernivåpyramide er slått på."
        }
        VariablesKey::LargeSelectionNoMeshes => {
            "3D-globus og 3D-overflate er slått av for dette store utvalget ({} celler, {} MB). 2D-plan og 1D-linje er fortsatt tilgjengelige."
        }
        VariablesKey::LargeVolumeDisabled => {
            "3D-volum og punktsky er slått av for dette utvalget: volumstørrelsen ({} MB) overskrider grensen på 128 MB for GPU-lagringsbuffere. 2D-plan, 1D-linje og 3D-globus er fortsatt tilgjengelige."
        }
        VariablesKey::RoleNone => "Ingen",
        VariablesKey::RoleAnimated => "Animert",
        VariablesKey::RoleGrid => "Rutenett",
        VariablesKey::Index => "Indeks:",
        VariablesKey::ChannelN => "Kanal {}",
    })
}
