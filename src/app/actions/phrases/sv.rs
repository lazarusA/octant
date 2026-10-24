//! Swedish phrasings of the verbs.

use super::Phrases;
use super::Verb::*;

pub(super) static PHRASES: Phrases = Phrases {
    verbs: &[
        (Hide, &["dölj", "göm"]),
        (Reveal, &["visa igen"]),
        (Remove, &["ta bort", "radera"]),
        (Range, &["intervall", "färgintervall", "gränser"]),
        (Export, &["exportera", "spara bilden"]),
        (Describe, &["beskriv", "vad är", "information", "info"]),
        (Search, &["sök", "hitta", "leta efter"]),
        (List, &["lista", "vilka variabler"]),
        (Goto, &["gå till", "hoppa till", "sätt"]),
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
    months: [
        &["januari"],
        &["februari"],
        &["mars"],
        &["april"],
        &["maj"],
        &["juni"],
        &["juli"],
        &["augusti"],
        &["september"],
        &["oktober"],
        &["november"],
        &["december"],
    ],
    axes: [
        &["tid", "datum"],
        &["latitud", "breddgrad"],
        &["longitud", "längdgrad"],
        &["nivå", "djup", "höjd"],
    ],
    dataset: &["datamängd", "data"],
    overlay: &["överlägg", "ovanpå", "som lager"],
    well: &["brunn"],
    field: &["fält"],
    catalog: &["katalog", "katalogen"],
    then: &["sedan", "därefter", "och sedan"],
    until: &["till", "och"],
};
