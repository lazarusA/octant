use crate::i18n::keys::NoticesKey;

pub fn text(key: NoticesKey) -> Option<&'static str> {
    Some(match key {
        NoticesKey::DismissHover => "Ignorer",
        NoticesKey::CopyDetails => "Copier les détails",
        NoticesKey::RevealInFinder => "Afficher dans le Finder",
        NoticesKey::OpenFolder => "Ouvrir le dossier",
        NoticesKey::NoVariablesFound => "Aucune variable trouvée",
        NoticesKey::CouldntOpenDataset => "Impossible d'ouvrir le jeu de données",
        NoticesKey::UnsupportedFile => "Fichier non pris en charge",
        NoticesKey::CouldntLoadData => "Impossible de charger les données",
        NoticesKey::CoordinatesUnavailable => "Coordonnées indisponibles",
        NoticesKey::CoordinatesFailedDetail => {
            "Les coordonnées de '{}' n'ont pas pu être chargées ({}) ; les axes affichent des indices"
        }
        NoticesKey::CoastlinesUnavailable => "Côtes indisponibles",
        NoticesKey::VolumeTooLargeForGpu => "Volume trop grand pour le GPU",
        NoticesKey::VolumeBufferTooLargeDetail => {
            "La taille du tampon de données du volume ({} éléments / {} octets) dépasse la limite maximale du tampon de stockage GPU ({})"
        }
        NoticesKey::VolumeTextureTooLargeDetail => {
            "Le volume {}x{}x{} dépasse la limite de texture 3D du GPU ({})"
        }
        NoticesKey::VolumeUploadSkipped => "Envoi du volume ignoré",
        NoticesKey::VolumeUploadMismatchDetail => "{} valeurs pour une texture {}x{}x{}",
        NoticesKey::DataTooLargeForGpuBuffer => "Données trop grandes pour le tampon GPU",
        NoticesKey::BufferTooLargeDetail => {
            "{} : la taille des données ({} octets) dépasse la capacité du tampon GPU ({} octets), écriture ignorée"
        }
        NoticesKey::BufferRangeTooLargeDetail => {
            "{} : la plage à l'élément {} (+{} éléments) dépasse la capacité du tampon GPU ({} octets), écriture ignorée"
        }
        NoticesKey::LinePlotTooLargeForGpu => "Graphique en lignes trop grand pour le GPU",
        NoticesKey::LinePayloadTooLargeDetail => {
            "Les lignes occupent {} octets ; ce GPU en lie au plus {}."
        }
        NoticesKey::FontLoadFailed => "Impossible de charger une police",
        NoticesKey::FontLoadFailedDetail => {
            "Le texte en {} peut s'afficher sous forme de carrés : {}"
        }
    })
}
