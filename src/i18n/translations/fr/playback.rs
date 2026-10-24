use crate::i18n::keys::PlaybackKey;

pub fn text(key: PlaybackKey) -> Option<&'static str> {
    Some(match key {
        PlaybackKey::Play => "Lire",
        PlaybackKey::Pause => "Pause",
        PlaybackKey::Loop => "Boucle",
        PlaybackKey::FirstStep => "Premier pas",
        PlaybackKey::PreviousStep => "Pas précédent",
        PlaybackKey::NextStep => "Pas suivant",
        PlaybackKey::LastStep => "Dernier pas",
        PlaybackKey::Crop => "Recadrer",
        PlaybackKey::CropHover => "Lignes de recadrage",
        PlaybackKey::PlaybackSpeed => "Vitesse de lecture",
        PlaybackKey::PlaybackBuffer => "Mise en mémoire tampon avant lecture",
        PlaybackKey::Buffering => "Mise en mémoire tampon {}/{}",
        PlaybackKey::SaveExportHover => {
            "Enregistrer / exporter la figure (Cmd+S pour un enregistrement rapide, Cmd+Shift+S pour les paramètres)"
        }
        PlaybackKey::CollapsedHint => "Lecture (cliquez ou faites glisser pour déplier)",
        PlaybackKey::TimelineDetails => "Détails de la chronologie",
        PlaybackKey::TimelineDimension => "Dimension : {}",
        PlaybackKey::TimelineStep => "Pas : {} / {}",
        PlaybackKey::TimelineCurrent => "Actuel : {}",
        PlaybackKey::TimelineRange => "Plage : {} -> {}",
        PlaybackKey::TimelineStepSize => "Taille du pas : {}",
        PlaybackKey::DefaultStepSize => "Pas : 1",
        PlaybackKey::VoiceListen => "Commencer l'écoute",
        PlaybackKey::VoiceStop => "Arrêter l'écoute",
        PlaybackKey::VoiceListening => "Écoute...",
        PlaybackKey::VoiceMicOff => "Microphone coupé",
        PlaybackKey::VoiceMicError => "Microphone indisponible",
        PlaybackKey::VoiceSilent => "Le microphone n'envoie que du silence",
        PlaybackKey::VoiceSilentHint => {
            "Autorisez l'accès au microphone pour l'application qui a lancé Octant dans les réglages de confidentialité du système, puis relancez-la."
        }
        PlaybackKey::VoiceSttFailed => "La transcription a échoué",
        PlaybackKey::VoiceDownloading => "Téléchargement du modèle vocal : {} %",
        PlaybackKey::VoiceLoading => "Chargement du modèle vocal...",
        PlaybackKey::VoiceModelHover => "Modèle de reconnaissance vocale",
        PlaybackKey::VoiceModelSize => "{} ({} Mo)",
        PlaybackKey::VoiceCloseHover => "Fermer la saisie vocale",
        PlaybackKey::VoiceDidOpen => "Ouverture de {}",
        PlaybackKey::VoiceNoSource => "Aucun jeu de données ne correspond à '{}'",
        PlaybackKey::VoiceDidSelect => "{} sélectionné",
        PlaybackKey::VoiceDidSetDim => "{} : {}",
        PlaybackKey::VoiceDidSetRange => "{} : {} - {}",
        PlaybackKey::VoiceNoDimension => "Aucune dimension ne correspond à '{}'",
        PlaybackKey::VoiceNoValue => "Aucune valeur de {} ne correspond à '{}'",
        PlaybackKey::VoiceDidPlot => "{} tracé en {}",
        PlaybackKey::VoiceDidColormap => "Palette {}",
        PlaybackKey::VoiceNotUnderstood => "Commande non comprise",
        PlaybackKey::VoiceNoDataset => "Ouvrez d'abord un jeu de données",
        PlaybackKey::VoiceNoVariable => "Aucune variable ne correspond à '{}'",
        PlaybackKey::VoiceNoPlotType => "Aucun type de tracé ne correspond à '{}'",
        PlaybackKey::VoiceNoColormap => "Aucune palette ne correspond à '{}'",
        PlaybackKey::VoiceAmbiguous => "Plusieurs variables correspondent à '{}'",
        PlaybackKey::VoiceDidYouMean => "Vouliez-vous dire : {}",
        PlaybackKey::VoiceNothingToAnimate => "Le tracé n'a pas de dimension animée",
        PlaybackKey::VoiceCannotPlot => "Impossible de tracer en {}",
    })
}
