//! German phrasings of the verbs.

use super::Phrases;
use super::Verb::*;

pub(super) static PHRASES: Phrases = Phrases {
    verbs: &[
        (Goto, &["gehe zu", "geh zu", "springe zu", "setze"]),
        (Open, &["öffne", "öffnen", "lade", "laden", "wechsle zu"]),
        (Select, &["wähle", "wählen", "auswählen", "wähle aus"]),
        (
            Plot,
            &[
                "zeige",
                "zeig",
                "zeichne",
                "plotte",
                "darstellen",
                "stelle dar",
                "anzeigen",
                "visualisiere",
            ],
        ),
        (
            Play,
            &[
                "abspielen",
                "spiele",
                "spiel ab",
                "starte",
                "animiere",
                "fortsetzen",
            ],
        ),
        (
            Pause,
            &["pause", "pausiere", "anhalten", "halte an", "stopp", "stop"],
        ),
        (Next, &["weiter", "nächster", "nächster schritt"]),
        (Previous, &["zurück", "vorheriger", "vorheriger schritt"]),
        (
            Colormap,
            &[
                "farbskala",
                "farbpalette",
                "farbkarte",
                "palette",
                "farben",
                "colormap",
            ],
        ),
        (
            ResetView,
            &[
                "ansicht zurücksetzen",
                "setze die ansicht zurück",
                "ansicht zurück",
            ],
        ),
    ],
    months: [
        &["januar", "jänner"],
        &["februar"],
        &["märz"],
        &["april"],
        &["mai"],
        &["juni"],
        &["juli"],
        &["august"],
        &["september"],
        &["oktober"],
        &["november"],
        &["dezember"],
    ],
    axes: [
        &["zeit", "datum"],
        &["breite", "breitengrad"],
        &["länge", "längengrad"],
        &["ebene", "tiefe", "höhe"],
    ],
    catalog: &["katalog"],
    then: &["dann", "danach", "und dann"],
};
