use crate::i18n::keys::PlaybackKey;

pub fn text(key: PlaybackKey) -> Option<&'static str> {
    Some(match key {
        PlaybackKey::Play => "Lire",
        PlaybackKey::Pause => "Pause",
        PlaybackKey::Loop => "Boucle",
        PlaybackKey::FirstStep => "Première étape",
        PlaybackKey::PreviousStep => "Étape précédente",
        PlaybackKey::NextStep => "Étape suivante",
        PlaybackKey::LastStep => "Dernière étape",
        PlaybackKey::Crop => "Recadrer",
        PlaybackKey::CropHover => "Lignes de recadrage (C)",
        PlaybackKey::PlaybackSpeed => "Vitesse de lecture",
        PlaybackKey::SaveExportHover => {
            "Enregistrer / exporter la figure (Cmd+S pour un enregistrement rapide, Cmd+Shift+S pour les paramètres)"
        }
        PlaybackKey::CollapsedHint => "Lecture (cliquez ou faites glisser pour déplier)",
        PlaybackKey::TimelineDetails => "Détails de la chronologie",
        PlaybackKey::TimelineDimension => "Dimension : {}",
        PlaybackKey::TimelineStep => "Étape : {} / {}",
        PlaybackKey::TimelineCurrent => "Actuel : {}",
        PlaybackKey::TimelineRange => "Plage : {} -> {}",
        PlaybackKey::TimelineStepSize => "Taille du pas : {}",
        PlaybackKey::DefaultStepSize => "Pas : 1",
    })
}
