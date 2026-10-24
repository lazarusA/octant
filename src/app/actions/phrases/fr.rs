//! French phrasings of the verbs.

use super::Phrases;
use super::Verb::*;

pub(super) static PHRASES: Phrases = Phrases {
    verbs: &[
        (
            Describe,
            &["décris", "qu'est-ce que", "informations", "infos"],
        ),
        (Search, &["cherche", "chercher", "trouve", "recherche"]),
        (List, &["liste", "quelles variables"]),
        (Goto, &["va à", "aller à", "saute à", "mets"]),
        (Open, &["ouvre", "ouvrir", "charge", "charger", "passe à"]),
        (
            Select,
            &["sélectionne", "sélectionner", "choisis", "choisir"],
        ),
        (
            Plot,
            &[
                "affiche",
                "afficher",
                "trace",
                "tracer",
                "dessine",
                "dessiner",
                "montre",
                "montrer",
                "visualise",
                "visualiser",
            ],
        ),
        (
            Play,
            &[
                "lecture", "lance", "lancer", "joue", "jouer", "anime", "animer", "démarre",
                "reprends",
            ],
        ),
        (Pause, &["pause", "arrête", "arrêter", "stop"]),
        (Next, &["suivant", "étape suivante", "avance"]),
        (
            Previous,
            &["précédent", "étape précédente", "recule", "retour"],
        ),
        (
            Colormap,
            &["carte de couleurs", "palette", "couleurs", "colormap"],
        ),
        (
            ResetView,
            &[
                "réinitialise la vue",
                "réinitialiser la vue",
                "vue par défaut",
            ],
        ),
    ],
    months: [
        &["janvier"],
        &["février"],
        &["mars"],
        &["avril"],
        &["mai"],
        &["juin"],
        &["juillet"],
        &["août"],
        &["septembre"],
        &["octobre"],
        &["novembre"],
        &["décembre"],
    ],
    axes: [
        &["temps", "date"],
        &["latitude"],
        &["longitude"],
        &["niveau", "profondeur", "altitude"],
    ],
    dataset: &["jeu de données", "données"],
    catalog: &["catalogue"],
    then: &["puis", "ensuite", "et puis"],
};
