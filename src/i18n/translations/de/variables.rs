use crate::i18n::keys::VariablesKey;

pub fn text(key: VariablesKey) -> Option<&'static str> {
    Some(match key {
        VariablesKey::Group => "Gruppe",
        VariablesKey::DataType => "Typ",
        VariablesKey::Shape => "Form",
        VariablesKey::Description => "Beschreibung",
        VariablesKey::SearchVariables => "Variablen suchen...",
        VariablesKey::NoVariablesMatching => "Keine Variablen passend zu '{}'",
        VariablesKey::CloseVariablesWindow => "Variablenfenster schließen",
        VariablesKey::NoMetadataLoaded => "Noch keine Store-Metadaten geladen.",
        VariablesKey::FetchStoreMetadata => "Store-Metadaten abrufen / laden",
        VariablesKey::NoVariablesFound => "Keine Variablen in diesem Datensatz gefunden.",
        VariablesKey::RefreshStoreMetadata => "Store-Metadaten aktualisieren",
        VariablesKey::TruncatedVariables => {
            "{} von {} Variablen in diesem Ordner angezeigt. Nutzen Sie die Suche, um alle zu finden."
        }
        VariablesKey::TruncatedFolders => {
            "{} von {} Ordnern angezeigt. Nutzen Sie die Suche, um alle zu finden."
        }
        VariablesKey::NoVariableSelected => "Keine Variable ausgewählt.",
        VariablesKey::DimensionSliders => "Dimensionsregler",
        VariablesKey::CloseDimensionPanel => "Dimensionsfenster schließen",
        VariablesKey::PlotData => "Plotten",
        VariablesKey::PlotDataHover => "Diese Variable abrufen und plotten",
        VariablesKey::PlotDataOverlayHover => {
            "Diese Variable abrufen und als Überlagerung über den Plot legen"
        }
        VariablesKey::AddOverlay => "Überlagern",
        VariablesKey::AddOverlayHover => {
            "Wenn aktiv, fügt Plotten diese Variable als Überlagerung hinzu"
        }
        VariablesKey::InfoPath => "Pfad:",
        VariablesKey::InfoUnits => "Einheiten: {}",
        VariablesKey::InfoDescription => "Beschreibung: {}",
        VariablesKey::InfoShape => "Form: {}",
        VariablesKey::InfoDimensions => "Dimensionen: {}",
        VariablesKey::InfoTime => "Zeit: {} -> {}",
        VariablesKey::InfoResolution => "Auflösung: {}",
        VariablesKey::InfoSize => "Größe: {} MB",
        VariablesKey::InfoAttributes => "Attribute (.zattrs)",
        VariablesKey::Download => "Herunterladen",
        VariablesKey::DownloadCells => "{} ({} Zellen)",
        VariablesKey::LargeSelectionPyramid => {
            "Große 2D-Auswahl ({} Zellen, {} MB): Automatische Multi-Resolution-Pyramidenaggregation ist aktiviert."
        }
        VariablesKey::LargeSelectionNoMeshes => {
            "3D-Globus- und 3D-Oberflächennetze sind für diese große Auswahl deaktiviert ({} Zellen, {} MB). 2D-Fläche und 1D-Linie bleiben voll verfügbar."
        }
        VariablesKey::LargeVolumeDisabled => {
            "3D-Volumen und Punktwolke sind für diese Auswahl deaktiviert: Die Volumengröße ({} MB) überschreitet das GPU-Speicherpufferlimit von {} MB. 2D-Fläche, 1D-Linie und 3D-Globus bleiben verfügbar."
        }
        VariablesKey::RoleNone => "Keine",
        VariablesKey::RoleAnimated => "Animiert",
        VariablesKey::RoleGrid => "Gitter",
        VariablesKey::Index => "Index:",
        VariablesKey::ChannelN => "Kanal {}",
    })
}
