//! Norwegian phrasings of the verbs.

use super::Phrases;
use super::Verb::*;

pub(super) static PHRASES: Phrases = Phrases {
    verbs: &[
        (Select, &["velg", "marker"]),
        (Plot, &["vis", "tegn", "plott", "visualiser"]),
        (Play, &["spill", "spill av", "start", "animer"]),
        (Pause, &["pause", "stopp", "stans"]),
        (Next, &["neste", "neste steg", "fremover"]),
        (Previous, &["forrige", "forrige steg", "bakover", "tilbake"]),
        (Colormap, &["fargeskala", "fargekart", "palett", "farger"]),
        (
            ResetView,
            &["tilbakestill visningen", "tilbakestill visning"],
        ),
    ],
    then: &["deretter", "og så", "så"],
};
