//! Finnish phrasings of the verbs.

use super::Phrases;
use super::Verb::*;

pub(super) static PHRASES: Phrases = Phrases {
    verbs: &[
        (Hide, &["piilota"]),
        (Reveal, &["näytä uudelleen"]),
        (Remove, &["poista"]),
        (Range, &["väli", "väriväli", "rajat"]),
        (Export, &["vie", "tallenna kuva"]),
        (Describe, &["kuvaile", "mikä on", "tiedot"]),
        (Search, &["etsi", "hae"]),
        (List, &["luettele", "mitkä muuttujat"]),
        (Goto, &["siirry", "hyppää", "aseta"]),
        (Open, &["avaa", "lataa", "vaihda"]),
        (Select, &["valitse"]),
        (Plot, &["näytä", "piirrä", "visualisoi", "plottaa"]),
        (Play, &["toista", "käynnistä", "aloita", "animoi"]),
        (Pause, &["tauko", "pysäytä", "keskeytä"]),
        (Next, &["seuraava", "seuraava vaihe", "eteenpäin"]),
        (
            Previous,
            &["edellinen", "edellinen vaihe", "taaksepäin", "takaisin"],
        ),
        (Colormap, &["väriskaala", "värikartta", "paletti", "värit"]),
        (ResetView, &["palauta näkymä", "nollaa näkymä"]),
    ],
    months: [
        &["tammikuu", "tammikuuta"],
        &["helmikuu", "helmikuuta"],
        &["maaliskuu", "maaliskuuta"],
        &["huhtikuu", "huhtikuuta"],
        &["toukokuu", "toukokuuta"],
        &["kesäkuu", "kesäkuuta"],
        &["heinäkuu", "heinäkuuta"],
        &["elokuu", "elokuuta"],
        &["syyskuu", "syyskuuta"],
        &["lokakuu", "lokakuuta"],
        &["marraskuu", "marraskuuta"],
        &["joulukuu", "joulukuuta"],
    ],
    axes: [
        &["aika", "päivämäärä"],
        &["leveysaste"],
        &["pituusaste"],
        &["taso", "syvyys", "korkeus"],
    ],
    dataset: &["aineisto", "data"],
    overlay: &["päällekkäin", "päälle", "tasona"],
    well: &["kuoppa"],
    field: &["kenttä"],
    catalog: &["luettelo", "katalogi"],
    then: &["sitten", "sen jälkeen", "ja sitten"],
};
