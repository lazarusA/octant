//! Norwegian phrasings of the verbs.

use super::Phrases;
use super::Verb::*;

pub(super) static PHRASES: Phrases = Phrases {
    verbs: &[
        (Describe, &["beskriv", "hva er", "informasjon", "info"]),
        (Search, &["søk", "finn", "let etter"]),
        (List, &["list opp", "hvilke variabler"]),
        (Goto, &["gå til", "hopp til", "sett"]),
        (Open, &["åpne", "last inn", "bytt til"]),
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
    months: [
        &["januar"],
        &["februar"],
        &["mars"],
        &["april"],
        &["mai"],
        &["juni"],
        &["juli"],
        &["august"],
        &["september"],
        &["oktober"],
        &["november"],
        &["desember"],
    ],
    axes: [
        &["tid", "dato"],
        &["breddegrad"],
        &["lengdegrad"],
        &["nivå", "dybde", "høyde"],
    ],
    dataset: &["datasett", "data"],
    catalog: &["katalog", "katalogen"],
    then: &["deretter", "og så", "så"],
};
