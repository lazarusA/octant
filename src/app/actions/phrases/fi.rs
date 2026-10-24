//! Finnish phrasings of the verbs.

use super::Phrases;
use super::Verb::*;

pub(super) static PHRASES: Phrases = Phrases {
    verbs: &[
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
    then: &["sitten", "sen jälkeen", "ja sitten"],
};
