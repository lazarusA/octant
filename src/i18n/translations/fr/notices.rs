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
        NoticesKey::NoDataAtStep => "Aucune donnée à cette étape",
        NoticesKey::NoDataAtStepDetail => "'{}' n'a aucune valeur à cette étape : le magasin ne contient pas de données ici. Essayez une autre étape.",
        NoticesKey::LargeChunks => "Chunks volumineux",
        NoticesKey::LargeChunksDetail => "'{}' est stocké en chunks de {} : chaque étape télécharge au moins un chunk entier, les étapes se chargent donc lentement.",
        NoticesKey::VolumeTooLargeForGpu => "Volume trop grand pour le GPU",
        NoticesKey::VolumeLodOff => "Le détail selon la vue est désactivé pour ce volume",
        NoticesKey::VolumeLodOffDetail => {
            "Son niveau affiché demande {} briques ; l'atlas en contient {} et en garde la moitié pour les plus fines. Le volume est dessiné sur un seul niveau entier."
        }
        NoticesKey::VolumeLodUnevenDetail => {
            "Ses niveaux ne sont pas des multiples entiers les uns des autres avec des cellules alignées, comme l'exigent les briques. Le volume est dessiné d'un seul niveau entier."
        }
        NoticesKey::VolumeLodWideIntegersDetail => {
            "Ses valeurs entières (comme des identifiants d'étiquettes) couvrent une plage trop large pour que les briques les distinguent. Le volume est dessiné d'un seul niveau entier."
        }
        NoticesKey::VolumeLodUnsupportedDetail => {
            "Ce GPU ne peut pas écrire depuis le shader du volume, ce que les briques exigent. Le volume est dessiné d'un seul niveau entier."
        }
        NoticesKey::VolumeLodBrickFailed => "Une partie du volume n'a pas pu être lue plus finement",
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
        NoticesKey::OmeMetadataNonConforming => "Les métadonnées OME-Zarr ne sont pas conformes",
        NoticesKey::LabelIdsBeyond32Bits => {
            "Les identifiants d'étiquette au-delà de 32 bits partagent les couleurs d'identifiants plus petits"
        }
        NoticesKey::OmeMetadataNonConformingDetail => {
            "Les métadonnées en version {} de {} ne suivent pas la spécification ; affichage de ce qui a pu être lu."
        }
        NoticesKey::PlateWellStopped => "Puits {} : la lecture s'est interrompue",
        NoticesKey::PlateWellNotJson => "Puits {} : ses métadonnées ne sont pas un JSON valide",
        NoticesKey::PlateWellNotOme => "Puits {} : pas un puits OME-Zarr",
        NoticesKey::PlateWellNoMetadata => "Puits {} : aucune métadonnée de puits",
    })
}
