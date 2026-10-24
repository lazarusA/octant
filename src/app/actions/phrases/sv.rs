//! Swedish phrasings of the verbs.

use super::Phrases;
use super::Verb::*;

pub(super) static PHRASES: Phrases = Phrases {
    verbs: &[
        (Open, &["öppna", "ladda", "byt till"]),
        (Select, &["välj", "markera"]),
        (Plot, &["visa", "rita", "plotta", "visualisera"]),
        (Play, &["spela", "spela upp", "starta", "animera"]),
        (Pause, &["pausa", "paus", "stoppa", "stanna"]),
        (Next, &["nästa", "nästa steg", "framåt"]),
        (
            Previous,
            &["föregående", "föregående steg", "bakåt", "tillbaka"],
        ),
        (Colormap, &["färgskala", "färgkarta", "palett", "färger"]),
        (ResetView, &["återställ vyn", "återställ vy"]),
    ],
    catalog: &["katalog", "katalogen"],
    then: &["sedan", "därefter", "och sedan"],
};
