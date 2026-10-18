use crate::i18n::keys::KeymapKey;

pub fn text(key: KeymapKey) -> Option<&'static str> {
    Some(match key {
        KeymapKey::Title => "Raccourcis clavier",
        KeymapKey::CloseHover => "Fermer les raccourcis (Échap)",
        KeymapKey::ScopeGlobal => "Partout",
        KeymapKey::ScopePanel => "Dans un panneau",
        KeymapKey::ScopeFree => "Hors des champs de texte",
        KeymapKey::ScopeCanvas => "Sur le graphique",
        KeymapKey::ScopePlayback => "Lecture",
        KeymapKey::Arrows => "Flèches",
        KeymapKey::Hint => "{} ({})",
        KeymapKey::QuickSave => "Enregistrer la figure",
        KeymapKey::ExportDialog => "Exporter la figure avec réglages",
        KeymapKey::ToggleCrop => "Afficher ou masquer le recadrage",
        KeymapKey::ToggleDataset => "Afficher ou masquer le panneau Jeu de données",
        KeymapKey::ToggleVariables => "Afficher ou masquer le panneau Variables",
        KeymapKey::ToggleDimensions => "Afficher ou masquer le panneau Dimensions",
        KeymapKey::ToggleSettings => "Afficher ou masquer les paramètres",
        KeymapKey::SearchVariables => "Rechercher des variables",
        KeymapKey::ClosePanel => "Fermer le panneau",
        KeymapKey::NextRegion => "Aller à la zone suivante",
        KeymapKey::PreviousRegion => "Aller à la zone précédente",
        KeymapKey::FocusTopBar => "Aller à la barre du haut",
        KeymapKey::PlayPause => "Lire ou mettre en pause",
        KeymapKey::StepPrev => "Pas précédent",
        KeymapKey::StepNext => "Pas suivant",
        KeymapKey::FirstStep => "Premier pas",
        KeymapKey::LastStep => "Dernier pas",
        KeymapKey::ToggleLoop => "Activer ou désactiver la boucle",
        KeymapKey::Move => {
            "Déplacer (2D) ou tourner (3D) le graphique ; déplacer la sonde ou le recadrage"
        }
        KeymapKey::MoveAlt => {
            "Déplacer davantage (2D) ou déplacer (3D) ; redimensionner le recadrage"
        }
        KeymapKey::ZoomIn => "Zoom avant",
        KeymapKey::ZoomOut => "Zoom arrière",
        KeymapKey::ResetView => "Réinitialiser la vue",
        KeymapKey::ToggleProbe => "Lire les valeurs avec les flèches",
        KeymapKey::LeaveProbe => "Arrêter la lecture des valeurs",
        KeymapKey::ShowShortcuts => "Afficher les raccourcis clavier",
    })
}
