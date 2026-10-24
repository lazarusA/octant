//! Finnish phrasings of the verbs.

use super::Phrases;
use super::Verb::*;

pub(super) static PHRASES: Phrases = Phrases {
    verbs: &[
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
    catalog: &["luettelo", "katalogi"],
    then: &["sitten", "sen jälkeen", "ja sitten"],
};
