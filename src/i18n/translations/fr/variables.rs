use crate::i18n::keys::VariablesKey;

pub fn text(key: VariablesKey) -> Option<&'static str> {
    Some(match key {
        VariablesKey::Group => "Groupe",
        VariablesKey::DataType => "Type",
        VariablesKey::Shape => "Forme",
        VariablesKey::Description => "Description",
        VariablesKey::SearchVariables => "Rechercher des variables...",
        VariablesKey::NoVariablesMatching => "Aucune variable ne correspond à '{}'",
        VariablesKey::CloseVariablesWindow => "Fermer la fenêtre des variables",
        VariablesKey::NoMetadataLoaded => "Aucune métadonnée de dépôt chargée pour l'instant.",
        VariablesKey::FetchStoreMetadata => "Récupérer / charger les métadonnées du dépôt",
        VariablesKey::NoVariablesFound => "Aucune variable trouvée dans ce dépôt de données.",
        VariablesKey::RefreshStoreMetadata => "Actualiser les métadonnées du dépôt",
        VariablesKey::TruncatedVariables => {
            "Affichage de {} variables sur {} dans ce dossier. Utilisez la recherche pour tout trouver."
        }
        VariablesKey::TruncatedFolders => {
            "Affichage de {} dossiers sur {}. Utilisez la recherche pour tout trouver."
        }
        VariablesKey::NoVariableSelected => "Aucune variable sélectionnée.",
        VariablesKey::DimensionSliders => "Curseurs de dimensions",
        VariablesKey::CloseDimensionPanel => "Fermer le panneau des dimensions",
        VariablesKey::PlotData => "Tracer",
        VariablesKey::PlotDataHover => "Récupérer cette variable et la tracer",
        VariablesKey::PlotDataOverlayHover => {
            "Récupérer cette variable et l'ajouter en superposition sur le graphique"
        }
        VariablesKey::AddOverlay => "Superposer",
        VariablesKey::AddOverlayHover => "Activé, Tracer ajoute cette variable en superposition",
        VariablesKey::InfoPath => "Chemin :",
        VariablesKey::InfoUnits => "Unités : {}",
        VariablesKey::InfoDescription => "Description : {}",
        VariablesKey::InfoShape => "Forme : {}",
        VariablesKey::InfoDimensions => "Dimensions : {}",
        VariablesKey::InfoTime => "Temps : {} -> {}",
        VariablesKey::InfoResolution => "Résolution : {}",
        VariablesKey::InfoSize => "Taille : {} MB",
        VariablesKey::InfoAttributes => "Attributs (.zattrs)",
        VariablesKey::Download => "Télécharger",
        VariablesKey::DownloadCells => "{} ({} cellules)",
        VariablesKey::LargeSelectionPyramid => {
            "Grande sélection 2D ({} cellules, {} MB) : l'agrégation automatique en pyramide multi-résolutions est activée."
        }
        VariablesKey::LargeSelectionNoMeshes => {
            "Les maillages Globe 3D et Surface 3D sont désactivés pour cette grande sélection ({} cellules, {} MB). Les graphiques Plan 2D et Ligne 1D restent entièrement disponibles."
        }
        VariablesKey::LargeVolumeDisabled => {
            "Volume 3D et Nuage de points sont désactivés pour cette sélection : la taille du volume ({} MB) dépasse la limite de {} MB du tampon de stockage GPU. Plan 2D, Ligne 1D et Globe 3D restent disponibles."
        }
        VariablesKey::RoleNone => "Aucun",
        VariablesKey::RoleAnimated => "Animé",
        VariablesKey::RoleGrid => "Grille",
        VariablesKey::Index => "Indice :",
        VariablesKey::ChannelN => "Canal {}",
    })
}
